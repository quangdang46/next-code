//! Stub of upstream `xai-grok-shell::agent::folder_trust`.

use std::path::Path;

pub fn grant_folder_trust(_cwd: &Path) {}

pub fn is_trusted(_cwd: &Path) -> bool {
    true
}
