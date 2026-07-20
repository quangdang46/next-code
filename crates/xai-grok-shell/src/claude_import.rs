//! Stub of upstream `xai-grok-shell::claude_import`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::util::config::McpServerConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportScope {
    Global,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathKind {
    Skill,
    Rule,
}

impl PathKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Rule => "rule",
        }
    }
}

pub use xai_grok_workspace::permission::types::PermissionRule;

#[derive(Debug, Clone)]
pub enum ImportableItem {
    Permission(PermissionRule),
    EnvVar { key: String, value: String },
    McpServer {
        name: String,
        config: Box<McpServerConfig>,
    },
    Hook {
        event: String,
        matcher: Option<String>,
        command: String,
        timeout: Option<u64>,
    },
    PathEntry { kind: PathKind, path: String },
}

#[derive(Debug, Clone, Default)]
pub struct ImportPlan {
    pub global_items: Vec<ImportableItem>,
    pub project_items: Vec<ImportableItem>,
}

impl ImportPlan {
    pub fn total_items(&self) -> usize {
        self.global_items.len() + self.project_items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.global_items.is_empty() && self.project_items.is_empty()
    }

    pub fn summary(&self, _cwd: &Path) -> String {
        if self.is_empty() {
            "No Claude settings found to import.".to_string()
        } else {
            format!("Found {} items to import.", self.total_items())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub modified_files: Vec<String>,
}

impl ImportResult {
    pub fn total(&self) -> usize {
        self.imported + self.skipped
    }
}

pub fn scan_importable_settings(_cwd: &Path) -> ImportPlan {
    ImportPlan::default()
}

pub fn find_project_root(cwd: &Path) -> PathBuf {
    cwd.to_path_buf()
}

pub fn is_claude_import_marked() -> bool {
    false
}

pub fn mark_claude_imported() -> anyhow::Result<()> {
    Ok(())
}

pub fn apply_import(_plan: &ImportPlan, _cwd: &Path) -> anyhow::Result<ImportResult> {
    Ok(ImportResult::default())
}

pub fn expand_home(s: &str) -> PathBuf {
    PathBuf::from(s)
}
