use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};

use rustcode_auth as auth;
use rustcode_config::i18n::{t, Msg};

use crate::{
    client_mode::ClientMode,
    coded_json_error, json_error,
    login_state::{
        ApplyPoll, BeginPoll, LoginRecord, LoginStateSnapshot, PollCompletion,
        LOGIN_RETRY_AFTER_MS, LOGIN_TTL,
    },
    AppState, LoginSessionsStore,
};

const MAX_LOGIN_RECORDS: usize = 64;

pub(crate) enum LoginPollStep {
    Pending,
    Authorized { user: auth::UserInfo },
    Expired,
    Cancelled,
    Failed { code: String, message: String },
    Retryable { code: String, message: String },
}

pub(crate) struct LoginPollResult {
    pub step: LoginPollStep,
}

pub(crate) struct LoginPollError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
}

// ============================================================================
// Response DTOs
// ============================================================================

#[derive(Debug, Serialize)]
struct AuthStatusResponse {
    logged_in: bool,
    /// Credentials exist on disk but the token can't be made valid (expired
    /// and the refresh_token was refused / absent). The sidebar used to show
    /// "logged in" here while chat returned "登录已过期" -- this flag lets the
    /// frontend surface a distinct "session expired, re-login" state instead.
    expired: bool,
    /// Whether this build ships a managed sign-in service at all (non-empty
    /// platform server). Neutral / open builds report false: the WebUI then
    /// hides the account / sign-in surfaces entirely, since `/auth/login/start`
    /// can only fail -- the same predicate as the TUI's command/wizard gating
    /// (`rustcode_auth::managed_login_available`).
    managed_available: bool,
    auth_path: String,
    user: Option<auth::UserInfo>,
    token: Option<TokenInfo>,
}

/// Classify the reported auth state from what disk holds (`present`) and
/// whether that stored token is actually usable (`token_usable` -- valid now
/// or successfully refreshed). `expired` means present-but-dead: the exact
/// mismatch that made the sidebar disagree with chat.
fn classify_auth_status(present: bool, token_usable: bool) -> (bool, bool) {
    let logged_in = present;
    let expired = present && !token_usable;
    (logged_in, expired)
}

#[derive(Debug, Serialize)]
struct TokenInfo {
    token_type: String,
    expires_in: Option<i64>,
    created_at: i64,
    has_refresh_token: bool,
}

#[derive(Debug, Serialize)]
struct LoginStartResponse {
    login_id: String,
    url: String,
    expires_in_seconds: u64,
    daemon_instance_id: String,
}

