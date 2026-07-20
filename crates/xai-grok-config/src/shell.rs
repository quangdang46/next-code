//! Shell PATH helpers (Face stub). Upstream lives in grok-build `shell`.

use std::path::{Path, PathBuf};

/// True when `program` resolves on `PATH` (bare name) or as an existing path.
pub fn is_command_available(program: &str) -> bool {
    if program.is_empty() {
        return false;
    }
    let as_path = Path::new(program);
    if as_path.components().count() > 1 || as_path.is_absolute() {
        return as_path.is_file();
    }
    path_entries().any(|dir| candidate_exists(&dir.join(program)))
}

fn path_entries() -> impl Iterator<Item = PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).collect::<Vec<_>>().into_iter()
}

fn candidate_exists(path: &Path) -> bool {
    if path.is_file() {
        return true;
    }
    #[cfg(windows)]
    {
        // Match `CreateProcess` / cmd.exe PATHEXT probing for bare names.
        let pathext = std::env::var_os("PATHEXT").unwrap_or_else(|| {
            std::ffi::OsString::from(".COM;.EXE;.BAT;.CMD")
        });
        for ext in std::env::split_paths(&pathext) {
            let mut with_ext = path.as_os_str().to_owned();
            with_ext.push(&ext);
            if Path::new(&with_ext).is_file() {
                return true;
            }
        }
    }
    false
}
