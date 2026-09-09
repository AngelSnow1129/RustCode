use std::sync::Arc;

use rustcode_coding::cc_hooks::HookConfig;
use rustcode_coding::PluginHookSource;

#[derive(Debug, Default)]
pub struct InstalledPluginHookSource;

impl PluginHookSource for InstalledPluginHookSource {
    fn load(&self) -> Result<Vec<HookConfig>, String> {
        rustcode_capabilities::plugin::hook_trust::ensure_migrated();
        Ok(
            rustcode_capabilities::plugin::loader::installed_plugin_cc_hooks()
                .into_iter()
                .filter_map(|hook| {
                    HookConfig::from_plugin_spec(
                        &hook.event,
                        hook.matcher,
                        hook.command,
                        hook.timeout_secs,
                        hook.plugin_root,
                    )
                })
                .collect(),
        )
    }
}

pub fn installed_plugin_hook_source() -> Arc<dyn PluginHookSource> {
    Arc::new(InstalledPluginHookSource)
}

pub fn gather_plugin_skill_dirs() -> Vec<(std::path::PathBuf, String)> {
    let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    gather_plugin_skill_dirs_for(&working_dir)
}

pub fn gather_plugin_skill_dirs_for(
    working_dir: &std::path::Path,
) -> Vec<(std::path::PathBuf, String)> {
    rustcode_capabilities::plugin::loader::installed_plugin_skill_dirs(working_dir)
}

pub fn coding_provider_factory() -> Arc<dyn rustcode_coding::CodingProviderFactory> {
    rustcode_coding::codingplan_provider_factory(rustcode_auth::RUSTCODE_USER_AGENT)
}
