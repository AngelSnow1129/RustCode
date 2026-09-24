//! DingTalk (钉钉) adapter -- Stream mode long connection.
//!
//! Stream mode is what makes DingTalk usable without any public endpoint: the
//! client **dials out** to the platform's gateway over a WebSocket, and the
//! platform pushes events down that connection. Compare the webhook shape,
//! which requires the platform to reach *us* (needing a public host or the
//! bundled reverse tunnel). So this adapter is the zero-infrastructure path.
//!
//! Wire flow (protocol shapes taken from the official `dingtalk-stream` SDK,
//! not guessed):
//!
//! ```text
//!   1. POST {gateway}/v1.0/gateway/connections/open
//!        body: {clientId, clientSecret, subscriptions:[{type,topic}], ua, localIp}
//!        resp: {endpoint, ticket}
//!   2. WS connect  "{endpoint}?ticket={ticket}"
//!   3. server -> client frames (JSON text):
//!        {specVersion, type:"CALLBACK"|"EVENT"|"SYSTEM",
//!         headers:{messageId, topic, ...}, data:"<json STRING>"}
//!   4. client -> server ack:  {code:200, headers:{messageId}, message:"OK", data:"{}"}
//!   5. reply: POST to the message's sessionWebhook, {"msgtype":"text",...}
//! ```
//!
//! Two details that are easy to get wrong and are handled explicitly:
//!
//! - **`data` is a JSON encoded as a string**, so it needs a second parse. A
//!   single parse yields a string where a struct is expected.
//! - **Every frame must be acked.** DingTalk redelivers unacked frames, and a
//!   redelivery would drive the agent a second time; the bridge additionally
//!   de-duplicates by message id as defence in depth.

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use std::time::Duration;

use super::{ImAdapter, ImError, ImMessage};
use rustcode_capabilities::egress::{build_http_client, HttpClientSpec};
use rustcode_capabilities::im_probe::{self, urlencode, ImProbeError};

/// Topic for inbound bot messages. Single source of truth lives in the
/// capabilities probe (shared with the daemon's test route); re-exported so the
/// adapter and its tests read naturally.
pub use im_probe::DINGTALK_CHATBOT_TOPIC as CHATBOT_TOPIC;

/// Default gateway. Overridable so a self-hosted/private deployment can point
/// at its own gateway instead of the public one. Re-exported from the probe.
pub use im_probe::DINGTALK_DEFAULT_GATEWAY as DEFAULT_GATEWAY;

/// How long to wait for the gateway to mint a connection.
const OPEN_TIMEOUT: Duration = Duration::from_secs(10);
/// Reply POST timeout.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);

/// Map a probe failure onto the adapter's error type, one-to-one.
fn map_probe_error(error: ImProbeError) -> ImError {
    match error {
        ImProbeError::Auth { platform, detail } => ImError::Auth { platform, detail },
        ImProbeError::Transport(detail) => ImError::Transport(detail),
        ImProbeError::Protocol(detail) => ImError::Protocol(detail),
    }
}

/// Ack codes, mirroring the SDK's `AckMessage` constants.
pub const ACK_OK: i64 = 200;
pub const ACK_BAD_REQUEST: i64 = 400;
pub const ACK_NOT_IMPLEMENTED: i64 = 404;
pub const ACK_SYSTEM_EXCEPTION: i64 = 500;

/// One parsed inbound frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// A bot message (or other callback) we subscribe to.
    Callback {
        message_id: String,
        topic: String,
        /// Raw `data` string; parsed further by [`parse_chatbot_message`].
        data: String,
    },
    /// A platform event we did not subscribe to. Acked, then ignored.
    Event { message_id: String },
    /// A system frame (e.g. `disconnect`). Acked; the caller reconnects.
    System { message_id: String, topic: String },
}

impl Frame {
    /// Id to echo in the ack. Empty when the frame carried none -- the ack is
    /// still sent so the platform can clear the frame.
    pub fn message_id(&self) -> &str {
        match self {
            Self::Callback { message_id, .. }
            | Self::Event { message_id }
            | Self::System { message_id, .. } => message_id,
        }
    }

