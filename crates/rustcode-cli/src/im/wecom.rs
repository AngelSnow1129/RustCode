//! WeCom (企业微信) smart-bot adapter -- long connection over WebSocket.
//!
//! WeCom's smart-bot long connection is the inverse of a webhook: the client
//! **dials out** to `wss://openws.work.weixin.qq.com`, sends a one-time
//! `aibot_subscribe` frame with the bot's `bot_id` + `secret`, and the platform
//! then pushes `aibot_msg_callback` frames down that same socket. This needs **no
//! public endpoint and no reverse tunnel** -- the same zero-infrastructure story
//! DingTalk's Stream mode has. (Our config doc previously claimed WeCom was a
//! webhook; that was wrong and has been corrected.)
//!
//! Wire flow (frame shapes taken from the official smart-bot long-connection
//! documentation, not guessed):
//!
//! ```text
//!   1. WS connect  wss://openws.work.weixin.qq.com
//!   2. client -> server: {cmd:"aibot_subscribe", headers:{req_id},
//!                         body:{bot_id, secret}}
//!   3. server -> client: {cmd:"aibot_subscribe", headers:{req_id},
//!                         body:{code, msg}}   (code != 0 => auth failure)
//!   4. server -> client: {cmd:"aibot_msg_callback", headers:{req_id},
//!                         body:{msgid, aibotid, chatid, chattype,
//!                               from:{userid}, msgtype, text:{content}}}
//!   5. client -> server: {cmd:"aibot_respond_msg", headers:{req_id},
//!                         body:{chatid, chattype, msgid, msgtype:"text",
//!                               text:{content}}, stream:{id, finish:true}}
//!   6. heartbeat: {cmd:"ping", headers:{req_id}}  about every 30 seconds
//! ```
//!
//! Three properties that differ from DingTalk and drive the structure below:
//!
//! - **Replies go back over the *same* socket, correlated by the inbound frame's
//!   `req_id`** -- not over an independent HTTP call. So `send_text` cannot be
//!   the no-socket operation it is for DingTalk; it shares the live connection.
//!   The reader and every reply therefore both touch the socket, which is why the
//!   stream is split into independent read/write halves behind separate mutexes.
//! - **At most one long connection may be alive per bot at a time.** Opening a
//!   second one *kicks the first* (the old connection receives a
//!   `disconnected_event` and is closed by the server). Single-instance is
//!   mandatory; multi-instance would lose messages rather than duplicate them.
//! - **`aibot_enter_chat` must be answered with `aibot_respond_welcome_msg`
//!   within 5 seconds**, or the session is rejected. The reader handles that
//!   inline before routing the message.
//!
//! `respond_msg`'s body in step 5 mirrors the inbound `aibot_msg_callback` body
//! (same `chatid`/`chattype`/`msgid`/`text` shape). That body schema is taken
//! from the official docs' inbound example and is the most likely thing to need
//! a tweak against a live tenant; the `cmd`/`headers` envelope is verified.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use super::{ImAdapter, ImError, ImMessage};
use tokio_tungstenite::tungstenite::Message;

/// Default WeCom smart-bot long-connection gateway.
pub const DEFAULT_GATEWAY: &str = "wss://openws.work.weixin.qq.com";

/// How long to wait for the WebSocket dial to complete.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Heartbeat cadence. WeCom expects roughly a 30s ping; the reader sends one
/// whenever it has been idle this long.
const PING_INTERVAL: Duration = Duration::from_secs(30);

type WsStream = tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>;
type Ws = tokio_tungstenite::WebSocketStream<WsStream>;

/// Per-chat context captured from the inbound callback, replayed into the reply
/// so `aibot_respond_msg` carries the right `chattype` + `msgid`. Without it the
/// reply would have to guess `chattype`, which is wrong for group chats. The map
/// is tiny for a chatbot; it is intentionally not evicted (a note, not a leak in
/// practice -- one entry per conversation).
#[derive(Debug, Clone)]
struct ChatContext {
    chattype: String,
    msgid: String,
}

