//! Folder-trust decision stubs. Local/dev builds treat the feature as off.

use std::path::Path;

use xai_grok_config::RemoteSettings;

use crate::trust::{TrustStore, is_unsafe_trust_root};

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

/// Folder-trust is inert on local builds (always auto-trust).
pub fn folder_trust_inert() -> bool {
    true
}

pub fn feature_enabled(remote: Option<&RemoteSettings>) -> bool {
    let _ = remote;
    false
}

pub fn grant_folder_trust(cwd: &Path) {
    let _ = cwd;
}

pub fn revoke_folder_trust_store(cwd: &Path) -> bool {
    let _ = cwd;
    false
}