#[derive(Debug, Serialize)]
struct LoginPollResponse {
    status: String,
    user: Option<auth::UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginStartRequest {
    #[serde(default = "default_true")]
    open_browser: bool,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /auth/status - Returns whether the user is signed in.
pub(crate) async fn auth_status() -> impl IntoResponse {
    let auth_path = auth::auth_file_path();
    let auth_path_str = auth_path.to_string_lossy().to_string();

    match auth::get_stored_auth() {
        Some(info) => {
            let has_refresh = info.refresh_token.is_some();
            // Presence of auth.toml is not the same as a usable session: an
            // expired token whose refresh_token is dead/absent still parses
            // fine but every chat turn 401s. Probe real usability the same
            // way chat does -- `get_valid_token` returns the stored token when
            // valid, refreshes (and saves) near expiry, and errors when it
            // can't be made valid. It's disk-only in the common case and
            // network-bounded (5s connect / 10s total) only when a refresh is
            // actually attempted; run it off the async runtime.
            let token_usable = tokio::task::spawn_blocking(|| auth::get_valid_token().is_ok())
                .await
                .unwrap_or(false);
            let (logged_in, expired) = classify_auth_status(true, token_usable);
            Json(AuthStatusResponse {
                logged_in,
                expired,
                managed_available: auth::managed_login_available(),
                auth_path: auth_path_str,
                user: Some(info.user),
                token: Some(TokenInfo {
                    token_type: info.token_type,
                    expires_in: info.expires_in,
                    created_at: info.created_at,
                    has_refresh_token: has_refresh,
                }),
            })
            .into_response()
        }
        None => Json(AuthStatusResponse {
            logged_in: false,
            expired: false,
            managed_available: auth::managed_login_available(),
            auth_path: auth_path_str,
            user: None,
            token: None,
        })
        .into_response(),
    }
}

/// Neutral-build response for managed-login entry points: 501 with the
/// `managed_login_unavailable` code. Clients key on this code to steer
/// users to bring-your-own-key provider setup instead of a dead OAuth
/// flow. Deliberately non-retryable.
pub(crate) fn managed_login_unavailable_response() -> axum::response::Response {
    coded_json_error(
        StatusCode::NOT_IMPLEMENTED,
        "managed_login_unavailable",
        t(Msg::DaemonApiManagedUnavailable).into_owned(),
        false,
    )
    .into_response()
}

/// POST /auth/login/start - Starts OAuth login and returns URL + login_id.
pub(crate) async fn auth_login_start(
    State(state): State<AppState>,
    Json(req): Json<LoginStartRequest>,
) -> impl IntoResponse {
    let _start_guard = state.login_start_lock.lock().await;
    cleanup_login_sessions(&state.login_sessions).await;
    if state.login_sessions.read().await.len() >= MAX_LOGIN_RECORDS {
        return coded_json_error(
            StatusCode::TOO_MANY_REQUESTS,
            "login_session_limit",
            t(Msg::DaemonApiLoginSessionLimit).into_owned(),
            true,
        )
        .into_response();
    }

    let open_browser = req.open_browser;

    // Neutral build: there is no managed sign-in service to talk to. Fail
    // fast with an actionable code instead of letting `start_login()` error
    // out into a generic 500 -- clients key on `managed_login_unavailable`
    // to steer users to bring-your-own-key provider setup.
    if !auth::managed_login_available() {
        return managed_login_unavailable_response();
    }

    let start_result =
        tokio::task::spawn_blocking(move || -> anyhow::Result<(auth::LoginSession, String)> {
            let session = auth::start_login()?;
            let url = session.url().to_string();
            if open_browser {
                session.open_browser_best_effort();
            }
            Ok((session, url))
        })
        .await;

    let (session, url) = match start_result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            tracing::warn!(error = ?e, "failed to start OAuth login");
            return coded_json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "login_start_failed",
                t(Msg::DaemonApiLoginStartFailed).into_owned(),
                true,
            )
            .into_response();
        }
        Err(e) => {
            tracing::error!(error = ?e, "OAuth login start task failed");
            return coded_json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "login_task_failed",
                t(Msg::DaemonApiLoginTaskFailed).into_owned(),
                true,
            )
            .into_response();
        }
    };

    let login_id = uuid::Uuid::new_v4().to_string();
    let entry = Arc::new(tokio::sync::Mutex::new(LoginRecord::new(
        session,
        Instant::now(),
    )));
    let mut sessions = state.login_sessions.write().await;
    sessions.insert(login_id.clone(), entry);
    drop(sessions);

    Json(LoginStartResponse {
        login_id,
        url,
        expires_in_seconds: LOGIN_TTL.as_secs(),
        daemon_instance_id: state.daemon_instance_id.to_string(),
    })
    .into_response()
}

/// POST /auth/login/:login_id/poll - Polls one OAuth login session.
pub(crate) async fn auth_login_poll(
    State(state): State<AppState>,
    // `ClientMode` stays extracted so the route contract is unchanged (a
    // missing extension still fails the same way); a neutral build has no
    // post-login gateway sync to dispatch it to.
    axum::Extension(_client_mode): axum::Extension<ClientMode>,
    Path(login_id): Path<String>,
) -> impl IntoResponse {
    match poll_login_session(&state, &login_id).await {
        Ok(result) => login_poll_response(result),
        Err(error) => coded_json_error(error.status, error.code, error.message, error.retryable)
            .into_response(),
    }
}

