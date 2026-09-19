//! End-to-end coverage for the self-hosted tunnel: relay (frps) + client (frpc).
//!
//! These tests prove the tunnel actually forwards bytes, which is the one item
//! `docs/relay.md` had left unchecked. Each test builds a full stack in-process:
//!
//! ```text
//! test TCP client -> relay public port -> WS control channel -> tunnel client -> echo endpoint
//! ```
//!
//! Everything is bound to `127.0.0.1:0`, so the OS assigns the ports and the
//! tests never collide with each other or with a real relay. Every test is
//! wrapped in a hard [`tokio::time::timeout`]: a regression fails fast instead
//! of hanging CI.

use std::time::Duration;

use rustcode_tunnel::{client, server};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Shared secret the relay and the client are configured with.
const TOKEN: &str = "test-token";
/// A token the relay must refuse.
const WRONG_TOKEN: &str = "wrong-token";
/// Hard ceiling for a whole test: a hang must surface as a failure.
const TEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Budget for one forwarding attempt while polling for the tunnel to come up.
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(1);
/// How many times we retry before declaring the tunnel dead.
const MAX_ATTEMPTS: usize = 20;

/// Spin up a full stack: echo endpoint, relay, tunnel client.
///
/// Returns the relay's public port -- the address a remote client would hit.
async fn spawn_stack(token: &str) -> u16 {
    // 1. The "local endpoint" the tunnel client forwards to: a byte echoer.
    let echo = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind echo endpoint");
    let echo_port = echo.local_addr().expect("echo endpoint addr").port();
    tokio::spawn(echo_forever(echo));

    // 2. The relay, on OS-assigned ports we can read back with `local_addr()`.
    let control = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind control port");
    let public = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind public port");
    let control_port = control.local_addr().expect("control addr").port();
    let public_port = public.local_addr().expect("public addr").port();

    let relay_token = token.to_string();
    tokio::spawn(async move {
        // Background task: its effect is observed through the assertions below.
        let _ = server::run_relay_with(control, public, &relay_token).await;
    });

    // 3. The tunnel client (the frpc half the daemon runs).
    let client_token = token.to_string();
    tokio::spawn(async move {
        let url = format!("ws://127.0.0.1:{control_port}/tunnel");
        let _ = client::start_tunnel_client(&url, &client_token, echo_port).await;
    });

    public_port
}

/// Accept forever on `listener`, echoing back every byte of every connection.
async fn echo_forever(listener: TcpListener) {
    while let Ok((mut sock, _peer)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                match sock.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if sock.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

/// Send `payload` into the relay's public port and read the same number of
/// bytes back, with a per-attempt timeout so a dead tunnel cannot hang the test.
async fn roundtrip(public_port: u16, payload: &[u8]) -> std::io::Result<Vec<u8>> {
    let attempt = async {
        let mut sock = TcpStream::connect(("127.0.0.1", public_port)).await?;
        sock.write_all(payload).await?;
        let mut got = vec![0u8; payload.len()];
        sock.read_exact(&mut got).await?;
        Ok(got)
    };
    match tokio::time::timeout(ATTEMPT_TIMEOUT, attempt).await {
        Ok(r) => r,
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!("no answer from public port {public_port}"),
        )),
    }
}

