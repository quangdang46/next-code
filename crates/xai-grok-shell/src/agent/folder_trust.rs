//! Folder-trust consume façade. Decision/persist live in `xai_grok_workspace`.

use std::path::Path;

pub use xai_grok_workspace::folder_trust::grant_folder_trust;

pub fn is_trusted(cwd: &Path) -> bool {
    let key = xai_grok_workspace::trust::workspace_key(cwd);
    xai_grok_workspace::trust::TrustStore::load().is_trusted(&key)
}
