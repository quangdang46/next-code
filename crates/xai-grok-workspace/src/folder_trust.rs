//! Folder-trust decision + grant/revoke helpers.

use std::path::Path;

use crate::trust::{TrustStore, is_unsafe_trust_root, workspace_key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustOutcome {
    Trusted,
    Untrusted,
    Prompt,
}

#[derive(Debug, Clone, Copy)]
pub struct DecideInputs {
    pub store_trusted: bool,
    pub repo_configs_present: bool,
    pub is_interactive: bool,
    pub key_recordable: bool,
}

pub fn decide(feature_enabled: bool, i: &DecideInputs) -> TrustOutcome {
    if !feature_enabled {
        return TrustOutcome::Trusted;
    }
    if i.store_trusted {
        return TrustOutcome::Trusted;
    }
    if !i.key_recordable {
        return TrustOutcome::Trusted;
    }
    if !i.repo_configs_present {
        return TrustOutcome::Trusted;
    }
    if i.is_interactive {
        return TrustOutcome::Prompt;
    }
    TrustOutcome::Untrusted
}

pub fn decide_inputs(cwd: &Path, key: &Path) -> DecideInputs {
    decide_inputs_with_interactive(cwd, key, true)
}

pub fn decide_inputs_with_interactive(
    cwd: &Path,
    key: &Path,
    is_interactive: bool,
) -> DecideInputs {
    let _ = cwd;
    DecideInputs {
        store_trusted: TrustStore::load().is_trusted(key),
        repo_configs_present: false,
        is_interactive,
        key_recordable: !is_unsafe_trust_root(key),
    }
}

pub fn folder_trust_inert() -> bool {
    is_local_build()
}

fn is_local_build() -> bool {
    if std::env::var(xai_grok_version::TEST_VERSION_ENV).is_ok() {
        return false;
    }
    option_env!("GROK_VERSION").is_none()
}

pub fn feature_enabled<R>(_remote: Option<&R>) -> bool {
    if is_local_build() {
        return false;
    }
    true
}

pub fn grant_folder_trust(cwd: &Path) {
    if folder_trust_inert() {
        return;
    }
    persist_trust(&mut TrustStore::load(), &workspace_key(cwd));
}

pub fn revoke_folder_trust_store(cwd: &Path) -> bool {
    if folder_trust_inert() {
        return false;
    }
    let key = workspace_key(cwd);
    let mut store = TrustStore::load();
    let was_trusted = store.is_trusted(&key);
    if was_trusted {
        let _ = store.set_untrusted(&key);
    }
    was_trusted
}

pub fn persist_trust(store: &mut TrustStore, key: &Path) {
    let _ = store.set_trusted(key);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn grant_persists_when_release_simulated() {
        let _guard = crate::trust::tests::env_lock();
        let home = TempDir::new().unwrap();
        let prev_home = std::env::var_os("GROK_HOME");
        let prev_ver = std::env::var_os(xai_grok_version::TEST_VERSION_ENV);
        unsafe {
            std::env::set_var("GROK_HOME", home.path());
            std::env::set_var(xai_grok_version::TEST_VERSION_ENV, "0.0.0-sim");
        }
        let repo = TempDir::new().unwrap();
        let key = workspace_key(repo.path());
        assert!(!folder_trust_inert());
        grant_folder_trust(repo.path());
        let store_path = home.path().join(crate::trust::TRUST_FILE_NAME);
        assert!(
            TrustStore::load_from(store_path).is_trusted(&key),
            "grant must write trusted_folders.toml under GROK_HOME"
        );
        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("GROK_HOME", v),
                None => std::env::remove_var("GROK_HOME"),
            }
            match prev_ver {
                Some(v) => std::env::set_var(xai_grok_version::TEST_VERSION_ENV, v),
                None => std::env::remove_var(xai_grok_version::TEST_VERSION_ENV),
            }
        }
    }
}
