//! Stub memory storage — API surface matching pager `memory_cmd` + effects.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope {
    Workspace,
    Global,
    All,
}

#[derive(Debug, Clone)]
pub struct MemoryStorage {
    workspace_root: PathBuf,
    global_root: PathBuf,
}

impl MemoryStorage {
    /// `cwd` is the workspace root; `_override_home` unused in the stub.
    pub fn new(cwd: &Path, _override_home: Option<&Path>) -> Self {
        let home = dirs_home();
        Self {
            workspace_root: cwd.join(".next-code").join("memory"),
            global_root: home.join("memory"),
        }
    }

    pub fn workspace_dir(&self) -> &Path {
        &self.workspace_root
    }

    pub fn global_memory_file(&self) -> PathBuf {
        self.global_root.join("MEMORY.md")
    }

    pub fn clear_workspace(&self) -> std::io::Result<bool> {
        Ok(false)
    }

    pub fn clear_global(&self) -> std::io::Result<bool> {
        Ok(false)
    }

    pub fn clear(&self, scope: MemoryScope) -> anyhow::Result<()> {
        match scope {
            MemoryScope::Workspace => {
                let _ = self.clear_workspace()?;
            }
            MemoryScope::Global => {
                let _ = self.clear_global()?;
            }
            MemoryScope::All => {
                let _ = self.clear_workspace()?;
                let _ = self.clear_global()?;
            }
        }
        Ok(())
    }

    pub fn append_to_memory(&self, _scope: MemoryScope, _text: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn workspace_path(&self) -> PathBuf {
        self.workspace_root.join("MEMORY.md")
    }

    pub fn global_path(&self) -> PathBuf {
        self.global_memory_file()
    }
}

fn dirs_home() -> PathBuf {
    crate::util::grok_home::grok_home()
}