/// WeCom adapter over a smart-bot long connection.
///
/// The socket is split into read/write halves behind two mutexes so a reply task
/// (writer) never blocks on the reader's lock, and the reader's periodic ping
/// (writer) never blocks a turn's reply. `next_message` is called by the dispatch
/// reader task alone; `send_text` is safe to call from many tasks because it only
/// locks the write half.
pub struct WeComAdapter {
    bot_id: String,
    secret: String,
    gateway: String,
    /// Serializes connection establishment so two tasks cannot both dial.
    connect_lock: Mutex<()>,
    read: Mutex<Option<futures::stream::SplitStream<Ws>>>,
    write: Mutex<Option<futures::stream::SplitSink<Ws, Message>>>,
    chat_meta: Mutex<HashMap<String, ChatContext>>,
}

impl WeComAdapter {
    /// Build from resolved credentials (already env-expanded by the caller).
    pub fn new(bot_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            bot_id: bot_id.into(),
            secret: secret.into(),
            gateway: DEFAULT_GATEWAY.to_string(),
            connect_lock: Mutex::new(()),
            read: Mutex::new(None),
            write: Mutex::new(None),
            chat_meta: Mutex::new(HashMap::new()),
        }
    }

    /// Point at a non-default gateway (private/sovereign deployment).
    pub fn with_gateway(mut self, gateway: impl Into<String>) -> Self {
        let mut g = gateway.into();
        while g.ends_with('/') {
            g.pop();
        }
        self.gateway = g;
        self
    }

    /// Establish the long connection if it is not already up.
    ///
    /// Safe to call repeatedly and from any task: an existing connection is
    /// returned as-is. Does not hold the read/write locks across the dial, so a
    /// concurrent `send_text` cannot deadlock against a reader that holds the
    /// read half during `next_message`.
    async fn ensure_connected(&self) -> Result<(), ImError> {
        let _lock = self.connect_lock.lock().await;
        {
            let r = self.read.lock().await;
            let w = self.write.lock().await;
            if r.is_some() && w.is_some() {
                return Ok(());
            }
        }
        let (ws, _resp) = tokio::time::timeout(
            CONNECT_TIMEOUT,
            tokio_tungstenite::connect_async(&self.gateway),
        )
        .await
        .map_err(|_| ImError::Transport("timed out connecting to the WeCom gateway".into()))?
        .map_err(|e| ImError::Transport(format!("WeCom WebSocket connect failed: {e}")))?;

        let req_id = new_req_id();
        let subscribe = build_subscribe(&self.bot_id, &self.secret, &req_id);
        let (mut writer, reader) = ws.split();
        writer
            .send(Message::text(subscribe))
            .await
            .map_err(|e| ImError::Transport(format!("WeCom subscribe failed: {e}")))?;
        *self.read.lock().await = Some(reader);
        *self.write.lock().await = Some(writer);
        Ok(())
    }

    /// Send one already-serialized frame on the write half. Assumes the
    /// connection is up; `send_text` calls `ensure_connected` first, while the
    /// reader's ping/welcome paths only run while already connected.
    async fn send_raw(&self, frame: &str) -> Result<(), ImError> {
        let mut guard = self.write.lock().await;
        let sink = guard
            .as_mut()
            .ok_or_else(|| ImError::Transport("WeCom connection not established".into()))?;
        match sink.send(Message::text(frame.to_string())).await {
            Ok(()) => Ok(()),
            // A write failure means the socket is dead; clear it so the next
            // call reconnects instead of retrying a broken handle.
            Err(e) => {
                *guard = None;
                Err(ImError::Transport(format!("WeCom send failed: {e}")))
            }
        }
    }

    async fn send_ping(&self) -> Result<(), ImError> {
        let req_id = new_req_id();
        self.send_raw(&build_ping(&req_id)).await
    }

    async fn send_welcome(
        &self,
        req_id: &str,
        chatid: &str,
        chattype: &str,
    ) -> Result<(), ImError> {
        self.send_raw(&build_welcome(req_id, chatid, chattype))
            .await
    }
}

/// Monotonic-per-process id for `headers.req_id`. WeCom only requires it be
/// unique per request; a nanosecond timestamp plus a counter is plenty.
fn new_req_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{now:x}-{seq:x}")
}

/// One parsed inbound frame.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Inbound {
    /// `aibot_subscribe` response. `code != 0` is an auth failure.
    SubscribeAck {
        req_id: String,
        code: i64,
        msg: String,
    },
    /// `aibot_msg_callback` -- the message we route to the agent.
    Message {
        req_id: String,
        msgid: String,
        chatid: String,
        chattype: String,
        userid: String,
        text: String,
    },
    /// `aibot_event_callback` (e.g. `enter_chat`).
    Event {
        req_id: String,
        eventtype: String,
        chatid: String,
        chattype: String,
    },
    /// Anything else the client does not act on.
    Unknown { cmd: String },
}

