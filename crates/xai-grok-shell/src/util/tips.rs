//! Stub of upstream `xai-grok-shell::util::tips`.

use std::path::Path;

/// Pick one tip and advance the rotation cursor (Face stub: first tip).
pub fn pick_and_advance(tips: &[String], _grok_home: &Path) -> Option<String> {
    tips.first().cloned()
}
