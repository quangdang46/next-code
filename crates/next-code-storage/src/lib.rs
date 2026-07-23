use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::path::{Path, PathBuf};

mod active_pids;
pub use active_pids::{
    SessionCounts, SessionPresence, StreamingGuard, active_pids_dir, active_session_ids,
    find_active_session_id_by_pid, mark_streaming, register_active_pid, session_counts,
    session_presence, streaming_pids_dir, unmark_streaming, unregister_active_pid,
};

/// Shared env lock for tests that mutate process-global product home vars.
/// Must be crate-level so `lib` and `active_pids` tests cannot race.
#[cfg(test)]
pub(crate) mod test_env {
    use std::sync::{Mutex, MutexGuard};

    pub fn lock_env() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn clear_home_env() {
        next_code_core::env::remove_var("NEXT_CODE_HOME");
    }
}

/// Platform-aware runtime directory for sockets and ephemeral state.
///
/// - Linux: `$XDG_RUNTIME_DIR` (typically `/run/user/<uid>`)
/// - macOS: `$TMPDIR` (per-user, e.g. `/var/folders/xx/.../T/`)
/// - Fallback: `std::env::temp_dir()`
///
/// Can be overridden with `$NEXT_CODE_RUNTIME_DIR`.
pub fn runtime_dir() -> PathBuf {
    if let Ok(dir) = next_code_core::env::product_env("RUNTIME_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(dir) = std::env::var("TMPDIR") {
            return PathBuf::from(dir);
        }
    }

    let dir = fallback_runtime_dir();
    ensure_private_runtime_dir(&dir);
    dir
}

fn fallback_runtime_dir() -> PathBuf {
    std::env::temp_dir().join(format!("next-code-{}", runtime_user_discriminator()))
}

#[cfg(unix)]
fn runtime_user_discriminator() -> String {
    unsafe { libc::geteuid() }.to_string()
}

#[cfg(not(unix))]
fn runtime_user_discriminator() -> String {
    let raw = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string());
    let sanitized: String = raw
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .take(64)
        .collect();
    if sanitized.is_empty() {
        "user".to_string()
    } else {
        sanitized
    }
}

fn ensure_private_runtime_dir(path: &Path) {
    let _ = std::fs::create_dir_all(path);
    #[cfg(unix)]
    {
        let _ = next_code_core::fs::set_directory_permissions_owner_only(path);
    }
}

/// Resolve the next-code home directory.
///
/// Resolution order:
/// 1. `$NEXT_CODE_HOME`
/// 2. `~/.next-code`
pub fn next_code_dir() -> Result<PathBuf> {
    if let Ok(path) = next_code_core::env::product_env("HOME") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory"))?;
    Ok(home.join(".next-code"))
}

/// Project-local product directory name.
pub const PROJECT_DIR_CANDIDATES: &[&str] = &[".next-code"];

/// Resolve a path under the project-local product directory
/// (`<root>/.next-code/<relative>`).
pub fn project_product_path(root: &Path, relative: impl AsRef<Path>) -> PathBuf {
    root.join(".next-code").join(relative)
}

/// Resolve the project-local product root directory (`.next-code`).
pub fn project_product_dir(root: &Path) -> PathBuf {
    root.join(".next-code")
}

pub fn logs_dir() -> Result<PathBuf> {
    Ok(next_code_dir()?.join("logs"))
}

/// Durable state directory for state that must survive reboots.
///
/// [`runtime_dir`] typically resolves to a tmpfs (for example
/// `/run/user/<uid>` on Linux) that is wiped on reboot, so it must only hold
/// sockets and truly ephemeral state. State that has to outlive a reboot,
/// such as swarm plans and member records, belongs here instead: it resolves
/// to `~/.next-code/state` (respecting `NEXT_CODE_HOME`).
///
/// When `NEXT_CODE_RUNTIME_DIR` is set (tests and
/// sandboxed temp servers), it takes precedence so isolated runs never touch
/// the real next-code home.
pub fn durable_state_dir() -> PathBuf {
    if let Ok(dir) = next_code_core::env::product_env("RUNTIME_DIR") {
        return PathBuf::from(dir).join("durable-state");
    }
    match next_code_dir() {
        Ok(dir) => dir.join("state"),
        Err(_) => runtime_dir().join("durable-state"),
    }
}

