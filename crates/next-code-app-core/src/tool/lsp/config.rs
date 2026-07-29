//! LSP server configuration: project / user `lsp.json` + built-in defaults.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct LspServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Map of file extension (with leading `.`) → LSP language id.
    #[serde(rename = "extensionToLanguage")]
    pub extension_to_language: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default, rename = "initializationOptions")]
    pub initialization_options: Option<Value>,
    #[serde(default, rename = "workspaceFolder")]
    pub workspace_folder: Option<String>,
    #[serde(default, rename = "startupTimeout")]
    pub startup_timeout_ms: Option<u64>,
    #[serde(default, rename = "maxRestarts")]
    pub max_restarts: Option<u32>,
}

impl LspServerConfig {
    pub fn language_id_for_path(&self, path: &Path) -> Option<String> {
        let ext = path.extension()?.to_str()?;
        let with_dot = format!(".{ext}");
        self.extension_to_language
            .get(&with_dot)
            .or_else(|| self.extension_to_language.get(ext))
            .cloned()
    }
}

/// Merge order (later wins): builtins (if on PATH) → user → project.
pub fn load_server_configs(workspace: &Path) -> HashMap<String, LspServerConfig> {
    let mut servers = HashMap::new();

    for (name, config) in builtin_candidates() {
        if command_on_path(&config.command) {
            servers.insert(name, config);
        }
    }

    if let Some(user_path) = user_lsp_json_path() {
        merge_from_file(&mut servers, &user_path);
    }

    let project_path = workspace.join(".next-code").join("lsp.json");
    merge_from_file(&mut servers, &project_path);

    servers
}

fn merge_from_file(servers: &mut HashMap<String, LspServerConfig>, path: &Path) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<HashMap<String, LspServerConfig>>(&raw) else {
        return;
    };
    for (name, config) in parsed {
        servers.insert(name, config);
    }
}

fn user_lsp_json_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".next-code").join("lsp.json"))
}

fn builtin_candidates() -> Vec<(String, LspServerConfig)> {
    vec![
        (
            "rust-analyzer".into(),
            LspServerConfig {
                command: "rust-analyzer".into(),
                args: vec![],
                extension_to_language: HashMap::from([(".rs".into(), "rust".into())]),
                env: HashMap::new(),
                initialization_options: None,
                workspace_folder: None,
                startup_timeout_ms: Some(30_000),
                max_restarts: Some(3),
            },
        ),
        (
            "typescript".into(),
            LspServerConfig {
                command: "typescript-language-server".into(),
                args: vec!["--stdio".into()],
                extension_to_language: HashMap::from([
                    (".ts".into(), "typescript".into()),
                    (".tsx".into(), "typescriptreact".into()),
                    (".js".into(), "javascript".into()),
                    (".jsx".into(), "javascriptreact".into()),
                    (".mts".into(), "typescript".into()),
                    (".cts".into(), "typescript".into()),
                    (".mjs".into(), "javascript".into()),
                    (".cjs".into(), "javascript".into()),
                ]),
                env: HashMap::new(),
                initialization_options: None,
                workspace_folder: None,
                startup_timeout_ms: Some(30_000),
                max_restarts: Some(3),
            },
        ),
        (
            "pyright".into(),
            LspServerConfig {
                command: "pyright-langserver".into(),
                args: vec!["--stdio".into()],
                extension_to_language: HashMap::from([
                    (".py".into(), "python".into()),
                    (".pyi".into(), "python".into()),
                ]),
                env: HashMap::new(),
                initialization_options: None,
                workspace_folder: None,
                startup_timeout_ms: Some(30_000),
                max_restarts: Some(3),
            },
        ),
        (
            "gopls".into(),
            LspServerConfig {
                command: "gopls".into(),
                args: vec![],
                extension_to_language: HashMap::from([(".go".into(), "go".into())]),
                env: HashMap::new(),
                initialization_options: None,
                workspace_folder: None,
                startup_timeout_ms: Some(30_000),
                max_restarts: Some(3),
            },
        ),
    ]
}

fn command_on_path(command: &str) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("where")
            .arg(command)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} >/dev/null 2>&1", shell_escape(command)))
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(not(windows))]
fn shell_escape(value: &str) -> String {
    value.replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_id_looks_up_dotted_extension() {
        let cfg = LspServerConfig {
            command: "x".into(),
            args: vec![],
            extension_to_language: HashMap::from([(".rs".into(), "rust".into())]),
            env: HashMap::new(),
            initialization_options: None,
            workspace_folder: None,
            startup_timeout_ms: None,
            max_restarts: None,
        };
        assert_eq!(
            cfg.language_id_for_path(Path::new("src/main.rs")).as_deref(),
            Some("rust")
        );
    }

    #[test]
    fn merge_from_file_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lsp.json");
        std::fs::write(
            &path,
            r#"{
              "custom": {
                "command": "echo",
                "extensionToLanguage": { ".txt": "plaintext" }
              }
            }"#,
        )
        .unwrap();
        let mut servers = HashMap::new();
        merge_from_file(&mut servers, &path);
        assert!(servers.contains_key("custom"));
        assert_eq!(servers["custom"].command, "echo");
    }
}
