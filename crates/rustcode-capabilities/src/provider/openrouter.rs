//! OpenRouter free-model quick connect: OAuth PKCE key exchange + free model discovery.
//!
//! Independent of any managed OAuth flow (that uses state-based polling; this
//! uses PKCE S256 with a loopback callback). Lives in the `provider` feature so
//! a provider-only embedder can opt in via the `openrouter` Cargo feature.

use anyhow::{Context as _, Result};
use base64::Engine as _;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const OPENROUTER_AUTH_URL: &str = "https://openrouter.ai/auth";
pub const OPENROUTER_KEYS_URL: &str = "https://openrouter.ai/api/v1/auth/keys";
pub const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
}

/// base64url(sha256(verifier)), no padding -- PKCE S256.
pub fn code_challenge_s256(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

/// Generate 96 random bytes -> base64url(no-pad) to get a 128-char verifier
/// (unreserved charset), plus its S256 challenge.
pub fn generate_pkce() -> PkcePair {
    let mut bytes = [0u8; 96];
    getrandom::getrandom(&mut bytes).expect("getrandom fill failed");
    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let challenge = code_challenge_s256(&verifier);
    PkcePair {
        verifier,
        challenge,
    }
}

/// Build the OpenRouter authorization URL. `callback_url=None` means headless
/// (no callback, code shown on screen).
pub fn build_auth_url(callback_url: Option<&str>, code_challenge: &str) -> String {
    let mut url = format!(
        "{OPENROUTER_AUTH_URL}?code_challenge={}&code_challenge_method=S256",
        urlencoding_component(code_challenge),
    );
    if let Some(cb) = callback_url {
        url.push_str(&format!("&callback_url={}", urlencoding_component(cb)));
    }
    url
}

/// Minimal RFC3986 component encoding (everything outside unreserved -> %XX).
/// Avoids pulling a dedicated encoding dependency.
fn urlencoding_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct FreeModel {
    pub id: String,
    pub name: Option<String>,
    pub context_length: u64,
}

pub fn parse_key_response(body: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct KeyResp {
        key: Option<String>,
    }
    let parsed: KeyResp = serde_json::from_str(body).context("parse /auth/keys response")?;
    parsed
        .key
        .filter(|k| !k.trim().is_empty())
        .context("/auth/keys response missing `key`")
}

pub fn select_top_free_models(models_json: &str, limit: usize) -> Result<Vec<FreeModel>> {
    #[derive(Deserialize)]
    struct ModelsResp {
        data: Vec<RawModel>,
    }
    #[derive(Deserialize)]
    struct RawModel {
        id: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        context_length: u64,
        #[serde(default)]
        pricing: Option<Pricing>,
        /// OpenRouter's per-model capability list (e.g. `"tools"`, `"reasoning"`).
        /// A model WITHOUT `"tools"` 404s a coding agent's first (tool-carrying)
        /// request with "No endpoints found that support tool use".
        #[serde(default)]
        supported_parameters: Vec<String>,
        #[serde(default)]
        architecture: Option<Architecture>,
    }
    #[derive(Deserialize)]
    struct Architecture {
        /// What the model EMITS. A chat model outputs `"text"`; a media model
        /// (e.g. Google Lyria music-gen) outputs `"audio"`/`"image"` -- free and
        /// large-context, but unusable for chat/tool use, so it must be excluded.
        #[serde(default)]
        output_modalities: Vec<String>,
    }
    #[derive(Deserialize)]
    struct Pricing {
        #[serde(default)]
        prompt: String,
        #[serde(default)]
        completion: String,
    }

    fn is_zero(p: &str) -> bool {
        p.trim().parse::<f64>().map(|v| v == 0.0).unwrap_or(false)
    }

    let resp: ModelsResp = serde_json::from_str(models_json).context("parse /models response")?;
    let mut free: Vec<FreeModel> = resp
        .data
        .into_iter()
        .filter(|m| {
            m.id.ends_with(":free")
                || m.pricing
                    .as_ref()
                    .map(|p| is_zero(&p.prompt) && is_zero(&p.completion))
                    .unwrap_or(false)
        })
        // Capability gate -- only surface models a coding agent can actually use,
        // so the auto-added set never includes a model that 404/403s on the first
        // message.
        .filter(|m| {
            m.supported_parameters.iter().any(|p| p == "tools")
                && match &m.architecture {
                    Some(a) if !a.output_modalities.is_empty() => {
                        a.output_modalities.iter().any(|x| x == "text")
                    }
                    _ => true,
                }
        })
        .map(|m| FreeModel {
            id: m.id,
            name: m.name,
            context_length: m.context_length,
        })
        .collect();
    // context descending; ties broken by id for deterministic test ordering.
    free.sort_by(|a, b| {
        b.context_length
            .cmp(&a.context_length)
            .then_with(|| a.id.cmp(&b.id))
    });
    free.truncate(limit);
    Ok(free)
}

pub fn parse_code_from_request_line(line: &str) -> Option<String> {
    let target = line.split_whitespace().nth(1)?; // "/callback?code=..."
    let query = target.split_once('?')?.1;
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("code=") {
            if !v.is_empty() {
                return Some(percent_decode(v));
            }
        }
    }
    None
}

