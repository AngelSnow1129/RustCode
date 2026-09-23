// crates/rustcode-tuix/src/modals/provider_panel.rs
//
// `/provider` full-panel manager, in the style of `/plugin` (PluginManager).
// Two tabs -- Accounts and Models -- with in-panel forms; the main input box is
// hidden (MenuKind::Plugin). See docs/plans/2026-07-28-provider-panel-ui-design.md.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use rustcode_config::config::provider::{ModelProfileConfig, ProviderAccountConfig};
use rustcode_config::config::{provider_preset, Config};
use unicode_segmentation::UnicodeSegmentation;

/// Discovered model from remote provider discovery API.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscoveredModel {
    id: String,
    name: Option<String>,
    context_window: Option<usize>,
    max_tokens: Option<usize>,
}

/// Lifecycle of the panel's one-shot model discovery.
///
/// Only `Pending` refuses a new request: a finished attempt leaves a `Notice`
/// behind (so the panel can say WHY nothing was loaded -- a silent failure was
/// indistinguishable from "this provider has no models") while a later add/edit
/// is still allowed to try again. The selection itself lives in
/// `Mode::DiscoveryResults`, never duplicated here.
#[derive(Clone, Debug, PartialEq, Eq)]
enum DiscoveryState {
    /// Nothing in flight, no notice to show.
    Idle,
    /// Request sent, waiting for the answer.
    Pending {
        account_id: String,
        base_url: String,
    },
    /// Terminal notice about the last attempt, rendered in the panel.
    Notice { message: String },
}

/// Result from async discovery thread.
struct DiscoveryResult {
    account_id: String,
    result: Result<Vec<DiscoveredModel>, String>,
}

/// The network facts one discovery request needs, resolved at SAVE time.
///
/// Discovery must NOT read `LoopCtx::config` right after a save: the add/edit
/// handlers stage an ASYNCHRONOUS provider reload (`stage_committed_config_reload`
/// only records `ctx.pending_provider_reload`; `ctx.config` is rewritten when the
/// runtime reports the reload finished). A freshly added account is therefore
/// absent from `ctx.config` for at least one event-loop turn, so the old
/// `start_discovery(&account_id, &ctx.config)` call found no account, returned
/// early and left the panel on an empty model list with no error at all.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscoveryTarget {
    account_id: String,
    /// Discovery protocol: OpenAI-compatible `/models` or Ollama `/api/tags`.
    provider_type: &'static str,
    base_url: String,
    api_key: Option<String>,
    /// Transport settings copied off the ACCOUNT, mirroring the daemon's discovery
    /// handler (`stored_discovery_transport`). The panel used to hardcode these,
    /// so an account that worked from the WebUI (self-signed gateway behind a
    /// corporate proxy, custom UA allowlist) silently failed from the TUI.
    user_agent: Option<String>,
    proxy: Option<String>,
    skip_tls_verify: bool,
}

impl DiscoveryTarget {
    /// An account with no endpoint (neither its own nor the preset default) has
    /// nothing to discover.
    fn is_discoverable(&self) -> bool {
        !self.base_url.trim().is_empty()
    }

    fn pending_state(&self) -> DiscoveryState {
        DiscoveryState::Pending {
            account_id: self.account_id.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

/// Whether a new discovery request may start. A request already in flight wins;
/// every terminal state (including a notice) stays re-tryable.
fn can_start_discovery(state: &DiscoveryState) -> bool {
    !matches!(state, DiscoveryState::Pending { .. })
}

/// Discovery speaks exactly two list endpoints; Anthropic accounts are probed
/// over the OpenAI-compatible listing, matching the daemon's discovery handler.
fn discovery_protocol(ty: provider_preset::ProviderType) -> &'static str {
    match ty {
        provider_preset::ProviderType::Ollama => "ollama",
        provider_preset::ProviderType::OpenAi | provider_preset::ProviderType::Anthropic => {
            "openai"
        }
    }
}

/// Resolve the discovery target of one persisted account row.
fn discovery_target_for_account(
    account: &ProviderAccountConfig,
    account_id: &str,
) -> DiscoveryTarget {
    let preset = provider_preset::preset_or_compatible(&account.provider);
    DiscoveryTarget {
        account_id: account_id.to_string(),
        provider_type: discovery_protocol(preset.provider_type),
        base_url: account
            .base_url
            .clone()
            .or_else(|| preset.default_base_url.map(str::to_string))
            .unwrap_or_default(),
        api_key: account.api_key.clone(),
        user_agent: account.user_agent.clone(),
        proxy: account.proxy.clone().filter(|p| !p.trim().is_empty()),
        skip_tls_verify: account.skip_tls_verify,
    }
}

use super::{tab_chip, Modal, ModalAction};
use crate::event_loop::{
    build_status, set_default_provider_and_reload, update_config_and_reload, Buffer,
    ConfigReloadSelection, LoopCtx,
};
use crate::render::{MenuKind, MenuPayload, Renderer, UiLine};
use crate::state::UiState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Accounts,
    Models,
}

/// A unique account id derived from a preset id, avoiding collisions with
/// existing accounts or legacy provider names.
fn unique_account_id(base: &str, config: &Config) -> String {
    let taken =
        |id: &str| config.provider_accounts.contains_key(id) || config.providers.contains_key(id);
    if !taken(base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !taken(candidate))
        .unwrap_or_else(|| base.to_string())
}

/// Which add-form field has focus.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FormField {
    Name,
    Preset,
    BaseUrl,
    ApiKey,
}

/// Add a provider ACCOUNT (name + protocol + endpoint + credential). Models are
/// added separately on the 模型 tab, so this form has no model field.
#[derive(Clone)]
struct AddForm {
    name: String,
    preset_idx: usize,
    base_url: String,
    api_key: String,
    focus: FormField,
    /// UTF-8 byte cursor for the currently focused text field.
    cursor_byte: usize,
}

/// Protocol presets the fully-custom add/edit form cycles through with `←/->`,
/// in display order. Each id resolves to a real `PRESETS` entry: the two generic
/// `*-compatible` custom endpoints plus the keyless local `ollama` preset.
const CYCLE_PROTOCOL_IDS: [&str; 3] = ["openai-compatible", "anthropic-compatible", "ollama"];

/// `PRESETS` index for a preset id. Falls back to the first entry, which the
/// registry guarantees is the neutral OpenAI-compatible custom endpoint (never a
/// featured vendor), so an unknown id can't pre-select a specific provider.
fn preset_idx_by_id(id: &str) -> usize {
    provider_preset::PRESETS
        .iter()
        .position(|p| p.id == id)
        .unwrap_or(0)
}

/// The `PRESETS` index of the protocol-toggle preset matching a stored account's
/// wire protocol -- so opening an existing account starts the toggle on the right
/// choice (and an Ollama account shows as Ollama, not OpenAI). Also seeds a fresh
/// add-form on its OpenAI default.
fn protocol_preset_idx(ty: provider_preset::ProviderType) -> usize {
    preset_idx_by_id(match ty {
        provider_preset::ProviderType::Anthropic => "anthropic-compatible",
        provider_preset::ProviderType::Ollama => "ollama",
        provider_preset::ProviderType::OpenAi => "openai-compatible",
    })
}

/// Human protocol label shown next to the `←/->` toggle. Derived from the wire
/// protocol (exhaustive) so a new `ProviderType` can't silently mislabel.
fn protocol_label(ty: provider_preset::ProviderType) -> &'static str {
    match ty {
        provider_preset::ProviderType::Anthropic => "Anthropic",
        provider_preset::ProviderType::Ollama => "Ollama",
        provider_preset::ProviderType::OpenAi => "OpenAI",
    }
}

/// Next protocol id in the toggle cycle (`forward` picks direction; wraps). An id
/// outside the cycle restarts at the first entry.
fn cycle_protocol_id(current_id: &str, forward: bool) -> &'static str {
    let len = CYCLE_PROTOCOL_IDS.len();
    let cur = CYCLE_PROTOCOL_IDS
        .iter()
        .position(|id| *id == current_id)
        .unwrap_or(0);
    let next = if forward {
        (cur + 1) % len
    } else {
        (cur + len - 1) % len
    };
    CYCLE_PROTOCOL_IDS[next]
}

/// Move `preset_idx` to the next protocol in the cycle. When landing on a preset
/// that ships a default endpoint (only Ollama does) and `base_url` is still
/// blank, offer that endpoint as a convenience. A value the user typed -- or one
/// an existing account was opened with -- is NEVER overwritten or cleared, so no
/// edit can silently lose an endpoint.
fn cycle_protocol(preset_idx: &mut usize, base_url: &mut String, forward: bool) {
    let next_id = cycle_protocol_id(provider_preset::PRESETS[*preset_idx].id, forward);
    *preset_idx = preset_idx_by_id(next_id);
    if base_url.trim().is_empty() {
        if let Some(url) = provider_preset::PRESETS[*preset_idx].default_base_url {
            *base_url = url.to_string();
        }
    }
}

impl AddForm {
    /// A fully-custom provider, protocol defaulting to OpenAI-compatible.
    fn new() -> Self {
        Self {
            name: String::new(),
            preset_idx: protocol_preset_idx(provider_preset::ProviderType::OpenAi),
            base_url: String::new(),
            api_key: String::new(),
            focus: FormField::Name,
            cursor_byte: 0,
        }
    }

    /// Human protocol label for the toggle.
    fn protocol_label(&self) -> &'static str {
        protocol_label(self.preset().provider_type)
    }

    fn preset(&self) -> &'static provider_preset::ProviderPreset {
        &provider_preset::PRESETS[self.preset_idx]
    }

    /// Field sequence: custom name, vendor preset, base_url (always editable),
    /// api key (only for keyed presets).
    fn fields(&self) -> Vec<FormField> {
        let mut v = vec![FormField::Name, FormField::Preset, FormField::BaseUrl];
        if !matches!(self.preset().auth_kind, provider_preset::AuthKind::None) {
            v.push(FormField::ApiKey);
        }
        v
    }

    fn advance_focus(&mut self, forward: bool) {
        let fields = self.fields();
        let cur = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = if forward {
            (cur + 1) % fields.len()
        } else {
            (cur + fields.len() - 1) % fields.len()
        };
        self.focus = fields[next];
        self.cursor_byte = self.focused_text().map(str::len).unwrap_or(0);
    }

    fn focused_text(&self) -> Option<&str> {
        match self.focus {
            FormField::Name => Some(&self.name),
            FormField::BaseUrl => Some(&self.base_url),
            FormField::ApiKey => Some(&self.api_key),
            FormField::Preset => None,
        }
    }

    fn cycle_preset(&mut self, forward: bool) {
        // OpenAI-compatible -> Anthropic-compatible -> Ollama -> ... (see cycle_protocol,
        // which also manages Ollama's auto-filled local endpoint).
        cycle_protocol(&mut self.preset_idx, &mut self.base_url, forward);
        if !self.fields().contains(&self.focus) {
            self.focus = FormField::Name;
        }
    }
}

/// Sanitize a user-typed account name into a TOML-key-safe id: keep
/// alphanumerics / `-` / `_` / `.`, collapse everything else to `-`, trim stray
/// dashes. Empty result ⇒ caller falls back to the preset id.
fn sanitize_account_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// Edit an existing account's vendor/connection/credential. `api_key` blank keeps
/// the current secret; `base_url` and the vendor preset are pre-filled and
/// editable.
#[derive(Clone)]
struct EditForm {
    id: String,
    is_legacy: bool,
    /// An unconfigured preset row has no persisted account yet. Keep its preset
    /// id so save can materialize the account instead of reporting a no-op
    /// success.
    materialize_provider: Option<String>,
    preset_idx: usize,
    /// The preset the account started on -- so save_edit rewrites the vendor ONLY
    /// when the user actually changed it (a no-op edit must not lossily normalize
    /// a `deepseek`/custom provider to the `openai` fallback).
    original_preset_idx: usize,
    /// A curated preset quick-add row has a fixed wire protocol. Its endpoint
    /// and key are editable, but changing the protocol would turn (for example)
    /// a DeepSeek row into an Anthropic-compatible account with a DeepSeek URL.
    protocol_locked: bool,
    api_key: String,
    base_url: String,
    focus: FormField,
    /// UTF-8 byte cursor for the currently focused text field.
    cursor_byte: usize,
}

impl EditForm {
    fn preset(&self) -> &'static provider_preset::ProviderPreset {
        &provider_preset::PRESETS[self.preset_idx]
    }

    fn protocol_label(&self) -> &'static str {
        protocol_label(self.preset().provider_type)
    }

    /// Field sequence. A protocol-locked account omits the preset toggle.
    fn fields(&self) -> Vec<FormField> {
        let mut v = Vec::new();
        if !self.protocol_locked {
            v.push(FormField::Preset);
        }
        v.push(FormField::BaseUrl);
        if !matches!(self.preset().auth_kind, provider_preset::AuthKind::None) {
            v.push(FormField::ApiKey);
        }
        v
    }

    fn advance_focus(&mut self, forward: bool) {
        let fields = self.fields();
        let cur = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = if forward {
            (cur + 1) % fields.len()
        } else {
            (cur + fields.len() - 1) % fields.len()
        };
        self.focus = fields[next];
        self.cursor_byte = self.focused_text().map(str::len).unwrap_or(0);
    }

    fn focused_text(&self) -> Option<&str> {
        match self.focus {
            FormField::BaseUrl => Some(&self.base_url),
            FormField::ApiKey => Some(&self.api_key),
            FormField::Name | FormField::Preset => None,
        }
    }

    fn cycle_preset(&mut self, forward: bool) {
        if self.protocol_locked {
            return; // curated vendor -- protocol not editable
        }
        // OpenAI-compatible -> Anthropic-compatible -> Ollama -> ... (see cycle_protocol,
        // which also manages Ollama's auto-filled local endpoint).
        cycle_protocol(&mut self.preset_idx, &mut self.base_url, forward);
        if !self.fields().contains(&self.focus) {
            self.focus = FormField::Preset;
        }
    }
}

fn previous_grapheme_boundary(text: &str, cursor: usize) -> usize {
    let cursor = cursor.min(text.len());
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .rfind(|index| *index < cursor)
        .unwrap_or(0)
}

fn next_grapheme_boundary(text: &str, cursor: usize) -> usize {
    let cursor = cursor.min(text.len());
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .find(|index| *index > cursor)
        .unwrap_or(text.len())
}

fn insert_at_cursor(text: &mut String, cursor: &mut usize, inserted: &str) {
    *cursor = (*cursor).min(text.len());
    text.insert_str(*cursor, inserted);
    *cursor += inserted.len();
}

fn backspace_at_cursor(text: &mut String, cursor: &mut usize) {
    let start = previous_grapheme_boundary(text, *cursor);
    if start < *cursor {
        text.drain(start..*cursor);
        *cursor = start;
    }
}

fn delete_at_cursor(text: &mut String, cursor: &mut usize) {
    let end = next_grapheme_boundary(text, *cursor);
    if *cursor < end {
        text.drain(*cursor..end);
    }
}

/// Build the transient value projection for a focused form field. The source
/// value is never changed: the visible caret and ellipses exist only in the
/// `PluginInfo` row sent to the renderer.
fn editable_field_row(
    label: &str,
    value: &str,
    focused: bool,
    cursor_byte: usize,
    max_cols: usize,
) -> (String, String) {
    let marker = if focused { "> " } else { "  " };
    let prefix = format!("{marker}{label}: ");
    let value_cols = max_cols.saturating_sub(crate::width::display_width(&prefix));
    let displayed = if focused {
        crate::width::editable_value_projection(value, cursor_byte, value_cols)
    } else {
        crate::width::truncate_with_ellipsis(value, value_cols)
    };
    (format!("{prefix}{displayed}"), String::new())
}

/// Provider forms are also used in classic Windows conhost, where the active
/// font commonly lacks the geometric symbols used as focus/caret chrome. Keep
/// user content intact while applying the shared decorative-glyph fallback to
/// both columns before the payload reaches the renderer.
fn downgrade_panel_items(items: &mut [(String, String)], unicode_symbols: bool) {
    if unicode_symbols {
        return;
    }

    fn provider_chrome_ascii(text: &str) -> String {
        crate::glyph::downgrade_glyphs(text, false)
            .chars()
            .map(|ch| match ch {
                '‹' => '<',
                '›' => '>',
                '–' | '—' => '-',
                // Form projections allocate one display cell for an ellipsis;
                // keep the fallback one cell wide as well.
                '…' => '.',
                '＋' => '+',
                other => other,
            })
            .collect()
    }

    for (label, detail) in items {
        *label = provider_chrome_ascii(label);
        *detail = provider_chrome_ascii(detail);
    }
}

/// Which model-form field has focus.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ModelField {
    Account,
    ApiKey,
    Model,
    Vision,
    Effort,
    EffortLevels,
    Window,
    MakeDefault,
}

/// True iff adding a model to `account_id` should prompt for the provider's
/// api_key: an account that has no explicit api_key yet. Filled once, stored
/// on the account.
fn account_needs_key(config: &Config, account_id: &str) -> bool {
    match config.provider_accounts.get(account_id) {
        Some(a) => a.api_key.as_deref().unwrap_or("").trim().is_empty(),
        // Not yet configured (a preset-vendor quick-add) -- needs a key iff the
        // preset is keyed (account_id == preset id).
        None => !matches!(
            provider_preset::preset_or_compatible(account_id).auth_kind,
            provider_preset::AuthKind::None
        ),
    }
}

/// Add a model to an EXISTING account (the 模型 tab's `a`). Optionally editing an
/// existing model in place (`edit_id` set -> account is fixed, id preserved).
#[derive(Clone)]
struct ModelForm {
    account_ids: Vec<String>,
    /// Parallel to `account_ids`: whether that account still needs an api_key.
    needs_key: Vec<bool>,
    account_idx: usize,
    api_key: String,
    model: String,
    /// `None` = Auto, `Some(true)` = Enabled, `Some(false)` = Disabled.
    supports_vision: Option<bool>,
    /// `None` = unsupported, `Some("auto")` = supported with API default.
    reasoning_effort: Option<String>,
    /// Per-level toggles for `reasoning_effort_levels` (index = canonical
    /// low/medium/high/xhigh/max order). All-true ⇒ unrestricted (persisted as None).
    effort_levels: [bool; EFFORT_LEVEL_COUNT],
    /// Sub-cursor for the EffortLevels multi-select (which level Space toggles).
    effort_level_cursor: usize,
    window: String,
    make_default: bool,
    focus: ModelField,
    /// UTF-8 byte cursor for the currently focused text field.
    cursor_byte: usize,
    /// When set, this is an edit of an existing model id (account locked).
    edit_id: Option<String>,
}

/// Number of canonical reasoning-effort levels; the per-level toggle arrays track
/// [`REASONING_EFFORT_LEVELS`](rustcode_config::config::REASONING_EFFORT_LEVELS) so
/// adding a level (e.g. `xhigh`) can never desync a hardcoded array length.
const EFFORT_LEVEL_COUNT: usize = rustcode_config::config::REASONING_EFFORT_LEVELS.len();

/// Convert a persisted `reasoning_effort_levels` list into per-level toggles
/// (canonical low/medium/high/xhigh/max order). `None`/empty ⇒ all levels enabled.
fn effort_levels_from_config(declared: Option<&[String]>) -> [bool; EFFORT_LEVEL_COUNT] {
    // Derive from the single source of truth so the toggles agree with what every
    // other surface offers (incl. the "unknown-only list ⇒ all levels" rule).
    let allowed = rustcode_config::config::allowed_effort_levels(declared);
    let mut bits = [false; EFFORT_LEVEL_COUNT];
    for (i, level) in rustcode_config::config::REASONING_EFFORT_LEVELS
        .iter()
        .enumerate()
    {
        bits[i] = allowed.contains(level);
    }
    bits
}

