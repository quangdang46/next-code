//! Install plugins from a marketplace source (stub).

use xai_grok_agent::plugins::install_registry::InstallRegistry;

/// Check if a plugin from a specific marketplace source is already installed.
///
/// Stub: always `None`.
pub fn find_installed_marketplace_plugin(
    _registry: &InstallRegistry,
    _source_url_or_path: &str,
    _plugin_subdir: &str,
) -> Option<(String, String)> {
    None
}