/// DELETE /auth/login/:login_id - Cancels and removes an in-flight login session.
pub(crate) async fn auth_login_cancel(
    State(state): State<AppState>,
    Path(login_id): Path<String>,
) -> impl IntoResponse {
    if uuid::Uuid::parse_str(&login_id).is_err() {
        return coded_json_error(
            StatusCode::BAD_REQUEST,
            "invalid_login_id",
            t(Msg::DaemonApiInvalidLoginId).into_owned(),
            false,
        )
        .into_response();
    }

    cleanup_login_sessions(&state.login_sessions).await;
    let record = state.login_sessions.read().await.get(&login_id).cloned();
    let cancelled = if let Some(record) = record {
        matches!(
            record.lock().await.cancel(Instant::now()),
            LoginStateSnapshot::Cancelled
        )
    } else {
        false
    };

    Json(serde_json::json!({"success": true, "cancelled": cancelled})).into_response()
}

/// POST /auth/logout - Logs out (removes stored auth).
pub(crate) async fn auth_logout(
    State(state): State<AppState>,
    axum::Extension(_client_mode): axum::Extension<ClientMode>,
) -> impl IntoResponse {
    let _state_inner = state.clone();
    match auth::logout() {
        Ok(()) => {
            // Return auth status after logout
            let auth_path = auth::auth_file_path();
            Json(AuthStatusResponse {
                logged_in: false,
                expired: false,
                managed_available: auth::managed_login_available(),
                auth_path: auth_path.to_string_lossy().to_string(),
                user: None,
                token: None,
            })
            .into_response()
        }
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            t(Msg::DaemonApiLogoutFailed {
                error: &format!("{e:#}"),
            })
            .into_owned(),
        )
        .into_response(),
    }
}

pub(crate) async fn poll_login_session(
    state: &AppState,
    login_id: &str,
) -> Result<LoginPollResult, LoginPollError> {
    if uuid::Uuid::parse_str(login_id).is_err() {
        return Err(LoginPollError {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_login_id",
            message: t(Msg::DaemonApiInvalidLoginId).into_owned(),
            retryable: false,
        });
    }

    cleanup_login_sessions(&state.login_sessions).await;
    let record = state
        .login_sessions
        .read()
        .await
        .get(login_id)
        .cloned()
        .ok_or_else(|| LoginPollError {
            status: StatusCode::GONE,
            code: "login_session_gone",
            message: t(Msg::DaemonApiLoginSessionGone).into_owned(),
            retryable: false,
        })?;

    let work = {
        let mut record = record.lock().await;
        record.begin_poll(Instant::now())
    };

    let step = match work {
        BeginPoll::Current(snapshot) => step_from_snapshot(snapshot),
        BeginPoll::Poll {
            generation,
            session,
        } => {
            let completion = tokio::task::spawn_blocking(move || match session.poll_once() {
                Ok(auth::PollOutcome::Pending) => PollCompletion::Pending(session),
                Err(error) => {
                    tracing::warn!(error = ?error, "OAuth login poll failed");
                    PollCompletion::Retryable {
                        session,
                        code: "login_poll_unavailable".to_string(),
                        message: t(Msg::DaemonApiLoginPollUnavailable).into_owned(),
                    }
                }
                Ok(auth::PollOutcome::Authorized) => match session.finish() {
                    Err(error) => {
                        tracing::warn!(error = ?error, "OAuth token exchange failed");
                        PollCompletion::Failed {
                            code: "login_exchange_failed".to_string(),
                            message: t(Msg::DaemonApiLoginExchangeFailed).into_owned(),
                        }
                    }
                    Ok(auth_info) => PollCompletion::AuthorizationReady(auth_info),
                },
            })
            .await
            .unwrap_or_else(|error| {
                tracing::error!(error = ?error, "OAuth poll task failed");
                PollCompletion::Failed {
                    code: "login_task_failed".to_string(),
                    message: t(Msg::DaemonApiLoginTaskFailed).into_owned(),
                }
            });

            apply_poll_completion(&record, generation, completion).await
        }
        BeginPoll::Persist { generation, auth } => {
            let completion = tokio::task::spawn_blocking(move || match auth::save_auth(&auth) {
                Ok(()) => PollCompletion::Authorized(auth.user),
                Err(error) => {
                    tracing::warn!(error = ?error, "failed to persist OAuth credentials");
                    PollCompletion::PersistFailed {
                        auth,
                        code: "auth_persist_failed".to_string(),
                        message: t(Msg::DaemonApiAuthPersistFailed).into_owned(),
                    }
                }
            })
            .await
            .unwrap_or_else(|error| {
                tracing::error!(error = ?error, "OAuth credential persistence task failed");
                PollCompletion::Failed {
                    code: "login_task_failed".to_string(),
                    message: t(Msg::DaemonApiLoginTaskFailed).into_owned(),
                }
            });

            apply_poll_completion(&record, generation, completion).await
        }
    };

    Ok(LoginPollResult { step })
}

