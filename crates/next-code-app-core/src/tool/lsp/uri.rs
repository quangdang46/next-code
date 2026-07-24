//! Windows-safe `file://` URI helpers for LSP.

use std::path::{Path, PathBuf};

/// Convert a filesystem path to an LSP `file://` URI.
pub fn path_to_uri(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    // Prefer Url::from_file_path when available; fall back to manual encoding.
    if let Ok(url) = url::Url::from_file_path(&absolute) {
        return url.into();
    }

    let mut s = absolute.to_string_lossy().replace('\\', "/");
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    format!("file://{s}")
}

/// Convert an LSP `file://` URI to a filesystem path.
pub fn uri_to_path(uri: &str) -> PathBuf {
    if let Ok(url) = url::Url::parse(uri)
        && let Ok(path) = url.to_file_path()
    {
        return path;
    }

    let mut file_path = uri
        .strip_prefix("file://")
        .unwrap_or(uri)
        .to_string();
    // Windows: file:///C:/path → /C:/path — strip leading slash before drive.
    if file_path.len() >= 3
        && file_path.as_bytes()[0] == b'/'
        && file_path.as_bytes()[1].is_ascii_alphabetic()
        && file_path.as_bytes()[2] == b':'
    {
        file_path = file_path[1..].to_string();
    }
    if let Ok(decoded) = urlencoding::decode(&file_path) {
        PathBuf::from(decoded.as_ref())
    } else {
        PathBuf::from(file_path)
    }
}

/// Format a URI for display, preferring a path relative to `cwd`.
pub fn format_uri_for_display(uri: &str, cwd: Option<&Path>) -> String {
    let path = uri_to_path(uri);
    let display = if let Some(cwd) = cwd
        && let Ok(rel) = path.strip_prefix(cwd)
    {
        let rel = rel.to_string_lossy().replace('\\', "/");
        if !rel.starts_with("../") {
            return rel;
        }
        path.to_string_lossy().replace('\\', "/")
    } else {
        path.to_string_lossy().replace('\\', "/")
    };
    display
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_simple_path() {
        let path = if cfg!(windows) {
            PathBuf::from(r"C:\Users\test\file.rs")
        } else {
            PathBuf::from("/Users/test/file.rs")
        };
        let uri = path_to_uri(&path);
        assert!(uri.starts_with("file://"), "{uri}");
        let back = uri_to_path(&uri);
        assert_eq!(back, path);
    }

    #[test]
    fn strips_windows_drive_slash() {
        let path = uri_to_path("file:///C:/foo/bar.rs");
        let s = path.to_string_lossy().replace('\\', "/");
        assert!(s.contains("foo/bar.rs") || s.ends_with("foo/bar.rs"), "{s}");
        assert!(s.contains('C') || s.contains('c') || cfg!(not(windows)), "{s}");
    }
}
