//! Where a piece of configuration was loaded from (vendored from upstream
//! for SkillInfo / pager extensions modal).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ConfigSource {
    Builtin,
    Bundled { path: PathBuf },
    Server { path: PathBuf },
    Project { path: PathBuf },
    User { path: PathBuf },
    Plugin { plugin_name: String, path: PathBuf },
    ConfigToml { path: PathBuf },
    ClaudeJson { path: PathBuf },
    McpJson { path: PathBuf },
    Cli { path: PathBuf },
    Managed {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
    },
}