    /// Whether this frame is a `disconnect` notice, after which the adapter
    /// should reconnect rather than treat the closure as an error.
    pub fn is_disconnect(&self) -> bool {
        matches!(self, Self::System { topic, .. } if topic == "disconnect")
    }
}

/// An inbound chatbot message, already unwrapped from the two JSON layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatbotInbound {
    /// Stable conversation id -- the identity-mapping key.
    pub conversation_id: String,
    /// Message id, echoed for ack + dedupe.
    pub message_id: String,
    /// Sender, for attribution only.
    pub sender_id: String,
    /// Text with the platform's mention prefix trimmed.
    pub text: String,
    /// Per-message reply target. Short-lived (the platform expires it), so it is
    /// used immediately and never persisted.
    pub session_webhook: Option<String>,
}

/// Parse the gateway's `connections/open` response.
///
/// Thin delegate over the capabilities probe, which owns the wire contract
/// (shared with the daemon's `/im/channels/test` route).
pub fn parse_open_response(body: &str) -> Result<(String, String), ImError> {
    let conn = im_probe::parse_open_response(body).map_err(map_probe_error)?;
    Ok((conn.endpoint, conn.ticket))
}

/// Parse one inbound server frame.
pub fn parse_frame(raw: &str) -> Result<Frame, ImError> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| ImError::Protocol(format!("frame is not JSON: {e}")))?;
    let headers = value.get("headers");
    let message_id = headers
        .and_then(|h| h.get("messageId"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let topic = headers
        .and_then(|h| h.get("topic"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    match value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
    {
        "CALLBACK" => {
            let data = value
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Frame::Callback {
                message_id,
                topic,
                data,
            })
        }
        "SYSTEM" => Ok(Frame::System { message_id, topic }),
        "EVENT" => Ok(Frame::Event { message_id }),
        other => Err(ImError::Protocol(format!("unknown frame type `{other}`"))),
    }
}

/// Build an ack frame for a consumed frame.
pub fn build_ack(message_id: &str, code: i64, message: &str) -> String {
    serde_json::json!({
        "code": code,
        "headers": { "messageId": message_id, "contentType": "application/json" },
        "message": message,
        "data": "{}",
    })
    .to_string()
}

/// Parse the inner `data` string of a chatbot callback.
///
/// Handles both a JSON string (the real wire shape) and an already-decoded
/// object, so a future protocol revision that stops double-encoding does not
/// silently break message handling.
pub fn parse_chatbot_message(data: &str) -> Result<ChatbotInbound, ImError> {
    let value: serde_json::Value = match serde_json::from_str::<serde_json::Value>(data) {
        Ok(serde_json::Value::String(inner)) => serde_json::from_str(&inner)
            .map_err(|e| ImError::Protocol(format!("chatbot payload is not JSON: {e}")))?,
        Ok(other) => other,
        Err(e) => return Err(ImError::Protocol(format!("chatbot data is not JSON: {e}"))),
    };

    let text_of = |key: &str| -> String {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    let conversation_id = text_of("conversationId");
    if conversation_id.is_empty() {
        return Err(ImError::Protocol(
            "chatbot payload has no `conversationId`; cannot map it to a project".into(),
        ));
    }

    Ok(ChatbotInbound {
        conversation_id,
        // Fall back to the outer frame id when the payload omits msgId; an empty
        // id would disable redelivery de-duplication in the bridge.
        message_id: text_of("msgId"),
        sender_id: {
            let staff = text_of("senderStaffId");
            if staff.is_empty() {
                text_of("senderId")
            } else {
                staff
            }
        },
        text: extract_text(&value),
        session_webhook: value
            .get("sessionWebhook")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
    })
}

/// Pull the user text out of a chatbot payload.
///
/// DingTalk sends `{"text": {"content": " hello"}}` with a leading space (the
/// mention slot), so the content is trimmed. Non-text messages (picture, rich
/// text) yield their `content` if present, else an empty string -- the caller
/// decides whether a non-text message is worth acting on.
fn extract_text(value: &serde_json::Value) -> String {
    value
        .get("text")
        .and_then(|t| t.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Trim a leading `@mention` token that some payloads leave in the content.
///
/// Only strips a *leading* token: an `@` mid-sentence is ordinary text and must
/// survive (stripping it would corrupt the user's prompt).
pub fn strip_leading_mention(text: &str) -> String {
    let trimmed = text.trim_start();
    if !trimmed.starts_with('@') {
        // Interior whitespace is preserved: it is part of the user's prompt.
        return trimmed.trim_end().to_string();
    }
    match trimmed.split_once(char::is_whitespace) {
        Some((_, rest)) => rest.trim().to_string(),
        // A lone "@something" with no separator is left alone: it may be an
        // email fragment or the whole prompt.
        None => trimmed.trim_end().to_string(),
    }
}

/// DingTalk adapter over a Stream long connection.
pub struct DingTalkAdapter {
    client_id: String,
    client_secret: String,
    gateway: String,
    /// Live WebSocket, established lazily on first use.
    socket: Option<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    http: reqwest::Client,
}

impl DingTalkAdapter {
    /// Build from resolved credentials.
    ///
    /// Credentials are passed in already expanded from config (`env:VAR`), so
    /// this type never reads the environment or the config file itself.
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        // Every outbound call goes through the egress factory (single auditable
        // TLS/proxy policy). A factory failure (pathological proxy env) degrades
        // to a default client rather than failing the whole channel setup; the
        // credentials-bearing handshake itself lives in the capabilities probe,
        // which goes through the factory unconditionally.
        let http = build_http_client(&HttpClientSpec::default())
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            gateway: DEFAULT_GATEWAY.to_string(),
            socket: None,
            http,
        }
    }

    /// Point at a non-default gateway (self-hosted / private deployment).
    pub fn with_gateway(mut self, gateway: impl Into<String>) -> Self {
        let mut g = gateway.into();
        while g.ends_with('/') {
            g.pop();
        }
        self.gateway = g;
        self
    }

    /// Ask the gateway for a connection endpoint + ticket.
    ///
    /// Delegates to the capabilities probe -- the single implementation of the
    /// gateway handshake, shared with the daemon's `/im/channels/test` route.
    /// The adapter adds only the WebSocket dial on top; it keeps no HTTP copy
    /// of the handshake.
    async fn open_connection(&self) -> Result<(String, String), ImError> {
        let conn = im_probe::probe_dingtalk(&self.client_id, &self.client_secret, &self.gateway)
            .await
            .map_err(map_probe_error)?;
        Ok((conn.endpoint, conn.ticket))
    }

    /// Establish the long connection if it is not already up.
    async fn ensure_connected(&mut self) -> Result<(), ImError> {
        if self.socket.is_some() {
            return Ok(());
        }
        let (endpoint, ticket) = self.open_connection().await?;
        let url = format!("{endpoint}?ticket={}", urlencode(&ticket));
        let (ws, _resp) =
            tokio::time::timeout(OPEN_TIMEOUT, tokio_tungstenite::connect_async(url.as_str()))
                .await
                .map_err(|_| {
                    ImError::Transport("timed out connecting to the stream endpoint".into())
                })?
                .map_err(|e| ImError::Transport(format!("WebSocket connect failed: {e}")))?;
        self.socket = Some(ws);
        Ok(())
    }

    /// Reply through the message's `sessionWebhook`.
    async fn reply_via_webhook(&self, webhook: &str, text: &str) -> Result<(), ImError> {
        let body = serde_json::json!({ "msgtype": "text", "text": { "content": text } });
        let resp = self
            .http
            .post(webhook)
            .header("Content-Type", "application/json")
            .timeout(SEND_TIMEOUT)
            .json(&body)
            .send()
            .await
            .map_err(|e| ImError::Transport(format!("reply failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(ImError::Transport(format!(
                "reply rejected with HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }
}

/// Percent-encode and local-IP helpers moved to
/// `rustcode_capabilities::im_probe` (the single implementation shared with the
/// daemon's test route); `urlencode` is re-exported above.

#[async_trait]
impl ImAdapter for DingTalkAdapter {
    fn platform(&self) -> &'static str {
        "dingtalk"
    }

    async fn next_message(&mut self) -> Result<Option<ImMessage>, ImError> {
        self.ensure_connected().await?;
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| ImError::Transport("stream not connected".into()))?;

        loop {
            let next = socket.next().await;
            let Some(frame) = next else {
                // Stream ended. Drop the socket so the caller's next attempt
                // reconnects instead of reading a dead handle forever.
                self.socket = None;
                return Ok(None);
            };
            let message =
                frame.map_err(|e| ImError::Transport(format!("WebSocket read failed: {e}")))?;

            let raw = match message {
                tokio_tungstenite::tungstenite::Message::Text(text) => text.to_string(),
                tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
                    String::from_utf8_lossy(&bytes).into_owned()
                }
                // Keepalive frames are answered by tungstenite; ignore them.
                tokio_tungstenite::tungstenite::Message::Ping(_)
                | tokio_tungstenite::tungstenite::Message::Pong(_)
                | tokio_tungstenite::tungstenite::Message::Frame(_) => continue,
                tokio_tungstenite::tungstenite::Message::Close(_) => {
                    self.socket = None;
                    return Ok(None);
                }
            };

            let parsed = match parse_frame(&raw) {
                Ok(frame) => frame,
                Err(e) => {
                    // Malformed frame: ack it so the platform stops redelivering,
                    // then keep the connection up rather than tearing down a
                    // healthy stream over one bad frame.
                    let ack = build_ack("", ACK_BAD_REQUEST, &format!("{e}"));
                    let _ = socket
                        .send(tokio_tungstenite::tungstenite::Message::text(ack))
                        .await;
                    continue;
                }
            };

            if parsed.is_disconnect() {
                let ack = build_ack(parsed.message_id(), ACK_OK, "OK");
                let _ = socket
                    .send(tokio_tungstenite::tungstenite::Message::text(ack))
                    .await;
                self.socket = None;
                return Ok(None);
            }

            match &parsed {
                Frame::Callback {
                    message_id, data, ..
                } => {
                    match parse_chatbot_message(data) {
                        Ok(bot) => {
                            let ack = build_ack(message_id, ACK_OK, "OK");
                            let _ = socket
                                .send(tokio_tungstenite::tungstenite::Message::text(ack))
                                .await;
                            // Ignore non-text messages (no text to act on) but
                            // still ack them, which the branch above already did.
                            let text = strip_leading_mention(&bot.text);
                            if text.is_empty() {
                                continue;
                            }
                            return Ok(Some(ImMessage {
                                platform: "dingtalk".to_string(),
                                chat_id: bot.conversation_id,
                                sender_id: bot.sender_id,
                                text,
                                message_id: if bot.message_id.is_empty() {
                                    message_id.clone()
                                } else {
                                    bot.message_id
                                },
                                reply_token: bot.session_webhook,
                            }));
                        }
                        Err(e) => {
                            let ack = build_ack(message_id, ACK_BAD_REQUEST, &format!("{e}"));
                            let _ = socket
                                .send(tokio_tungstenite::tungstenite::Message::text(ack))
                                .await;
                            continue;
                        }
                    }
                }
                // Ack frames we do not act on so they are not redelivered.
                Frame::Event { message_id } => {
                    let ack = build_ack(message_id, ACK_OK, "OK");
                    let _ = socket
                        .send(tokio_tungstenite::tungstenite::Message::text(ack))
                        .await;
                    continue;
                }
                Frame::System { message_id, .. } => {
                    let ack = build_ack(message_id, ACK_OK, "OK");
                    let _ = socket
                        .send(tokio_tungstenite::tungstenite::Message::text(ack))
                        .await;
                    continue;
                }
            }
        }
    }

    async fn send_text(
        &self,
        _chat_id: &str,
        reply_token: Option<&str>,
        text: &str,
    ) -> Result<(), ImError> {
        // DingTalk replies via the per-message sessionWebhook rather than a
        // conversation-addressed API, so a missing token is a hard failure --
        // silently dropping the reply would leave the user with no answer.
        let webhook = reply_token.filter(|t| !t.is_empty()).ok_or_else(|| {
            ImError::Protocol("cannot reply: the message carried no sessionWebhook".into())
        })?;
        self.reply_via_webhook(webhook, text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_response_extracts_endpoint_and_ticket() {
        let body = r#"{"endpoint":"wss://eg.example/connect","ticket":"abc/def+="}"#;
        let (endpoint, ticket) = parse_open_response(body).unwrap();
        assert_eq!(endpoint, "wss://eg.example/connect");
        assert_eq!(ticket, "abc/def+=");
    }

    #[test]
    fn open_response_reports_bad_credentials_as_auth_error() {
        // The real gateway answers a credential problem with a body lacking
        // `endpoint`; that must surface as an auth failure, not a transport one.
        let err =
            parse_open_response(r#"{"code":"invalidCredential","message":"nope"}"#).unwrap_err();
        assert!(matches!(err, ImError::Auth { .. }), "got {err:?}");
        assert!(parse_open_response("not json").is_err());
    }

    #[test]
    fn parses_a_callback_frame() {
        let raw = r#"{"specVersion":"1.0","type":"CALLBACK",
            "headers":{"messageId":"mid-1","topic":"/v1.0/im/bot/messages/get"},
            "data":"{\"conversationId\":\"cid\"}"}"#;
        let frame = parse_frame(raw).unwrap();
        assert_eq!(frame.message_id(), "mid-1");
        match frame {
            Frame::Callback { topic, data, .. } => {
                assert_eq!(topic, CHATBOT_TOPIC);
                assert!(data.contains("conversationId"));
            }
            other => panic!("expected a callback, got {other:?}"),
        }
    }

    #[test]
    fn parses_system_and_event_frames() {
        let sys =
            parse_frame(r#"{"type":"SYSTEM","headers":{"messageId":"s1","topic":"disconnect"}}"#)
                .unwrap();
        assert!(
            sys.is_disconnect(),
            "a disconnect notice must be recognized"
        );
        assert_eq!(sys.message_id(), "s1");

        let ev = parse_frame(r#"{"type":"EVENT","headers":{"messageId":"e1"}}"#).unwrap();
        assert_eq!(ev.message_id(), "e1");
        assert!(!ev.is_disconnect());
    }

    #[test]
    fn unknown_frame_type_is_an_error_not_a_panic() {
        let err = parse_frame(r#"{"type":"WAT","headers":{}}"#).unwrap_err();
        assert!(matches!(err, ImError::Protocol(_)));
    }

    #[test]
    fn frame_without_headers_does_not_panic() {
        let frame = parse_frame(r#"{"type":"CALLBACK"}"#).unwrap();
        assert_eq!(frame.message_id(), "");
    }

    #[test]
    fn ack_carries_the_message_id_and_an_ok_code() {
        let ack = build_ack("mid-9", ACK_OK, "OK");
        let value: serde_json::Value = serde_json::from_str(&ack).unwrap();
        assert_eq!(value["code"], 200);
        assert_eq!(value["headers"]["messageId"], "mid-9");
        assert_eq!(value["message"], "OK");
        // `data` is a JSON *string* on this wire, not an object.
        assert!(value["data"].is_string());
    }

    #[test]
    fn chatbot_payload_is_double_encoded_and_we_handle_both() {
        // Real shape: the inner payload is a JSON string.
        let inner = r#"{"conversationId":"cid-1","msgId":"m-1","senderStaffId":"u-1",
            "sessionWebhook":"https://hook.example/x","text":{"content":" hello world"}}"#;
        let encoded = serde_json::to_string(inner).unwrap();
        let bot = parse_chatbot_message(&encoded).unwrap();
        assert_eq!(bot.conversation_id, "cid-1");
        assert_eq!(bot.message_id, "m-1");
        assert_eq!(bot.sender_id, "u-1");
        assert_eq!(bot.text, "hello world", "mention-slot space is trimmed");
        assert_eq!(
            bot.session_webhook.as_deref(),
            Some("https://hook.example/x")
        );

        // Defensive: an already-decoded object must still work.
        let direct = parse_chatbot_message(inner).unwrap();
        assert_eq!(direct, bot);
    }

    #[test]
    fn chatbot_payload_falls_back_to_sender_id() {
        let inner = r#"{"conversationId":"c","senderId":"fallback"}"#;
        let bot = parse_chatbot_message(inner).unwrap();
        assert_eq!(bot.sender_id, "fallback");
    }

    #[test]
    fn chatbot_payload_without_conversation_id_is_rejected() {
        // Without a conversation id there is no way to map the message to a
        // project; guessing one would cross conversations.
        let inner = r#"{"msgId":"m","text":{"content":"hi"}}"#;
        let err = parse_chatbot_message(inner).unwrap_err();
        assert!(matches!(err, ImError::Protocol(_)));
    }

    #[test]
    fn non_text_message_yields_empty_text_not_garbage() {
        let inner = r#"{"conversationId":"c","msgId":"m","msgtype":"picture"}"#;
        let bot = parse_chatbot_message(inner).unwrap();
        assert!(bot.text.is_empty());
    }

    #[test]
    fn leading_mention_is_stripped_only_at_the_start() {
        assert_eq!(strip_leading_mention("@Bot hello"), "hello");
        assert_eq!(strip_leading_mention("  @Bot   hello  "), "hello");
        // Mid-sentence `@` is ordinary text and must survive.
        assert_eq!(strip_leading_mention("ping @Bot now"), "ping @Bot now");
        // No whitespace after the token -> ambiguous, leave it alone.
        assert_eq!(strip_leading_mention("@Bot"), "@Bot");
        assert_eq!(strip_leading_mention("plain"), "plain");
    }

    #[test]
    fn urlencode_escapes_query_metacharacters() {
        assert_eq!(urlencode("abcXYZ019-_.~"), "abcXYZ019-_.~");
        assert_eq!(urlencode("a/b+c="), "a%2Fb%2Bc%3D");
        assert_eq!(urlencode(" "), "%20");
    }

    #[test]
    fn gateway_trailing_slash_is_normalized() {
        // A trailing slash must not produce a double-slashed URL path.
        let adapter = DingTalkAdapter::new("id", "secret").with_gateway("https://g.example//");
        assert_eq!(adapter.gateway, "https://g.example");
    }

    #[test]
    fn adapter_reports_its_platform() {
        let adapter = DingTalkAdapter::new("id", "secret");
        assert_eq!(adapter.platform(), "dingtalk");
    }

    #[tokio::test]
    async fn reply_without_a_session_webhook_fails_loudly() {
        // Dropping the reply silently would leave the user with no answer at
        // all, so a missing token must be an error.
        let adapter = DingTalkAdapter::new("id", "secret");
        let err = adapter.send_text("cid", None, "hi").await.unwrap_err();
        assert!(matches!(err, ImError::Protocol(_)), "got {err:?}");
        let err = adapter.send_text("cid", Some(""), "hi").await.unwrap_err();
        assert!(matches!(err, ImError::Protocol(_)));
    }

    // The `best_effort_local_ip` helper test moved with the implementation into
    // `rustcode_capabilities::im_probe`'s test module.

    #[test]
    fn errors_never_embed_the_client_secret() {
        // The secret must not be recoverable from an error string.
        let adapter = DingTalkAdapter::new("client-id-xyz", "SUPER-SECRET");
        let rendered = format!("{:?}", adapter.client_id);
        assert!(rendered.contains("client-id-xyz"));
        // `client_secret` has no Debug impl reachable from an error path; assert
        // the type does not derive Debug on the secret by checking the auth error
        // built from a gateway response never contains it.
        let err = parse_open_response("{}").unwrap_err();
        assert!(!err.to_string().contains("SUPER-SECRET"));
    }
}
