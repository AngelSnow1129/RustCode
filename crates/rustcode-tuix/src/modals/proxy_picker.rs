use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use rustcode_config::proxy::{self, ProxyMode};

use super::{Modal, ModalAction};
use crate::event_loop::{build_status, save_proxy_and_reload, Buffer, LoopCtx};
use crate::render::{MenuPayload, Renderer, UiLine};
use crate::state::UiState;

pub struct ProxyPicker {
    pub selected: usize,
}

impl ProxyPicker {
    pub fn open(config: &rustcode_config::config::Config) -> Self {
        let selected = match config.network.proxy.mode {
            ProxyMode::FollowSystem => 0,
            ProxyMode::DefaultProxy => 1,
            ProxyMode::NoProxy => 2,
        };
        Self { selected }
    }
}

impl Modal for ProxyPicker {
    fn handle_key(
        &mut self,
        code: KeyCode,
        _mods: KeyModifiers,
        buf: &mut Buffer,
        state: &mut UiState,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Result<ModalAction> {
        match code {
            KeyCode::Up => {
                // Wraps: three options (direct / env / manual).
                self.selected = crate::modals::step_up(self.selected, 3);
                self.draw(buf, state, ctx, renderer);
                Ok(ModalAction::Continue)
            }
            KeyCode::Down => {
                // Wraps: three options.
                self.selected = crate::modals::step_down(self.selected, 3);
                self.draw(buf, state, ctx, renderer);
                Ok(ModalAction::Continue)
            }
            KeyCode::Enter => {
                let mut desired = ctx.config.network.proxy.clone();
                match self.selected {
                    0 => {
                        desired.mode = ProxyMode::FollowSystem;
                    }
                    1 => {
                        let pinned = proxy::ProxyConfig::capture_from_env();
                        desired = pinned;
                    }
                    _ => {
                        desired.mode = ProxyMode::NoProxy;
                    }
                }
                let success = crate::i18n::t(crate::i18n::Msg::ProxyModeLine {
                    mode: &proxy_summary_word(&desired),
                })
                .into_owned();
                if save_proxy_and_reload(ctx, desired, renderer, success) {
                    Ok(ModalAction::Close)
                } else {
                    Ok(ModalAction::Continue)
                }
            }
            KeyCode::Esc => Ok(ModalAction::Close),
            _ => Ok(ModalAction::Continue),
        }
    }

    fn draw(&self, buf: &Buffer, state: &UiState, ctx: &LoopCtx, renderer: &mut dyn Renderer) {
        let items = vec![
            (
                crate::i18n::t(crate::i18n::Msg::ProxyTitleFollowSystem).into_owned(),
                crate::i18n::t(crate::i18n::Msg::ProxyDescFollowSystem).into_owned(),
            ),
            (
                crate::i18n::t(crate::i18n::Msg::ProxyTitleDefaultProxy).into_owned(),
                crate::i18n::t(crate::i18n::Msg::ProxyDescDefaultProxy).into_owned(),
            ),
            (
                crate::i18n::t(crate::i18n::Msg::ProxyTitleNoProxy).into_owned(),
                crate::i18n::t(crate::i18n::Msg::ProxyDescNoProxy).into_owned(),
            ),
        ];
        let payload = MenuPayload {
            items,
            selected: self.selected,
            kind: crate::render::MenuKind::SlashCommand,
        };
        renderer.render(UiLine::InputPrompt {
            buf: buf.text.clone(),
            cursor_byte: buf.cursor,
            menu: Some(payload),
            status: build_status(state, ctx),
            attachments: Vec::new(),
        });
        renderer.flush();
    }
}

/// Localized mode word for the proxy confirmation line. Mirrors
/// `ProxyConfig::summary()` (rustcode-config), whose stable enum-id string stays
/// on the daemon/webui wire; the TUI renders a localized variant.
fn proxy_summary_word(cfg: &proxy::ProxyConfig) -> String {
    match cfg.mode {
        ProxyMode::FollowSystem => {
            crate::i18n::t(crate::i18n::Msg::ProxyTitleFollowSystem).into_owned()
        }
        ProxyMode::NoProxy => crate::i18n::t(crate::i18n::Msg::ProxyTitleNoProxy).into_owned(),
        ProxyMode::DefaultProxy => {
            let count = [
                cfg.http.as_ref(),
                cfg.https.as_ref(),
                cfg.all.as_ref(),
                cfg.no_proxy.as_ref(),
            ]
            .into_iter()
            .flatten()
            .count();
            if count == 0 {
                crate::i18n::t(crate::i18n::Msg::ProxyDefaultEmpty).into_owned()
            } else {
                crate::i18n::t(crate::i18n::Msg::ProxyDefaultPinned { count }).into_owned()
            }
        }
    }
}
