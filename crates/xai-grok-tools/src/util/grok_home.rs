//! Session path helpers (shim).

use std::path::{Path, PathBuf};

use xai_grok_config::grok_home;

/// Directory for sessions under a given cwd key.
pub fn sessions_cwd_dir(cwd: &str) -> PathBuf {
    let key = encode_cwd_dirname(cwd);
    grok_home().join("sessions").join(key)
}

pub fn ensure_sessions_cwd_dir(cwd: &str) -> std::io::Result<PathBuf> {
    let dir = sessions_cwd_dir(cwd);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn encode_cwd_dirname(cwd: &str) -> String {
    cwd.replace(['\\', '/', ':', '<', '>', '"', '|', '?', '*'], "_")
}

pub fn decode_cwd_from_dirname(name: &str) -> String {
    name.to_string()
}

pub fn grok_application() -> PathBuf {
    grok_home()
}

pub fn encode_path_key(path: &Path) -> String {
    encode_cwd_dirname(&path.to_string_lossy())
}