async fn apply_poll_completion(
    record: &Arc<tokio::sync::Mutex<LoginRecord>>,
    generation: u64,
    completion: PollCompletion<auth::LoginSession>,
) -> LoginPollStep {
    match record
        .lock()
        .await
        .apply_poll(generation, completion, Instant::now())
    {
        ApplyPoll::NewlyAuthorized(user) => LoginPollStep::Authorized { user },
        ApplyPoll::Retryable { code, message } => LoginPollStep::Retryable { code, message },
        ApplyPoll::Current(snapshot) | ApplyPoll::Ignored(snapshot) => step_from_snapshot(snapshot),
    }
}

fn step_from_snapshot(snapshot: LoginStateSnapshot) -> LoginPollStep {
    match snapshot {
        LoginStateSnapshot::Pending => LoginPollStep::Pending,
        LoginStateSnapshot::Authorized(user) => LoginPollStep::Authorized { user },
        LoginStateSnapshot::Expired => LoginPollStep::Expired,
        LoginStateSnapshot::Cancelled => LoginPollStep::Cancelled,
        LoginStateSnapshot::Failed { code, message } => LoginPollStep::Failed { code, message },
    }
}

fn login_poll_response(result: LoginPollResult) -> axum::response::Response {
    let response = |status: &str,
                    user: Option<auth::UserInfo>,
                    code: Option<String>,
                    message: Option<String>,
                    retry_after_ms: Option<u64>| {
        Json(LoginPollResponse {
            status: status.to_string(),
            user,
            code,
            message,
            retry_after_ms,
        })
        .into_response()
    };

    match result.step {
        LoginPollStep::Pending => response("pending", None, None, None, Some(LOGIN_RETRY_AFTER_MS)),
        LoginPollStep::Authorized { user } => response("authorized", Some(user), None, None, None),
        LoginPollStep::Retryable { code, message } => {
            coded_json_error(StatusCode::SERVICE_UNAVAILABLE, code, message, true).into_response()
        }
        LoginPollStep::Expired => terminal_login_response(
            StatusCode::GONE,
            "login_session_expired",
            t(Msg::DaemonApiLoginExpired),
        ),
        LoginPollStep::Cancelled => terminal_login_response(
            StatusCode::GONE,
            "login_session_cancelled",
            t(Msg::DaemonApiLoginCancelled),
        ),
        LoginPollStep::Failed { code, message } => {
            terminal_login_response(StatusCode::INTERNAL_SERVER_ERROR, code, message)
        }
    }
}

fn terminal_login_response(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
) -> axum::response::Response {
    coded_json_error(status, code, message, false).into_response()
}

