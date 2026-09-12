//! Tunnel client -- the "frpc" half.
//!
//! Opens a WebSocket control channel to the relay, authenticates with the
//! tunnel token, and forwards every inbound relay stream to a local TCP port
//! (the daemon's tunnel endpoint). See `docs/relay.md` for the design.
//!
//! The bridge is a pure bidirectional byte pump: no request/response framing is
//! assumed, so long-lived streams (SSE) work unchanged.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::protocol::{decode, Frame};

/// Query parameter carrying the tunnel token on the control-channel handshake.
pub const TOKEN_QUERY_PARAM: &str = "token";

/// Local address the relay streams are forwarded to.
pub const LOCAL_HOST: &str = "127.0.0.1";

/// Read buffer size for the local->relay direction.
const BUF_SIZE: usize = 16 * 1024;

/// Start the tunnel client. Returns when the control channel closes.
///
/// * `relay_url`  -- e.g. `ws://host:7000/tunnel` or `wss://host:7000/tunnel`
/// * `token`      -- shared secret the relay checks before granting the tunnel
/// * `local_port` -- the daemon's local endpoint that inbound streams reach
pub async fn start_tunnel_client(relay_url: &str, token: &str, local_port: u16) -> Result<()> {
    let url = with_token(relay_url, token);
    info!("tunnel: connecting to relay {relay_url}");
    let (ws, _resp) = tokio_tungstenite::connect_async(&url)
        .await
        .with_context(|| format!("connect to relay {relay_url}"))?;
    info!("tunnel: relay connected, forwarding to {LOCAL_HOST}:{local_port}");

    let (mut sink, mut stream) = ws.split();

    // Everything we send to the relay goes through this channel.
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
    tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    // stream id -> sender feeding bytes into that stream's local TCP socket.
    let streams: Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<Vec<u8>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Binary(bytes)) => {
                let frame = match decode(&bytes) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("tunnel: dropped a malformed frame: {e}");
                        continue;
                    }
                };
                match frame {
                    Frame::Open(id) => spawn_stream(id, local_port, out_tx.clone(), streams.clone()),
                    Frame::Data(id, payload) => {
                        let tx = streams.lock().await.get(&id).cloned();
                        match tx {
                            Some(tx) => {
                                if tx.send(payload).is_err() {
                                    streams.lock().await.remove(&id);
                                }
                            }
                            // Late data for a stream we no longer hold: tell the relay to close it.
                            None => {
                                let _ = out_tx.send(Message::Binary(Frame::Close(id).encode()));
                            }
                        }
                    }
                    Frame::Close(id) => {
                        streams.lock().await.remove(&id);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("tunnel: relay closed the control channel");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("tunnel: control channel error: {e}");
                break;
            }
        }
    }
    Ok(())
}

/// Append the (percent-encoded) tunnel token as a query parameter.
pub fn with_token(relay_url: &str, token: &str) -> String {
    let encoded: String = token
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect();
    let sep = if relay_url.contains('?') { '&' } else { '?' };
    format!("{relay_url}{sep}{TOKEN_QUERY_PARAM}={encoded}")
}

/// Bridge one relay stream to a fresh local TCP connection.
fn spawn_stream(
    id: u32,
    local_port: u16,
    out_tx: mpsc::UnboundedSender<Message>,
    streams: Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<Vec<u8>>>>>,
) {
    tokio::spawn(async move {
        let local = match TcpStream::connect((LOCAL_HOST, local_port)).await {
            Ok(s) => s,
            Err(e) => {
                warn!("tunnel: stream {id}: local endpoint unreachable: {e}");
                let _ = out_tx.send(Message::Binary(Frame::Close(id).encode()));
                return;
            }
        };
        let (in_tx, mut in_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        streams.lock().await.insert(id, in_tx);

        let (mut rd, mut wr) = local.into_split();

        // local TCP -> relay
        let out_tx_relay = out_tx.clone();
        let to_relay = tokio::spawn(async move {
            let mut buf = vec![0u8; BUF_SIZE];
            loop {
                match rd.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let frame = Frame::Data(id, buf[..n].to_vec()).encode();
                        if out_tx_relay.send(Message::Binary(frame)).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // relay -> local TCP
        let to_local = tokio::spawn(async move {
            while let Some(bytes) = in_rx.recv().await {
                if wr.write_all(&bytes).await.is_err() {
                    break;
                }
            }
            let _ = wr.shutdown().await;
        });

        let _ = to_relay.await;
        let _ = to_local.await;
        streams.lock().await.remove(&id);
        let _ = out_tx.send(Message::Binary(Frame::Close(id).encode()));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_appended_as_query_param() {
        assert_eq!(
            with_token("ws://h:7000/tunnel", "abc"),
            "ws://h:7000/tunnel?token=abc"
        );
    }

    #[test]
    fn existing_query_is_preserved() {
        assert_eq!(
            with_token("ws://h:7000/tunnel?x=1", "abc"),
            "ws://h:7000/tunnel?x=1&token=abc"
        );
    }

    #[test]
    fn unsafe_token_chars_are_percent_encoded() {
        assert_eq!(with_token("ws://h/t", "a b/c"), "ws://h/t?token=a%20b%2Fc");
    }
}
