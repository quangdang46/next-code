//! Stub of upstream `xai-grok-shell::claude_import_state`.

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportState {
    #[serde(default)]
    pub scopes: Vec<ScopeState>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeState {
    pub cwd: String,
    pub imported: bool,
    pub dismissed: bool,
}

pub fn load_import_state() -> ImportState {
    ImportState::default()
}

pub fn save_import_state(_state: &ImportState) -> std::io::Result<()> {
    Ok(())
}

pub fn has_new_changes(_cwd: &Path) -> bool {
    false
}

pub fn mark_imported(_cwd: &Path) {}

pub fn mark_dismissed(_cwd: &Path) {}
