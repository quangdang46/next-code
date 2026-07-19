use std::path::{Path, PathBuf};

/// Normalize a path lexically without resolving symlinks or touching the
/// filesystem. Uses `dunce::canonicalize` as a best-effort canonicalization.
pub fn normalize_lexically<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    // First try dunce to strip `\\?\` prefixes on Windows and normalize.
    // On failure, fall back to a simple lexical clean.
    dunce::canonicalize(path).unwrap_or_else(|_| {
        let mut result = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    result.pop();
                }
                std::path::Component::CurDir => {
                    // skip
                }
                other => {
                    result.push(other);
                }
            }
        }
        result
    })
}

/// Return the `.grok` subdirectory of the home directory.
pub fn grok_dir() -> PathBuf {
    PathBuf::from(".grok")
}