/// Resolve next-code's app-owned config directory.
///
/// Default location is the platform config dir + `next-code` (for example
/// `~/.config/next-code` on Linux). When `NEXT_CODE_HOME` is set, sandbox this
/// under `$HOME/config/next-code` so self-dev/tests do not leak into the user's
/// real config directory.
///
pub fn app_config_dir() -> Result<PathBuf> {
    if let Ok(path) = next_code_core::env::product_env("HOME") {
        return Ok(PathBuf::from(path).join("config").join("next-code"));
    }

    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config directory found"))?;
    Ok(config_dir.join("next-code"))
}


/// Resolve a path under the user's home directory, but sandbox it under
/// `$NEXT_CODE_HOME/external/` when a
/// product home override is set.
///
/// This keeps external provider auth files isolated during tests and sandboxed
/// runs without changing default on-disk locations for normal users.
pub fn user_home_path(relative: impl AsRef<Path>) -> Result<PathBuf> {
    let relative = relative.as_ref();
    if relative.is_absolute() {
        anyhow::bail!(
            "user_home_path expects a relative path, got {}",
            relative.display()
        );
    }

    if let Ok(path) = next_code_core::env::product_env("HOME") {
        return Ok(PathBuf::from(path).join("external").join(relative));
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory"))?;
    Ok(home.join(relative))
}

/// Best-effort startup hardening for local config dirs that may store credentials.
///
/// This intentionally ignores failures so startup does not fail on exotic
/// filesystems, but it narrows exposure on typical Unix systems. Hardens both
/// the `next-code` config segment when it exists.
pub fn harden_user_config_permissions() {
    if let Some(config_dir) = dirs::config_dir() {
        for segment in ["next-code"] {
            let dir = config_dir.join(segment);
            if dir.exists() {
                let _ = next_code_core::fs::set_directory_permissions_owner_only(&dir);
            }
        }
    }

    if let Ok(next_code_home) = next_code_dir()
        && next_code_home.exists()
    {
        let _ = next_code_core::fs::set_directory_permissions_owner_only(&next_code_home);
    }
}

/// Best-effort hardening for a secret-bearing file and its parent directory.
///
/// This is used before reading credential files so legacy permissive modes can
/// be tightened opportunistically. Missing paths are a no-op: presence probes
/// (e.g. openai-compatible autodetection) must not pay Windows ACL rewrites
/// for every absent `*.env` candidate on cold start.
pub fn harden_secret_file_permissions(path: &Path) {
    if !path.exists() {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = next_code_core::fs::set_directory_permissions_owner_only(parent);
    }
    let _ = next_code_core::fs::set_permissions_owner_only(path);
}

/// Validate an external auth file managed by another tool before reading it.
///
/// next-code intentionally avoids mutating these files. We also reject obvious
/// risky cases like symlinks so a remembered trust decision stays bound to a
/// real file path rather than an arbitrary redirect.
pub fn validate_external_auth_file(path: &Path) -> Result<PathBuf> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to inspect external auth file {}: {}",
            path.display(),
            e
        )
    })?;
    if metadata.file_type().is_symlink() {
        anyhow::bail!(
            "Refusing to read external auth file via symlink: {}",
            path.display()
        );
    }
    if !metadata.is_file() {
        anyhow::bail!(
            "External auth path is not a regular file: {}",
            path.display()
        );
    }
    std::fs::canonicalize(path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to canonicalize external auth file {}: {}",
            path.display(),
            e
        )
    })
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
        next_code_core::fs::set_directory_permissions_owner_only(path)?;
    }
    Ok(())
}

pub fn write_text_secret(path: &Path, content: &str) -> Result<()> {
    write_bytes_inner(path, content.as_bytes(), true)?;
    if let Some(parent) = path.parent() {
        next_code_core::fs::set_directory_permissions_owner_only(parent)?;
    }
    next_code_core::fs::set_permissions_owner_only(path)?;
    Ok(())
}

