//! OpenRouter quick connect: background connect task + events + config provisioning.
//!
//! The network-heavy PKCE exchange and model discovery run on a background
//! std::thread (blocking reqwest). Provisioning (config mutation + persist +
//! reload) happens on the main loop select! arm so it shares the same config
//! store revision path as every other config mutation.

use rustcode_capabilities::provider::openrouter::FreeModel;
use rustcode_config::config::provider::{
    default_context_window_for, ModelProfileConfig, ProviderAccountConfig,
};
use rustcode_config::config::provider_preset::preset_or_compatible;
use rustcode_config::config::Config;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;

const OPENROUTER_ACCOUNT_ID: &str = "openrouter";

/// Connect mode: empty arg -> OAuth PKCE browser flow, non-empty -> use provided key.
pub enum ConnectMode {
    Oauth,
    ProvidedKey(String),
}

/// Parse `/openrouter [arg]`: trim-empty -> Oauth, else ProvidedKey.
pub fn parse_connect_mode(arg: &str) -> ConnectMode {
    let t = arg.trim();
    if t.is_empty() {
        ConnectMode::Oauth
    } else {
        ConnectMode::ProvidedKey(t.to_string())
    }
}

/// Event sent from the background connect thread to the main loop select! arm.
pub enum OpenRouterConnectEvent {
    /// OAuth auth URL opened in browser; surfaced so the user can copy-paste
    /// if the browser did not auto-open (headless / no DISPLAY / SSH).
    AwaitingBrowser {
        auth_url: String,
    },
    Ready {
        api_key: String,
        models: Vec<FreeModel>,
    },
    Failed(String),
}

pub struct ProvisionOutcome {
    #[allow(dead_code)]
    pub account_id: String,
    pub added: Vec<String>,
    #[allow(dead_code)]
    pub default_model: String,
}

const FREE_MODEL_LIMIT: usize = 5;

/// Background thread: get key (OAuth or provided) + discover top-5 free models
/// -> send event + wake loop. All network ops here; provisioning+save+reload
/// in the main loop select! arm.
pub fn spawn_openrouter_connect(
    mode: ConnectMode,
    event_tx: mpsc::UnboundedSender<OpenRouterConnectEvent>,
    wake_tx: mpsc::Sender<()>,
    cancel: Arc<AtomicBool>,
) {
    use rustcode_capabilities::provider::openrouter as or;
    std::thread::spawn(move || {
        let result: Result<(String, Vec<or::FreeModel>), String> = (|| {
            let key = match mode {
                ConnectMode::ProvidedKey(k) => k,
                ConnectMode::Oauth => {
                    let pkce = or::generate_pkce();
                    let cb = or::start_local_callback().map_err(|e| format!("{e:#}"))?;
                    let callback_url = format!("http://127.0.0.1:{}/callback", cb.port());
                    let auth_url = or::build_auth_url(Some(&callback_url), &pkce.challenge);
                    let _ = or::open_browser(&auth_url);
                    let _ = event_tx.send(OpenRouterConnectEvent::AwaitingBrowser { auth_url });
                    let _ = wake_tx.blocking_send(());
                    let code = cb
                        .wait_for_code(std::time::Duration::from_secs(180), &cancel)
                        .map_err(|e| format!("{e:#}"))?
                        .ok_or_else(|| "cancelled or timed out".to_string())?;
                    or::exchange_code_for_key(&code, &pkce.verifier)
                        .map_err(|e| format!("{e:#}"))?
                }
            };
            let models =
                or::fetch_top_free_models(&key, FREE_MODEL_LIMIT).map_err(|e| format!("{e:#}"))?;
            if models.is_empty() {
                return Err("OpenRouter returned no usable free models".to_string());
            }
            Ok((key, models))
        })();

        let event = match result {
            Ok((api_key, models)) => OpenRouterConnectEvent::Ready { api_key, models },
            Err(reason) => OpenRouterConnectEvent::Failed(reason),
        };
        let _ = event_tx.send(event);
        let _ = wake_tx.blocking_send(());
    });
}

