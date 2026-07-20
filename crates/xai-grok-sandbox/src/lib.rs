//! Facade of `xai-org/grok-build` `xai-grok-sandbox` (Apache-2.0) for the
//! next-code Grok Face migration (PR7).
//!
//! Upstream enforces OS-level sandboxing. This stub only reproduces
//! [`profile_name`], [`ProfileName`], and [`sandbox_profile_conflicts`].

use std::path::Path;

/// Built-in / custom sandbox profile name.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProfileName {
    /// Default workspace-writable profile.
    #[default]
    Workspace,
    /// Devbox profile.
    Devbox,
    /// Read-only profile.
    ReadOnly,
    /// Strict profile.
    Strict,
    /// Sandbox disabled.
    Off,
    /// Custom profile from sandbox.toml.
    Custom(String),
}

impl std::fmt::Display for ProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace => write!(f, "workspace"),
            Self::Devbox => write!(f, "devbox"),
            Self::ReadOnly => write!(f, "read-only"),
            Self::Strict => write!(f, "strict"),
            Self::Off => write!(f, "off"),
            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

impl std::str::FromStr for ProfileName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "workspace" => Ok(Self::Workspace),
            "devbox" => Ok(Self::Devbox),
            "read-only" | "readonly" => Ok(Self::ReadOnly),
            "strict" => Ok(Self::Strict),
            "off" | "none" => Ok(Self::Off),
            other => Ok(Self::Custom(other.to_string())),
        }
    }
}

/// The active sandbox profile name, or `None` if sandbox is not applied.
///
/// Stub: always `None` (no sandbox installed).
pub fn profile_name() -> Option<&'static str> {
    None
}

/// Profile names that conflict between global and project sandbox.toml.
///
/// Stub: always empty.
pub fn sandbox_profile_conflicts(_workspace: &Path) -> Vec<String> {
    Vec::new()
}
