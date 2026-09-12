//! Relay server -- the "frps" half.
//!
//! Exposes a public TCP port and multiplexes every inbound connection to the
//! connected tunnel client over a single WebSocket control channel. The client
//! authenticates with the tunnel token on the handshake. See `docs/relay.md`.
//!
//! # Lifecycle
//!
//! The relay is a long-running process: it accepts control connections in a
//! loop, so a rejected handshake or a client that simply disconnects never
//! takes it down -- it cleans up and waits for the next client.
//!
//! # One client at a time
//!
//! The relay serves **one** tunnel client at a time (single-tenant by design).
//! A second dev machine connecting while one is active simply waits in the
//! backlog until the first disconnects; it does NOT get a second tunnel, and
//! the relay never multiplexes two clients onto the same public port. Clients
//! therefore use a connect timeout so they fail fast rather than waiting
//! indefinitely.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

use crate::protocol::{decode, percent_decode, Frame, TOKEN_QUERY_PARAM};

/// Read buffer size for the public->client direction.
const BUF_SIZE: usize = 16 * 1024;

/// Queue depth for the control channel and for each stream. Bounded on purpose.
const QUEUE_DEPTH: usize = 64;

/// Maximum number of concurrent streams.
const MAX_STREAMS: usize = 512;

/// How often we ping the client to keep an idle (but healthy) tunnel alive.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// After one side hits EOF, give the other direction this long to drain.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(10);

/// stream id -> sender that feeds bytes into that stream's public socket.
type Streams = Arc<Mutex<HashMap<u32, mpsc::Sender<Vec<u8>>>>>;

/// Bounded send queue + frame cap, so one peer cannot exhaust memory.
fn ws_config() -> WebSocketConfig {
    WebSocketConfig {
        // NOTE: `max_send_queue` is deprecated in tungstenite 0.24; backpressure
        // is provided by the bounded mpsc queues in front of the socket instead.
        max_message_size: Some(1 << 20), // 1 MiB
        max_frame_size: Some(1 << 20),
        ..Default::default()
    }
}

/// Run the relay.
///
/// * `control_addr` -- where the tunnel client connects (e.g. `0.0.0.0:7000`)
/// * `public_addr`  -- the port remote clients hit (e.g. `0.0.0.0:8080`)
/// * `token`        -- shared secret the client must present (must be non-empty)
pub async fn run_relay(control_addr: &str, public_addr: &str, token: &str) -> Result<()> {
    let control = TcpListener::bind(control_addr)
        .await
        .with_context(|| format!("bind control address {control_addr}"))?;
    let public = TcpListener::bind(public_addr)
        .await
        .with_context(|| format!("bind public address {public_addr}"))?;
    run_relay_with(control, public, token).await
}

/// Run the relay on already-bound listeners (see [`run_relay`]).
pub async fn run_relay_with(control: TcpListener, public: TcpListener, token: &str) -> Result<()> {
    // An empty token would compare equal to a *missing* token below, letting
    // unauthenticated clients in. Refuse to start instead.
    if token.is_empty() {
        bail!("relay token must not be empty (it would let any client in)");
    }
    let caddr = control
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "?".to_string());
    let paddr = public
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "?".to_string());
    info!("relay: control={caddr} public={paddr}");

    let public = Arc::new(public);

    loop {
        let (stream, peer) = control
            .accept()
            .await
            .context("accept control connection")?;
        debug!("relay: control connection from {peer}");

        let expected = token.to_string();
        let handshake = tokio_tungstenite::accept_hdr_async_with_config(
            stream,
            move |req: &Request, resp: Response| {
                // `None` when the parameter is absent -- an absent token must
                // never be treated as an empty (valid) one.
                let presented = req.uri().query().and_then(|q| {
                    q.split('&').find_map(|kv| {
                        let mut it = kv.splitn(2, '=');
                        let key = it.next()?;
                        let value = it.next().unwrap_or("");
                        (key == TOKEN_QUERY_PARAM).then(|| percent_decode(value))
                    })
                });
                if presented.as_deref() == Some(expected.as_str()) {
                    Ok(resp)
                } else {
                    warn!("relay: rejected control handshake (bad or missing token)");
                    Err(Response::builder()
                        .status(401)
                        .body(None)
                        .expect("a fixed 401 response is always valid"))
                }
            },
            Some(ws_config()),
        )
        .await;

        let ws = match handshake {
            Ok(ws) => ws,
            Err(e) => {
                // Never let a bad handshake take the relay down.
                warn!("relay: control handshake from {peer} failed: {e}");
                continue;
            }
        };

        info!("relay: tunnel client authenticated: {peer}");
        serve_control(ws, public.clone()).await;
        info!("relay: control channel closed; waiting for the next client");
    }
}

