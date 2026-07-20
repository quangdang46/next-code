//! Stub of upstream `xai-grok-agent::plugins::git_install`. Only
//! `InstallSource` (imported by the future pager's `plugin_cmd.rs`) plus
//! `parse_install_source` are kept faithful to upstream shape; the actual
//! git clone / local copy / discovery logic is out of scope for this
//! compile-stub layer (real installs are a runtime-side concern, PR8+).

use std::path::{Path, PathBuf};

/// Source of a plugin installation.
#[derive(Debug, Clone)]
pub enum InstallSource {
    Git {
        url: String,
        git_ref: Option<String>,
        git_sha: Option<String>,
        subdir: Option<String>,
    },
    Local {
        path: PathBuf,
        subdir: Option<String>,
    },
}

pub struct InstallResult {
    pub repo_key: String,
    pub repo_path: PathBuf,
    pub plugins: Vec<DiscoveredPlugin>,
    pub commit: Option<String>,
}

pub struct DiscoveredPlugin {
    pub name: String,
    pub subdir: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("git-install stub: not implemented")]
    NotImplemented,
}

/// Faithful to upstream's parsing rules (git URL / SSH / GitHub shorthand /
/// local path), since the future pager's `plugin add <source>` slash
/// command depends on this returning the right variant even before real
/// installs are wired.
pub fn parse_install_source(input: &str, cwd: &Path) -> InstallSource {
    let (base, subdir) = match input.split_once('#') {
        Some((b, s)) => (b, Some(s.to_string())),
        None => (input, None),
    };

    if base.starts_with("http://")
        || base.starts_with("https://")
        || base.starts_with("git@")
        || base.ends_with(".git")
    {
        let (url, git_ref) = match base.split_once('@') {
            Some((u, r)) => (u.to_string(), Some(r.to_string())),
            None => (base.to_string(), None),
        };
        return InstallSource::Git {
            url,
            git_ref,
            git_sha: None,
            subdir,
        };
    }

    // `user/repo[@ref]` GitHub shorthand: exactly one `/`, no leading `.`/`~`/path sep.
    if !base.starts_with('.')
        && !base.starts_with('~')
        && !base.starts_with('/')
        && !base.contains('\\')
        && base.matches('/').count() == 1
    {
        let (rest, git_ref) = match base.split_once('@') {
            Some((r, gr)) => (r.to_string(), Some(gr.to_string())),
            None => (base.to_string(), None),
        };
        return InstallSource::Git {
            url: format!("https://github.com/{rest}"),
            git_ref,
            git_sha: None,
            subdir,
        };
    }

    let path = if let Some(stripped) = base.strip_prefix('~') {
        dirs::home_dir()
            .unwrap_or_default()
            .join(stripped.trim_start_matches(['/', '\\']))
    } else {
        cwd.join(base)
    };
    InstallSource::Local { path, subdir }
}

pub fn install_from_source(
    _source: &InstallSource,
    _install_dir: &Path,
) -> Result<InstallResult, InstallError> {
    Err(InstallError::NotImplemented)
}
