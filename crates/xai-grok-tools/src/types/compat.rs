//! Stub of upstream `xai-grok-tools::types::compat`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorCompat {
    #[serde(default = "default_true")]
    pub sessions: bool,
    #[serde(default = "default_true")]
    pub skills: bool,
    #[serde(default = "default_true")]
    pub rules: bool,
    #[serde(default = "default_true")]
    pub agents: bool,
    #[serde(default = "default_true")]
    pub mcps: bool,
    #[serde(default = "default_true")]
    pub hooks: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allow_claude: bool,
    #[serde(default)]
    pub claude: VendorCompat,
    #[serde(default)]
    pub cursor: VendorCompat,
    #[serde(default)]
    pub codex: VendorCompat,
}
