//! Tunnel client -- the "frpc" half.
//!
//! Opens a WebSocket control channel to the relay, authenticates with the
//! tunnel token, and forwards every inbound relay stream to a local TCP port
//! (the daemon's tunnel endpoint). See `docs/relay.md`.
//!
//! The bridge is a pure bidirectional byte pump: no request/response framing is
//! assumed, so long-lived streams (SSE) work unchanged.
//!
//! Resource discipline: all queues are bounded (a stalled consumer applies
//! backpressure instead of growing without bound), streams are capped, and a
//! silent control channel is treated as dead so the caller can reconnect.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

use crate::protocol::{decode, Frame, TOKEN_QUERY_PARAM};

/// Local address the relay streams are forwarded to.
pub const LOCAL_HOST: &str = "127.0.0.1";

/// Read buffer size for the local->relay direction.
const BUF_SIZE: usize = 16 * 1024;

/// Queue depth for the control channel and for each stream. Bounded on purpose.
const QUEUE_DEPTH: usize = 64;

/// Maximum number of concurrent streams we will bridge.
const MAX_STREAMS: usize = 512;

/// How often we ping the relay to keep an idle (but healthy) tunnel alive.
/// Without this, a quiet tunnel would be indistinguishable from a dead one.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// After one side hits EOF, give the other direction this long to drain.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(10);

/// Bail out instead of hanging forever when the relay never completes the
/// handshake (unreachable, or already serving another client -- the relay
/// serves one tunnel client at a time).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for reaching the local endpoint.
const LOCAL_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Bounded send queue + frame cap, so a single peer cannot exhaust memory.
fn ws_config() -> WebSocketConfig {
    WebSocketConfig {
        // NOTE: `max_send_queue` is deprecated in tungstenite 0.24; backpressure
        // is provided by the bounded mpsc queues in front of the socket instead.
        max_message_size: Some(1 << 20), // 1 MiB
        max_frame_size: Some(1 << 20),
        ..Default::default()
    }
}

/// Start the tunnel client. Returns when the control channel closes (the caller
/// is expected to retry with backoff).
///
/// * `relay_url`  -- e.g. `ws://host:7000/tunnel` or `wss://host:7000/tunnel`
/// * `token`      -- shared secret the relay checks before granting the tunnel
/// * `local_port` -- the daemon's local endpoint that inbound streams reach
pub async fn start_tunnel_client(relay_url: &str, token: &str, local_port: u16) -> Result<()> {
    let url = with_token(relay_url, token);
    info!("tunnel: connecting to relay {relay_url}");
    let (ws, _resp) = tokio::time::timeout(
        CONNECT_TIMEOUT,
        tokio_tungstenite::connect_async_with_config(url.as_str(), Some(ws_config()), false),
    )
    .await
    .with_context(|| format!("timed out connecting to relay {relay_url}"))?
    .with_context(|| format!("connect to relay {relay_url}"))?;
    info!("tunnel: relay connected, forwarding to {LOCAL_HOST}:{local_port}");

    let (mut sink, mut stream) = ws.split();

    // Everything we send to the relay goes through this bounded channel, so a
    // slow WebSocket applies backpressure to the streams instead of queueing
    // without bound.
    let (out_tx, mut out_rx) = mpsc::channel::<Message>(QUEUE_DEPTH);
    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    // stream id -> sender feeding bytes into that stream's local TCP socket.
    let streams: Arc<Mutex<HashMap<u32, mpsc::Sender<Vec<u8>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut keepalive = tokio::time::interval(KEEPALIVE_INTERVAL);
    keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        let msg = tokio::select! {
            m = stream.next() => match m {
                Some(m) => m,
                None => {
                    info!("tunnel: relay closed the control channel");
                    break;
                }
            },
            _ = keepalive.tick() => {
                // Keepalive: an idle tunnel must not look dead. A failure to
                // send here means the peer really is gone.
                if out_tx.send(Message::Ping(Vec::new())).await.is_err() {
                    warn!("tunnel: keepalive failed, treating the relay as dead");
                    break;
                }
                continue;
            }
        };
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
                    Frame::Open(id) => {
                        // Register before spawning: the relay can push Data for
                        // this id immediately (protocols where the local side
                        // speaks first), and an unregistered id would be dropped.
                        let (in_tx, in_rx) = mpsc::channel(QUEUE_DEPTH);
                        {
                            let mut guard = streams.lock().await;
                            if guard.len() >= MAX_STREAMS {
                                warn!("tunnel: stream limit reached, rejecting stream {id}");
                                let _ = out_tx.try_send(Message::Binary(Frame::Close(id).encode()));
                                continue;
                            }
                            guard.insert(id, in_tx);
                        }
                        spawn_stream(id, local_port, in_rx, out_tx.clone(), streams.clone());
                    }
                    Frame::Data(id, payload) => {
                        let tx = streams.lock().await.get(&id).cloned();
                        match tx {
                            Some(tx) => {
                                // try_send keeps one stalled stream from
                                // head-of-line blocking the whole tunnel.
                                if tx.try_send(payload).is_err() {
                                    warn!("tunnel: stream {id}: queue full or gone, closing it");
                                    streams.lock().await.remove(&id);
                                    let _ =
                                        out_tx.try_send(Message::Binary(Frame::Close(id).encode()));
                                }
                            }
                            None => {
                                debug!("tunnel: stream {id}: data for an unknown stream");
                                let _ = out_tx.try_send(Message::Binary(Frame::Close(id).encode()));
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

    // Drop every sender so each pump sees EOF and shuts its socket down.
    streams.lock().await.clear();
    writer.abort();
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
    mut in_rx: mpsc::Receiver<Vec<u8>>,
    out_tx: mpsc::Sender<Message>,
    streams: Arc<Mutex<HashMap<u32, mpsc::Sender<Vec<u8>>>>>,
) {
    tokio::spawn(async move {
        let local = match tokio::time::timeout(
            LOCAL_CONNECT_TIMEOUT,
            TcpStream::connect((LOCAL_HOST, local_port)),
        )
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                warn!("tunnel: stream {id}: local endpoint unreachable: {e}");
                let _ = out_tx
                    .send(Message::Binary(Frame::Close(id).encode()))
                    .await;
                return;
            }
            Err(_) => {
                warn!("tunnel: stream {id}: local connect timed out");
                let _ = out_tx
                    .send(Message::Binary(Frame::Close(id).encode()))
                    .await;
                return;
            }
        };
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
                        if out_tx_relay.send(Message::Binary(frame)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // relay -> local TCP
        let from_relay = tokio::spawn(async move {
            while let Some(bytes) = in_rx.recv().await {
                if wr.write_all(&bytes).await.is_err() {
                    break;
                }
            }
            let _ = wr.shutdown().await;
        });

        let _ = to_relay.await;
        // Half-close: the local side is finished. Announce it now and only give
        // the reverse direction a bounded grace period, so the remote peer can
        // never hang forever waiting on us.
        let _ = out_tx
            .send(Message::Binary(Frame::Close(id).encode()))
            .await;
        if tokio::time::timeout(DRAIN_TIMEOUT, from_relay)
            .await
            .is_err()
        {
            warn!("tunnel: stream {id}: drain timed out, forcing close");
        }
        streams.lock().await.remove(&id);
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
