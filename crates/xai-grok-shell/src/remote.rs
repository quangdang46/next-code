//! Stub of upstream `xai-grok-shell::remote`.

use crate::util::config::RemoteSettings;

pub fn proxy_url() -> String {
    String::new()
}

/// Blocking settings fetch used by the pager gate-refresh path (Face stub).
pub fn fetch_settings_blocking(
    _proxy_base: &str,
    _auth: &serde_json::Value,
    _extra: Option<&str>,
) -> Option<RemoteSettings> {
    Some(RemoteSettings::default())
}