/// Atomically write binary secret content with owner-only (0600) permissions.
///
/// Like [`write_text_secret`] but for non-UTF-8 payloads (e.g. age-encrypted
/// blobs). Uses the same temp-file + fsync + atomic-rename path, so a crash
/// mid-write cannot truncate or corrupt the destination.
pub fn write_bytes_secret(path: &Path, content: &[u8]) -> Result<()> {
    write_bytes_inner(path, content, true)?;
    if let Some(parent) = path.parent() {
        next_code_core::fs::set_directory_permissions_owner_only(parent)?;
    }
    next_code_core::fs::set_permissions_owner_only(path)?;
    Ok(())
}

pub fn upsert_env_file_value(path: &Path, env_key: &str, value: Option<&str>) -> Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let prefix = format!("{}=", env_key);

    let mut lines = Vec::new();
    let mut replaced = false;
    for line in existing.lines() {
        if line.starts_with(&prefix) {
            replaced = true;
            if let Some(value) = value {
                lines.push(format!("{}={}", env_key, value));
            }
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced && let Some(value) = value {
        lines.push(format!("{}={}", env_key, value));
    }

    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    write_text_secret(path, &content)
}

pub fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    write_json_inner(path, value, true)
}

pub fn write_json_secret<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    write_json_inner(path, value, true)?;
    if let Some(parent) = path.parent() {
        next_code_core::fs::set_directory_permissions_owner_only(parent)?;
    }
    next_code_core::fs::set_permissions_owner_only(path)?;
    Ok(())
}

/// Fast JSON write: atomic rename but no fsync. Good for frequent saves where
/// durability on power loss is not critical (e.g., session saves during tool execution).
/// Data is still safe against process crashes (atomic rename protects against partial writes).
pub fn write_json_fast<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    write_json_inner(path, value, false)
}

/// Atomically write raw bytes to `path` (temp file + rename), fsync'd for
/// durability. Used for editing user config files where a torn write would be
/// catastrophic.
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    write_bytes_inner(path, bytes, true)
}

fn write_json_inner<T: Serialize + ?Sized>(path: &Path, value: &T, durable: bool) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    write_bytes_inner(path, &bytes, durable)
}

fn write_bytes_inner(path: &Path, bytes: &[u8], durable: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let pid = std::process::id();
    let nonce: u64 = rand::random();
    let tmp_path = path.with_extension(format!("tmp.{}.{}", pid, nonce));

    let result = (|| -> Result<()> {
        let file = std::fs::File::create(&tmp_path)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(bytes)?;
        let file = writer
            .into_inner()
            .map_err(|e| anyhow::anyhow!("flush failed: {}", e))?;

        if durable {
            file.sync_all()?;
        }

        if path.exists() {
            let bak_path = path.with_extension("bak");
            // Preserve the previous version as .bak without ever leaving the
            // primary path missing. On Unix, rename(tmp, path) atomically
            // replaces the destination, so the backup can be a hard link to
            // the old inode: concurrent readers always see either the old or
            // the new content, never ENOENT. (The old rename-away approach
            // opened a window where the primary did not exist, which made
            // concurrent load-all style readers silently drop entries, e.g.
            // self-dev build requests "disappearing" from the queue.)
            #[cfg(unix)]
            {
                let _ = std::fs::remove_file(&bak_path);
                let _ = std::fs::hard_link(path, &bak_path);
            }
            // On Windows, rename fails when the destination exists, so the
            // primary must be moved away first; the brief missing window is
            // unavoidable without platform-specific replace APIs.
            #[cfg(not(unix))]
            {
                let _ = std::fs::rename(path, &bak_path);
            }
        }

        std::fs::rename(&tmp_path, path)?;

        #[cfg(unix)]
        if durable
            && let Some(parent) = path.parent()
            && let Ok(dir) = std::fs::File::open(parent)
        {
            let _ = dir.sync_all();
        }

        Ok(())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }

    result
}

pub enum StorageRecoveryEvent<'a> {
    CorruptPrimary {
        path: &'a Path,
        error: &'a serde_json::Error,
    },
    RecoveredFromBackup {
        backup_path: &'a Path,
    },
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    read_json_with_recovery_handler(path, |event| match event {
        StorageRecoveryEvent::CorruptPrimary { path, error } => {
            eprintln!(
                "Corrupt JSON at {}, trying backup: {}",
                path.display(),
                error
            );
        }
        StorageRecoveryEvent::RecoveredFromBackup { backup_path } => {
            eprintln!("Recovered from backup: {}", backup_path.display());
        }
    })
}