/// Parse a server frame. A malformed frame is a `Protocol` error; callers ack
/// nothing for WeCom (there is no per-frame ack -- the reply *is* the ack) and
/// simply skip it.
fn parse_frame(raw: &str) -> Result<Inbound, ImError> {
    let v: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| ImError::Protocol(format!("WeCom frame is not JSON: {e}")))?;
    let cmd = v
        .get("cmd")
        .and_then(|c| c.as_str())
        .unwrap_or_default()
        .to_string();
    let req_id = v
        .get("headers")
        .and_then(|h| h.get("req_id"))
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string();

    let body = v.get("body");
    match cmd.as_str() {
        "aibot_subscribe" => {
            let code = body
                .and_then(|b| b.get("code"))
                .and_then(|x| x.as_i64())
                .unwrap_or(0);
            let msg = body
                .and_then(|b| b.get("msg"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Inbound::SubscribeAck { req_id, code, msg })
        }
        "aibot_msg_callback" => {
            let str_of = |key: &str| -> String {
                body.and_then(|b| b.get(key))
                    .and_then(|x| x.as_str())
                    .unwrap_or_default()
                    .to_string()
            };
            let msgid = str_of("msgid");
            let chatid = str_of("chatid");
            let chattype = str_of("chattype");
            let userid = body
                .and_then(|b| b.get("from"))
                .and_then(|f| f.get("userid"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let text = body
                .and_then(|b| b.get("text"))
                .and_then(|t| t.get("content"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            if chatid.is_empty() {
                return Err(ImError::Protocol(
                    "WeCom message has no `chatid`; cannot map it to a project".into(),
                ));
            }
            Ok(Inbound::Message {
                req_id,
                msgid,
                chatid,
                chattype,
                userid,
                text,
            })
        }
        "aibot_event_callback" => {
            let eventtype = body
                .and_then(|b| b.get("eventtype"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let chatid = body
                .and_then(|b| b.get("chatid"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let chattype = body
                .and_then(|b| b.get("chattype"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Inbound::Event {
                req_id,
                eventtype,
                chatid,
                chattype,
            })
        }
        other => Ok(Inbound::Unknown {
            cmd: other.to_string(),
        }),
    }
}

/// Build the `aibot_subscribe` frame sent immediately after the dial.
pub fn build_subscribe(bot_id: &str, secret: &str, req_id: &str) -> String {
    serde_json::json!({
        "cmd": "aibot_subscribe",
        "headers": { "req_id": req_id },
        "body": { "bot_id": bot_id, "secret": secret },
    })
    .to_string()
}

/// Build the `aibot_respond_msg` frame. Body mirrors the inbound callback shape
/// (same `chatid`/`chattype`/`msgid`/`text`); `stream.finish` closes the (single
/// chunk) response.
pub fn build_respond_msg(
    req_id: &str,
    chatid: &str,
    chattype: &str,
    msgid: &str,
    text: &str,
) -> String {
    serde_json::json!({
        "cmd": "aibot_respond_msg",
        "headers": { "req_id": req_id },
        "body": {
            "chatid": chatid,
            "chattype": chattype,
            "msgid": msgid,
            "msgtype": "text",
            "text": { "content": text },
        },
        "stream": { "id": msgid, "finish": true },
    })
    .to_string()
}

/// Build the `aibot_respond_welcome_msg` frame for `enter_chat`.
pub fn build_welcome(req_id: &str, chatid: &str, chattype: &str) -> String {
    serde_json::json!({
        "cmd": "aibot_respond_welcome_msg",
        "headers": { "req_id": req_id },
        "body": { "chatid": chatid, "chattype": chattype },
    })
    .to_string()
}

/// Build the heartbeat frame.
pub fn build_ping(req_id: &str) -> String {
    serde_json::json!({
        "cmd": "ping",
        "headers": { "req_id": req_id },
    })
    .to_string()
}

#[async_trait]
impl ImAdapter for WeComAdapter {
    fn platform(&self) -> &'static str {
        "wecom"
    }

    async fn next_message(&self) -> Result<Option<ImMessage>, ImError> {
        self.ensure_connected().await?;
        loop {
            // Read one frame, releasing the read lock after each iteration so a
            // concurrent `send_text` (which also briefly takes the read lock via
            // `ensure_connected`) cannot deadlock against the reader.
            let raw = {
                let mut guard = self.read.lock().await;
                let stream = match guard.as_mut() {
                    Some(s) => s,
                    None => return Ok(None),
                };
                let ping = tokio::time::sleep(PING_INTERVAL);
                tokio::pin!(ping);
                match tokio::select! {
                    f = stream.next() => f,
                    _ = &mut ping => {
                        drop(guard);
                        let _ = self.send_ping().await;
                        continue;
                    }
                } {
                    None => {
                        // Stream ended -> clean close; caller reconnects.
                        *guard = None;
                        return Ok(None);
                    }
                    Some(Err(e)) => {
                        *guard = None;
                        return Err(ImError::Transport(format!(
                            "WeCom WebSocket read failed: {e}"
                        )));
                    }
                    Some(Ok(msg)) => match msg {
                        Message::Text(t) => t.to_string(),
                        Message::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
                        // Keepalive / control frames: ignore.
                        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
                        Message::Close(_) => {
                            *guard = None;
                            return Ok(None);
                        }
                    },
                }
            };

            let inbound = match parse_frame(&raw) {
                Ok(inb) => inb,
                // No per-frame ack in WeCom; skip malformed frames.
                Err(_) => continue,
            };

            match inbound {
                Inbound::Message {
                    req_id,
                    msgid,
                    chatid,
                    chattype,
                    userid,
                    text,
                } => {
                    let text = text.trim().to_string();
                    if text.is_empty() {
                        // Non-text (image/file/voice) or empty: nothing to act on.
                        continue;
                    }
                    self.chat_meta.lock().await.insert(
                        chatid.clone(),
                        ChatContext {
                            chattype: chattype.clone(),
                            msgid: msgid.clone(),
                        },
                    );
                    return Ok(Some(ImMessage {
                        platform: "wecom".into(),
                        chat_id: chatid,
                        sender_id: userid,
                        text,
                        message_id: msgid,
                        // The inbound frame's req_id -- `send_text` echoes it so
                        // the reply is correlated to this exact callback.
                        reply_token: Some(req_id),
                    }));
                }
                Inbound::Event {
                    req_id,
                    eventtype,
                    chatid,
                    chattype,
                } => {
                    if eventtype == "enter_chat" {
                        let _ = self.send_welcome(&req_id, &chatid, &chattype).await;
                    }
                    continue;
                }
                Inbound::SubscribeAck { code, msg, .. } => {
                    if code != 0 {
                        return Err(ImError::Auth {
                            platform: "wecom",
                            detail: format!("WeCom subscribe rejected (code {code}): {msg}"),
                        });
                    }
                    continue;
                }
                Inbound::Unknown { .. } => continue,
            }
        }
    }

    async fn send_text(
        &self,
        chat_id: &str,
        reply_token: Option<&str>,
        text: &str,
    ) -> Result<(), ImError> {
        // The reply must carry the req_id of the inbound frame it answers; without
        // it WeCom cannot correlate the response, so a missing token is a hard
        // failure (silently dropping the reply would leave the user with no answer).
        let req_id = reply_token.filter(|t| !t.is_empty()).ok_or_else(|| {
            ImError::Protocol("cannot reply: the WeCom message carried no req_id".into())
        })?;
        let ctx = self
            .chat_meta
            .lock()
            .await
            .get(chat_id)
            .cloned()
            .unwrap_or(ChatContext {
                chattype: "single".into(),
                msgid: String::new(),
            });
        let frame = build_respond_msg(req_id, chat_id, &ctx.chattype, &ctx.msgid, text);
        self.send_raw(&frame).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_message_callback_frame() {
        let raw = r#"{"cmd":"aibot_msg_callback","headers":{"req_id":"r-1"},
            "body":{"msgid":"m-1","aibotid":"a","chatid":"c-1","chattype":"group",
                    "from":{"userid":"u-1"},"msgtype":"text","text":{"content":" hello"}}}"#;
        match parse_frame(raw).unwrap() {
            Inbound::Message {
                req_id,
                msgid,
                chatid,
                chattype,
                userid,
                ..
            } => {
                assert_eq!(req_id, "r-1");
                assert_eq!(msgid, "m-1");
                assert_eq!(chatid, "c-1");
                assert_eq!(chattype, "group");
                assert_eq!(userid, "u-1");
            }
            other => panic!("expected a message, got {other:?}"),
        }
    }

    #[test]
    fn parses_an_event_frame_and_a_subscribe_ack() {
        let ev = parse_frame(
            r#"{"cmd":"aibot_event_callback","headers":{"req_id":"r-2"},
                "body":{"eventtype":"enter_chat","chatid":"c-2","chattype":"single"}}"#,
        )
        .unwrap();
        assert!(matches!(ev, Inbound::Event { eventtype, .. } if eventtype == "enter_chat"));

        let ok = parse_frame(
            r#"{"cmd":"aibot_subscribe","headers":{"req_id":"r-3"},"body":{"code":0,"msg":"ok"}}"#,
        )
        .unwrap();
        assert!(matches!(ok, Inbound::SubscribeAck { code: 0, .. }));

        let bad = parse_frame(
            r#"{"cmd":"aibot_subscribe","headers":{"req_id":"r-4"},"body":{"code":40001,"msg":"bad secret"}}"#,
        )
        .unwrap();
        assert!(matches!(bad, Inbound::SubscribeAck { code: 40001, .. }));
    }

    #[test]
    fn non_json_frame_is_a_protocol_error() {
        assert!(matches!(parse_frame("not json"), Err(ImError::Protocol(_))));
    }

    #[test]
    fn message_without_chatid_is_rejected() {
        let raw = r#"{"cmd":"aibot_msg_callback","headers":{"req_id":"r"},"body":{"msgid":"m"}}"#;
        assert!(matches!(parse_frame(raw), Err(ImError::Protocol(_))));
    }

    #[test]
    fn build_subscribe_carries_bot_id_and_secret() {
        let frame = build_subscribe("BOT", "SEC", "r-9");
        let v: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(v["cmd"], "aibot_subscribe");
        assert_eq!(v["headers"]["req_id"], "r-9");
        assert_eq!(v["body"]["bot_id"], "BOT");
        assert_eq!(v["body"]["secret"], "SEC");
    }

    #[test]
    fn build_respond_msg_carries_req_id_and_text_and_closes_stream() {
        let frame = build_respond_msg("r-1", "c-1", "group", "m-1", "hi");
        let v: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(v["cmd"], "aibot_respond_msg");
        assert_eq!(v["headers"]["req_id"], "r-1");
        assert_eq!(v["body"]["chatid"], "c-1");
        assert_eq!(v["body"]["chattype"], "group");
        assert_eq!(v["body"]["msgid"], "m-1");
        assert_eq!(v["body"]["msgtype"], "text");
        assert_eq!(v["body"]["text"]["content"], "hi");
        assert_eq!(v["stream"]["finish"], true);
    }

    #[test]
    fn build_welcome_and_ping_are_well_formed() {
        let w = build_welcome("r-2", "c-2", "single");
        let v: serde_json::Value = serde_json::from_str(&w).unwrap();
        assert_eq!(v["cmd"], "aibot_respond_welcome_msg");
        assert_eq!(v["body"]["chatid"], "c-2");

        let p = build_ping("r-3");
        let v: serde_json::Value = serde_json::from_str(&p).unwrap();
        assert_eq!(v["cmd"], "ping");
        assert_eq!(v["headers"]["req_id"], "r-3");
    }

    #[test]
    fn adapter_reports_its_platform() {
        let adapter = WeComAdapter::new("bot", "secret");
        assert_eq!(adapter.platform(), "wecom");
    }

    #[test]
    fn gateway_trailing_slash_is_normalized() {
        let adapter = WeComAdapter::new("bot", "secret").with_gateway("wss://example.com//");
        assert_eq!(adapter.gateway, "wss://example.com");
    }

    #[tokio::test]
    async fn reply_without_a_req_id_fails_loudly() {
        let adapter = WeComAdapter::new("bot", "secret");
        let err = adapter.send_text("c-1", None, "hi").await.unwrap_err();
        assert!(matches!(err, ImError::Protocol(_)), "got {err:?}");
        let err = adapter.send_text("c-1", Some(""), "hi").await.unwrap_err();
        assert!(matches!(err, ImError::Protocol(_)));
    }

    #[test]
    fn req_id_is_unique_per_call() {
        let a = new_req_id();
        let b = new_req_id();
        assert_ne!(a, b);
    }
}
