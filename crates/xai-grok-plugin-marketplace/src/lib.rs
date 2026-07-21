//! Facade of `xai-org/grok-build` `xai-grok-plugin-marketplace` (Apache-2.0)
//! for the next-code Grok Face migration (PR7).
//!
//! Upstream browses/installs marketplace plugins. This stub only reproduces
//! the types and empty/Err entry points the pager imports.

pub mod git;
pub mod install_resolve;
pub mod installer;
pub mod matcher;
pub mod types;

pub use types::*;

/// Display name of the official xAI marketplace source.
pub const OFFICIAL_SOURCE_NAME: &str = "xAI Official";

/// Git URL of the official xAI marketplace source.
pub const OFFICIAL_SOURCE_GIT_URL: &str = "https://github.com/xai-org/plugin-marketplace.git";

/// Whether `url` is the official xAI marketplace source (normalized compare).
pub fn is_official_source_url(url: &str) -> bool {
    canonical_github_owner_repo(url).as_deref() == Some("xai-org/plugin-marketplace")
}

/// Normalized lowercase `owner/repo` from a GitHub URL, or `None`.
pub(crate) fn canonical_github_owner_repo(url: &str) -> Option<String> {
    let s = url.trim();
    let s = s.strip_suffix('/').unwrap_or(s);
    let s = s.strip_suffix(".git").unwrap_or(s);
    let lower = s.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .or_else(|| lower.strip_prefix("ssh://"))
        .unwrap_or(&lower);
    let rest = rest.strip_prefix("git@").unwrap_or(rest);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    let owner_repo = rest
        .strip_prefix("github.com/")
        .or_else(|| rest.strip_prefix("github.com:"))?;
    if owner_repo.is_empty() {
        None
    } else {
        Some(owner_repo.to_string())
    }
}

/// Load marketplace sources from config. Stub: always empty.
pub fn load_sources(_config: &toml::Value) -> Vec<MarketplaceSource> {
    Vec::new()
}

/// Extra sources from settings. Stub: always empty.
pub fn load_extra_sources_from_settings(
    _existing: &[MarketplaceSource],
) -> Vec<MarketplaceSource> {
    Vec::new()
}

/// Scan a marketplace root. Stub: always empty.
pub fn scan_marketplace(_root: &std::path::Path) -> MarketplaceScan {
    MarketplaceScan {
        entries: Vec::new(),
        catalog_loaded: false,
    }
}
