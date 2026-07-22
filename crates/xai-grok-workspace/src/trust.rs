//! Folder-trust store ("do you trust this folder?").
//!
//! Persists per-folder trust decisions to `<user home>/trusted_folders.toml`.
//! Copied from upstream `xai-org/grok-build` `xai-grok-workspace::trust`.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const TRUST_FILE_NAME: &str = "trusted_folders.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTrust {
    pub trusted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TrustDocument {
    #[serde(default)]
    folders: BTreeMap<String, FolderTrust>,
}

#[derive(Debug, Clone)]
pub struct TrustStore {
    doc: TrustDocument,
    path: Option<PathBuf>,
}

impl TrustStore {
    pub fn load() -> Self {
        match Self::default_path() {
            Some(path) => Self::load_from(path),
            None => Self::empty(),
        }
    }

    pub fn load_from(path: PathBuf) -> Self {
        let doc = Self::read_doc(&path);
        Self {
            doc,
            path: Some(path),
        }
    }

    fn empty() -> Self {
        Self {
            doc: TrustDocument::default(),
            path: None,
        }
    }

    pub fn default_path() -> Option<PathBuf> {
        Some(resolve_user_home()?.join(TRUST_FILE_NAME))
    }

    pub fn is_trusted(&self, workspace_key: &Path) -> bool {
        let workspace_key = canonicalize_or_owned(workspace_key);
        let mut best_depth: Option<usize> = None;
        let mut trusted = false;
        for (folder, record) in &self.doc.folders {
            let folder = Path::new(folder);
            if is_unsafe_trust_root(folder) || !workspace_key.starts_with(folder) {
                continue;
            }
            let depth = folder.components().count();
            match best_depth {
                Some(d) if depth < d => {}
                Some(d) if depth == d => trusted &= record.trusted,
                _ => {
                    best_depth = Some(depth);
                    trusted = record.trusted;
                }
            }
        }
        trusted
    }

    pub fn set_trusted(&mut self, workspace_key: &Path) -> io::Result<()> {
        self.record_decision(workspace_key, true)
    }

    pub fn set_untrusted(&mut self, workspace_key: &Path) -> io::Result<()> {
        self.record_decision(workspace_key, false)
    }

    pub fn grant(&mut self, key: &Path) {
        let _ = self.set_trusted(key);
    }

    pub fn len(&self) -> usize {
        self.doc.folders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.doc.folders.is_empty()
    }

    fn record_decision(&mut self, workspace_key: &Path, trusted: bool) -> io::Result<()> {
        let canonical = canonicalize_or_owned(workspace_key);
        if is_unsafe_trust_root(&canonical) {
            return Ok(());
        }
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "trust store path has no parent",
            )
        })?;
        std::fs::create_dir_all(parent)?;
        let _lock = ExclusiveLock::acquire(&path.with_extension("toml.lock"))?;
        let mut doc = Self::read_doc(path);
        doc.folders.insert(
            canonical.to_string_lossy().to_string(),
            FolderTrust {
                trusted,
                decided_at: now_unix(),
            },
        );
        Self::persist_doc(path, &doc)?;
        self.doc = doc;
        Ok(())
    }

    fn read_doc(path: &Path) -> TrustDocument {
        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return TrustDocument::default(),
            Err(_) => return TrustDocument::default(),
        };
        toml::from_str(&contents).unwrap_or_default()
    }

    fn persist_doc(path: &Path, doc: &TrustDocument) -> io::Result<()> {
        use std::io::Write;
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "trust store path has no parent",
            )
        })?;
        std::fs::create_dir_all(parent)?;
        let body = toml::to_string_pretty(doc)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        tmp.write_all(body.as_bytes())?;
        tmp.as_file().sync_all()?;
        tmp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
}

fn resolve_user_home() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("GROK_HOME") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    if let Ok(v) = std::env::var("NEXT_CODE_HOME") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    dirs::home_dir().map(|h| {
        dunce::canonicalize(&h)
            .unwrap_or(h)
            .join(".next-code")
    })
}

fn now_unix() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
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

pub fn workspace_key(cwd: &Path) -> PathBuf {
    let key = canonicalize_or_owned(cwd);
    if is_unsafe_trust_root(&key) {
        return canonicalize_or_owned(cwd);
    }
    key
}

struct ExclusiveLock {
    file: std::fs::File,
}

impl ExclusiveLock {
    fn acquire(lock_path: &Path) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        fs2::FileExt::lock_exclusive(&file)?;
        Ok(Self { file })
    }
}

impl Drop for ExclusiveLock {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn set_trusted_round_trips() {
        let _guard = env_lock();
        let home = TempDir::new().unwrap();
        let path = home.path().join(TRUST_FILE_NAME);
        let repo = TempDir::new().unwrap();
        let key = workspace_key(repo.path());

        let mut store = TrustStore::load_from(path.clone());
        assert!(!store.is_trusted(&key));
        store.set_trusted(&key).unwrap();
        assert!(TrustStore::load_from(path).is_trusted(&key));
    }
}
