//! Trust store + workspace key stubs (no git2).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct TrustStore {
    trusted: Vec<PathBuf>,
}

impl TrustStore {
    pub fn load() -> Self {
        Self::default()
    }

    pub fn is_trusted(&self, key: &Path) -> bool {
        self.trusted.iter().any(|p| p == key)
    }

    pub fn grant(&mut self, key: &Path) {
        let key = key.to_path_buf();
        if !self.trusted.contains(&key) {
            self.trusted.push(key);
        }
    }
}

fn canonicalize_or_owned(path: &Path) -> PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn is_home_dir(path: &Path) -> bool {
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    canonicalize_or_owned(path) == canonicalize_or_owned(&home)
}

pub fn is_unsafe_trust_root(key: &Path) -> bool {
    if !key.is_absolute() {
        return true;
    }
    if key.parent().is_none() {
        return true;
    }
    is_home_dir(key)
}

/// Workspace trust key: canonicalize cwd (no git discovery in stub).
pub fn workspace_key(cwd: &Path) -> PathBuf {
    let key = canonicalize_or_owned(cwd);
    if is_unsafe_trust_root(&key) {
        return canonicalize_or_owned(cwd);
    }
    key
}