/// Idempotent provisioning: account fixed id, update key if exists; model
/// selection `openrouter/<model>`, skip if exists. All persistent.
pub fn provision_openrouter(
    config: &mut Config,
    api_key: &str,
    models: &[FreeModel],
) -> ProvisionOutcome {
    let preset = preset_or_compatible(OPENROUTER_ACCOUNT_ID);
    let provider_type_wire = preset.provider_type.wire().to_string();

    // upsert account (update key only, preserve other fields).
    config
        .provider_accounts
        .entry(OPENROUTER_ACCOUNT_ID.to_string())
        .and_modify(|a| a.api_key = Some(api_key.to_string()))
        .or_insert_with(|| ProviderAccountConfig {
            provider: OPENROUTER_ACCOUNT_ID.to_string(),
            display_name: None,
            api_key: Some(api_key.to_string()),
            base_url: None,
            extra_headers: None,
            proxy: None,
            user_agent: None,
            skip_tls_verify: false,
            enterprise_url: None,
            ephemeral: false,
        });

    let mut added = Vec::new();
    let mut first_selection: Option<String> = None;
    for m in models {
        let selection_id = format!("{OPENROUTER_ACCOUNT_ID}/{}", m.id);
        if first_selection.is_none() {
            first_selection = Some(selection_id.clone());
        }
        if config.selection_exists(&selection_id) {
            continue;
        }
        config.models.insert(
            selection_id.clone(),
            ModelProfileConfig {
                account: OPENROUTER_ACCOUNT_ID.to_string(),
                model: m.id.clone(),
                display_name: m.name.clone(),
                system_prompt: None,
                supports_vision: None,
                context_window: if m.context_length > 0 {
                    m.context_length as usize
                } else {
                    default_context_window_for(&provider_type_wire)
                },
                max_tokens: None,
                capable_model: None,
                thinking_type: None,
                thinking_keep: None,
                reasoning_history: None,
                reasoning_effort: None,
                reasoning_effort_levels: None,
                thinking_enabled: None,
                thinking_budget: None,
                retry_max_attempts: None,
                model_mapping: Default::default(),
                // Discovered models start with no chain; the user opts in.
                fallback: Vec::new(),
                added_at: rustcode_config::util::now_epoch_secs(),
            },
        );
        added.push(selection_id);
    }

    let default_model = first_selection.unwrap_or_default();
    if config.default_model.is_none() && !default_model.is_empty() {
        config.default_model = Some(default_model.clone());
    }

    ProvisionOutcome {
        account_id: OPENROUTER_ACCOUNT_ID.to_string(),
        added,
        default_model,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_capabilities::provider::openrouter::FreeModel;
    use rustcode_config::config::Config;

    fn models() -> Vec<FreeModel> {
        vec![
            FreeModel {
                id: "vendor/big:free".into(),
                name: Some("Big".into()),
                context_length: 128000,
            },
            FreeModel {
                id: "vendor/small:free".into(),
                name: None,
                context_length: 8000,
            },
        ]
    }

    #[test]
    fn arg_parsing_selects_mode() {
        assert!(matches!(parse_connect_mode(""), ConnectMode::Oauth));
        assert!(matches!(parse_connect_mode("   "), ConnectMode::Oauth));
        match parse_connect_mode("  sk-or-v1-abc  ") {
            ConnectMode::ProvidedKey(k) => assert_eq!(k, "sk-or-v1-abc"),
            _ => panic!("expected ProvidedKey"),
        }
    }

    #[test]
    fn fresh_config_gets_account_models_and_default() {
        let mut c = Config::default();
        let out = provision_openrouter(&mut c, "sk-or-v1-x", &models());
        assert_eq!(out.account_id, "openrouter");
        assert!(c.provider_accounts.contains_key("openrouter"));
        assert_eq!(
            c.provider_accounts["openrouter"].api_key.as_deref(),
            Some("sk-or-v1-x")
        );
        assert!(!c.provider_accounts["openrouter"].ephemeral);
        assert!(c.models.contains_key("openrouter/vendor/big:free"));
        assert!(c.models.contains_key("openrouter/vendor/small:free"));
        assert_eq!(out.default_model, "openrouter/vendor/big:free");
        assert_eq!(
            c.default_model.as_deref(),
            Some("openrouter/vendor/big:free")
        );
    }

    #[test]
    fn existing_account_key_updated_not_duplicated() {
        let mut c = Config::default();
        provision_openrouter(&mut c, "sk-or-v1-OLD", &models());
        let out = provision_openrouter(&mut c, "sk-or-v1-NEW", &models());
        assert_eq!(
            c.provider_accounts["openrouter"].api_key.as_deref(),
            Some("sk-or-v1-NEW")
        );
        assert_eq!(
            c.models
                .keys()
                .filter(|k| k.starts_with("openrouter/"))
                .count(),
            2
        );
        assert!(out.added.is_empty());
    }

    #[test]
    fn preexisting_default_model_is_preserved() {
        let mut c = Config::default();
        c.default_model = Some("someacct/somemodel".into());
        let out = provision_openrouter(&mut c, "k", &models());
        assert_eq!(c.default_model.as_deref(), Some("someacct/somemodel"));
        assert_eq!(out.default_model, "openrouter/vendor/big:free");
    }
}