/// Convert per-level toggles back to a persisted `reasoning_effort_levels`.
/// All-enabled (or, degenerately, all-disabled) ⇒ `None` = unrestricted; there is
/// no "zero levels" state, since `allowed_effort_levels` treats empty as all.
fn effort_levels_to_config(bits: [bool; EFFORT_LEVEL_COUNT]) -> Option<Vec<String>> {
    if bits.iter().all(|b| *b) || bits.iter().all(|b| !*b) {
        return None;
    }
    Some(
        rustcode_config::config::REASONING_EFFORT_LEVELS
            .iter()
            .enumerate()
            .filter(|(i, _)| bits[*i])
            .map(|(_, level)| level.to_string())
            .collect(),
    )
}

impl ModelForm {
    fn new_add(config: &Config, preferred: Option<&str>) -> Option<Self> {
        let account_ids: Vec<String> = ProviderPanel::account_ids(config);
        if account_ids.is_empty() {
            return None;
        }
        // Preselect the drilled-into account (if any) so "add a model to THIS
        // account" is one keystroke, not a hunt through the ‹account› cycle.
        let account_idx = preferred
            .and_then(|p| account_ids.iter().position(|a| a == p))
            .unwrap_or(0);
        let needs_key = account_ids
            .iter()
            .map(|id| account_needs_key(config, id))
            .collect();
        Some(Self {
            account_ids,
            needs_key,
            account_idx,
            api_key: String::new(),
            model: String::new(),
            supports_vision: None,
            reasoning_effort: None,
            effort_levels: [true; EFFORT_LEVEL_COUNT],
            effort_level_cursor: 0,
            window: String::new(),
            make_default: true,
            focus: ModelField::Account,
            cursor_byte: 0,
            edit_id: None,
        })
    }

    fn new_edit(config: &Config, id: &str) -> Option<Self> {
        let m = config.logical_models().get(id).cloned()?;
        Some(Self {
            account_ids: vec![m.account.clone()],
            needs_key: vec![false], // account already exists; edit its key via 账号页
            account_idx: 0,
            api_key: String::new(),
            model: m.model.clone(),
            supports_vision: m.supports_vision,
            reasoning_effort: m.reasoning_effort.clone(),
            effort_levels: effort_levels_from_config(m.reasoning_effort_levels.as_deref()),
            effort_level_cursor: 0,
            window: m.context_window.to_string(),
            make_default: config.effective_model_selection().as_deref() == Some(id),
            focus: ModelField::Model,
            cursor_byte: m.model.len(),
            edit_id: Some(id.to_string()),
        })
    }

    fn account_id(&self) -> &str {
        &self.account_ids[self.account_idx]
    }

    /// Whether the currently-selected account still needs an api_key.
    fn account_needs_key(&self) -> bool {
        self.needs_key
            .get(self.account_idx)
            .copied()
            .unwrap_or(false)
    }

    fn fields(&self) -> Vec<ModelField> {
        let mut v = Vec::new();
        if self.edit_id.is_none() {
            v.push(ModelField::Account);
            if self.account_needs_key() {
                v.push(ModelField::ApiKey);
            }
        }
        v.push(ModelField::Model);
        v.push(ModelField::Vision);
        v.push(ModelField::Effort);
        v.push(ModelField::EffortLevels);
        v.push(ModelField::Window);
        v.push(ModelField::MakeDefault);
        v
    }

    fn advance_focus(&mut self, forward: bool) {
        let fields = self.fields();
        let cur = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = if forward {
            (cur + 1) % fields.len()
        } else {
            (cur + fields.len() - 1) % fields.len()
        };
        self.focus = fields[next];
        self.cursor_byte = self.focused_text().map(str::len).unwrap_or(0);
    }

    fn focused_text(&self) -> Option<&str> {
        match self.focus {
            ModelField::ApiKey => Some(&self.api_key),
            ModelField::Model => Some(&self.model),
            ModelField::Window => Some(&self.window),
            ModelField::Account
            | ModelField::Vision
            | ModelField::Effort
            | ModelField::EffortLevels
            | ModelField::MakeDefault => None,
        }
    }

    fn cycle_account(&mut self, forward: bool) {
        let n = self.account_ids.len();
        if n == 0 {
            return;
        }
        self.account_idx = if forward {
            (self.account_idx + 1) % n
        } else {
            (self.account_idx + n - 1) % n
        };
        // The ApiKey field appears/disappears with the account.
        if !self.fields().contains(&self.focus) {
            self.focus = ModelField::Account;
        }
    }

    fn cycle_vision(&mut self, forward: bool) {
        self.supports_vision = match (self.supports_vision, forward) {
            (None, true) => Some(true),
            (Some(true), true) => Some(false),
            (Some(false), true) => None,
            (None, false) => Some(false),
            (Some(false), false) => Some(true),
            (Some(true), false) => None,
        };
    }

    fn vision_label(&self) -> String {
        match self.supports_vision {
            None => crate::i18n::t(crate::i18n::Msg::ProviderPanelVisionAuto).into_owned(),
            Some(true) => crate::i18n::t(crate::i18n::Msg::ProviderPanelVisionEnabled).into_owned(),
            Some(false) => {
                crate::i18n::t(crate::i18n::Msg::ProviderPanelVisionDisabled).into_owned()
            }
        }
    }

    fn cycle_effort(&mut self, forward: bool) {
        // Cycle the DEFAULT value through None, "auto", then only the ENABLED
        // levels -- so a model that dropped `medium` can't take `medium` as its
        // default.
        let mut values: Vec<Option<&str>> = vec![None, Some("auto")];
        for (i, level) in rustcode_config::config::REASONING_EFFORT_LEVELS
            .iter()
            .enumerate()
        {
            if self.effort_levels[i] {
                values.push(Some(level));
            }
        }
        let current = values
            .iter()
            .position(|value| *value == self.reasoning_effort.as_deref())
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % values.len()
        } else {
            (current + values.len() - 1) % values.len()
        };
        self.reasoning_effort = values[next].map(str::to_string);
    }

    fn move_effort_cursor(&mut self, forward: bool) {
        let n = self.effort_levels.len();
        self.effort_level_cursor = if forward {
            (self.effort_level_cursor + 1) % n
        } else {
            (self.effort_level_cursor + n - 1) % n
        };
    }

    fn toggle_effort_level(&mut self) {
        let i = self.effort_level_cursor.min(self.effort_levels.len() - 1);
        self.effort_levels[i] = !self.effort_levels[i];
        // Keep the default value valid: if it now names a disabled level, drop it
        // back to the API default.
        if let Some(current) = self.reasoning_effort.clone() {
            let still_valid = current.eq_ignore_ascii_case("auto")
                || rustcode_config::config::REASONING_EFFORT_LEVELS
                    .iter()
                    .enumerate()
                    .any(|(idx, level)| {
                        self.effort_levels[idx] && level.eq_ignore_ascii_case(&current)
                    });
            if !still_valid {
                self.reasoning_effort = Some("auto".to_string());
            }
        }
    }

    /// Render the level toggles with the sub-cursor marked, e.g.
    /// ` ● low  ‹○ medium›  ● high  ● max ` (focused level in guillemets).
    ///
    /// Use text glyphs rather than emoji so each marker stays monochrome and
    /// occupies one terminal cell on the terminals supported by the TUI.
    fn effort_levels_label(&self, focused: bool) -> String {
        rustcode_config::config::REASONING_EFFORT_LEVELS
            .iter()
            .enumerate()
            .map(|(i, level)| {
                let mark = if self.effort_levels[i] { '●' } else { '○' };
                let cell = format!("{mark} {level}");
                if focused && i == self.effort_level_cursor {
                    format!("‹{cell}›")
                } else {
                    format!(" {cell} ")
                }
            })
            .collect::<String>()
    }

    fn effort_label(&self) -> String {
        match self.reasoning_effort.as_deref() {
            None => crate::i18n::t(crate::i18n::Msg::ProviderPanelVisionDisabled).into_owned(),
            Some("auto") => {
                crate::i18n::t(crate::i18n::Msg::ProviderPanelDefaultValue).into_owned()
            }
            Some(value) => value.to_string(),
        }
    }
}

enum Mode {
    List,
    Add(AddForm),
    EditAccount(EditForm),
    Model(ModelForm),
    /// Show discovered models and let user select which to add.
    DiscoveryResults {
        account_id: String,
        models: Vec<DiscoveredModel>,
        selected: Vec<bool>,
        cursor: usize, // Current cursor position in the list
    },
}

pub struct ProviderPanel {
    tab: Tab,
    selected: usize,
    mode: Mode,
    /// Search/filter query for the list (the plugin-style search box).
    query: String,
    /// UTF-8 byte cursor for the list search query.
    query_cursor_byte: usize,
    /// Whether Left/Right edit the query instead of switching list tabs.
    search_focused: bool,
    /// When set (via drilling into an account with ↵), the Models tab shows only
    /// this account's models. Cleared by Tab / Esc.
    account_filter: Option<String>,
    /// The row armed by the first Ctrl+D. A second Ctrl+D deletes only when the
    /// same logical row is still selected; every other list action disarms it.
    pending_delete: Option<(String, bool)>,
    /// Async model discovery state.
    discovery_state: DiscoveryState,
    /// Channel receiver for discovery results (polled in main loop).
    discovery_rx: Option<std::sync::mpsc::Receiver<DiscoveryResult>>,
}

/// Rows the List layout pushes before the first account/model row: the tab bar,
/// a blank, the reserved plugin search box (index 2), and a blank separator.
/// The selection offset MUST equal the number of these header pushes -- keep this
/// in lockstep with the `items.push(...)` calls at the top of the List arm in
/// [`ProviderPanel::draw`].
const LIST_HEADER_ROWS: usize = 4;

/// Virtual last row on the 账号 tab: "+ 添加自定义 provider". Not a real id, so it
/// never collides with an account; selecting it opens the add-account form.
const ADD_PROVIDER_ROW: &str = "\u{1}add-provider";
const ADD_MODEL_ROW: &str = "\u{1}add-model";
/// Virtual row on the 模型 tab: re-load the drilled-in account's model list from
/// its endpoint and open the picker. Offered only when there is exactly one
/// unambiguous target (the account the user drilled into), so Enter never has to
/// guess which provider to ask.
const REFRESH_MODELS_ROW: &str = "\u{1}refresh-models";

fn is_add_shortcut(code: &KeyCode, mods: KeyModifiers) -> bool {
    matches!(
        code,
        KeyCode::Char('a' | 'A') if mods.contains(KeyModifiers::CONTROL)
    ) || matches!(code, KeyCode::Char('\u{1}'))
}

/// Multi-select transitions of the discovery picker, kept as free functions on
/// the raw state so they can be tested without a `LoopCtx` (the panel's key
/// handler is a thin dispatcher over these). Each one is total: an out-of-range
/// cursor is a no-op rather than a panic, because the cursor and the toggle list
/// are updated by different keystrokes.
fn pick_cursor_up(cursor: &mut usize, len: usize) {
    *cursor = crate::modals::step_up(*cursor, len);
}

fn pick_cursor_down(cursor: &mut usize, len: usize) {
    *cursor = crate::modals::step_down(*cursor, len);
}

fn pick_toggle(selected: &mut [bool], cursor: usize) {
    if let Some(flag) = selected.get_mut(cursor) {
        *flag = !*flag;
    }
}

fn pick_select_all(selected: &mut [bool]) {
    for flag in selected.iter_mut() {
        *flag = true;
    }
}

fn pick_deselect_all(selected: &mut [bool]) {
    for flag in selected.iter_mut() {
        *flag = false;
    }
}

/// The chosen models in listing order. `selected` shorter than `models` (a stale
/// pick next to a refreshed listing) means the missing tail is NOT chosen --
/// zipping is deliberate, so a shorter list can never invent selections.
fn pick_chosen(models: &[DiscoveredModel], selected: &[bool]) -> Vec<DiscoveredModel> {
    models
        .iter()
        .zip(selected.iter())
        .filter(|(_, &flag)| flag)
        .map(|(model, _)| model.clone())
        .collect()
}

/// Actions that are not rows of the config: Ctrl+E / Ctrl+D must skip them.
fn is_virtual_row(id: &str) -> bool {
    id == ADD_PROVIDER_ROW || id == ADD_MODEL_ROW || id == REFRESH_MODELS_ROW
}

impl ProviderPanel {
    /// A model can be added whenever there is an account to attach it to: the
    /// drilled-into account when the 模型 tab is filtered, else any account row.
    fn can_add_model(&self, config: &Config) -> bool {
        self.account_filter.is_some() || !Self::account_ids(config).is_empty()
    }

    /// The account the refresh row would re-load, when that row is offered.
    ///
    /// Requires a drill-in (so the target is unambiguous) AND a resolvable
    /// endpoint (so the row is never a no-op). A virtual preset row has no
    /// persisted account, hence no credential to ask with -- excluded.
    fn refresh_account(&self, config: &Config) -> Option<String> {
        if self.tab != Tab::Models {
            return None;
        }
        let account_id = self.account_filter.as_deref()?;
        let account = config.provider_accounts.get(account_id)?;
        discovery_target_for_account(account, account_id)
            .is_discoverable()
            .then(|| account_id.to_string())
    }