/// Minimal percent-decode: `%XX` -> byte, `+` left as-is (query values).
/// Invalid `%XX` is preserved verbatim.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub struct LocalCallback {
    listener: TcpListener,
}

pub fn start_local_callback() -> Result<LocalCallback> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).context("bind loopback callback port")?;
    listener
        .set_nonblocking(true)
        .context("set callback listener non-blocking")?;
    Ok(LocalCallback { listener })
}

impl LocalCallback {
    pub fn port(&self) -> u16 {
        self.listener.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    /// Block until the browser hits the callback and returns `code`.
    /// `Ok(None)` = cancelled or timed out (not an error).
    ///
    /// Connections without a code (browser preflight, favicon, etc.) receive a
    /// response but are skipped; the loop continues waiting for the real
    /// callback. A slow connection that doesn't send a request within 2s is
    /// treated as invalid and likewise skipped.
    pub fn wait_for_code(self, timeout: Duration, cancel: &AtomicBool) -> Result<Option<String>> {
        let deadline = Instant::now() + timeout;
        let success_body = "<html><body>OpenRouter connected. You can close this page and return to the terminal.</body></html>";
        let waiting_body = "<html><body>Waiting for authorization...</body></html>";
        loop {
            if cancel.load(Ordering::Relaxed) || Instant::now() >= deadline {
                return Ok(None);
            }
            match self.listener.accept() {
                Ok((mut stream, _)) => {
                    // Critical: the listener is non-blocking, and on macOS/BSD the
                    // accepted stream inherits O_NONBLOCK. set_read_timeout is
                    // ineffective and read returns WouldBlock before the HTTP
                    // request arrives, causing the real callback to be discarded.
                    // Explicitly switch back to blocking so the read timeout below
                    // works.
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));

                    let mut buf = [0u8; 2048];
                    let n = match stream.read(&mut buf) {
                        Ok(0) | Err(_) => {
                            continue;
                        }
                        Ok(n) => n,
                    };

                    let text = String::from_utf8_lossy(&buf[..n]);
                    let first_line = text.lines().next().unwrap_or("");
                    let code = parse_code_from_request_line(first_line);

                    if let Some(ref c) = code {
                        let _ = stream.write_all(
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                success_body.as_bytes().len(),
                                success_body,
                            )
                            .as_bytes(),
                        );
                        return Ok(Some(c.clone()));
                    }

                    let _ = stream.write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            waiting_body.as_bytes().len(),
                            waiting_body,
                        )
                        .as_bytes(),
                    );
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(anyhow::Error::new(e).context("accept callback connection")),
            }
        }
    }
}

/// Build a blocking reqwest client that shares the process-wide proxy policy
/// (RUSTCODE_PROXY_MODE / no_proxy) and a consistent user-agent. OpenRouter is
/// a standard TLS 1.3 endpoint, so no TLS 1.2 cap (force_tls12 = false).
fn blocking_client() -> Result<reqwest::blocking::Client> {
    crate::proxy::apply_blocking_proxy_policy(reqwest::blocking::Client::builder())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .user_agent(crate::egress::client::DEFAULT_USER_AGENT)
        .build()
        .context("failed to build OpenRouter HTTP client")
}

