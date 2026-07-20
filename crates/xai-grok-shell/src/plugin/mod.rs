//! Stub of upstream `xai-grok-shell::plugin`.

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum UninstallError {
    #[error("uninstall stub: {0}")]
    Stub(String),
    #[error("plugin not found: {name}")]
    NotFound { name: String },
    #[error("needs confirm")]
    NeedsConfirm {
        name: String,
        repo_key: String,
        other_plugins: Vec<String>,
        total: usize,
    },
}

#[derive(Debug, Clone)]
pub enum RepoUpdateOutcome {
    Updated {
        repo_key: String,
        old_commit: Option<String>,
        new_commit: Option<String>,
    },
    AlreadyUpToDate { repo_key: String },
    Pinned {
        repo_key: String,
        ref_name: String,
    },
    LiveLocal { repo_key: String },
    Failed {
        repo_key: String,
        error: String,
    },
}

#[derive(Debug, Clone)]
pub enum MarketplaceAddInput {
    GitUrl(String),
    LocalPath(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("install stub: {0}")]
    Stub(String),
}

impl InstallError {
    pub fn category(&self) -> &'static str {
        "stub"
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstallOutcome {
    pub repo_key: String,
    pub plugin_names: Vec<String>,
    pub warnings: Vec<String>,
    pub is_local: bool,
    pub name: String,
    pub path: PathBuf,
    pub source_display_name: String,
    pub source_is_git: bool,
    pub already_installed: bool,
    pub other_copies_note: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UninstallOutcome {
    pub removed_plugins: Vec<String>,
}

pub fn install_source_is_local(source: &str, _cwd: &Path) -> bool {
    Path::new(source).exists()
}

pub fn install_plugin(_source: &str, _cwd: &Path) -> Result<InstallOutcome, InstallError> {
    Err(InstallError::Stub("not implemented".into()))
}

pub fn uninstall_plugin(
    _name: &str,
    _confirm: bool,
    _keep_data: bool,
) -> Result<UninstallOutcome, UninstallError> {
    Err(UninstallError::Stub("not implemented".into()))
}

pub fn classify_install_error(_err: &InstallError) -> String {
    "stub".into()
}

pub fn update_plugins(_name: Option<&str>) -> Result<Vec<RepoUpdateOutcome>, String> {
    Ok(vec![])
}

pub fn install_marketplace_plugin(
    _name: &str,
    _qualifier: Option<&str>,
) -> Result<InstallOutcome, InstallError> {
    Err(InstallError::Stub("not implemented".into()))
}

pub fn resolve_marketplace_source_name(
    name: &str,
    _qualifier: Option<&str>,
) -> Result<String, String> {
    Ok(name.to_string())
}

pub fn resolve_qualified_source_name(qualifier: &str) -> Result<String, String> {
    Ok(qualifier.to_string())
}

pub fn uninstall_marketplace_source_plugins(_source_identity: &str) -> Vec<String> {
    vec![]
}

pub fn remove_toml_marketplace_block(_content: &str, _source_identity: &str) -> Option<String> {
    None
}

pub fn try_remove_source_from_json_files(_source_url_or_path: &str) -> bool {
    false
}

pub fn classify_marketplace_add_input(url: &str, cwd: &Path) -> MarketplaceAddInput {
    let p = Path::new(url);
    if p.is_absolute() || cwd.join(p).exists() {
        MarketplaceAddInput::LocalPath(if p.is_absolute() {
            p.to_path_buf()
        } else {
            cwd.join(p)
        })
    } else {
        MarketplaceAddInput::GitUrl(url.to_string())
    }
}

pub fn name_from_url(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_string()
}

pub fn name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("plugin")
        .to_string()
}

pub fn normalize_git_url(url: &str) -> String {
    url.to_string()
}