/// Serve one authenticated tunnel client until it disconnects, then clean up.
async fn serve_control(
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    public: Arc<TcpListener>,
) {
    let (mut sink, mut stream) = ws.split();

    let (out_tx, mut out_rx) = mpsc::channel::<Message>(QUEUE_DEPTH);
    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let next_id = Arc::new(AtomicU32::new(1));
    let streams: Streams = Arc::new(Mutex::new(HashMap::new()));

    // Public port: every inbound connection becomes one stream on the tunnel.
    let pub_out = out_tx.clone();
    let pub_streams = streams.clone();
    let pub_ids = next_id.clone();
    let accept_task = tokio::spawn(async move {
        loop {
            match public.accept().await {
                Ok((sock, peer)) => {
                    let id = pub_ids.fetch_add(1, Ordering::Relaxed);
                    debug!("relay: public connection from {peer} as stream {id}");
                    // Register BEFORE announcing: the client may answer with
                    // Data immediately, and data for an unregistered id would
                    // be dropped.
                    let (in_tx, in_rx) = mpsc::channel(QUEUE_DEPTH);
                    {
                        let mut guard = pub_streams.lock().await;
                        if guard.len() >= MAX_STREAMS {
                            warn!("relay: stream limit reached, dropping connection from {peer}");
                            drop(sock);
                            continue;
                        }
                        guard.insert(id, in_tx);
                    }
                    if pub_out
                        .send(Message::Binary(Frame::Open(id).encode()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                    spawn_public_stream(id, sock, pub_out.clone(), pub_streams.clone(), in_rx);
                }
                Err(e) => {
                    error!("relay: public accept failed: {e}");
                    break;
                }
            }
        }
    });

    let mut keepalive = tokio::time::interval(KEEPALIVE_INTERVAL);
    keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        let msg = tokio::select! {
            m = stream.next() => match m {
                Some(m) => m,
                None => {
                    info!("relay: tunnel client disconnected");
                    break;
                }
            },
            _ = keepalive.tick() => {
                // Keepalive: an idle tunnel must not look dead. A failure to
                // send here means the client really is gone.
                if out_tx.send(Message::Ping(Vec::new())).await.is_err() {
                    warn!("relay: keepalive failed, dropping the client");
                    break;
                }
                continue;
            }
        };
        match msg {
            Ok(Message::Binary(bytes)) => match decode(&bytes) {
                Ok(Frame::Data(id, payload)) => {
                    let tx = streams.lock().await.get(&id).cloned();
                    if let Some(tx) = tx {
                        // try_send keeps one stalled stream from head-of-line
                        // blocking the whole tunnel.
                        if tx.try_send(payload).is_err() {
                            warn!("relay: stream {id}: queue full or gone, closing it");
                            streams.lock().await.remove(&id);
                            let _ = out_tx.try_send(Message::Binary(Frame::Close(id).encode()));
                        }
                    } else {
                        debug!("relay: stream {id}: data for an unknown stream");
                        let _ = out_tx.try_send(Message::Binary(Frame::Close(id).encode()));
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

    // Cleanup: stop accepting, then drop every sender so each pump sees EOF and
    // shuts its socket down, instead of hanging forever on a closed channel.
    accept_task.abort();
    streams.lock().await.clear();
    writer.abort();
}

/// Bridge one public TCP connection onto the tunnel as stream `id`.
fn spawn_public_stream(
    id: u32,
    sock: TcpStream,
    out_tx: mpsc::Sender<Message>,
    streams: Streams,
    mut in_rx: mpsc::Receiver<Vec<u8>>,
) {
    tokio::spawn(async move {
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
                        if out_tx_client.send(Message::Binary(frame)).await.is_err() {
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
        // Half-close: the public side is done. Announce it now and only give the
        // reverse direction a bounded grace period so the remote peer can never
        // hang forever waiting on us.
        let _ = out_tx.send(Message::Binary(Frame::Close(id).encode())).await;
        if tokio::time::timeout(DRAIN_TIMEOUT, from_client)
            .await
            .is_err()
        {
            warn!("relay: stream {id}: drain timed out, forcing close");
        }
        streams.lock().await.remove(&id);
    });
}