pub(crate) async fn cleanup_login_sessions(login_sessions: &LoginSessionsStore) {
    let now = Instant::now();
    let records: Vec<_> = login_sessions
        .read()
        .await
        .iter()
        .map(|(id, record)| (id.clone(), record.clone()))
        .collect();
    let mut removable = Vec::new();
    for (id, record) in records {
        let mut guard = record.lock().await;
        guard.expire_if_due(now);
        if guard.removable_at(now) {
            removable.push((id, record.clone()));
        }
    }

    if removable.is_empty() {
        return;
    }
    let mut sessions = login_sessions.write().await;
    for (id, record) in removable {
        if sessions
            .get(&id)
            .is_some_and(|current| Arc::ptr_eq(current, &record))
        {
            sessions.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The sidebar login indicator reads `/auth/status`. Before the fix it
    // reported `logged_in` purely from the presence of `auth.toml`, so an
    // expired/rejected token still showed "logged in" while chat correctly
    // returned "登录已过期". These lock the three states the classifier must
    // distinguish so the frontend can tell "present-and-usable" from
    // "present-but-dead".

    #[test]
    fn no_credentials_is_logged_out_not_expired() {
        assert_eq!(classify_auth_status(false, false), (false, false));
    }

    #[test]
    fn present_and_usable_token_is_logged_in_not_expired() {
        assert_eq!(classify_auth_status(true, true), (true, false));
    }

    #[test]
    fn present_but_unusable_token_is_expired() {
        // File on disk exists (so the user "looks" logged in) but the token
        // can't be made valid -- this is the exact sidebar/chat mismatch.
        assert_eq!(classify_auth_status(true, false), (true, true));
    }

    #[tokio::test]
    async fn neutral_build_reports_managed_unavailable() {
        // The WebUI hides its account / sign-in surfaces on this flag. A
        // neutral/open build ships no platform server, so it must be false
        // (and with the test ctor's temp RUSTCODE_HOME there is no stored
        // auth either); the field must always be present in the JSON.
        let response = auth_status().await.into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            !json["logged_in"].as_bool().unwrap_or(true),
            "neutral build with no stored auth must report logged_in=false: {json}"
        );
        assert!(
            !json["managed_available"].as_bool().unwrap_or(true),
            "neutral build must report managed_available=false so the WebUI hides \
             account surfaces: {json}"
        );
        assert!(
            !auth::managed_login_available(),
            "predicate and /auth/status must agree in a neutral build"
        );
    }

    #[tokio::test]
    async fn neutral_build_login_start_returns_actionable_501() {
        // This test asserts the English BYO-guidance prose; pin the
        // locale because the product default is Simplified Chinese.
        let _locale = rustcode_config::i18n::test_lock();
        rustcode_config::i18n::set_locale(rustcode_config::i18n::Locale::En);
        // Old client builds that still show a sign-in button get a
        // deterministic, non-retryable code instead of a generic 500, so
        // they can surface bring-your-own-key guidance.
        let response = managed_login_unavailable_response();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "managed_login_unavailable");
        assert!(
            !json["retryable"].as_bool().unwrap_or(true),
            "neutral unavailability must not be retried: {json}"
        );
        let message = json["error"].as_str().unwrap_or_default();
        assert!(
            message.contains("third-party provider"),
            "501 must steer to BYO provider setup: {message}"
        );
    }

    #[test]
    fn login_start_response_has_no_runtime_protocol_selector() {
        let response = LoginStartResponse {
            login_id: "login-id".to_string(),
            url: "https://example.invalid/login".to_string(),
            expires_in_seconds: 600,
            daemon_instance_id: "daemon-instance".to_string(),
        };

        let json = serde_json::to_value(response).unwrap();
        assert!(json.get("protocol_version").is_none());
    }

    #[tokio::test]
    async fn login_terminal_states_are_non_success_and_non_retryable() {
        let cases = [
            (
                LoginPollStep::Expired,
                StatusCode::GONE,
                "login_session_expired",
            ),
            (
                LoginPollStep::Cancelled,
                StatusCode::GONE,
                "login_session_cancelled",
            ),
            (
                LoginPollStep::Failed {
                    code: "login_exchange_failed".to_string(),
                    message: "Login authorization exchange failed".to_string(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                "login_exchange_failed",
            ),
        ];

        for (step, expected_http_status, expected_code) in cases {
            let response = login_poll_response(LoginPollResult { step });
            assert_eq!(response.status(), expected_http_status);
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["code"], expected_code);
            assert_eq!(json["retryable"], false);
        }
    }

    #[test]
    fn login_pending_and_authorized_states_remain_successful() {
        let pending = login_poll_response(LoginPollResult {
            step: LoginPollStep::Pending,
        });
        assert_eq!(pending.status(), StatusCode::OK);

        let authorized = login_poll_response(LoginPollResult {
            step: LoginPollStep::Authorized {
                user: auth::UserInfo {
                    id: "user-id".to_string(),
                    username: "tester".to_string(),
                    name: None,
                    email: None,
                    avatar_url: None,
                },
            },
        });
        assert_eq!(authorized.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn login_retryable_failure_remains_service_unavailable() {
        let response = login_poll_response(LoginPollResult {
            step: LoginPollStep::Retryable {
                code: "login_poll_unavailable".to_string(),
                message: "Login service is temporarily unavailable".to_string(),
            },
        });
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "login_poll_unavailable");
        assert_eq!(json["retryable"], true);
    }
}