/// POST /api/v1/auth/keys {code, code_verifier, code_challenge_method:"S256"} -> key.
pub fn exchange_code_for_key(code: &str, verifier: &str) -> Result<String> {
    let client = blocking_client()?;
    let resp = client
        .post(OPENROUTER_KEYS_URL)
        .json(&serde_json::json!({
            "code": code,
            "code_verifier": verifier,
            "code_challenge_method": "S256",
        }))
        .send()
        .context("call OpenRouter /auth/keys")?;
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("OpenRouter /auth/keys returned HTTP {}", status.as_u16());
    }
    parse_key_response(&body)
}

/// GET /api/v1/models (Bearer) -> filter free, sort by context desc, take limit.
pub fn fetch_top_free_models(api_key: &str, limit: usize) -> Result<Vec<FreeModel>> {
    let client = blocking_client()?;
    let resp = client
        .get(OPENROUTER_MODELS_URL)
        .bearer_auth(api_key)
        .send()
        .context("call OpenRouter /models")?;
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("OpenRouter /models returned HTTP {}", status.as_u16());
    }
    select_top_free_models(&body, limit)
}

/// Best-effort cross-platform browser open. Errors are non-fatal: the caller
/// also surfaces the auth URL as text so the user can copy-paste it.
pub fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .context("Failed to open browser")?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("Failed to open browser")?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("cmd")
            .raw_arg(format!("/C start \"\" \"{}\"", url))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .context("Failed to open browser")?;
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = url;
        anyhow::bail!("unsupported platform for browser open");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s256_challenge_matches_rfc7636_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            code_challenge_s256(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn generated_pair_roundtrips() {
        let p = generate_pkce();
        assert!(
            (43..=128).contains(&p.verifier.len()),
            "len={}",
            p.verifier.len()
        );
        assert!(p
            .verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-._~".contains(c)));
        assert_eq!(p.challenge, code_challenge_s256(&p.verifier));
        assert!(!p.challenge.contains(['=', '+', '/']));
    }

    #[test]
    fn auth_url_has_callback_and_challenge() {
        let url = build_auth_url(Some("http://localhost:51234/callback"), "CHAL");
        assert!(url.starts_with("https://openrouter.ai/auth?"));
        assert!(url.contains("code_challenge=CHAL"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("callback_url=http%3A%2F%2Flocalhost%3A51234%2Fcallback"));
    }

    #[test]
    fn auth_url_headless_omits_callback() {
        let url = build_auth_url(None, "CHAL");
        assert!(!url.contains("callback_url="));
        assert!(url.contains("code_challenge=CHAL"));
    }

    #[test]
    fn parse_key_extracts_field() {
        assert_eq!(
            parse_key_response(r#"{"key":"sk-or-v1-abc"}"#).unwrap(),
            "sk-or-v1-abc"
        );
    }

    #[test]
    fn parse_key_errors_on_missing() {
        assert!(parse_key_response(r#"{"error":"bad"}"#).is_err());
    }

    const MODELS_FIXTURE: &str = r#"{
      "data": [
        {"id":"vendor/big:free","name":"Big Free","context_length":128000,
         "pricing":{"prompt":"0","completion":"0"},
         "supported_parameters":["tools","temperature"],
         "architecture":{"output_modalities":["text"]}},
        {"id":"vendor/paid","name":"Paid","context_length":200000,
         "pricing":{"prompt":"0.001","completion":"0.002"},
         "supported_parameters":["tools"]},
        {"id":"vendor/small:free","name":"Small Free","context_length":8000,
         "pricing":{"prompt":"0","completion":"0"},
         "supported_parameters":["tools"],
         "architecture":{"output_modalities":["text"]}},
        {"id":"vendor/zero-priced","name":"Zero Priced","context_length":32000,
         "pricing":{"prompt":"0","completion":"0"},
         "supported_parameters":["tools"],
         "architecture":{"output_modalities":["text"]}},
        {"id":"vendor/nopricing","context_length":16000,
         "supported_parameters":["tools"]},
        {"id":"vendor/music:free","name":"Music Gen","context_length":256000,
         "pricing":{"prompt":"0","completion":"0"},
         "supported_parameters":["temperature"],
         "architecture":{"output_modalities":["audio"]}},
        {"id":"vendor/notools:free","name":"No Tools","context_length":300000,
         "pricing":{"prompt":"0","completion":"0"},
         "architecture":{"output_modalities":["text"]}}
      ]
    }"#;

    #[test]
    fn top_free_filters_paid_and_sorts_by_context_desc() {
        let got = select_top_free_models(MODELS_FIXTURE, 5).unwrap();
        let ids: Vec<&str> = got.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["vendor/big:free", "vendor/zero-priced", "vendor/small:free"]
        );
    }

    #[test]
    fn top_free_excludes_non_tool_and_non_text_models() {
        let got = select_top_free_models(MODELS_FIXTURE, 5).unwrap();
        let ids: Vec<&str> = got.iter().map(|m| m.id.as_str()).collect();
        assert!(
            !ids.contains(&"vendor/music:free"),
            "audio-gen model excluded"
        );
        assert!(
            !ids.contains(&"vendor/notools:free"),
            "non-tool model excluded"
        );
    }

    #[test]
    fn top_free_respects_limit() {
        let got = select_top_free_models(MODELS_FIXTURE, 2).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].id, "vendor/big:free");
    }

    #[test]
    fn top_free_empty_when_none_free() {
        let json = r#"{"data":[{"id":"x/paid","context_length":9,"pricing":{"prompt":"0.01","completion":"0"}}]}"#;
        assert!(select_top_free_models(json, 5).unwrap().is_empty());
    }

    #[test]
    fn code_parsed_from_request_line() {
        let line = "GET /callback?code=abc123&scope=x HTTP/1.1";
        assert_eq!(
            parse_code_from_request_line(line).as_deref(),
            Some("abc123")
        );
    }

    #[test]
    fn code_percent_decoded() {
        assert_eq!(
            parse_code_from_request_line("GET /callback?code=abc%2Bxyz HTTP/1.1").as_deref(),
            Some("abc+xyz")
        );
        assert_eq!(
            parse_code_from_request_line("GET /callback?code=ab%2 HTTP/1.1").as_deref(),
            Some("ab%2")
        );
    }

    #[test]
    fn code_none_when_absent() {
        assert_eq!(parse_code_from_request_line("GET /callback HTTP/1.1"), None);
    }

    #[test]
    fn local_callback_receives_code_over_loopback() {
        use std::io::Write;
        use std::sync::atomic::AtomicBool;
        let cb = start_local_callback().unwrap();
        let port = cb.port();
        let h = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            s.write_all(b"GET /callback?code=deadbeef HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
        });
        let cancel = AtomicBool::new(false);
        let code = cb
            .wait_for_code(std::time::Duration::from_secs(3), &cancel)
            .unwrap();
        h.join().unwrap();
        assert_eq!(code.as_deref(), Some("deadbeef"));
    }

    #[test]
    fn delayed_request_after_handshake_still_returns_code() {
        use std::io::Write;
        use std::sync::atomic::AtomicBool;
        let cb = start_local_callback().unwrap();
        let port = cb.port();
        let h = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(250));
            s.write_all(b"GET /callback?code=cafef00d HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
        });
        let cancel = AtomicBool::new(false);
        let code = cb
            .wait_for_code(std::time::Duration::from_secs(4), &cancel)
            .unwrap();
        h.join().unwrap();
        assert_eq!(code.as_deref(), Some("cafef00d"));
    }

    #[test]
    fn no_code_request_is_skipped_real_code_returned() {
        use std::io::Write;
        use std::sync::atomic::AtomicBool;

        let cb = start_local_callback().unwrap();
        let port = cb.port();

        let h = std::thread::spawn(move || {
            let mut s1 = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            s1.write_all(b"GET /favicon.ico HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
            drop(s1);

            std::thread::sleep(std::time::Duration::from_millis(100));

            let mut s2 = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            s2.write_all(b"GET /callback?code=realcode42 HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
        });

        let cancel = AtomicBool::new(false);
        let code = cb
            .wait_for_code(std::time::Duration::from_secs(5), &cancel)
            .unwrap();
        h.join().unwrap();
        assert_eq!(code.as_deref(), Some("realcode42"));
    }
}
