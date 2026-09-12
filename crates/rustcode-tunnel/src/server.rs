//! Relay server -- the "frps" half.
//!
//! Exposes a public TCP port and multiplexes every inbound connection to the
//! connected tunnel client over a single WebSocket control channel. The client
//! authenticates with the tunnel token on the handshake. See `docs/relay.md`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

use crate::protocol::{decode, Frame, TOKEN_QUERY_PARAM};

/// Read buffer size for the public->client direction.
const BUF_SIZE: usize = 16 * 1024;

/// Run the relay on freshly bound listeners.
///
/// * `control_addr` -- where the tunnel client connects (e.g. `0.0.0.0:7000`)
/// * `public_addr`  -- the port remote clients hit (e.g. `0.0.0.0:8080`)
/// * `token`        -- shared secret the client must present
pub async fn run_relay(control_addr: &str, public_addr: &str, token: &str) -> Result<()> {
    let control = TcpListener::bind(control_addr)
        .await
        .with_context(|| format!("bind control address {control_addr}"))?;
    let public = TcpListener::bind(public_addr)
        .await
        .with_context(|| format!("bind public address {public_addr}"))?;
    info!("relay: control={control_addr} public={public_addr}");
    run_relay_with(control, public, token).await
}

/// Run the relay on already-bound listeners.
///
/// Same as [`run_relay`] minus the `bind` step: it takes ownership of two
/// listeners, so callers that bind ephemeral ports (`127.0.0.1:0`) can read the
/// OS-assigned addresses with [`TcpListener::local_addr`] before handing the
/// listeners over. This is what the end-to-end test uses.
pub async fn run_relay_with(
    control: TcpListener,
    public: TcpListener,
    token: &str,
) -> Result<()> {
    info!(
        "relay: control={} public={}",
        control
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "?".to_string()),
        public
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "?".to_string())
    );

    // Accept the tunnel client, validating its token on the handshake.
    let (stream, peer) = control
        .accept()
        .await
        .context("accept control connection")?;
    info!("relay: control connection from {peer}");
    let expected = token.to_string();
    let ws = tokio_tungstenite::accept_hdr_async(stream, move |req: &Request, resp: Response| {
        let presented = req
            .uri()
            .query()
            .and_then(|q| {
                q.split('&').find_map(|kv| {
                    let mut it = kv.splitn(2, '=');
                    let key = it.next()?;
                    let value = it.next().unwrap_or("");
                    (key == TOKEN_QUERY_PARAM).then_some(value.to_string())
                })
            })
            .unwrap_or_default();
        if presented == expected {
            Ok(resp)
        } else {
            warn!("relay: rejected control handshake (bad token)");
            Err(Response::builder().status(401).body(None).unwrap())
        }
    })
    .await
    .context("websocket handshake with tunnel client")?;
    info!("relay: tunnel client authenticated");

    let (mut sink, mut stream) = ws.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
    tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let next_id = Arc::new(AtomicU32::new(1));
    let streams: Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<Vec<u8>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Public port: every inbound connection becomes one stream on the tunnel.
    let pub_out = out_tx.clone();
    let pub_streams = streams.clone();
    let pub_ids = next_id.clone();
    tokio::spawn(async move {
        loop {
            match public.accept().await {
                Ok((sock, peer)) => {
                    debug!("relay: public connection from {peer}");
                    let id = pub_ids.fetch_add(1, Ordering::Relaxed);
                    if pub_out
                        .send(Message::Binary(Frame::Open(id).encode()))
                        .is_err()
                    {
                        break;
                    }
                    spawn_public_stream(id, sock, pub_out.clone(), pub_streams.clone());
                }
                Err(e) => {
                    error!("relay: public accept failed: {e}");
                    break;
                }
            }
        }
    });

    // Control channel: route Data/Close frames to the right public socket.
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Binary(bytes)) => match decode(&bytes) {
                Ok(Frame::Data(id, payload)) => {
                    let tx = streams.lock().await.get(&id).cloned();
                    match tx {
                        Some(tx) => {
                            if tx.send(payload).is_err() {
                                streams.lock().await.remove(&id);
                            }
                        }
                        None => warn!("relay: stream {id}: data for an unknown stream, dropping"),
                    }
                }
                Ok(Frame::Close(id)) => {
                    streams.lock().await.remove(&id);
                }
                Ok(Frame::Open(_)) => {
                    // Only the relay opens streams; ignore a stray Open.
                }
                Err(e) => warn!("relay: dropped a malformed frame: {e}"),
            },
            Ok(Message::Close(_)) => {
                info!("relay: tunnel client disconnected");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("relay: control channel error: {e}");
                break;
            }
        }
    }
    Ok(())
}

/// Bridge one public TCP connection onto the tunnel as stream `id`.
fn spawn_public_stream(
    id: u32,
    sock: TcpStream,
    out_tx: mpsc::UnboundedSender<Message>,
    streams: Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<Vec<u8>>>>>,
) {
    tokio::spawn(async move {
        let (in_tx, mut in_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        streams.lock().await.insert(id, in_tx);
        let (mut rd, mut wr) = sock.into_split();

        // public socket -> tunnel client
        let out_tx_client = out_tx.clone();
        let to_client = tokio::spawn(async move {
            let mut buf = vec![0u8; BUF_SIZE];
            loop {
                match rd.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let frame = Frame::Data(id, buf[..n].to_vec()).encode();
                        if out_tx_client.send(Message::Binary(frame)).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // tunnel client -> public socket
        let from_client = tokio::spawn(async move {
            while let Some(bytes) = in_rx.recv().await {
                if wr.write_all(&bytes).await.is_err() {
                    break;
                }
            }
            let _ = wr.shutdown().await;
        });

        let _ = to_client.await;
        let _ = from_client.await;
        streams.lock().await.remove(&id);
        let _ = out_tx.send(Message::Binary(Frame::Close(id).encode()));
    });
}
