//! Façade re-export of upstream `xai-grok-shell::util::grok_home`. Home
//! resolution itself already lives in `xai-grok-config` (PR3: `$GROK_HOME`
//! > `$NEXT_CODE_HOME` > `~/.next-code`) — re-exported here so
//! `xai_grok_shell::util::grok_home::grok_home()` keeps working.
pub use xai_grok_config::grok_home;

/// Upstream derives a filesystem-safe dirname from a cwd for per-project
/// session storage under the home dir; this stub does a simple
/// lossy/sanitizing encode (not upstream's exact scheme) since no
/// consumer round-trips it yet in this compile-stub layer.
pub fn encode_cwd_dirname(cwd: impl AsRef<str>) -> String {
    cwd.as_ref()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}