pub fn read_json_with_recovery_handler<T, F>(path: &Path, mut on_recovery: F) -> Result<T>
where
    T: DeserializeOwned,
    F: FnMut(StorageRecoveryEvent<'_>),
{
    let data = std::fs::read_to_string(path)?;
    match serde_json::from_str(&data) {
        Ok(val) => Ok(val),
        Err(e) => {
            let bak_path = path.with_extension("bak");
            if bak_path.exists() {
                on_recovery(StorageRecoveryEvent::CorruptPrimary { path, error: &e });
                let bak_data = std::fs::read_to_string(&bak_path)?;
                match serde_json::from_str(&bak_data) {
                    Ok(val) => {
                        on_recovery(StorageRecoveryEvent::RecoveredFromBackup {
                            backup_path: &bak_path,
                        });
                        let _ = std::fs::copy(&bak_path, path);
                        Ok(val)
                    }
                    Err(bak_err) => Err(anyhow::anyhow!(
                        "Corrupt JSON at {} ({}), backup also corrupt ({})",
                        path.display(),
                        e,
                        bak_err
                    )),
                }
            } else {
                Err(anyhow::anyhow!("Corrupt JSON at {}: {}", path.display(), e))
            }
        }
    }
}

/// Fast append of a single JSON value followed by a newline.
/// Intended for append-only journals where per-write fsync is not required.
///
/// The entire line (value + trailing newline) is serialized into one buffer
/// and appended with a single `write_all`. Streaming the serializer straight
/// into the file issued many small writes, so a concurrent reader (or a
/// process killed mid-append) could observe a torn half-line, and two
/// concurrent appenders could interleave fragments. A single `O_APPEND` write
/// of the complete line keeps each journal line intact.
pub fn append_json_line_fast<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let mut line = serde_json::to_vec(value)?;
    line.push(b'\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(&line)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::{clear_home_env, lock_env};

    #[test]
    fn prefers_next_code_home() {
        let _g = lock_env();
        clear_home_env();
        let temp = tempfile::tempdir().expect("tempdir");
        let preferred = temp.path().join("preferred");
        std::fs::create_dir_all(&preferred).unwrap();
        next_code_core::env::set_var("NEXT_CODE_HOME", &preferred);
        let got = next_code_dir().unwrap();
        assert_eq!(got, preferred);
        clear_home_env();
    }

    #[test]
    fn fresh_user_gets_next_code() {
        let _g = lock_env();
        clear_home_env();
        // Without NEXT_CODE_HOME, resolve relative to the platform home dir.
        // (Windows Known Folder APIs ignore temporary HOME/USERPROFILE overrides.)
        let got = next_code_dir().unwrap();
        let expected = dirs::home_dir()
            .expect("home directory")
            .join(".next-code");
        assert_eq!(got, expected);
    }

    #[test]
    fn app_config_dir_sandboxed_under_product_home() {
        let _g = lock_env();
        clear_home_env();
        let temp = tempfile::tempdir().expect("tempdir");
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path());
        let cfg = app_config_dir().unwrap();
        assert_eq!(cfg, temp.path().join("config").join("next-code"));
        clear_home_env();
    }

    #[test]
    fn runtime_dir_reads_next_code() {
        let _g = lock_env();
        next_code_core::env::remove_var("NEXT_CODE_RUNTIME_DIR");
        let temp = tempfile::tempdir().expect("tempdir");
        next_code_core::env::set_var("NEXT_CODE_RUNTIME_DIR", temp.path());
        assert_eq!(runtime_dir(), temp.path());
        next_code_core::env::remove_var("NEXT_CODE_RUNTIME_DIR");
    }
}