    /// Re-load one account's model list, reporting WHY when it cannot start.
    ///
    /// Both entry points (the account row's Enter and the models-tab refresh row)
    /// go through here so they cannot drift. A request already in flight is left
    /// alone rather than replaced: the answer belongs to the account the user
    /// asked about a moment ago.
    fn start_discovery_for_account(&mut self, account_id: &str, config: &Config) -> bool {
        if !can_start_discovery(&self.discovery_state) {
            return false;
        }
        let Some(account) = config.provider_accounts.get(account_id) else {
            // A virtual preset row has no persisted account and no credential.
            self.note_discovery(
                crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryNoEndpoint {
                    account: account_id,
                })
                .into_owned(),
            );
            return false;
        };
        let target = discovery_target_for_account(account, account_id);
        if !target.is_discoverable() {
            // No base_url on the account and none from its preset: the request
            // would go nowhere. Say so instead of doing nothing visible -- a
            // silent no-op is what made the old flow look broken.
            self.note_discovery(
                crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryNoEndpoint {
                    account: account_id,
                })
                .into_owned(),
            );
            return false;
        }
        self.start_discovery(target)
    }

    /// Virtual rows appended after the real rows, in draw order. Keeping them in
    /// ONE place is what lets `selected_id` and `current_len` agree with the
    /// renderer; the old single-sentinel check had to be duplicated per row.
    fn virtual_rows(&self, config: &Config) -> Vec<String> {
        match self.tab {
            Tab::Accounts => vec![ADD_PROVIDER_ROW.to_string()],
            Tab::Models => {
                let mut rows = Vec::new();
                if self.can_add_model(config) {
                    rows.push(ADD_MODEL_ROW.to_string());
                }
                if self.refresh_account(config).is_some() {
                    rows.push(REFRESH_MODELS_ROW.to_string());
                }
                rows
            }
        }
    }

    /// The virtual rows as they are RENDERED, in display order. Derived from
    /// [`Self::virtual_rows`] so the selectable rows and the drawn rows cannot
    /// drift -- an off-by-one between them would make Enter act on a row the user
    /// is not looking at. `ids` are the real rows above them; they only drive the
    /// add row's empty-state detail.
    fn virtual_row_items(&self, config: &Config, ids: &[String]) -> Vec<(String, String)> {
        self.virtual_rows(config)
            .into_iter()
            .filter_map(|id| match id.as_str() {
                ADD_PROVIDER_ROW => Some((
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelAddAccountRow).into_owned(),
                    String::new(),
                )),
                ADD_MODEL_ROW => {
                    let detail = if ids.is_empty() {
                        if self.query.trim().is_empty() {
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelEmptyModels).into_owned()
                        } else {
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelNoMatchingModels)
                                .into_owned()
                        }
                    } else {
                        String::new()
                    };
                    Some((
                        crate::i18n::t(crate::i18n::Msg::ProviderPanelAddModelRow).into_owned(),
                        detail,
                    ))
                }
                REFRESH_MODELS_ROW => Some((
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelRefreshModelsRow).into_owned(),
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelRefreshModelsRowDetail {
                        account: &self.refresh_account(config).unwrap_or_default(),
                    })
                    .into_owned(),
                )),
                _ => None,
            })
            .collect()
    }

    /// Apply a single-line paste to the field currently being edited.
    ///
    /// Provider form values are all single-line. Normalize terminal line
    /// endings and use only the first pasted line so an accidental trailing
    /// newline cannot leak into an account id, URL, key, or model name.
    fn apply_paste_text(&mut self, text: &str) {
        let clean = text.trim().split(['\r', '\n']).next().unwrap_or("").trim();
        match &mut self.mode {
            Mode::Add(form) => match form.focus {
                FormField::ApiKey => {
                    insert_at_cursor(&mut form.api_key, &mut form.cursor_byte, clean)
                }
                FormField::BaseUrl => {
                    insert_at_cursor(&mut form.base_url, &mut form.cursor_byte, clean)
                }
                FormField::Name => insert_at_cursor(&mut form.name, &mut form.cursor_byte, clean),
                FormField::Preset => {}
            },
            Mode::EditAccount(form) => match form.focus {
                FormField::ApiKey => {
                    insert_at_cursor(&mut form.api_key, &mut form.cursor_byte, clean)
                }
                FormField::BaseUrl => {
                    insert_at_cursor(&mut form.base_url, &mut form.cursor_byte, clean)
                }
                FormField::Name | FormField::Preset => {}
            },
            Mode::Model(form) => match form.focus {
                ModelField::ApiKey => {
                    insert_at_cursor(&mut form.api_key, &mut form.cursor_byte, clean)
                }
                ModelField::Model => {
                    insert_at_cursor(&mut form.model, &mut form.cursor_byte, clean)
                }
                ModelField::Window => {
                    let digits: String = clean.chars().filter(char::is_ascii_digit).collect();
                    insert_at_cursor(&mut form.window, &mut form.cursor_byte, &digits)
                }
                ModelField::Account
                | ModelField::Vision
                | ModelField::Effort
                | ModelField::EffortLevels
                | ModelField::MakeDefault => {}
            },
            Mode::List => {
                insert_at_cursor(&mut self.query, &mut self.query_cursor_byte, clean);
                self.search_focused = true;
                self.selected = 0;
                self.pending_delete = None;
            }
            Mode::DiscoveryResults { .. } => {
                // Paste in discovery results mode does nothing
            }
        }
    }

    pub fn open() -> Self {
        Self {
            tab: Tab::Accounts,
            selected: 0,
            mode: Mode::List,
            query: String::new(),
            query_cursor_byte: 0,
            search_focused: false,
            account_filter: None,
            pending_delete: None,
            discovery_state: DiscoveryState::Idle,
            discovery_rx: None,
        }
    }

    /// The 账号 tab list: configured accounts first (new-schema, sorted by
    /// model-count DESC), then every unconfigured preset
    /// VENDOR (deepseek/openai/... -- name only) so the user can pick one and add a
    /// model to it. Pure-legacy `[providers.*]` are excluded (they show flattened
    /// on the 模型 tab); the custom-endpoint presets are reached via the trailing
    /// "＋ 添加自定义 provider" row instead.
    fn account_ids(config: &Config) -> Vec<String> {
        let accounts = config.logical_accounts();
        let models = config.logical_models();
        let mut with_count: Vec<(String, usize)> = accounts
            .keys()
            .filter(|id| config.provider_accounts.contains_key(*id))
            .map(|id| {
                let count = models.values().filter(|m| &m.account == id).count();
                (id.clone(), count)
            })
            .collect();
        with_count.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let mut ids: Vec<String> = with_count.into_iter().map(|(id, _)| id).collect();
        // Unconfigured preset vendors as quick-add rows. A vendor is only
        // quick-addable as a raw-key account when it has a concrete dispatch
        // endpoint: the compat presets are reached via the trailing custom
        // row; and presets without a default base_url (the `*-compatible`
        // presets) have nothing to dispatch against.
        for p in provider_preset::PRESETS {
            if p.default_base_url.is_none()
                || matches!(p.id, "openai-compatible" | "anthropic-compatible")
                || ids.iter().any(|i| i == p.id)
            {
                continue;
            }
            ids.push(p.id.to_string());
        }
        ids
    }

    /// Human-facing account label. Stable account ids remain the selection and
    /// persistence keys; only preset-shaped accounts inherit the preset's
    /// display name so custom account ids are never relabelled as their wire
    /// provider.
    fn account_label(config: &Config, id: &str) -> String {
        if let Some(account) = config.provider_accounts.get(id) {
            if let Some(display_name) = account
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
            {
                return display_name.to_string();
            }
            if account.provider != id {
                return id.to_string();
            }
        }
        provider_preset::preset(id)
            .map(|preset| preset.display_name.to_string())
            .unwrap_or_else(|| id.to_string())
    }

    /// Model selection ids grouped by account (matches the /model order).
    fn model_ids(config: &Config) -> Vec<String> {
        let models = config.logical_models();
        let mut ids: Vec<String> = models.keys().cloned().collect();
        ids.sort_by(|a, b| {
            let key = |id: &String| {
                models
                    .get(id)
                    .map(|m| (m.account.clone(), m.model.clone()))
                    .unwrap_or_else(|| (id.clone(), String::new()))
            };
            key(a).cmp(&key(b))
        });
        ids
    }

    /// Ids for the current tab, filtered by the search query (matched against
    /// the id, vendor/preset, and model name).
    fn filtered_ids(&self, config: &Config) -> Vec<String> {
        let mut all = match self.tab {
            Tab::Accounts => Self::account_ids(config),
            Tab::Models => Self::model_ids(config),
        };
        let models = config.logical_models();
        // Drill-in: on the Models tab, restrict to a single account when the
        // user entered via ↵ on an account row (Tab / Esc clears it).
        if self.tab == Tab::Models {
            if let Some(acct) = &self.account_filter {
                all.retain(|id| models.get(id).is_some_and(|m| &m.account == acct));
            }
        }
        if self.query.trim().is_empty() {
            return all;
        }
        let q = self.query.to_lowercase();
        let accounts = config.logical_accounts();
        all.into_iter()
            .filter(|id| {
                if id.to_lowercase().contains(&q) {
                    return true;
                }
                match self.tab {
                    Tab::Accounts => accounts
                        .get(id)
                        .is_some_and(|a| a.provider.to_lowercase().contains(&q)),
                    Tab::Models => models.get(id).is_some_and(|m| {
                        m.model.to_lowercase().contains(&q) || m.account.to_lowercase().contains(&q)
                    }),
                }
            })
            .collect()
    }

    /// Switch to `tab`, resetting the selection and clearing both filters (the
    /// search query and the account drill-in) so the destination shows its full
    /// list.
    fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.selected = 0;
        self.query.clear();
        self.query_cursor_byte = 0;
        self.search_focused = false;
        self.account_filter = None;
        self.pending_delete = None;
    }

    /// Keep the panel open after creating an account and drill directly into
    /// that account's model list.
    fn show_models_for_account(&mut self, account_id: &str) {
        self.tab = Tab::Models;
        self.selected = 0;
        self.mode = Mode::List;
        self.query.clear();
        self.query_cursor_byte = 0;
        self.search_focused = false;
        self.account_filter = Some(account_id.to_string());
        self.pending_delete = None;
    }

    /// Start async model discovery for one already-resolved target.
    /// Returns true if discovery was started, false when a request is already in
    /// flight or the target has no endpoint to probe.
    fn start_discovery(&mut self, target: DiscoveryTarget) -> bool {
        if !can_start_discovery(&self.discovery_state) || !target.is_discoverable() {
            return false;
        }

        // Create channel for async result
        let (tx, rx) = std::sync::mpsc::channel();

        // Set pending state
        self.discovery_state = target.pending_state();
        self.discovery_rx = Some(rx);

        // Spawn background task
        let DiscoveryTarget {
            account_id,
            provider_type,
            base_url,
            api_key,
            user_agent,
            proxy,
            skip_tls_verify,
        } = target;

        std::thread::spawn(move || {
            let result = Self::discover_models_sync(
                &base_url,
                api_key.as_deref(),
                provider_type,
                user_agent.as_deref(),
                proxy.as_deref(),
                skip_tls_verify,
            );
            let _ = tx.send(DiscoveryResult { account_id, result });
        });

        true
    }

    /// Record why the last discovery attempt loaded nothing, so the panel does not
    /// look like it silently ignored the request.
    fn note_discovery(&mut self, message: String) {
        self.discovery_state = DiscoveryState::Notice { message };
    }

    /// Synchronous model discovery (runs in background thread).
    ///
    /// The transport settings come from the ACCOUNT, mirroring the daemon's
    /// discovery handler: a self-signed gateway, a corporate proxy or a UA
    /// allowlist that works from the WebUI must work here too. The client comes
    /// from the SHARED egress factory's blocking flavour
    /// (`egress::build_blocking_http_client`), so this path gets the same trust
    /// roots, proxy policy and timeout discipline as every other egress call
    /// site instead of a hand-rolled builder that drifted from them.
    #[allow(clippy::too_many_arguments)]
    fn discover_models_sync(
        base_url: &str,
        api_key: Option<&str>,
        provider_type: &str,
        user_agent: Option<&str>,
        proxy: Option<&str>,
        skip_tls_verify: bool,
    ) -> Result<Vec<DiscoveredModel>, String> {
        // One-shot discovery budget (historic 30s) on top of the shared policy.
        let spec = rustcode_capabilities::egress::HttpClientSpec {
            user_agent: user_agent
                .map(str::trim)
                .filter(|a| !a.is_empty())
                .map(str::to_string),
            proxy: proxy
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(str::to_string),
            skip_tls_verify,
            request_timeout: Some(std::time::Duration::from_secs(30)),
            ..rustcode_capabilities::egress::HttpClientSpec::default()
        };
        let client = rustcode_capabilities::egress::build_blocking_http_client(&spec)
            .map_err(|e| format!("HTTP client error: {}", e.detail()))?;

        // Construct discovery URL
        let suffix = if provider_type == "ollama" {
            "/api/tags"
        } else {
            "/models"
        };
        let url = format!("{}{}", base_url.trim_end_matches('/'), suffix);

        // Build request
        let mut request = client.get(&url).header("accept", "application/json");

        if let Some(key) = api_key {
            if !key.trim().is_empty() {
                request = request.bearer_auth(key.trim());
            }
        }

        // Execute request (blocking)
        let response = request.send().map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status().as_u16()));
        }

        let body = response
            .text()
            .map_err(|e| format!("Read body failed: {e}"))?;

        // Parse response
        Self::parse_models_response(provider_type, &body)
    }

    /// Parse models from provider response.
    fn parse_models_response(
        provider_type: &str,
        body: &str,
    ) -> Result<Vec<DiscoveredModel>, String> {
        let json: serde_json::Value =
            serde_json::from_str(body).map_err(|e| format!("JSON parse error: {e}"))?;

        let models = if provider_type == "ollama" {
            // Ollama: { models: [{ name: "...", model: "..." }] }
            json["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m["model"].as_str().or_else(|| m["name"].as_str())?;
                            Some(DiscoveredModel {
                                id: id.to_string(),
                                name: m["name"].as_str().map(String::from),
                                context_window: None,
                                max_tokens: None,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            // OpenAI: { data: [{ id: "...", name: "...", context_length: N }] }
            json["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m["id"].as_str()?;
                            Some(DiscoveredModel {
                                id: id.to_string(),
                                name: m["name"].as_str().map(String::from),
                                context_window: m["context_length"]
                                    .as_u64()
                                    .or_else(|| m["context_window"].as_u64())
                                    .map(|v| v as usize),
                                max_tokens: m["max_output_tokens"]
                                    .as_u64()
                                    .or_else(|| m["max_tokens"].as_u64())
                                    .map(|v| v as usize),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(models)
    }

    /// Poll discovery results (call from main loop). The outcome is handed back to
    /// the caller and the state left re-tryable -- parking it in a terminal variant
    /// is what used to block every later attempt for the rest of the panel's life.
    fn poll_discovery(&mut self) -> Option<(String, Result<Vec<DiscoveredModel>, String>)> {
        let rx = self.discovery_rx.take()?;
        let outcome = match rx.try_recv() {
            Ok(outcome) => outcome,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.discovery_rx = Some(rx);
                return None;
            }
            // The worker died without answering (panic / killed thread). Report it
            // instead of leaving `Pending` behind, which would refuse every later
            // discovery request.
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                let account_id =
                    match std::mem::replace(&mut self.discovery_state, DiscoveryState::Idle) {
                        DiscoveryState::Pending { account_id, .. } => account_id,
                        _ => String::new(),
                    };
                return Some((
                    account_id,
                    Err(
                        crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryWorkerStopped)
                            .into_owned(),
                    ),
                ));
            }
        };
        self.discovery_state = DiscoveryState::Idle;
        Some((outcome.account_id, outcome.result))
    }

    /// Arm a row on the first Ctrl+D and confirm it on the second. Returning
    /// true means the caller should perform the destructive operation.
    fn confirm_double_delete(&mut self, id: &str, is_account: bool) -> bool {
        let target = (id.to_string(), is_account);
        if self.pending_delete.as_ref() == Some(&target) {
            self.pending_delete = None;
            true
        } else {
            self.pending_delete = Some(target);
            false
        }
    }

    /// Selectable row count, including the trailing virtual rows on both tabs.
    fn current_len(&self, config: &Config) -> usize {
        self.filtered_ids(config).len() + self.virtual_rows(config).len()
    }

    fn selected_id(&self, config: &Config) -> Option<String> {
        let ids = self.filtered_ids(config);
        // Virtual rows sit just past the real ones. Index and count both come from
        // `virtual_rows`, so the mapping can never drift from what is rendered.
        if self.selected >= ids.len() {
            return self
                .virtual_rows(config)
                .get(self.selected - ids.len())
                .cloned();
        }
        ids.get(self.selected).cloned()
    }

    fn begin_add_for_current_tab(&mut self, config: &Config) {
        self.pending_delete = None;
        if self.tab == Tab::Models && !self.can_add_model(config) {
            return;
        }
        self.mode = match self.tab {
            Tab::Accounts => Mode::Add(AddForm::new()),
            Tab::Models => ModelForm::new_add(config, self.account_filter.as_deref())
                .map(Mode::Model)
                .unwrap_or_else(|| Mode::Add(AddForm::new())),
        };
    }

    /// Persist the add form as one provider ACCOUNT (no model -- models are added
    /// on the 模型 tab). Returns the account's discovery target when saved, so the
    /// caller can drill into its model list AND auto-load that list; `None` keeps
    /// the add form open.
    fn save_add(
        &self,
        form: &AddForm,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Option<DiscoveryTarget> {
        let preset = form.preset();
        // A fully-custom provider requires a name (it becomes the account id).
        let base_id = sanitize_account_name(form.name.trim());
        if base_id.is_empty() {
            return None;
        }
        // base_url is pre-filled with the preset default and editable. Persist
        // only a genuine override; blank + no preset default = missing endpoint.
        let base_url = {
            let b = form.base_url.trim();
            if b.is_empty() {
                preset.default_base_url?;
                None
            } else if Some(b) == preset.default_base_url {
                None // equals the preset default -- keep config clean
            } else {
                Some(b.to_string())
            }
        };
        let account = ProviderAccountConfig {
            provider: preset.id.to_string(),
            display_name: None,
            api_key: {
                let k = form.api_key.trim();
                (!k.is_empty()).then(|| k.to_string())
            },
            base_url,
            user_agent: None,
            skip_tls_verify: false,
            extra_headers: None,
            proxy: None,
            enterprise_url: None,
            ephemeral: false,
        };
        update_config_and_reload(
            ctx,
            renderer,
            ConfigReloadSelection::KeepCurrent,
            move |persisted| {
                let account_id = unique_account_id(&base_id, persisted);
                // Resolve discovery from the values being committed, BEFORE the
                // account is moved into the map -- `ctx.config` still holds the
                // pre-save snapshot here (see [`DiscoveryTarget`]).
                let target = discovery_target_for_account(&account, &account_id);
                persisted.provider_accounts.insert(account_id, account);
                Ok(target)
            },
            |target: &DiscoveryTarget| {
                crate::i18n::t(crate::i18n::Msg::ProviderAdded {
                    name: &target.account_id,
                })
                .into_owned()
            },
        )
    }

    /// Build an edit form pre-filled from the selected account.
    fn open_edit(config: &Config, id: &str) -> EditForm {
        let is_legacy =
            !config.provider_accounts.contains_key(id) && config.providers.contains_key(id);
        let configured_account = config.provider_accounts.get(id);
        let virtual_preset = !is_legacy && configured_account.is_none();
        let (base_url, provider) = if is_legacy {
            let p = config.providers.get(id);
            (
                p.and_then(|p| p.base_url.clone()),
                p.map(|p| p.provider_type.clone()).unwrap_or_default(),
            )
        } else {
            (
                configured_account.and_then(|a| a.base_url.clone()),
                configured_account
                    .map(|a| a.provider.clone())
                    .unwrap_or_else(|| id.to_string()),
            )
        };
        let effective_base_url = base_url.or_else(|| {
            provider_preset::preset(&provider)
                .and_then(|preset| preset.default_base_url)
                .map(str::to_string)
        });
        // Map the stored provider to a protocol toggle (OpenAI/Anthropic/Ollama).
        // original == preset so a no-op edit leaves the real stored provider
        // (e.g. "deepseek"/"openai"/"ollama") untouched (see save_edit's guard).
        let preset_idx =
            protocol_preset_idx(provider_preset::preset_or_compatible(&provider).provider_type);
        EditForm {
            id: id.to_string(),
            is_legacy,
            materialize_provider: virtual_preset.then_some(provider),
            preset_idx,
            original_preset_idx: preset_idx,
            protocol_locked: virtual_preset,
            api_key: String::new(),
            base_url: effective_base_url.clone().unwrap_or_default(),
            // A curated preset row exposes only the endpoint first.
            focus: if virtual_preset {
                FormField::BaseUrl
            } else {
                FormField::Preset
            },
            cursor_byte: if virtual_preset {
                effective_base_url.as_deref().map(str::len).unwrap_or(0)
            } else {
                0
            },
        }
    }

    /// Apply an account edit in place (blank fields keep the current value), save,
    /// and return the committed account's discovery target so the caller can
    /// auto-load the (possibly repointed) model list. See [`DiscoveryTarget`] for
    /// why the target is resolved from the commit and not from `ctx.config`.
    fn save_edit(
        &self,
        form: &EditForm,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Option<DiscoveryTarget> {
        let form = form.clone();
        let account_id = form.id.clone();
        update_config_and_reload(
            ctx,
            renderer,
            ConfigReloadSelection::KeepCurrent,
            move |persisted| {
                if form.materialize_provider.is_none()
                    && !persisted.provider_accounts.contains_key(&form.id)
                    && !persisted.providers.contains_key(&form.id)
                {
                    anyhow::bail!("provider account {:?} changed; reopen /provider", form.id);
                }
                Self::apply_account_edit(&form, persisted);
                Ok(Self::discovery_target_after_edit(&form, persisted))
            },
            |_| {
                crate::i18n::t(crate::i18n::Msg::ProviderUpdated { name: &account_id }).into_owned()
            },
        )
    }

    /// Discovery target of a just-edited account. The committed account row wins
    /// because it carries the MERGED credential -- a blank api_key edit keeps the
    /// saved key, so the form alone cannot reconstruct it. A pure-legacy
    /// `[providers.*]` row has no account entry and falls back to its own fields.
    fn discovery_target_after_edit(form: &EditForm, persisted: &Config) -> DiscoveryTarget {
        if let Some(account) = persisted.provider_accounts.get(&form.id) {
            return discovery_target_for_account(account, &form.id);
        }
        let preset = form.preset();
        let (provider_type, base_url, api_key, user_agent, proxy, skip_tls_verify) =
            match persisted.provider_accounts.get(&form.id) {
                // Prefer the COMMITTED account: it carries the transport settings
                // (UA / proxy / TLS) the user just saved.
                Some(account) => (
                    provider_preset::preset_or_compatible(&account.provider).provider_type,
                    account.base_url.clone(),
                    account.api_key.clone(),
                    account.user_agent.clone(),
                    account.proxy.clone().filter(|p| !p.trim().is_empty()),
                    account.skip_tls_verify,
                ),
                None => match persisted.providers.get(&form.id) {
                    Some(provider) => (
                        provider_preset::preset_or_compatible(&provider.provider_type)
                            .provider_type,
                        provider.base_url.clone(),
                        provider.resolved_api_key(),
                        provider.user_agent.clone(),
                        provider.proxy.clone().filter(|p| !p.trim().is_empty()),
                        provider.skip_tls_verify,
                    ),
                    // Unreachable after a successful save (the store closure bails
                    // first); keep the form's own protocol so the caller still gets
                    // a usable target.
                    None => (preset.provider_type, None, None, None, None, false),
                },
            };
        DiscoveryTarget {
            account_id: form.id.clone(),
            provider_type: discovery_protocol(provider_type),
            base_url: base_url
                .or_else(|| preset.default_base_url.map(str::to_string))
                .unwrap_or_default(),
            api_key,
            user_agent,
            proxy,
            skip_tls_verify,
        }
    }

    fn apply_account_edit(form: &EditForm, desired: &mut Config) {
        let api_key = form.api_key.trim();
        let base_url = form.base_url.trim();
        let preset = form.preset();
        // Only rewrite the vendor when the user actually changed it (and the
        // protocol isn't locked) -- a no-op edit must not normalize a
        // `deepseek`/custom provider to the fallback preset. When the new
        // preset is keyless, drop any stale api_key.
        let vendor_changed = !form.protocol_locked && form.preset_idx != form.original_preset_idx;
        let clear_key =
            vendor_changed && matches!(preset.auth_kind, provider_preset::AuthKind::None);
        if form.is_legacy {
            if let Some(p) = desired.providers.get_mut(&form.id) {
                if vendor_changed {
                    // Legacy dispatches on the wire `type`; store the preset's wire.
                    p.provider_type = preset.provider_type.wire().to_string();
                }
                if clear_key {
                    p.api_key = None;
                } else if !api_key.is_empty() {
                    p.api_key = Some(api_key.to_string());
                }
                if !base_url.is_empty() {
                    p.base_url = Some(base_url.to_string());
                }
            }
        } else if let Some(a) = desired.provider_accounts.get_mut(&form.id) {
            if vendor_changed {
                // New-schema stores the preset id.
                a.provider = preset.id.to_string();
            }
            if clear_key {
                a.api_key = None;
            } else if !api_key.is_empty() {
                a.api_key = Some(api_key.to_string());
            }
            if !base_url.is_empty() {
                let default = provider_preset::preset(&a.provider).and_then(|p| p.default_base_url);
                a.base_url = (Some(base_url) != default).then(|| base_url.to_string());
            }
        } else if let Some(original_provider) = &form.materialize_provider {
            let provider = if vendor_changed {
                preset.id.to_string()
            } else {
                original_provider.clone()
            };
            let provider_default =
                provider_preset::preset(&provider).and_then(|p| p.default_base_url);
            desired.provider_accounts.insert(
                form.id.clone(),
                ProviderAccountConfig {
                    provider,
                    display_name: None,
                    api_key: (!api_key.is_empty()).then(|| api_key.to_string()),
                    base_url: (!base_url.is_empty() && Some(base_url) != provider_default)
                        .then(|| base_url.to_string()),
                    user_agent: None,
                    skip_tls_verify: false,
                    extra_headers: None,
                    proxy: None,
                    enterprise_url: None,
                    ephemeral: false,
                },
            );
        }
    }

    fn is_virtual_account_row(config: &Config, id: &str) -> bool {
        !config.provider_accounts.contains_key(id) && !config.providers.contains_key(id)
    }

    /// Add a model to an existing account, or edit an existing model's wire name
    /// + window in place (preserving its other fields), then save.
    fn save_model(&self, form: &ModelForm, ctx: &mut LoopCtx, renderer: &mut dyn Renderer) -> bool {
        let account_id = form.account_id().to_string();
        let model_name = form.model.trim().to_string();
        let supports_vision = form.supports_vision;
        let reasoning_effort_levels = effort_levels_to_config(form.effort_levels);
        // Keep the persisted default within the enabled levels. The interactive
        // toggle already self-heals, but a value LOADED from a hand-edited config
        // (medium selected while medium is off) would otherwise be saved verbatim.
        let reasoning_effort = if form.reasoning_effort.as_deref().is_some_and(|v| {
            !v.eq_ignore_ascii_case("auto")
                && rustcode_config::config::clamp_effort_to_levels(
                    Some(v),
                    reasoning_effort_levels.as_deref(),
                )
                .is_none()
        }) {
            Some("auto".to_string())
        } else {
            form.reasoning_effort.clone()
        };
        if model_name.is_empty() {
            return false;
        }
        let requested_window = form
            .window
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|window| *window > 0);
        let edit_id = form.edit_id.clone();
        let needs_key = form.edit_id.is_none() && form.account_needs_key();
        let api_key = form.api_key.trim().to_string();
        let make_default = form.make_default;
        let was_virtual_account = Self::is_virtual_account_row(&ctx.config, &account_id);
        let selection = if make_default {
            ConfigReloadSelection::FollowPersisted
        } else {
            ConfigReloadSelection::KeepCurrent
        };
        update_config_and_reload(
            ctx,
            renderer,
            selection,
            move |persisted| {
                if edit_id.is_none()
                    && !persisted.provider_accounts.contains_key(&account_id)
                    && !persisted.providers.contains_key(&account_id)
                {
                    if !was_virtual_account {
                        anyhow::bail!("provider account {account_id:?} changed; reopen /provider");
                    }
                    let preset = provider_preset::preset_or_compatible(&account_id);
                    persisted.provider_accounts.insert(
                        account_id.clone(),
                        ProviderAccountConfig {
                            provider: account_id.clone(),
                            display_name: None,
                            api_key: None,
                            base_url: preset.default_base_url.map(str::to_string),
                            user_agent: None,
                            skip_tls_verify: false,
                            extra_headers: None,
                            proxy: None,
                            enterprise_url: None,
                            ephemeral: false,
                        },
                    );
                }

                let preset_id = persisted
                    .logical_accounts()
                    .get(&account_id)
                    .map(|account| account.provider.clone())
                    .unwrap_or_else(|| account_id.clone());
                let wire = provider_preset::preset_or_compatible(&preset_id)
                    .provider_type
                    .wire();
                let context_window = requested_window.unwrap_or_else(|| {
                    rustcode_config::config::provider::default_context_window_for(wire)
                });

                let selection_id = if let Some(id) = &edit_id {
                    if let Some(model) = persisted.models.get_mut(id) {
                        model.model = model_name.clone();
                        model.supports_vision = supports_vision;
                        model.reasoning_effort = reasoning_effort.clone();
                        model.reasoning_effort_levels = reasoning_effort_levels.clone();
                        model.context_window = context_window;
                    } else if let Some(provider) = persisted.providers.get_mut(id) {
                        provider.model = model_name.clone();
                        provider.supports_vision = supports_vision;
                        provider.reasoning_effort = reasoning_effort.clone();
                        provider.reasoning_effort_levels = reasoning_effort_levels.clone();
                        provider.context_window = context_window;
                    } else {
                        anyhow::bail!("model {id:?} changed; reopen /provider");
                    }
                    id.clone()
                } else {
                    let base = format!("{account_id}/{model_name}");
                    let model_id = if persisted.models.contains_key(&base)
                        || persisted.providers.contains_key(&base)
                    {
                        (2..)
                            .map(|n| format!("{base}-{n}"))
                            .find(|candidate| {
                                !persisted.models.contains_key(candidate)
                                    && !persisted.providers.contains_key(candidate)
                            })
                            .unwrap_or(base)
                    } else {
                        base
                    };
                    persisted.models.insert(
                        model_id.clone(),
                        ModelProfileConfig {
                            model_mapping: rustcode_config::config::provider::ModelMapping::default(
                            ),
                            account: account_id.clone(),
                            model: model_name.clone(),
                            display_name: None,
                            system_prompt: None,
                            supports_vision,
                            context_window,
                            max_tokens: None,
                            capable_model: None,
                            retry_max_attempts: None,
                            thinking_type: None,
                            thinking_keep: None,
                            reasoning_history: None,
                            reasoning_effort: reasoning_effort.clone(),
                            reasoning_effort_levels: reasoning_effort_levels.clone(),
                            thinking_enabled: None,
                            thinking_budget: None,
                            // A newly added model starts with no chain.
                            fallback: Vec::new(),
                        },
                    );
                    model_id
                };

                if needs_key && !api_key.is_empty() {
                    if let Some(account) = persisted.provider_accounts.get_mut(&account_id) {
                        account.api_key = Some(api_key);
                    }
                }
                if make_default {
                    persisted.default_model = Some(selection_id.clone());
                }
                Ok(selection_id)
            },
            |selection_id| {
                crate::i18n::t(crate::i18n::Msg::ProviderPanelModelSaved {
                    model: selection_id,
                })
                .into_owned()
            },
        )
        .is_some()
    }

    /// Delete the account (and its models) or a single model, then save.
    fn commit_delete(
        &self,
        id: &str,
        is_account: bool,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> bool {
        let active_selection = ctx.config.effective_model_selection();
        let deletes_active = if is_account {
            active_selection.as_deref().is_some_and(|selection| {
                ctx.config
                    .resolve_model(Some(selection))
                    .is_ok_and(|resolved| resolved.account_id == id)
            })
        } else {
            active_selection.as_deref() == Some(id)
        };
        let id = id.to_string();
        let selection = if deletes_active {
            ConfigReloadSelection::FollowPersisted
        } else {
            ConfigReloadSelection::KeepCurrent
        };
        let message_id = id.clone();
        update_config_and_reload(
            ctx,
            renderer,
            selection,
            move |persisted| {
                if is_account {
                    persisted.provider_accounts.remove(&id);
                    persisted.providers.remove(&id);
                    persisted.models.retain(|_, model| model.account != id);
                } else {
                    persisted.models.remove(&id);
                    persisted.providers.remove(&id);
                }
                // Clear a now-dangling default (both the canonical
                // `default_model` and legacy `default_provider`).
                if persisted
                    .default_model
                    .as_deref()
                    .is_some_and(|default| persisted.resolve_model(Some(default)).is_err())
                {
                    persisted.default_model = None;
                }
                if persisted
                    .resolve_model(Some(&persisted.default_provider))
                    .is_err()
                {
                    persisted.default_provider.clear();
                }
                Ok(())
            },
            |_| {
                crate::i18n::t(crate::i18n::Msg::ProviderDeleted { name: &message_id }).into_owned()
            },
        )
        .is_some()
    }
}

/// Selection-id plan for one discovery batch: the ids still free in `config`,
/// each paired with the model it will persist. Taken ids are skipped so re-running
/// discovery on an account is idempotent instead of an overwrite.
fn plan_discovered_models(
    account_id: &str,
    models: &[DiscoveredModel],
    config: &Config,
) -> Vec<(String, DiscoveredModel)> {
    models
        .iter()
        .filter_map(|model| {
            let selection_id = format!("{account_id}/{}", model.id);
            let taken = config.models.contains_key(&selection_id)
                || config.providers.contains_key(&selection_id);
            (!taken).then(|| (selection_id, model.clone()))
        })
        .collect()
}

/// Persist a discovery selection as model profiles under `account_id`, through the
/// same CAS path as every other provider write. Returns true when the config was
/// committed (a refused commit is rendered by `update_config_and_reload`).
///
/// The panel owns this write: `Modal::poll_background` only reports "needs redraw"
/// and exposes no channel for the event loop to read the selection, so the old
/// "return to list mode, models will be added by event loop" hand-off had no
/// consumer -- a confirmed selection was silently dropped.
fn commit_discovered_models(
    account_id: &str,
    models: &[DiscoveredModel],
    ctx: &mut LoopCtx,
    renderer: &mut dyn Renderer,
) -> bool {
    let account_id = account_id.to_string();
    let models = models.to_vec();
    update_config_and_reload(
        ctx,
        renderer,
        ConfigReloadSelection::KeepCurrent,
        move |persisted| {
            let preset_id = persisted
                .logical_accounts()
                .get(&account_id)
                .map(|account| account.provider.clone())
                .unwrap_or_else(|| account_id.clone());
            let wire = provider_preset::preset_or_compatible(&preset_id)
                .provider_type
                .wire();
            let default_window =
                rustcode_config::config::provider::default_context_window_for(wire);
            let plan = plan_discovered_models(&account_id, &models, persisted);
            let added = plan.len();
            for (selection_id, model) in plan {
                persisted.models.insert(
                    selection_id,
                    ModelProfileConfig {
                        account: account_id.clone(),
                        model: model.id,
                        display_name: model.name,
                        model_mapping: rustcode_config::config::provider::ModelMapping::default(),
                        system_prompt: None,
                        supports_vision: None,
                        context_window: model.context_window.unwrap_or(default_window),
                        max_tokens: model.max_tokens,
                        capable_model: None,
                        thinking_type: None,
                        thinking_keep: None,
                        reasoning_history: None,
                        reasoning_effort: None,
                        reasoning_effort_levels: None,
                        thinking_enabled: None,
                        thinking_budget: None,
                        retry_max_attempts: None,
                        // Discovered models start with no chain.
                        fallback: Vec::new(),
                    },
                );
            }
            Ok(added)
        },
        |added| {
            crate::i18n::t(crate::i18n::Msg::ProviderPanelModelsAdded { count: *added })
                .into_owned()
        },
    )
    .is_some()
}

impl Modal for ProviderPanel {
    fn handle_key(
        &mut self,
        code: KeyCode,
        mods: KeyModifiers,
        buf: &mut Buffer,
        state: &mut UiState,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Result<ModalAction> {
        // ^H/^? backspace-delete aliases are normalized upstream at the modal-dispatch
        // boundary (see `key_action::normalize_edit_key`), so `code`/`mods` here already
        // carry Backspace/Delete on terminals that emit those chords.

        // ── Add form ──
        if let Mode::Add(form) = &mut self.mode {
            match code {
                KeyCode::Esc => {
                    self.mode = Mode::List;
                }
                KeyCode::Tab | KeyCode::Down => form.advance_focus(true),
                KeyCode::BackTab | KeyCode::Up => form.advance_focus(false),
                KeyCode::Left if form.focus == FormField::Preset => form.cycle_preset(false),
                KeyCode::Right if form.focus == FormField::Preset => form.cycle_preset(true),
                KeyCode::Left => {
                    if let Some(text) = form.focused_text() {
                        form.cursor_byte = previous_grapheme_boundary(text, form.cursor_byte);
                    }
                }
                KeyCode::Right => {
                    if let Some(text) = form.focused_text() {
                        form.cursor_byte = next_grapheme_boundary(text, form.cursor_byte);
                    }
                }
                KeyCode::Home => form.cursor_byte = 0,
                KeyCode::End => {
                    form.cursor_byte = form.focused_text().map(str::len).unwrap_or(0);
                }
                KeyCode::Char(c) if !mods.contains(KeyModifiers::CONTROL) => match form.focus {
                    FormField::Name => insert_at_cursor(
                        &mut form.name,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    FormField::BaseUrl => insert_at_cursor(
                        &mut form.base_url,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    FormField::ApiKey => insert_at_cursor(
                        &mut form.api_key,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    _ => {}
                },
                KeyCode::Backspace => match form.focus {
                    FormField::Name => backspace_at_cursor(&mut form.name, &mut form.cursor_byte),
                    FormField::BaseUrl => {
                        backspace_at_cursor(&mut form.base_url, &mut form.cursor_byte)
                    }
                    FormField::ApiKey => {
                        backspace_at_cursor(&mut form.api_key, &mut form.cursor_byte)
                    }
                    _ => {}
                },
                KeyCode::Delete => match form.focus {
                    FormField::Name => delete_at_cursor(&mut form.name, &mut form.cursor_byte),
                    FormField::BaseUrl => {
                        delete_at_cursor(&mut form.base_url, &mut form.cursor_byte)
                    }
                    FormField::ApiKey => delete_at_cursor(&mut form.api_key, &mut form.cursor_byte),
                    _ => {}
                },
                KeyCode::Enter => {
                    let form = form.clone();
                    if let Some(target) = self.save_add(&form, ctx, renderer) {
                        // Auto-load the new account's model list. Discovery runs on
                        // the target resolved DURING the save because `ctx.config`
                        // still holds the pre-save snapshot here (the provider
                        // reload the save staged lands on a later event-loop turn).
                        let account_id = target.account_id.clone();
                        self.start_discovery(target);
                        self.show_models_for_account(&account_id);
                    } else {
                        // Save refused (missing endpoint): keep editing.
                        self.mode = Mode::Add(form);
                    }
                }
                _ => {}
            }
            self.draw(buf, state, ctx, renderer);
            return Ok(ModalAction::Continue);
        }

        // ── Edit account ──
        if let Mode::EditAccount(form) = &mut self.mode {
            match code {
                KeyCode::Esc => self.mode = Mode::List,
                KeyCode::Tab | KeyCode::Down => form.advance_focus(true),
                KeyCode::BackTab | KeyCode::Up => form.advance_focus(false),
                KeyCode::Left if form.focus == FormField::Preset => form.cycle_preset(false),
                KeyCode::Right if form.focus == FormField::Preset => form.cycle_preset(true),
                KeyCode::Left => {
                    if let Some(text) = form.focused_text() {
                        form.cursor_byte = previous_grapheme_boundary(text, form.cursor_byte);
                    }
                }
                KeyCode::Right => {
                    if let Some(text) = form.focused_text() {
                        form.cursor_byte = next_grapheme_boundary(text, form.cursor_byte);
                    }
                }
                KeyCode::Home => form.cursor_byte = 0,
                KeyCode::End => {
                    form.cursor_byte = form.focused_text().map(str::len).unwrap_or(0);
                }
                KeyCode::Char(c) if !mods.contains(KeyModifiers::CONTROL) => match form.focus {
                    FormField::ApiKey => insert_at_cursor(
                        &mut form.api_key,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    FormField::BaseUrl => insert_at_cursor(
                        &mut form.base_url,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    _ => {}
                },
                KeyCode::Backspace => match form.focus {
                    FormField::ApiKey => {
                        backspace_at_cursor(&mut form.api_key, &mut form.cursor_byte)
                    }
                    FormField::BaseUrl => {
                        backspace_at_cursor(&mut form.base_url, &mut form.cursor_byte)
                    }
                    _ => {}
                },
                KeyCode::Delete => match form.focus {
                    FormField::ApiKey => delete_at_cursor(&mut form.api_key, &mut form.cursor_byte),
                    FormField::BaseUrl => {
                        delete_at_cursor(&mut form.base_url, &mut form.cursor_byte)
                    }
                    _ => {}
                },
                KeyCode::Enter => {
                    let form = form.clone();
                    if let Some(target) = self.save_edit(&form, ctx, renderer) {
                        // Mirror the add flow: land on this account's model list and
                        // auto-load it. The edit may have repointed the endpoint or
                        // credential, so discovery uses the committed target rather
                        // than anything read back from the not-yet-reloaded config.
                        let account_id = target.account_id.clone();
                        self.start_discovery(target);
                        self.show_models_for_account(&account_id);
                    } else {
                        // Save refused (managed account / stale config): keep editing.
                        self.mode = Mode::EditAccount(form);
                    }
                }
                _ => {}
            }
            self.draw(buf, state, ctx, renderer);
            return Ok(ModalAction::Continue);
        }

        // ── Add / edit model ──
        if let Mode::Model(form) = &mut self.mode {
            match code {
                KeyCode::Esc => self.mode = Mode::List,
                KeyCode::Tab | KeyCode::Down => form.advance_focus(true),
                KeyCode::BackTab | KeyCode::Up => form.advance_focus(false),
                KeyCode::Left if form.focus == ModelField::Account => form.cycle_account(false),
                KeyCode::Right if form.focus == ModelField::Account => form.cycle_account(true),
                KeyCode::Left if form.focus == ModelField::Vision => form.cycle_vision(false),
                KeyCode::Right if form.focus == ModelField::Vision => form.cycle_vision(true),
                KeyCode::Left if form.focus == ModelField::Effort => form.cycle_effort(false),
                KeyCode::Right if form.focus == ModelField::Effort => form.cycle_effort(true),
                KeyCode::Left if form.focus == ModelField::EffortLevels => {
                    form.move_effort_cursor(false)
                }
                KeyCode::Right if form.focus == ModelField::EffortLevels => {
                    form.move_effort_cursor(true)
                }
                KeyCode::Left => {
                    if let Some(text) = form.focused_text() {
                        form.cursor_byte = previous_grapheme_boundary(text, form.cursor_byte);
                    }
                }
                KeyCode::Right => {
                    if let Some(text) = form.focused_text() {
                        form.cursor_byte = next_grapheme_boundary(text, form.cursor_byte);
                    }
                }
                KeyCode::Home => form.cursor_byte = 0,
                KeyCode::End => {
                    form.cursor_byte = form.focused_text().map(str::len).unwrap_or(0);
                }
                KeyCode::Char(' ') if form.focus == ModelField::Vision => {
                    form.cycle_vision(true);
                }
                KeyCode::Char(' ') if form.focus == ModelField::Effort => {
                    form.cycle_effort(true);
                }
                KeyCode::Char(' ') if form.focus == ModelField::EffortLevels => {
                    form.toggle_effort_level();
                }
                KeyCode::Char(' ') if form.focus == ModelField::MakeDefault => {
                    form.make_default = !form.make_default;
                }
                KeyCode::Char(c) if !mods.contains(KeyModifiers::CONTROL) => match form.focus {
                    ModelField::ApiKey => insert_at_cursor(
                        &mut form.api_key,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    ModelField::Model => insert_at_cursor(
                        &mut form.model,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    ModelField::Window if c.is_ascii_digit() => insert_at_cursor(
                        &mut form.window,
                        &mut form.cursor_byte,
                        c.encode_utf8(&mut [0; 4]),
                    ),
                    _ => {}
                },
                KeyCode::Backspace => match form.focus {
                    ModelField::ApiKey => {
                        backspace_at_cursor(&mut form.api_key, &mut form.cursor_byte)
                    }
                    ModelField::Model => {
                        backspace_at_cursor(&mut form.model, &mut form.cursor_byte)
                    }
                    ModelField::Window => {
                        backspace_at_cursor(&mut form.window, &mut form.cursor_byte)
                    }
                    _ => {}
                },
                KeyCode::Delete => match form.focus {
                    ModelField::ApiKey => {
                        delete_at_cursor(&mut form.api_key, &mut form.cursor_byte)
                    }
                    ModelField::Model => delete_at_cursor(&mut form.model, &mut form.cursor_byte),
                    ModelField::Window => delete_at_cursor(&mut form.window, &mut form.cursor_byte),
                    _ => {}
                },
                KeyCode::Enter => {
                    let form = form.clone();
                    if self.save_model(&form, ctx, renderer) {
                        return Ok(ModalAction::Close);
                    }
                    self.mode = Mode::Model(form);
                }
                _ => {}
            }
            self.draw(buf, state, ctx, renderer);
            return Ok(ModalAction::Continue);
        }

        // ── Discovery results mode ──
        if let Mode::DiscoveryResults {
            ref account_id,
            ref models,
            ref mut selected,
            ref mut cursor,
        } = &mut self.mode
        {
            let account_id = account_id.clone();
            match code {
                KeyCode::Esc => {
                    // Cancel and go back to list
                    self.mode = Mode::List;
                    self.account_filter = Some(account_id);
                }
                KeyCode::Up => pick_cursor_up(cursor, models.len()),
                KeyCode::Down => pick_cursor_down(cursor, models.len()),
                KeyCode::Char(' ') => pick_toggle(selected, *cursor),
                KeyCode::Char('a') if mods.contains(KeyModifiers::CONTROL) => {
                    pick_select_all(selected);
                }
                KeyCode::Char('n') if mods.contains(KeyModifiers::CONTROL) => {
                    pick_deselect_all(selected);
                }
                KeyCode::Enter => {
                    let models_to_add = pick_chosen(models, selected);

                    if !models_to_add.is_empty() {
                        // Persist HERE. The old code only stashed the pick in
                        // `discovery_state` and promised "models will be added by
                        // event loop", but the panel exposes no accessor for that
                        // state and `Modal::poll_background` returns only a redraw
                        // flag -- the selection was silently dropped. On a refused
                        // commit the picker stays open (`update_config_and_reload`
                        // has already rendered the reason) so nothing is lost.
                        if commit_discovered_models(&account_id, &models_to_add, ctx, renderer) {
                            self.mode = Mode::List;
                            self.account_filter = Some(account_id);
                        }
                    }
                }
                _ => {}
            }
            self.draw(buf, state, ctx, renderer);
            return Ok(ModalAction::Continue);
        }

        // ── List mode (plugin-style: type filters, Ctrl+key acts) ──
        let len = self.current_len(&ctx.config);
        let ctrl = mods.contains(KeyModifiers::CONTROL);
        match code {
            // Esc closes the panel outright. Tab / Shift-Tab switch tabs and
            // reset both filters; arrows edit the search query.
            KeyCode::Esc => return Ok(ModalAction::Close),
            KeyCode::Tab | KeyCode::BackTab => {
                let next = match self.tab {
                    Tab::Accounts => Tab::Models,
                    Tab::Models => Tab::Accounts,
                };
                self.switch_tab(next);
            }
            KeyCode::Up => {
                self.search_focused = false;
                // Wraps: from the first row to the last.
                self.selected = crate::modals::step_up(self.selected, len);
                self.pending_delete = None;
            }
            KeyCode::Down => {
                self.search_focused = false;
                // Wraps: from the last row back to the first.
                self.selected = crate::modals::step_down(self.selected, len);
                self.pending_delete = None;
            }
            // Ctrl+A: add. Letter keys are reserved for the search filter.
            code if is_add_shortcut(&code, mods) => {
                self.begin_add_for_current_tab(&ctx.config);
            }
            // Ctrl+E: edit the selected row.
            KeyCode::Char('e') if ctrl => {
                self.pending_delete = None;
                if let Some(id) = self.selected_id(&ctx.config).filter(|i| !is_virtual_row(i)) {
                    self.mode = match self.tab {
                        Tab::Accounts => Mode::EditAccount(Self::open_edit(&ctx.config, &id)),
                        Tab::Models => match ModelForm::new_edit(&ctx.config, &id) {
                            Some(f) => Mode::Model(f),
                            None => Mode::List,
                        },
                    };
                }
            }
            // Ctrl+D twice: the first press arms the selected logical row; the
            // second deletes it without leaving the list for a confirmation UI.
            KeyCode::Char('d') if ctrl => {
                if let Some(id) = self.selected_id(&ctx.config).filter(|i| !is_virtual_row(i)) {
                    let is_account = self.tab == Tab::Accounts;
                    // Unconfigured preset rows have no persisted object to
                    // delete.
                    let is_virtual_preset =
                        is_account && Self::is_virtual_account_row(&ctx.config, &id);
                    if is_virtual_preset {
                        self.pending_delete = None;
                    } else if self.confirm_double_delete(&id, is_account)
                        && self.commit_delete(&id, is_account, ctx, renderer)
                    {
                        return Ok(ModalAction::Close);
                    }
                } else {
                    self.pending_delete = None;
                }
            }
            // Type to filter.
            KeyCode::Char(c) if !ctrl => {
                insert_at_cursor(
                    &mut self.query,
                    &mut self.query_cursor_byte,
                    c.encode_utf8(&mut [0; 4]),
                );
                self.search_focused = true;
                self.selected = 0;
                self.pending_delete = None;
            }
            KeyCode::Backspace => {
                backspace_at_cursor(&mut self.query, &mut self.query_cursor_byte);
                self.search_focused = true;
                self.selected = 0;
                self.pending_delete = None;
            }
            KeyCode::Left if self.search_focused => {
                self.query_cursor_byte =
                    previous_grapheme_boundary(&self.query, self.query_cursor_byte);
            }
            KeyCode::Right if self.search_focused => {
                self.query_cursor_byte =
                    next_grapheme_boundary(&self.query, self.query_cursor_byte);
            }
            KeyCode::Left => self.switch_tab(Tab::Accounts),
            KeyCode::Right => self.switch_tab(Tab::Models),
            KeyCode::Home if self.search_focused => self.query_cursor_byte = 0,
            KeyCode::End if self.search_focused => self.query_cursor_byte = self.query.len(),
            KeyCode::Delete if self.search_focused => {
                delete_at_cursor(&mut self.query, &mut self.query_cursor_byte);
                self.selected = 0;
                self.pending_delete = None;
            }
            KeyCode::Enter => {
                self.pending_delete = None;
                if let Some(id) = self.selected_id(&ctx.config) {
                    match self.tab {
                        // Set default + switch session.
                        Tab::Models if id == ADD_MODEL_ROW => {
                            self.begin_add_for_current_tab(&ctx.config);
                        }
                        // Re-load the drilled-in account's list and open the picker.
                        // Same helper the account row uses, so both entry points
                        // report an unreachable endpoint identically.
                        Tab::Models if id == REFRESH_MODELS_ROW => {
                            if let Some(account_id) = self.refresh_account(&ctx.config) {
                                self.start_discovery_for_account(&account_id, &ctx.config);
                            }
                        }
                        Tab::Models => {
                            if set_default_provider_and_reload(ctx, &id, renderer) {
                                return Ok(ModalAction::Close);
                            }
                        }
                        Tab::Accounts if id == ADD_PROVIDER_ROW => {
                            self.mode = Mode::Add(AddForm::new());
                        }
                        // Drill in AND load this account's model list -- the same
                        // thing saving a new account does, so the two paths cannot
                        // disagree about what "open an account" means. Manual Tab /
                        // Esc clears the filter to show all models again.
                        Tab::Accounts => {
                            self.show_models_for_account(&id);
                            self.start_discovery_for_account(&id, &ctx.config);
                        }
                    }
                }
            }
            _ => self.pending_delete = None,
        }
        self.draw(buf, state, ctx, renderer);
        Ok(ModalAction::Continue)
    }

    fn draw(&self, _buf: &Buffer, state: &UiState, ctx: &LoopCtx, renderer: &mut dyn Renderer) {
        let mut items: Vec<(String, String)> = Vec::new();
        let t0 = tab_chip(
            &crate::i18n::t(crate::i18n::Msg::ProviderPanelTabAccounts),
            self.tab == Tab::Accounts,
        );
        let t1 = tab_chip(
            &crate::i18n::t(crate::i18n::Msg::ProviderPanelTabModels),
            self.tab == Tab::Models,
        );
        items.push((format!("{t0}   {t1}"), String::new()));
        items.push((String::new(), String::new()));

        let mut selected = items.len(); // nothing highlighted by default
        let hint: String; // assigned once per match arm below
                          // Forms use the box-less `PluginInfo` layout; the list uses the `Plugin`
                          // layout whose reserved index-2 slot is rendered as the search box.
        let mut kind = MenuKind::PluginInfo;
        let mut buf = String::new();
        // PluginInfo rows are flush-left inside a one-column rule margin.
        // Keep a small safety margin so the caret/ellipsis never lands in the
        // terminal's autowrap column on narrow Windows hosts.
        let form_cols = crossterm::terminal::size()
            .map(|(cols, _)| cols as usize)
            .unwrap_or(80)
            .saturating_sub(4);

        match &self.mode {
            Mode::List => {
                kind = MenuKind::Plugin;
                buf = self.query.clone();
                // Reserved search box (index 2) + blank separator (index 3): the
                // plugin menu renders index 2 as the bordered input field. With
                // the tab bar + blank already pushed, list rows start at
                // LIST_HEADER_ROWS.
                items.push((self.query.clone(), String::new()));
                items.push((String::new(), String::new()));
                let cur = ctx.config.effective_model_selection().unwrap_or_default();
                let accounts = ctx.config.logical_accounts();
                let models = ctx.config.logical_models();
                let default_account = models.get(&cur).map(|m| m.account.clone());
                match self.tab {
                    Tab::Accounts => {
                        let ids = self.filtered_ids(&ctx.config);
                        for id in &ids {
                            let a = accounts.get(id);
                            let count = models.values().filter(|m| m.account == *id).count();
                            // 0-model providers show just the name; configured
                            // ones show "vendor · N 模型 [默认]".
                            let desc = if count == 0 {
                                String::new()
                            } else {
                                let vendor = a.map(|a| a.provider.clone()).unwrap_or_default();
                                let mark = if default_account.as_deref() == Some(id) {
                                    format!(
                                        "  [{}]",
                                        crate::i18n::t(crate::i18n::Msg::ProviderPanelDefaultBadge)
                                    )
                                } else {
                                    String::new()
                                };
                                let model_count =
                                    crate::i18n::t(crate::i18n::Msg::ProviderPanelModelCount {
                                        count,
                                    });
                                format!("{vendor} · {model_count}{mark}")
                            };
                            items.push((Self::account_label(&ctx.config, id), desc));
                        }
                        // Trailing virtual rows (add account), drawn from the same
                        // source `selected_id` indexes into.
                        items.extend(self.virtual_row_items(&ctx.config, &ids));
                        hint = crate::i18n::t(crate::i18n::Msg::ProviderPanelAccountsHint)
                            .into_owned();
                    }
                    Tab::Models => {
                        let ids = self.filtered_ids(&ctx.config);
                        for id in &ids {
                            let m = models.get(id);
                            let mark = if *id == cur {
                                format!(
                                    "  * [{}]",
                                    crate::i18n::t(crate::i18n::Msg::ProviderPanelDefaultBadge)
                                )
                            } else {
                                String::new()
                            };
                            let desc = m
                                .map(|m| {
                                    let name = m.display_name.as_deref().unwrap_or(&m.model);
                                    format!("{} · {}{}", m.account, name, mark)
                                })
                                .unwrap_or_default();
                            items.push((id.clone(), desc));
                        }
                        // Trailing virtual rows (add model, refresh list), drawn from
                        // the same source `selected_id` indexes into.
                        items.extend(self.virtual_row_items(&ctx.config, &ids));
                        hint = if let Some(acct) = &self.account_filter {
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelFilteredModelsHint {
                                account: acct,
                            })
                            .into_owned()
                        } else {
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelModelsHint).into_owned()
                        };
                    }
                }
                // List rows begin at LIST_HEADER_ROWS (tab bar, blank, search
                // box, blank).
                if self.current_len(&ctx.config) > 0 {
                    selected =
                        (self.selected + LIST_HEADER_ROWS).min(items.len().saturating_sub(1));
                }
            }
            Mode::Add(form) => {
                let p = form.preset();
                let field_row = |label: &str, value: String, focused: bool| {
                    let marker = if focused { "> " } else { "  " };
                    (format!("{marker}{label}: {value}"), String::new())
                };
                items.push((
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelAddTitle).into_owned(),
                    String::new(),
                ));
                items.push((String::new(), String::new()));
                let name = if form.name.is_empty() {
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelRequiredMark).into_owned()
                } else {
                    form.name.clone()
                };
                items.push(editable_field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldName),
                    &name,
                    form.focus == FormField::Name,
                    form.cursor_byte,
                    form_cols,
                ));
                items.push(field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldProtocol),
                    format!(
                        "‹ {} ›   ({})",
                        form.protocol_label(),
                        crate::i18n::t(crate::i18n::Msg::ProviderPanelSwitchHint)
                    ),
                    form.focus == FormField::Preset,
                ));
                items.push(editable_field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldBaseUrl),
                    &form.base_url,
                    form.focus == FormField::BaseUrl,
                    form.cursor_byte,
                    form_cols,
                ));
                if !matches!(p.auth_kind, provider_preset::AuthKind::None) {
                    let masked = "*".repeat(form.api_key.chars().count());
                    let env_hint = p
                        .api_key_env
                        .map(|e| {
                            format!(
                                "   ({})",
                                crate::i18n::t(crate::i18n::Msg::ProviderPanelEnvHint { env: e })
                            )
                        })
                        .unwrap_or_default();
                    let masked_cursor = "*"
                        .repeat(
                            form.api_key[..form.cursor_byte.min(form.api_key.len())]
                                .chars()
                                .count(),
                        )
                        .len();
                    items.push(editable_field_row(
                        &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldApiKey),
                        &format!("{masked}{env_hint}"),
                        form.focus == FormField::ApiKey,
                        masked_cursor,
                        form_cols,
                    ));
                }
                // Account-only form -- model/window/default moved to the 模型 tab.
                hint =
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelAddAccountFormHint).into_owned();
            }
            Mode::EditAccount(form) => {
                let field_row = |label: &str, value: String, focused: bool| {
                    let marker = if focused { "> " } else { "  " };
                    (format!("{marker}{label}: {value}"), String::new())
                };
                items.push((
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelEditAccountTitle {
                        account: &form.id,
                    })
                    .into_owned(),
                    String::new(),
                ));
                items.push((String::new(), String::new()));
                let p = form.preset();
                if form.protocol_locked {
                    // Curated preset rows lock the protocol.
                    items.push((
                        crate::i18n::t(crate::i18n::Msg::ProviderPanelProtocolLocked {
                            protocol: form.protocol_label(),
                        })
                        .into_owned(),
                        String::new(),
                    ));
                } else {
                    items.push(field_row(
                        &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldProtocol),
                        format!(
                            "‹ {} ›   ({})",
                            form.protocol_label(),
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelSwitchHint)
                        ),
                        form.focus == FormField::Preset,
                    ));
                }
                items.push(editable_field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldBaseUrl),
                    &form.base_url,
                    form.focus == FormField::BaseUrl,
                    form.cursor_byte,
                    form_cols,
                ));
                if !matches!(p.auth_kind, provider_preset::AuthKind::None) {
                    let masked = "*".repeat(form.api_key.chars().count());
                    let masked_cursor = "*"
                        .repeat(
                            form.api_key[..form.cursor_byte.min(form.api_key.len())]
                                .chars()
                                .count(),
                        )
                        .len();
                    items.push(editable_field_row(
                        &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldApiKey),
                        &format!(
                            "{masked}   ({})",
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelKeepOriginal)
                        ),
                        form.focus == FormField::ApiKey,
                        masked_cursor,
                        form_cols,
                    ));
                }
                hint = if form.protocol_locked {
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelEditFormProtocolLockedHint)
                        .into_owned()
                } else {
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelEditAccountFormHint).into_owned()
                };
            }
            Mode::Model(form) => {
                let field_row = |label: &str, value: String, focused: bool| {
                    let marker = if focused { "> " } else { "  " };
                    (format!("{marker}{label}: {value}"), String::new())
                };
                let title = if form.edit_id.is_some() {
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelEditModelTitle)
                } else {
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelAddModelTitle)
                };
                items.push((title.into_owned(), String::new()));
                items.push((String::new(), String::new()));
                if form.edit_id.is_some() {
                    // Account locked on edit -- show it, not editable.
                    items.push((
                        format!(
                            "  {}: {}",
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldAccount),
                            form.account_id()
                        ),
                        String::new(),
                    ));
                } else {
                    items.push(field_row(
                        &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldAccount),
                        format!(
                            "‹ {} ›   ({})",
                            form.account_id(),
                            crate::i18n::t(crate::i18n::Msg::ProviderPanelSwitchHint)
                        ),
                        form.focus == ModelField::Account,
                    ));
                    // This provider has no api_key yet -- collect it once here.
                    if form.account_needs_key() {
                        let masked = "*".repeat(form.api_key.chars().count());
                        let masked_cursor = form.api_key
                            [..form.cursor_byte.min(form.api_key.len())]
                            .chars()
                            .count()
                            * '•'.len_utf8();
                        items.push(editable_field_row(
                            "api_key",
                            &format!(
                                "{masked}   ({})",
                                crate::i18n::t(
                                    crate::i18n::Msg::ProviderPanelProviderNotConfigured
                                )
                            ),
                            form.focus == ModelField::ApiKey,
                            masked_cursor,
                            form_cols,
                        ));
                    }
                }
                items.push(editable_field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldModel),
                    &form.model,
                    form.focus == ModelField::Model,
                    form.cursor_byte,
                    form_cols,
                ));
                items.push(field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldVision),
                    format!("‹ {} ›", form.vision_label()),
                    form.focus == ModelField::Vision,
                ));
                items.push(field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldEffort),
                    format!("‹ {} ›", form.effort_label()),
                    form.focus == ModelField::Effort,
                ));
                items.push(field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldEffortLevels),
                    form.effort_levels_label(form.focus == ModelField::EffortLevels),
                    form.focus == ModelField::EffortLevels,
                ));
                let win = if form.window.is_empty() {
                    format!(
                        "({})",
                        crate::i18n::t(crate::i18n::Msg::ProviderPanelDefaultValue)
                    )
                } else {
                    form.window.clone()
                };
                items.push(editable_field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldWindow),
                    &win,
                    form.focus == ModelField::Window,
                    form.cursor_byte,
                    form_cols,
                ));
                items.push(field_row(
                    &crate::i18n::t(crate::i18n::Msg::ProviderPanelFieldMakeDefault),
                    if form.make_default { "[[+]]" } else { "[ ]" }.to_string(),
                    form.focus == ModelField::MakeDefault,
                ));
                hint = crate::i18n::t(crate::i18n::Msg::ProviderPanelModelFormHint).into_owned();
            }
            Mode::DiscoveryResults {
                models,
                selected,
                cursor,
                ..
            } => {
                kind = MenuKind::PluginInfo;
                buf = String::new();

                // Header
                items.push((
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryTitle).into_owned(),
                    String::new(),
                ));
                items.push((String::new(), String::new()));

                // Model list with checkboxes
                for (i, (model, &sel)) in models.iter().zip(selected.iter()).enumerate() {
                    let checkbox = if sel { "[x]" } else { "[ ]" };
                    let marker = if i == *cursor { "> " } else { "  " };
                    let name = model.name.as_deref().unwrap_or(&model.id);
                    items.push((format!("{marker}{checkbox} {name}"), model.id.clone()));
                }

                // Footer with instructions
                hint = crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryHint).into_owned();
            }
        }

        // Surface the last discovery outcome (failure reason / empty listing). A
        // silent failure was indistinguishable from "this provider has no models",
        // which is how the auto-load flow used to look simply broken.
        match &self.discovery_state {
            DiscoveryState::Notice { message } => items.push((message.clone(), String::new())),
            // An in-flight request must be visible: the user just pressed Enter and
            // the list does not change until the answer lands.
            DiscoveryState::Pending { account_id, .. } => items.push((
                crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryPending {
                    account: account_id,
                })
                .into_owned(),
                String::new(),
            )),
            DiscoveryState::Idle => {}
        }

        items.push((format!("— {hint} —"), String::new()));

        downgrade_panel_items(&mut items, ctx.caps.unicode_symbols);

        let payload = MenuPayload {
            items,
            selected,
            kind,
        };
        let cursor_byte = if matches!(self.mode, Mode::List) {
            self.query_cursor_byte
        } else {
            buf.len()
        };
        renderer.render(UiLine::InputPrompt {
            buf,
            cursor_byte,
            menu: Some(payload),
            status: build_status(state, ctx),
            attachments: Vec::new(),
        });
        renderer.flush();
    }

    fn handle_paste(
        &mut self,
        text: &str,
        buf: &mut Buffer,
        state: &mut UiState,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Result<ModalAction> {
        self.apply_paste_text(text);
        self.draw(buf, state, ctx, renderer);
        Ok(ModalAction::Continue)
    }

    fn poll_background(&mut self) -> bool {
        let Some((account_id, result)) = self.poll_discovery() else {
            return false;
        };
        match result {
            Ok(models) if !models.is_empty() => {
                // Switch to discovery results mode (every row preselected).
                let selected = vec![true; models.len()];
                self.mode = Mode::DiscoveryResults {
                    account_id,
                    models,
                    selected,
                    cursor: 0,
                };
                self.tab = Tab::Models;
                true
            }
            Ok(_) => {
                self.note_discovery(
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryEmpty).into_owned(),
                );
                true
            }
            Err(error) => {
                self.note_discovery(
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelDiscoveryFailed {
                        reason: &error,
                    })
                    .into_owned(),
                );
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serve exactly one HTTP response on an ephemeral port, returning the base
    /// URL to point discovery at.
    ///
    /// A real socket (not a mocked transport) is the point: the panel's discovery
    /// path builds its OWN client and URL, so only a round trip proves the whole
    /// chain -- URL join, auth header, status check, body read, parse.
    fn serve_once(
        status_line: &'static str,
        body: &'static str,
        expect_path: &'static str,
        expect_auth: Option<&'static str>,
    ) -> (String, std::thread::JoinHandle<String>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            use std::io::{Read, Write};
            let mut buf = [0u8; 4096];
            let n = socket.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let head = format!(
                "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(head.as_bytes()).unwrap();
            socket.write_all(body.as_bytes()).unwrap();
            socket.flush().unwrap();
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or_default()
                .to_string();
            let auth = request
                .lines()
                .find(|line| line.to_ascii_lowercase().starts_with("authorization:"))
                .unwrap_or_default()
                .to_string();
            assert_eq!(
                path, expect_path,
                "discovery must hit the protocol's list endpoint"
            );
            if let Some(expected) = expect_auth {
                assert!(
                    auth.to_ascii_lowercase()
                        .contains(&expected.to_ascii_lowercase()),
                    "expected `{expected}` in the request, got `{auth}`"
                );
            } else {
                assert!(
                    auth.is_empty(),
                    "no api_key ⇒ no Authorization header, got `{auth}`"
                );
            }
            let _ = (path, auth);
            request
        });
        (format!("http://{addr}/v1"), handle)
    }

    /// The auto-load path must actually work end to end: adding an account fires a
    /// real request at `<base_url>/models`, and a 200 yields selectable models.
    #[test]
    fn discovery_round_trips_an_openai_compatible_listing() {
        let body = r#"{"data":[
            {"id":"vendor/model-a","name":"Model A","context_length":131072,"max_output_tokens":8192},
            {"id":"vendor/model-b"}
        ]}"#;
        let (base_url, server) = serve_once(
            "HTTP/1.1 200 OK",
            body,
            "/v1/models",
            Some("Bearer sk-test"),
        );

        let models = ProviderPanel::discover_models_sync(
            &base_url,
            Some("sk-test"),
            "openai",
            None,
            None,
            false,
        )
        .unwrap();

        assert_eq!(
            models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["vendor/model-a", "vendor/model-b"],
            "every id in the listing must become a selectable model"
        );
        // Metadata is carried through, not just the id.
        assert_eq!(models[0].name.as_deref(), Some("Model A"));
        assert_eq!(models[0].context_window, Some(131072));
        assert_eq!(models[0].max_tokens, Some(8192));
        assert_eq!(models[1].context_window, None, "absent metadata stays None");
        server.join().unwrap();
    }

    /// Ollama speaks a different endpoint AND a different body shape; a regression
    /// in either would silently produce an empty picker.
    #[test]
    fn discovery_round_trips_an_ollama_tag_listing() {
        let body = r#"{"models":[{"name":"llama3:8b","model":"llama3:8b"}]}"#;
        let (base_url, server) = serve_once("HTTP/1.1 200 OK", body, "/api/tags", None);

        // Note the base URL carries `/v1`: discovery must NOT inherit that suffix
        // for Ollama... it appends to whatever it was given, so assert what the
        // protocol actually requires -- the tags endpoint off the given base.
        let base = base_url.trim_end_matches("/v1").to_string();
        let models =
            ProviderPanel::discover_models_sync(&base, None, "ollama", None, None, false).unwrap();

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "llama3:8b");
        assert_eq!(models[0].name.as_deref(), Some("llama3:8b"));
        server.join().unwrap();
    }

    /// A failed discovery must surface the HTTP status, because "no models" and
    /// "the request was rejected" are indistinguishable to the user otherwise.
    /// The point of the fix: transport settings saved on the ACCOUNT must reach
    /// the request. A custom UA allowlist (or proxy) that works from the WebUI
    /// used to be ignored here, so discovery failed only in the TUI.
    #[test]
    fn discovery_sends_the_account_user_agent() {
        let (base_url, server) = serve_once(
            "HTTP/1.1 200 OK",
            r#"{"data":[{"id":"m"}]}"#,
            "/v1/models",
            None,
        );
        let models = ProviderPanel::discover_models_sync(
            &base_url,
            None,
            "openai",
            Some("my-gateway-agent/9"),
            None,
            false,
        )
        .unwrap();
        assert_eq!(models.len(), 1);
        let request = server.join().unwrap();
        assert!(
            request.to_ascii_lowercase().contains("my-gateway-agent/9"),
            "the account's User-Agent must be sent, got request:\n{request}"
        );
    }

    /// Without a UA on the account nothing custom is injected -- the shared egress
    /// factory's `DEFAULT_USER_AGENT` applies, the same one every other egress call
    /// site gets.
    #[test]
    fn discovery_omits_a_user_agent_the_account_never_set() {
        let (base_url, server) = serve_once(
            "HTTP/1.1 200 OK",
            r#"{"data":[{"id":"m"}]}"#,
            "/v1/models",
            None,
        );
        ProviderPanel::discover_models_sync(&base_url, None, "openai", Some("  "), None, false)
            .unwrap();
        let request = server.join().unwrap();
        assert!(
            !request.to_ascii_lowercase().contains("my-gateway-agent"),
            "blank UA must not be forwarded, got request:\n{request}"
        );
    }

    #[test]
    fn discovery_reports_a_non_success_status() {
        let (base_url, server) = serve_once(
            "HTTP/1.1 401 Unauthorized",
            r#"{"error":"nope"}"#,
            "/v1/models",
            None,
        );
        let err = ProviderPanel::discover_models_sync(&base_url, None, "openai", None, None, false)
            .unwrap_err();
        assert!(
            err.contains("401"),
            "the status must reach the user, got `{err}`"
        );
        server.join().unwrap();
    }

    /// A 200 with an unparseable body is its own failure mode: it must be an
    /// error, not an empty list (which would render as "provider has no models").
    #[test]
    fn discovery_rejects_a_malformed_body_instead_of_claiming_no_models() {
        let (base_url, server) = serve_once("HTTP/1.1 200 OK", "not json", "/v1/models", None);
        let err = ProviderPanel::discover_models_sync(&base_url, None, "openai", None, None, false)
            .unwrap_err();
        assert!(
            err.to_lowercase().contains("json"),
            "a malformed body must be reported as a parse failure, got `{err}`"
        );
        server.join().unwrap();
    }

    /// The refresh row is the ONLY way to re-load an existing account's list, so
    /// its visibility rules are the feature's contract.
    #[test]
    fn refresh_row_needs_a_drilled_in_account_with_an_endpoint() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": {
                "acc": { "provider": "deepseek" },
                "nourl": { "provider": "openai-compatible" }
            }
        }))
        .unwrap();

        let mut p = ProviderPanel::open();
        // Accounts tab: nothing to refresh -- Enter there drills in instead.
        p.tab = Tab::Accounts;
        p.account_filter = Some("acc".into());
        assert!(p.refresh_account(&cfg).is_none());

        // Models tab but no drill-in: which account would we ask? Refuse.
        p.tab = Tab::Models;
        p.account_filter = None;
        assert!(p.refresh_account(&cfg).is_none());

        // Drilled in, and the preset supplies the endpoint (`deepseek` has one).
        p.account_filter = Some("acc".into());
        assert_eq!(p.refresh_account(&cfg).as_deref(), Some("acc"));

        // A `*-compatible` account has no default endpoint and no base_url, so the
        // row must not appear (it would be a guaranteed no-op).
        p.account_filter = Some("nourl".into());
        assert!(p.refresh_account(&cfg).is_none());

        // An id with no persisted account (a virtual preset row) is never a target.
        p.account_filter = Some("not-configured".into());
        assert!(p.refresh_account(&cfg).is_none());
    }

    /// The refresh row must be selectable and carry its own sentinel -- otherwise
    /// Enter would fall through to "set as default".
    #[test]
    fn refresh_row_is_selectable_after_the_add_row() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": { "acc/deepseek-chat": { "account": "acc", "model": "deepseek-chat", "context_window": 8000 } }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        p.tab = Tab::Models;
        p.account_filter = Some("acc".into());

        // Rows: the one real model, then add-model, then refresh.
        assert_eq!(p.filtered_ids(&cfg), vec!["acc/deepseek-chat".to_string()]);
        assert_eq!(p.current_len(&cfg), 3);

        p.selected = 0;
        assert_eq!(p.selected_id(&cfg).as_deref(), Some("acc/deepseek-chat"));
        p.selected = 1;
        assert_eq!(p.selected_id(&cfg).as_deref(), Some(ADD_MODEL_ROW));
        p.selected = 2;
        assert_eq!(p.selected_id(&cfg).as_deref(), Some(REFRESH_MODELS_ROW));

        // Past the end resolves to nothing rather than repeating the last row.
        p.selected = 3;
        assert_eq!(p.selected_id(&cfg), None);
    }

    /// Without a drill-in the refresh row is absent, so the add row stays last --
    /// an off-by-one here would make Enter add a model instead of asking the
    /// provider, or vice versa.
    #[test]
    fn add_row_stays_last_when_the_refresh_row_is_not_offered() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": { "acc/a": { "account": "acc", "model": "a", "context_window": 8000 } }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        p.tab = Tab::Models;
        p.account_filter = None;

        assert_eq!(p.current_len(&cfg), 2);
        p.selected = 1;
        assert_eq!(p.selected_id(&cfg).as_deref(), Some(ADD_MODEL_ROW));
    }

    /// Ctrl+E / Ctrl+D must skip the refresh row: it is an action, not a config
    /// row, so deleting it must not arm a delete on some unrelated id.
    #[test]
    fn refresh_row_is_never_editable_or_deletable() {
        assert!(is_virtual_row(REFRESH_MODELS_ROW));
        assert!(is_virtual_row(ADD_MODEL_ROW));
        assert!(is_virtual_row(ADD_PROVIDER_ROW));
        assert!(!is_virtual_row("acc/deepseek-chat"));
    }

    /// Entering the refresh row must start a real request for the drilled-in
    /// account, and the panel must report it as in flight.
    #[test]
    fn enter_on_the_refresh_row_starts_discovery_for_the_drilled_in_account() {
        // A LOCAL endpoint, not the `deepseek` preset default: this test used to
        // point at api.deepseek.com and really fire a request at the internet.
        let (base_url, server) = serve_once(
            "HTTP/1.1 200 OK",
            r#"{"data":[{"id":"m"}]}"#,
            "/v1/models",
            None,
        );
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek", "base_url": base_url } }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        p.tab = Tab::Models;
        p.account_filter = Some("acc".into());
        p.selected = 1; // the refresh row (only virtual row here)

        let account_id = p.refresh_account(&cfg).unwrap();
        assert!(p.start_discovery_for_account(&account_id, &cfg));

        match &p.discovery_state {
            DiscoveryState::Pending { account_id, .. } => assert_eq!(account_id, "acc"),
            other => panic!("expected an in-flight request, got {other:?}"),
        }
        // A second press must not stack requests on the same panel.
        assert!(!p.start_discovery_for_account("acc", &cfg));
        server.join().unwrap();
    }

    /// The account row's Enter is the entry point the user asked for: it must both
    /// drill in AND load the list, matching what saving a new account does.
    #[test]
    fn account_enter_drills_in_and_loads_the_model_list() {
        let (base_url, server) = serve_once(
            "HTTP/1.1 200 OK",
            r#"{"data":[{"id":"m"}]}"#,
            "/v1/models",
            None,
        );
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek", "base_url": base_url } }
        }))
        .unwrap();

        let mut p = ProviderPanel::open();
        p.tab = Tab::Accounts;
        let ids = p.filtered_ids(&cfg);
        p.selected = ids.iter().position(|id| id == "acc").unwrap();

        // Exactly what the Enter arm does.
        let id = p.selected_id(&cfg).unwrap();
        p.show_models_for_account(&id);
        assert!(p.start_discovery_for_account(&id, &cfg));

        assert!(
            matches!(p.tab, Tab::Models),
            "Enter must drill into the account"
        );
        assert_eq!(p.account_filter.as_deref(), Some("acc"));
        assert!(
            p.virtual_rows(&cfg)
                .contains(&REFRESH_MODELS_ROW.to_string()),
            "the drilled-in list offers a manual refresh"
        );
        match &p.discovery_state {
            DiscoveryState::Pending { account_id, .. } => assert_eq!(account_id, "acc"),
            other => panic!("Enter must start loading, got {other:?}"),
        }
        server.join().unwrap();
    }

    /// An account with no endpoint (no base_url, no preset default) cannot be
    /// asked for models. The panel must SAY so -- the old silent no-op is exactly
    /// what made this look like a missing feature.
    #[test]
    fn start_discovery_for_account_reports_a_missing_endpoint() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            // `openai-compatible` carries no default base URL.
            "provider_accounts": { "nourl": { "provider": "openai-compatible" } }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        p.tab = Tab::Accounts;

        assert!(!p.start_discovery_for_account("nourl", &cfg));
        match &p.discovery_state {
            DiscoveryState::Notice { message } => assert!(
                message.contains("nourl"),
                "the notice must name the account, got `{message}`"
            ),
            other => panic!("expected a notice, got {other:?}"),
        }
        // Noticed, not stuck: a later attempt is still allowed.
        assert!(can_start_discovery(&p.discovery_state));
    }

    /// A virtual preset row (no persisted account, hence no credential) is refused
    /// the same way rather than silently doing nothing.
    #[test]
    fn start_discovery_for_an_unconfigured_row_reports_a_missing_endpoint() {
        let cfg = Config::default();
        let mut p = ProviderPanel::open();
        assert!(!p.start_discovery_for_account("not-configured", &cfg));
        assert!(matches!(p.discovery_state, DiscoveryState::Notice { .. }));
    }

    /// A request already in flight belongs to the account the user asked about a
    /// moment ago; a second Enter must not silently replace it with another
    /// account's answer.
    #[test]
    fn in_flight_discovery_is_not_replaced_by_another_account() {
        let (a_url, server_a) = serve_once(
            "HTTP/1.1 200 OK",
            r#"{"data":[{"id":"a"}]}"#,
            "/v1/models",
            None,
        );
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": {
                "a": { "provider": "deepseek", "base_url": a_url },
                // A dead port on purpose: no server thread, because this account's
                // request must never be sent -- joining a listener that expects a
                // connection would hang the suite forever.
                "b": { "provider": "deepseek", "base_url": "http://127.0.0.1:1/v1" }
            }
        }))
        .unwrap();

        let mut p = ProviderPanel::open();
        assert!(p.start_discovery_for_account("a", &cfg));
        assert!(
            !p.start_discovery_for_account("b", &cfg),
            "a second account must not displace the in-flight request"
        );
        match &p.discovery_state {
            DiscoveryState::Pending { account_id, .. } => assert_eq!(account_id, "a"),
            other => panic!("expected the FIRST request to still be pending, got {other:?}"),
        }
        server_a.join().unwrap();
    }

    /// The drawn virtual rows and the selectable ones must be the SAME list: an
    /// off-by-one here makes Enter act on a row the user is not looking at. This
    /// locks the invariant the old duplicated pushes could only hope to maintain.
    #[test]
    fn drawn_virtual_rows_match_the_selectable_ones() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": { "acc/a": { "account": "acc", "model": "a", "context_window": 8000 } }
        }))
        .unwrap();

        let mut p = ProviderPanel::open();
        p.tab = Tab::Models;
        p.account_filter = Some("acc".into());
        let ids = p.filtered_ids(&cfg);

        // Drilled in: add row + refresh row, same order as `virtual_rows`.
        assert_eq!(p.virtual_rows(&cfg).len(), 2);
        let drawn = p.virtual_row_items(&cfg, &ids);
        assert_eq!(drawn.len(), p.virtual_rows(&cfg).len());
        for (row, (label, _)) in p.virtual_rows(&cfg).iter().zip(drawn.iter()) {
            let expected = match row.as_str() {
                ADD_MODEL_ROW => crate::i18n::t(crate::i18n::Msg::ProviderPanelAddModelRow),
                REFRESH_MODELS_ROW => {
                    crate::i18n::t(crate::i18n::Msg::ProviderPanelRefreshModelsRow)
                }
                other => panic!("unexpected virtual row {other}"),
            };
            assert_eq!(*label, expected, "row {row} must render its own label");
        }

        // Every selectable index must land on the row drawn at that position.
        let total = p.current_len(&cfg);
        assert_eq!(total, ids.len() + drawn.len());
        for i in 0..total {
            p.selected = i;
            let id = p.selected_id(&cfg).expect("every row is selectable");
            let label = if i < ids.len() {
                id.clone()
            } else {
                drawn[i - ids.len()].0.clone()
            };
            assert!(!label.is_empty(), "row {i} ({id}) must have a label");
        }

        // Accounts tab: exactly the add-account row, no refresh affordance.
        p.tab = Tab::Accounts;
        p.account_filter = None;
        let acc_ids = p.filtered_ids(&cfg);
        let acc_drawn = p.virtual_row_items(&cfg, &acc_ids);
        assert_eq!(acc_drawn.len(), 1);
        assert_eq!(
            acc_drawn[0].0,
            crate::i18n::t(crate::i18n::Msg::ProviderPanelAddAccountRow)
        );
        p.selected = acc_ids.len();
        assert_eq!(p.selected_id(&cfg).as_deref(), Some(ADD_PROVIDER_ROW));
    }

    /// The multi-select state machine behind the discovery picker. It is easy to
    /// get wrong in a way no compile error catches -- an off-by-one cursor, a
    /// toggle that writes past the list, or a Confirm that adds rows the user
    /// unchecked -- and the picker writes to the user's config, so the wrong
    /// answer is persisted, not just displayed.
    #[test]
    fn discovery_picker_multiselect_state_machine() {
        let models = vec![
            discovered("a", "A"),
            discovered("b", "B"),
            discovered("c", "C"),
        ];

        // Starts fully selected (poll_background preselects every row).
        let mut selected = vec![true; models.len()];
        let mut cursor = 0usize;
        assert_eq!(pick_chosen(&models, &selected).len(), 3);

        // Space toggles ONLY the row under the cursor.
        pick_toggle(&mut selected, cursor);
        assert_eq!(selected, vec![false, true, true]);
        assert_eq!(
            pick_chosen(&models, &selected)
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "c"]
        );
        pick_toggle(&mut selected, cursor);
        assert_eq!(selected, vec![true, true, true], "toggling twice restores");

        // Ctrl+N clears everything: Confirm then has nothing to persist, which is
        // how the caller knows to stay open instead of writing an empty batch.
        pick_deselect_all(&mut selected);
        assert_eq!(selected, vec![false, false, false]);
        assert!(pick_chosen(&models, &selected).is_empty());

        // Ctrl+A restores all.
        pick_select_all(&mut selected);
        assert!(selected.iter().all(|&s| s));

        // Cursor movement WRAPS at both ends: a choice list must not dead-end at
        // the first or last entry. (A cursor left stale by a shorter list is still
        // a no-op -- see the step_* helpers.)
        assert_eq!(cursor, 0);
        pick_cursor_up(&mut cursor, models.len());
        assert_eq!(cursor, 2, "Up at the top goes to the LAST entry");
        pick_cursor_down(&mut cursor, models.len());
        assert_eq!(cursor, 0, "Down at the bottom returns to the FIRST entry");
        pick_cursor_down(&mut cursor, models.len());
        assert_eq!(cursor, 1);
        pick_cursor_down(&mut cursor, models.len());
        assert_eq!(cursor, 2);
        pick_cursor_down(&mut cursor, models.len());
        assert_eq!(cursor, 0, "and wraps again");

        // A cursor left stale by a shorter listing is a no-op, never a panic.
        let mut stale = vec![true, true];
        pick_toggle(&mut stale, 5);
        assert_eq!(stale, vec![true, true]);
        pick_toggle(&mut stale, 1);
        assert_eq!(stale, vec![true, false]);
    }

    /// A stale pick shorter than the listing must not invent selections for the
    /// missing tail -- silently adding unchecked models is the failure that
    /// matters, since it writes to the user's config.
    #[test]
    fn discovery_picker_never_selects_beyond_a_short_stale_pick() {
        let models = vec![
            discovered("a", "A"),
            discovered("b", "B"),
            discovered("c", "C"),
        ];
        let chosen = pick_chosen(&models, &[true, false]);
        assert_eq!(
            chosen.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["a"],
            "the third model has no flag, so it is NOT chosen"
        );
        assert!(pick_chosen(&models, &[]).is_empty());
    }

    #[test]
    fn provider_panel_chrome_downgrades_for_legacy_conhost() {
        let mut items = vec![
            ("▸ Model: │vendor/model…".to_string(), String::new()),
            ("  Image input: ‹ Auto ›".to_string(), "[✓]".to_string()),
            ("＋ Add model".to_string(), "— hint —".to_string()),
        ];

        downgrade_panel_items(&mut items, false);

        assert_eq!(items[0].0, "> Model: |vendor/model.");
        assert_eq!(items[1].0, "  Image input: < Auto >");
        assert_eq!(items[1].1, "[v]");
        assert_eq!(items[2].0, "+ Add model");
        assert_eq!(items[2].1, "- hint -");
    }

    #[test]
    fn provider_panel_chrome_is_unchanged_on_unicode_terminals() {
        let mut items = vec![("▸ Model: │…".to_string(), "[✓]".to_string())];
        let original = items.clone();

        downgrade_panel_items(&mut items, true);

        assert_eq!(items, original);
    }

    #[test]
    fn add_shortcut_accepts_terminal_ctrl_a_variants() {
        assert!(is_add_shortcut(&KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert!(is_add_shortcut(&KeyCode::Char('A'), KeyModifiers::CONTROL));
        assert!(is_add_shortcut(&KeyCode::Char('\u{1}'), KeyModifiers::NONE));
        assert!(!is_add_shortcut(&KeyCode::Char('a'), KeyModifiers::NONE));
    }

    #[test]
    fn both_provider_tabs_expose_a_selectable_add_row() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": { "acc/chat": { "account": "acc", "model": "chat", "context_window": 8000 } }
        }))
        .unwrap();
        let mut panel = ProviderPanel::open();

        let account_rows = panel.filtered_ids(&cfg).len();
        assert_eq!(panel.current_len(&cfg), account_rows + 1);
        panel.selected = account_rows;
        assert_eq!(panel.selected_id(&cfg).as_deref(), Some(ADD_PROVIDER_ROW));

        panel.tab = Tab::Models;
        panel.selected = panel.filtered_ids(&cfg).len();
        assert_eq!(panel.current_len(&cfg), 2);
        assert_eq!(panel.selected_id(&cfg).as_deref(), Some(ADD_MODEL_ROW));

        panel.query = "no-match".into();
        panel.selected = 0;
        assert_eq!(panel.current_len(&cfg), 1);
        assert_eq!(panel.selected_id(&cfg).as_deref(), Some(ADD_MODEL_ROW));
    }

    #[test]
    fn model_add_row_opens_the_model_form_for_the_drilled_in_account() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } }
        }))
        .unwrap();
        let mut panel = ProviderPanel::open();
        panel.tab = Tab::Models;
        panel.account_filter = Some("acc".into());

        panel.begin_add_for_current_tab(&cfg);

        let Mode::Model(form) = &panel.mode else {
            panic!("model add row should open the model form");
        };
        assert_eq!(form.account_id(), "acc");
    }

    #[test]
    fn add_form_is_custom_provider_with_protocol_toggle() {
        let mut f = AddForm::new();
        // Fully-custom: name, protocol, base_url, api_key; base_url starts blank.
        assert_eq!(
            f.fields(),
            vec![
                FormField::Name,
                FormField::Preset,
                FormField::BaseUrl,
                FormField::ApiKey,
            ]
        );
        assert!(f.base_url.is_empty());
        assert_eq!(f.protocol_label(), "OpenAI");
        // ←-> cycles OpenAI -> Anthropic -> Ollama -> OpenAI (never a vendor list).
        f.cycle_preset(true);
        assert_eq!(f.protocol_label(), "Anthropic");
        assert_eq!(f.preset().id, "anthropic-compatible");
        f.cycle_preset(true);
        assert_eq!(f.protocol_label(), "Ollama");
        assert_eq!(f.preset().id, "ollama");
        f.cycle_preset(true);
        assert_eq!(f.protocol_label(), "OpenAI");
        assert_eq!(f.preset().id, "openai-compatible");
    }

    #[test]
    fn add_form_protocol_toggle_cycles_backward() {
        let mut f = AddForm::new(); // OpenAI
        f.cycle_preset(false);
        assert_eq!(f.preset().id, "ollama");
        f.cycle_preset(false);
        assert_eq!(f.preset().id, "anthropic-compatible");
        f.cycle_preset(false);
        assert_eq!(f.preset().id, "openai-compatible");
    }

    #[test]
    fn add_form_ollama_offers_local_endpoint_and_hides_api_key() {
        let mut f = AddForm::new();
        f.cycle_preset(true); // Anthropic
        f.cycle_preset(true); // Ollama
        assert_eq!(f.preset().id, "ollama");
        // Local, keyless: auto-fill the well-known endpoint and drop the key field.
        assert_eq!(f.base_url, "http://localhost:11434");
        assert!(
            !f.fields().contains(&FormField::ApiKey),
            "Ollama is keyless local"
        );
        // The field is never silently wiped when cycling away -- the value stays
        // visible and editable (auto-fill only ever fills a blank field).
        f.cycle_preset(true); // OpenAI
        assert_eq!(f.preset().id, "openai-compatible");
        assert_eq!(f.base_url, "http://localhost:11434");
    }

    #[test]
    fn cycle_never_overwrites_or_clears_an_existing_base_url() {
        // Editing an account with a pre-filled endpoint: cycling the protocol
        // must never clobber or clear the URL the account already has -- auto-fill
        // is a convenience for a blank field only, so no save loses data.
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "openai", "base_url": "https://mirror/v1" } }
        }))
        .unwrap();
        let mut edit = ProviderPanel::open_edit(&cfg, "acc");
        assert_eq!(edit.base_url, "https://mirror/v1");
        edit.cycle_preset(true); // Anthropic
        assert_eq!(edit.base_url, "https://mirror/v1");
        edit.cycle_preset(true); // Ollama -- field non-empty, no auto-fill, no clear
        assert_eq!(edit.preset().id, "ollama");
        assert_eq!(edit.base_url, "https://mirror/v1");
        edit.cycle_preset(true); // OpenAI -- still intact
        assert_eq!(edit.base_url, "https://mirror/v1");
    }

    #[test]
    fn editing_ollama_account_and_cycling_away_keeps_the_default_endpoint() {
        // An existing Ollama account with no override shows the default endpoint
        // on open. Cycling the protocol away must NOT wipe that pre-filled value
        // (open_edit's default is indistinguishable from a within-session
        // auto-fill by string, so clearing on a match loses the endpoint).
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "local": { "provider": "ollama" } }
        }))
        .unwrap();
        let mut edit = ProviderPanel::open_edit(&cfg, "local");
        assert_eq!(edit.base_url, "http://localhost:11434");
        edit.cycle_preset(true); // Ollama -> OpenAI
        assert_eq!(edit.preset().id, "openai-compatible");
        assert_eq!(
            edit.base_url, "http://localhost:11434",
            "cycling away must not clear the account's shown endpoint"
        );
    }

    #[test]
    fn add_form_ollama_keeps_user_typed_base_url() {
        let mut f = AddForm::new();
        f.base_url = "http://box:11434".into();
        f.cycle_preset(true); // Anthropic
        f.cycle_preset(true); // Ollama
        assert_eq!(f.preset().id, "ollama");
        assert_eq!(
            f.base_url, "http://box:11434",
            "must not clobber a user-typed endpoint"
        );
        // A user-typed endpoint survives cycling away, too.
        f.cycle_preset(true); // OpenAI
        assert_eq!(f.base_url, "http://box:11434");
    }

    #[test]
    fn edit_form_protocol_toggle_reaches_ollama() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "openai" } }
        }))
        .unwrap();
        let mut edit = ProviderPanel::open_edit(&cfg, "acc");
        assert_eq!(edit.protocol_label(), "OpenAI");
        edit.cycle_preset(true); // Anthropic
        edit.cycle_preset(true); // Ollama
        assert_eq!(edit.protocol_label(), "Ollama");
        assert_eq!(edit.preset().id, "ollama");
    }

    #[test]
    fn open_edit_of_ollama_account_shows_ollama_and_noop_keeps_provider() {
        let mut cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "local": { "provider": "ollama" } }
        }))
        .unwrap();
        let edit = ProviderPanel::open_edit(&cfg, "local");
        assert_eq!(edit.protocol_label(), "Ollama");
        assert_eq!(
            edit.preset_idx, edit.original_preset_idx,
            "a no-op edit must not rewrite the stored provider"
        );
        ProviderPanel::apply_account_edit(&edit, &mut cfg);
        assert_eq!(
            cfg.provider_accounts.get("local").unwrap().provider,
            "ollama"
        );
    }

    #[test]
    fn sanitize_account_name_makes_toml_safe_ids() {
        assert_eq!(sanitize_account_name("Xiaomi MiMo"), "Xiaomi-MiMo");
        assert_eq!(sanitize_account_name("my/vendor@v1"), "my-vendor-v1");
        assert_eq!(sanitize_account_name("  --keep_me.1--  "), "keep_me.1");
        assert_eq!(sanitize_account_name("！！！"), "");
    }

    #[test]
    fn text_cursor_edits_at_utf8_boundaries() {
        let mut value = "ab前端cd".to_string();
        let mut cursor = value.len();

        cursor = previous_grapheme_boundary(&value, cursor);
        cursor = previous_grapheme_boundary(&value, cursor);
        insert_at_cursor(&mut value, &mut cursor, "/");
        assert_eq!(value, "ab前端/cd");

        backspace_at_cursor(&mut value, &mut cursor);
        assert_eq!(value, "ab前端cd");
        delete_at_cursor(&mut value, &mut cursor);
        assert_eq!(value, "ab前端d");
    }

    #[test]
    fn text_cursor_deletes_complete_combining_and_zwj_graphemes() {
        let mut combining = "Ae\u{301}".to_string();
        let mut cursor = combining.len();
        cursor = previous_grapheme_boundary(&combining, cursor);
        assert_eq!(cursor, 1, "cursor must jump over the complete e + accent");
        delete_at_cursor(&mut combining, &mut cursor);
        assert_eq!(combining, "A");

        let family = "👨‍👩‍👦";
        let mut emoji = format!("A{family}B");
        let mut cursor = 1;
        assert_eq!(next_grapheme_boundary(&emoji, cursor), 1 + family.len());
        delete_at_cursor(&mut emoji, &mut cursor);
        assert_eq!(emoji, "AB", "Delete must remove the whole ZWJ emoji");
    }

    #[test]
    fn editable_projection_keeps_the_url_tail_visible_at_end() {
        let url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1";
        let shown = crate::width::editable_value_projection(url, url.len(), 28);
        assert!(
            shown.starts_with('…'),
            "expected hidden-left marker: {shown}"
        );
        assert!(shown.ends_with('│'), "caret should remain visible: {shown}");
        assert!(
            shown.contains("compatible-mode/v1"),
            "URL tail missing: {shown}"
        );
        assert!(crate::width::display_width(&shown) <= 28);
    }

    #[test]
    fn editable_projection_marks_both_hidden_sides_around_middle_cursor() {
        let url = "https://example.test/a/very/long/provider/path/v1";
        let cursor = url.find("provider").expect("provider segment");
        let shown = crate::width::editable_value_projection(url, cursor, 20);
        assert!(shown.starts_with('…'), "left marker missing: {shown}");
        assert!(shown.ends_with('…'), "right marker missing: {shown}");
        assert!(shown.contains('│'), "caret missing: {shown}");
        assert!(crate::width::display_width(&shown) <= 20);
    }

    #[test]
    fn editable_projection_keeps_complete_value_when_it_fits() {
        assert_eq!(
            crate::width::editable_value_projection("https://x/v1", "https://".len(), 40),
            "https://│x/v1"
        );
    }

    #[test]
    fn open_edit_prefills_and_detects_legacy() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "providers": { "leg": { "type": "openai", "base_url": "https://legacy/v1", "model": "m", "context_window": 8000 } },
            "provider_accounts": { "acc": { "provider": "deepseek", "base_url": "https://mirror/v1" } },
            "models": { "acc/m": { "account": "acc", "model": "x", "context_window": 8000 } }
        }))
        .unwrap();
        let leg = ProviderPanel::open_edit(&cfg, "leg");
        assert!(leg.is_legacy);
        assert_eq!(leg.base_url, "https://legacy/v1");
        // Protocol toggle pre-filled from the wire (openai -> OpenAI-compatible),
        // and original == preset so a no-op edit won't rewrite the real provider.
        assert_eq!(leg.protocol_label(), "OpenAI");
        assert_eq!(leg.preset_idx, leg.original_preset_idx);
        let acc = ProviderPanel::open_edit(&cfg, "acc");
        assert!(!acc.is_legacy);
        assert_eq!(acc.base_url, "https://mirror/v1");
        assert!(acc.api_key.is_empty()); // blank = keep existing
                                         // deepseek is openai-wire -> OpenAI-compatible toggle.
        assert_eq!(acc.protocol_label(), "OpenAI");
    }

    #[test]
    fn open_edit_prefills_and_materializes_virtual_preset_account() {
        let mut cfg = Config::default();
        let mut edit = ProviderPanel::open_edit(&cfg, "deepseek");

        assert_eq!(edit.base_url, "https://api.deepseek.com/v1");
        assert_eq!(edit.materialize_provider.as_deref(), Some("deepseek"));
        assert!(edit.protocol_locked);
        assert!(!edit.fields().contains(&FormField::Preset));
        assert!(edit.fields().contains(&FormField::ApiKey));
        let original_preset = edit.preset_idx;
        edit.cycle_preset(true);
        assert_eq!(
            edit.preset_idx, original_preset,
            "curated vendor protocol must stay locked"
        );
        edit.api_key = "sk-test".into();
        ProviderPanel::apply_account_edit(&edit, &mut cfg);

        let account = cfg
            .provider_accounts
            .get("deepseek")
            .expect("editing a virtual preset must create its account");
        assert_eq!(account.provider, "deepseek");
        assert_eq!(account.api_key.as_deref(), Some("sk-test"));
        assert!(
            account.base_url.is_none(),
            "the preset default need not be duplicated in persisted config"
        );
    }

    #[test]
    fn configured_account_without_override_edits_with_preset_default_url() {
        let mut cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "main": { "provider": "openai" } }
        }))
        .unwrap();

        let edit = ProviderPanel::open_edit(&cfg, "main");
        assert_eq!(edit.base_url, "https://api.openai.com/v1");
        assert!(edit.materialize_provider.is_none());
        ProviderPanel::apply_account_edit(&edit, &mut cfg);
        assert!(
            cfg.provider_accounts["main"].base_url.is_none(),
            "a no-op edit should keep using the preset instead of persisting its default"
        );
    }

    #[test]
    fn only_unconfigured_rows_are_virtual_accounts() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "configured": { "provider": "deepseek" } },
            "providers": {
                "legacy": {
                    "type": "openai",
                    "base_url": "https://legacy/v1",
                    "model": "m",
                    "context_window": 8000
                }
            }
        }))
        .unwrap();

        assert!(ProviderPanel::is_virtual_account_row(&cfg, "deepseek"));
        assert!(!ProviderPanel::is_virtual_account_row(&cfg, "configured"));
        assert!(!ProviderPanel::is_virtual_account_row(&cfg, "legacy"));
    }

    #[test]
    fn model_form_add_vs_edit() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": { "acc/m": { "account": "acc", "model": "deepseek-chat", "context_window": 131072 } }
        }))
        .unwrap();
        // Add: account is a selectable field, defaults to an existing account.
        let add = ModelForm::new_add(&cfg, None).unwrap();
        assert!(add.fields().contains(&ModelField::Account));
        assert_eq!(add.account_id(), "acc");
        // A preferred (drilled-into) account is preselected.
        let cfg2: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "a1": { "provider": "deepseek" }, "z9": { "provider": "openai" } },
            "models": { "a1/m": { "account": "a1", "model": "x", "context_window": 8000 } }
        }))
        .unwrap();
        assert_eq!(
            ModelForm::new_add(&cfg2, Some("z9")).unwrap().account_id(),
            "z9"
        );
        // Edit: account locked; model + window pre-filled; id preserved.
        let edit = ModelForm::new_edit(&cfg, "acc/m").unwrap();
        assert!(!edit.fields().contains(&ModelField::Account));
        assert_eq!(edit.model, "deepseek-chat");
        assert_eq!(edit.window, "131072");
        assert_eq!(edit.edit_id.as_deref(), Some("acc/m"));
    }

    #[test]
    fn model_form_vision_cycles_auto_enabled_disabled_and_restores_edit_value() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "openai-compatible" } },
            "models": {
                "acc/qwen": {
                    "account": "acc",
                    "model": "qwen3.8max",
                    "supports_vision": true,
                    "context_window": 131072
                }
            }
        }))
        .unwrap();

        let mut add = ModelForm::new_add(&cfg, Some("acc")).unwrap();
        assert_eq!(add.supports_vision, None);
        add.cycle_vision(true);
        assert_eq!(add.supports_vision, Some(true));
        add.cycle_vision(true);
        assert_eq!(add.supports_vision, Some(false));
        add.cycle_vision(true);
        assert_eq!(add.supports_vision, None);
        add.cycle_vision(false);
        assert_eq!(add.supports_vision, Some(false));

        let edit = ModelForm::new_edit(&cfg, "acc/qwen").unwrap();
        assert_eq!(edit.supports_vision, Some(true));
        assert!(edit.fields().contains(&ModelField::Vision));
    }

    #[test]
    fn model_form_effort_defaults_off_cycles_and_restores_edit_value() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "openai-compatible" } },
            "models": {
                "acc/custom": {
                    "account": "acc",
                    "model": "vendor-model",
                    "reasoning_effort": "high",
                    "context_window": 131072
                }
            }
        }))
        .unwrap();

        let mut add = ModelForm::new_add(&cfg, Some("acc")).unwrap();
        assert_eq!(add.reasoning_effort, None);
        add.cycle_effort(true);
        assert_eq!(add.reasoning_effort.as_deref(), Some("auto"));
        add.cycle_effort(true);
        assert_eq!(add.reasoning_effort.as_deref(), Some("low"));
        add.cycle_effort(false);
        assert_eq!(add.reasoning_effort.as_deref(), Some("auto"));
        add.cycle_effort(false);
        assert_eq!(add.reasoning_effort, None);

        let edit = ModelForm::new_edit(&cfg, "acc/custom").unwrap();
        assert_eq!(edit.reasoning_effort.as_deref(), Some("high"));
        assert!(edit.fields().contains(&ModelField::Effort));
    }

    #[test]
    fn model_form_effort_levels_toggle_persist_and_couple_default() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "openai-compatible" } },
            "models": {
                "acc/sub": {
                    "account": "acc",
                    "model": "vendor-model",
                    "reasoning_effort_levels": ["low", "high", "max"],
                    "context_window": 131072
                }
            }
        }))
        .unwrap();

        // Pure config↔toggles round-trip (index = low/medium/high/max).
        assert_eq!(
            effort_levels_from_config(None),
            [true; EFFORT_LEVEL_COUNT],
            "None ⇒ all levels"
        );
        assert_eq!(
            effort_levels_to_config([true; EFFORT_LEVEL_COUNT]),
            None,
            "all ⇒ unrestricted"
        );
        assert_eq!(
            effort_levels_to_config([false; EFFORT_LEVEL_COUNT]),
            None,
            "none ⇒ unrestricted (no zero-levels state)"
        );
        assert_eq!(
            // low on, medium+xhigh off, high+max on (index = low/medium/high/xhigh/max).
            effort_levels_to_config([true, false, true, false, true]).as_deref(),
            Some(["low".to_string(), "high".to_string(), "max".to_string()].as_slice())
        );

        // new_add starts unrestricted and exposes the EffortLevels field.
        let mut add = ModelForm::new_add(&cfg, Some("acc")).unwrap();
        assert_eq!(add.effort_levels, [true; EFFORT_LEVEL_COUNT]);
        assert!(add.fields().contains(&ModelField::EffortLevels));
        // Toggle "medium" (cursor index 1) off.
        add.effort_level_cursor = 1;
        add.toggle_effort_level();
        assert_eq!(add.effort_levels, [true, false, true, true, true]);
        assert_eq!(
            add.effort_levels_label(true),
            " ● low ‹○ medium› ● high  ● xhigh  ● max "
        );
        assert_eq!(
            add.effort_levels_label(false),
            " ● low  ○ medium  ● high  ● xhigh  ● max "
        );
        // The DEFAULT cycle now skips medium: None -> auto -> low -> high.
        add.reasoning_effort = None;
        add.cycle_effort(true);
        add.cycle_effort(true);
        add.cycle_effort(true);
        assert_eq!(add.reasoning_effort.as_deref(), Some("high"));

        // Disabling the level that IS the current default resets it to auto.
        let mut f = ModelForm::new_add(&cfg, Some("acc")).unwrap();
        f.reasoning_effort = Some("medium".to_string());
        f.effort_level_cursor = 1;
        f.toggle_effort_level();
        assert_eq!(f.reasoning_effort.as_deref(), Some("auto"));

        // new_edit loads the persisted subset (declares low/high/max ⇒ medium+xhigh off).
        let edit = ModelForm::new_edit(&cfg, "acc/sub").unwrap();
        assert_eq!(edit.effort_levels, [true, false, true, false, true]);
    }

    #[test]
    fn paste_routes_to_account_and_model_form_fields() {
        let mut panel = ProviderPanel::open();

        let mut add = AddForm::new();
        add.focus = FormField::BaseUrl;
        panel.mode = Mode::Add(add);
        panel.apply_paste_text("  https://api.example.test/v1\r\nignored");
        let Mode::Add(add) = &panel.mode else {
            panic!("expected add-account form");
        };
        assert_eq!(add.base_url, "https://api.example.test/v1");

        panel.apply_paste_text("\n\n  /with-leading-newlines\r\nignored");
        let Mode::Add(add) = &panel.mode else {
            panic!("expected add-account form");
        };
        assert_eq!(
            add.base_url,
            "https://api.example.test/v1/with-leading-newlines"
        );

        let mut edit = EditForm {
            id: "account".into(),
            is_legacy: false,
            materialize_provider: None,
            preset_idx: protocol_preset_idx(provider_preset::ProviderType::OpenAi),
            original_preset_idx: protocol_preset_idx(provider_preset::ProviderType::OpenAi),
            protocol_locked: false,
            api_key: String::new(),
            base_url: String::new(),
            focus: FormField::ApiKey,
            cursor_byte: 0,
        };
        edit.focus = FormField::ApiKey;
        panel.mode = Mode::EditAccount(edit);
        panel.apply_paste_text("  sk-provider-key  \n");
        let Mode::EditAccount(edit) = &panel.mode else {
            panic!("expected edit-account form");
        };
        assert_eq!(edit.api_key, "sk-provider-key");

        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "openai", "api_key": "configured" } }
        }))
        .unwrap();
        let mut model = ModelForm::new_add(&cfg, Some("acc")).unwrap();
        model.focus = ModelField::Model;
        panel.mode = Mode::Model(model);
        panel.apply_paste_text("  vendor/model-name  \r");
        let Mode::Model(model) = &mut panel.mode else {
            panic!("expected model form");
        };
        assert_eq!(model.model, "vendor/model-name");

        model.focus = ModelField::Window;
        panel.apply_paste_text("128K tokens");
        let Mode::Model(model) = &panel.mode else {
            panic!("expected model form");
        };
        assert_eq!(model.window, "128");
    }

    #[test]
    fn model_list_groups_by_account() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": {
                "acc/z": { "account": "acc", "model": "z", "context_window": 8000 },
                "acc/a": { "account": "acc", "model": "a", "context_window": 8000 }
            }
        }))
        .unwrap();
        assert_eq!(
            ProviderPanel::model_ids(&cfg),
            vec!["acc/a".to_string(), "acc/z".to_string()]
        );
    }

    #[test]
    fn query_filters_accounts_by_id_and_vendor() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": {
                "openai-main": { "provider": "openai" },
                "deep": { "provider": "deepseek" }
            },
            "models": {
                "openai-main/gpt": { "account": "openai-main", "model": "gpt", "context_window": 8000 },
                "deep/chat": { "account": "deep", "model": "chat", "context_window": 8000 }
            }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        // Empty query -> configured accounts + all unconfigured preset vendors.
        let all = p.filtered_ids(&cfg);
        assert!(all.contains(&"openai-main".to_string()) && all.contains(&"deep".to_string()));
        assert!(all.len() > 2, "preset vendors are also listed");
        // Match by id substring: the "deep" account AND the "deepseek" preset.
        p.query = "deep".into();
        let d = p.filtered_ids(&cfg);
        assert!(d.contains(&"deep".to_string()) && d.contains(&"deepseek".to_string()));
        // Match by vendor: "openai-main" (provider openai) surfaces for "openai".
        p.query = "openai".into();
        assert!(p.filtered_ids(&cfg).contains(&"openai-main".to_string()));
        // No match -> empty.
        p.query = "zzznomatch".into();
        assert!(p.filtered_ids(&cfg).is_empty());
    }

    #[test]
    fn account_ids_lists_unconfigured_preset_vendors() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "RustCode": { "provider": "openai", "base_url": "https://gateway.test.example/v1" } }
        }))
        .unwrap();
        let ids = ProviderPanel::account_ids(&cfg);
        assert!(
            ids.first() == Some(&"RustCode".to_string()),
            "configured first"
        );
        assert_eq!(
            ids.get(1).map(String::as_str),
            Some("taotoken"),
            "TaoToken should be the first quick-add vendor below RustCode"
        );
        assert!(
            ids.contains(&"deepseek".to_string()),
            "unconfigured vendor listed"
        );
        // Custom-endpoint presets are reached via the add-custom row, not listed.
        assert!(!ids.contains(&"openai-compatible".to_string()));
        assert!(!ids.contains(&"anthropic-compatible".to_string()));
        // Neutrality guard: the legacy upstream gateway vendor id must never
        // appear as a quick-add raw-key account -- the managed gateway is
        // reached exclusively through the /login OAuth signer.
        assert!(!ids.contains(&"atomgit".to_string()));
        // A preset with a concrete default endpoint is quick-addable.
        assert!(ids.contains(&"xiaomi-mimo".to_string()));
        // A keyed preset vendor prompts for a key when you add its first model.
        assert!(account_needs_key(&cfg, "deepseek"));
    }

    #[test]
    fn account_label_uses_preset_display_name_without_replacing_custom_ids() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": {
                "custom-openai": { "provider": "openai" },
                "named": { "provider": "openai", "display_name": "My Gateway" },
                "taotoken": { "provider": "taotoken" }
            }
        }))
        .unwrap();

        assert_eq!(ProviderPanel::account_label(&cfg, "taotoken"), "TaoToken");
        assert_eq!(
            ProviderPanel::account_label(&Config::default(), "taotoken"),
            "TaoToken"
        );
        assert_eq!(
            ProviderPanel::account_label(&cfg, "custom-openai"),
            "custom-openai"
        );
        assert_eq!(ProviderPanel::account_label(&cfg, "named"), "My Gateway");
    }

    #[test]
    fn model_form_prompts_for_key_on_keyless_provider() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": {
                "custom": { "provider": "openai-compatible", "base_url": "https://x/v1" },
                "keyed": { "provider": "openai-compatible", "base_url": "https://y/v1", "api_key": "sk-1" }
            }
        }))
        .unwrap();
        assert!(account_needs_key(&cfg, "custom"));
        assert!(!account_needs_key(&cfg, "keyed"));
        // The model form shows an api_key field only for the keyless provider.
        assert!(ModelForm::new_add(&cfg, Some("custom"))
            .unwrap()
            .fields()
            .contains(&ModelField::ApiKey));
        assert!(!ModelForm::new_add(&cfg, Some("keyed"))
            .unwrap()
            .fields()
            .contains(&ModelField::ApiKey));
    }

    #[test]
    fn account_filter_restricts_models_tab_to_one_account() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "RustCode": { "provider": "openai" }, "other": { "provider": "openai" } },
            "models": {
                "RustCode-a": { "account": "RustCode", "model": "a", "context_window": 8000 },
                "RustCode-b": { "account": "RustCode", "model": "b", "context_window": 8000 },
                "other/x": { "account": "other", "model": "x", "context_window": 8000 }
            }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        p.tab = Tab::Models;
        // No filter -> all models.
        assert_eq!(p.filtered_ids(&cfg).len(), 3);
        // Drill into RustCode -> only its two models.
        p.account_filter = Some("RustCode".into());
        assert_eq!(
            p.filtered_ids(&cfg),
            vec!["RustCode-a".to_string(), "RustCode-b".to_string()]
        );
        // A typed query narrows further, within the account.
        p.query = "b".into();
        assert_eq!(p.filtered_ids(&cfg), vec!["RustCode-b".to_string()]);
        // The account filter only applies to the Models tab; the Accounts tab
        // lists both configured accounts (plus preset vendors).
        p.query.clear();
        p.tab = Tab::Accounts;
        let acc = p.filtered_ids(&cfg);
        assert!(acc.contains(&"RustCode".to_string()) && acc.contains(&"other".to_string()));
    }

    #[test]
    fn added_account_stays_open_on_its_models_page() {
        let mut panel = ProviderPanel::open();
        panel.selected = 3;
        panel.query = "stale".into();
        panel.pending_delete = Some(("old".into(), true));

        panel.show_models_for_account("taotoken");

        assert!(panel.tab == Tab::Models);
        assert_eq!(panel.selected, 0);
        assert!(matches!(panel.mode, Mode::List));
        assert_eq!(panel.account_filter.as_deref(), Some("taotoken"));
        assert!(panel.query.is_empty());
        assert!(panel.pending_delete.is_none());
    }

    #[test]
    fn query_filters_models_by_name_and_account() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": {
                "acc/chat": { "account": "acc", "model": "deepseek-chat", "context_window": 8000 },
                "acc/reason": { "account": "acc", "model": "deepseek-reasoner", "context_window": 8000 }
            }
        }))
        .unwrap();
        let mut p = ProviderPanel::open();
        p.tab = Tab::Models;
        // Match by model name substring.
        p.query = "reason".into();
        assert_eq!(p.filtered_ids(&cfg), vec!["acc/reason".to_string()]);
        // Account name matches both models.
        p.query = "acc".into();
        assert_eq!(p.filtered_ids(&cfg).len(), 2);
    }

    #[test]
    fn delete_requires_two_presses_on_the_same_logical_row() {
        let mut panel = ProviderPanel::open();

        assert!(!panel.confirm_double_delete("account-a", true));
        assert_eq!(panel.pending_delete, Some(("account-a".to_string(), true)));
        assert!(panel.confirm_double_delete("account-a", true));
        assert!(panel.pending_delete.is_none());

        assert!(!panel.confirm_double_delete("account-a", true));
        assert!(!panel.confirm_double_delete("account-b", true));
        assert_eq!(panel.pending_delete, Some(("account-b".to_string(), true)));

        // An account and model with the same id are still distinct targets.
        assert!(!panel.confirm_double_delete("account-b", false));
    }

    fn account(
        provider: &str,
        base_url: Option<&str>,
        api_key: Option<&str>,
    ) -> ProviderAccountConfig {
        let mut json = serde_json::json!({ "provider": provider });
        if let Some(url) = base_url {
            json["base_url"] = serde_json::json!(url);
        }
        if let Some(key) = api_key {
            json["api_key"] = serde_json::json!(key);
        }
        serde_json::from_value(json).unwrap()
    }

    fn discovered(id: &str, name: &str) -> DiscoveredModel {
        DiscoveredModel {
            id: id.to_string(),
            name: Some(name.to_string()),
            context_window: Some(64_000),
            max_tokens: None,
        }
    }

    #[test]
    fn discovery_target_prefers_the_account_endpoint_and_credential() {
        let target = discovery_target_for_account(
            &account("openai", Some("https://mirror.test/v1"), Some("sk-acc")),
            "acc",
        );
        assert_eq!(target.account_id, "acc");
        assert_eq!(target.provider_type, "openai");
        assert_eq!(target.base_url, "https://mirror.test/v1");
        assert_eq!(target.api_key.as_deref(), Some("sk-acc"));
        assert!(target.is_discoverable());
    }

    #[test]
    fn discovery_target_falls_back_to_the_preset_endpoint_and_protocol() {
        // No stored endpoint: the preset default is authoritative (DeepSeek ships
        // one) and the protocol follows the preset.
        let deepseek = discovery_target_for_account(&account("deepseek", None, None), "ds");
        assert_eq!(deepseek.base_url, "https://api.deepseek.com/v1");
        assert_eq!(deepseek.provider_type, "openai");

        // Ollama is keyless and lists through its own tags endpoint.
        let ollama = discovery_target_for_account(&account("ollama", None, None), "local");
        assert_eq!(ollama.provider_type, "ollama");
        assert_eq!(ollama.base_url, "http://localhost:11434");

        // Anthropic is probed over the OpenAI-compatible listing, as before.
        let anthropic =
            discovery_target_for_account(&account("anthropic", None, Some("sk-ant")), "an");
        assert_eq!(anthropic.provider_type, "openai");
    }

    #[test]
    fn discovery_target_without_any_endpoint_is_not_discoverable() {
        // The generic compatible presets ship no default endpoint; an account that
        // never set one has nothing to probe.
        let target = discovery_target_for_account(&account("openai-compatible", None, None), "c");
        assert_eq!(target.base_url, "");
        assert!(!target.is_discoverable());

        // ...and starting discovery for it must neither spawn a worker nor move the
        // panel out of Idle.
        let mut panel = ProviderPanel::open();
        assert!(!panel.start_discovery(target));
        assert_eq!(panel.discovery_state, DiscoveryState::Idle);
        assert!(panel.discovery_rx.is_none());
    }

    #[test]
    fn only_an_in_flight_request_blocks_the_next_discovery() {
        assert!(can_start_discovery(&DiscoveryState::Idle));
        // A failure/empty notice must stay re-tryable: the old terminal states
        // refused every later request for the rest of the panel's life.
        assert!(can_start_discovery(&DiscoveryState::Notice {
            message: "boom".into()
        }));
        assert!(!can_start_discovery(&DiscoveryState::Pending {
            account_id: "acc".into(),
            base_url: "https://example.test/v1".into(),
        }));
    }

    #[test]
    fn a_just_saved_account_yields_a_target_before_the_config_reload_lands() {
        // Regression: the add flow resolved discovery from `ctx.config`, which does
        // not contain a just-saved account until the asynchronous provider reload
        // lands -- so discovery never started and the panel showed an empty list
        // with no error. The target is now built from the values being committed.
        let pre_save = Config::default();
        assert!(!pre_save.provider_accounts.contains_key("deepseek"));

        let target =
            discovery_target_for_account(&account("deepseek", None, Some("sk-test")), "deepseek");

        assert_eq!(target.account_id, "deepseek");
        assert!(target.is_discoverable());
        assert_eq!(target.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn discovery_plan_skips_models_the_catalog_already_has() {
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "provider_accounts": { "acc": { "provider": "deepseek" } },
            "models": { "acc/deepseek-chat": { "account": "acc", "model": "deepseek-chat" } }
        }))
        .unwrap();
        let models = vec![
            discovered("deepseek-chat", "DeepSeek Chat"),
            discovered("deepseek-reasoner", "DeepSeek Reasoner"),
        ];

        let plan = plan_discovered_models("acc", &models, &cfg);

        // Re-discovering an account is idempotent: the already-configured model is
        // skipped, the new one keeps the "<account>/<wire id>" selection id.
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].0, "acc/deepseek-reasoner");
        assert_eq!(plan[0].1.id, "deepseek-reasoner");
        assert_eq!(plan[0].1.name.as_deref(), Some("DeepSeek Reasoner"));
    }
}