/// Same as [`roundtrip`], but retries while the tunnel is still coming up, so
/// the test depends on readiness rather than on a fixed sleep.
async fn roundtrip_when_ready(public_port: u16, payload: &[u8]) -> Vec<u8> {
    let mut last = None;
    for _ in 0..MAX_ATTEMPTS {
        match roundtrip(public_port, payload).await {
            Ok(got) => return got,
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
    panic!(
        "tunnel never forwarded traffic through public port {public_port} \
         after {MAX_ATTEMPTS} attempts: {last:?}"
    );
}

/// A remote client reaches the local endpoint through the relay, byte for byte.
#[tokio::test]
async fn tunnel_forwards_traffic_end_to_end() {
    tokio::time::timeout(TEST_TIMEOUT, async {
        let public_port = spawn_stack(TOKEN).await;
        let got = roundtrip_when_ready(public_port, b"ping").await;
        assert_eq!(got, b"ping".to_vec(), "echoed bytes must match");
    })
    .await
    .expect("e2e tunnel test timed out");
}

/// Several streams over one control channel stay independent and keep working
/// after earlier streams finished (stream ids must not leak between them).
#[tokio::test]
async fn tunnel_keeps_sequential_streams_independent() {
    tokio::time::timeout(TEST_TIMEOUT, async {
        let public_port = spawn_stack(TOKEN).await;
        assert_eq!(
            roundtrip_when_ready(public_port, b"ping").await,
            b"ping".to_vec()
        );

        // Distinct payloads, including one bigger than a single read buffer, so
        // a mix-up between stream ids or a truncated pump is visible.
        for (i, len) in [1usize, 1_024, 64 * 1024].iter().enumerate() {
            let payload: Vec<u8> = (0..*len).map(|b| (b as u8).wrapping_add(i as u8)).collect();
            let got = roundtrip(public_port, &payload)
                .await
                .unwrap_or_else(|e| panic!("stream {i} ({len} bytes) failed: {e}"));
            assert_eq!(got, payload, "stream {i} came back corrupted");
        }
    })
    .await
    .expect("sequential stream test timed out");
}

/// The relay multiplexes: many public connections are served at the same time.
#[tokio::test]
async fn tunnel_multiplexes_concurrent_streams() {
    tokio::time::timeout(TEST_TIMEOUT, async {
        let public_port = spawn_stack(TOKEN).await;
        // Wait until the tunnel is up before measuring concurrency.
        assert_eq!(
            roundtrip_when_ready(public_port, b"ping").await,
            b"ping".to_vec()
        );

        let mut in_flight = Vec::new();
        for i in 0..8u8 {
            in_flight.push(tokio::spawn(async move {
                let payload = vec![i; 4 * 1024];
                let got = roundtrip(public_port, &payload)
                    .await
                    .unwrap_or_else(|e| panic!("concurrent stream {i} failed: {e}"));
                assert_eq!(got, payload, "concurrent stream {i} came back corrupted");
            }));
        }
        for task in in_flight {
            task.await.expect("concurrent stream task panicked");
        }
    })
    .await
    .expect("concurrent stream test timed out");
}

/// A wrong tunnel token gets no tunnel: the relay answers `401` and the client
/// fails to connect. (Two fresh relays, because each one serves a single
/// control-connection attempt.)
#[tokio::test]
async fn relay_rejects_a_wrong_tunnel_token() {
    tokio::time::timeout(TEST_TIMEOUT, async {
        // (a) The relay itself answers 401 on the control handshake.
        let control_port = spawn_relay(TOKEN).await;
        let status = raw_handshake_status(control_port, WRONG_TOKEN).await;
        assert_eq!(status, 401, "relay must reject a wrong token with 401");

        // (b) Our own client cannot establish a tunnel with the wrong token.
        let control_port = spawn_relay(TOKEN).await;
        let url = format!("ws://127.0.0.1:{control_port}/tunnel");
        let err = client::start_tunnel_client(&url, WRONG_TOKEN, 1)
            .await
            .expect_err("start_tunnel_client must fail with a wrong token");
        let rendered = format!("{err:?}");
        assert!(
            rendered.contains("401"),
            "expected the 401 rejection in the error, got: {rendered}"
        );
    })
    .await
    .expect("wrong-token test timed out");
}

/// Start a relay on ephemeral ports; returns its control port.
async fn spawn_relay(token: &str) -> u16 {
    let control = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind control port");
    let public = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind public port");
    let control_port = control.local_addr().expect("control addr").port();
    let token = token.to_string();
    tokio::spawn(async move {
        let _ = server::run_relay_with(control, public, &token).await;
    });
    control_port
}

/// Do a raw WebSocket upgrade request against the control port and return the
/// HTTP status the relay answered with (`0` if it never answered).
async fn raw_handshake_status(control_port: u16, token: &str) -> u16 {
    let request = format!(
        "GET /tunnel?token={token} HTTP/1.1\r\n\
         Host: 127.0.0.1:{control_port}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\
         \r\n"
    );
    let read_status = async {
        let mut sock = TcpStream::connect(("127.0.0.1", control_port))
            .await
            .expect("connect to control port");
        sock.write_all(request.as_bytes())
            .await
            .expect("write handshake");
        // Read until we have at least the status line.
        let mut buf = Vec::new();
        let mut chunk = vec![0u8; 256];
        while !buf.ends_with(b"\r\n") {
            let n = sock.read(&mut chunk).await.expect("read handshake");
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        String::from_utf8_lossy(&buf).to_string()
    };
    let head = tokio::time::timeout(ATTEMPT_TIMEOUT, read_status)
        .await
        .expect("relay never answered the control handshake");
    // "HTTP/1.1 401 Unauthorized\r\n..."
    head.split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0)
}

/// A "local endpoint" that replies once and then immediately closes, used to
/// check that a close on one end reaches the other end of the tunnel.
async fn echo_once_then_close(listener: TcpListener) {
    while let Ok((mut sock, _peer)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            if let Ok(n) = sock.read(&mut buf).await {
                if n > 0 {
                    let _ = sock.write_all(&buf[..n]).await;
                }
            }
            let _ = sock.shutdown().await;
        });
    }
}

/// A close on the local side must reach the remote peer: the public client sees
/// EOF instead of hanging forever on a half-closed stream.
#[tokio::test]
async fn half_close_is_propagated_to_the_remote_peer() {
    tokio::time::timeout(TEST_TIMEOUT, async {
        let once = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind echo-once endpoint");
        let once_port = once.local_addr().expect("echo-once addr").port();
        tokio::spawn(echo_once_then_close(once));

        let control = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind control port");
        let public = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind public port");
        let control_port = control.local_addr().expect("control addr").port();
        let public_port = public.local_addr().expect("public addr").port();
        tokio::spawn(async move {
            let _ = server::run_relay_with(control, public, TOKEN).await;
        });
        let relay_client = tokio::spawn(async move {
            let url = format!("ws://127.0.0.1:{control_port}/tunnel");
            let _ = client::start_tunnel_client(&url, TOKEN, once_port).await;
        });

        // Wait for the tunnel to come up.
        assert_eq!(
            roundtrip_when_ready(public_port, b"bye").await,
            b"bye".to_vec()
        );

        // Then check that a fresh connection gets its reply and then EOF.
        let mut sock = TcpStream::connect(("127.0.0.1", public_port))
            .await
            .expect("connect to public port");
        sock.write_all(b"bye").await.expect("write payload");
        let mut got = vec![0u8; 3];
        sock.read_exact(&mut got).await.expect("read reply");
        assert_eq!(got, b"bye".to_vec());

        let eof = tokio::time::timeout(Duration::from_secs(5), async {
            let mut sink = Vec::new();
            sock.read_to_end(&mut sink).await
        })
        .await
        .expect("EOF never arrived -- half-close was not propagated");
        assert!(eof.is_ok(), "reading to EOF failed: {eof:?}");
        relay_client.abort();
    })
    .await
    .expect("half-close test timed out");
}
