/// Resolve the Grok CLI binary path, honoring an explicit override.
///
/// `NEXT_CODE_GROK_CLI_PATH` lets a user (or a test harness) point the Grok
/// Build ACP provider at a specific `grok` binary. Absent an override we assume
/// `grok` is on `PATH`.
pub fn cli_path() -> String {
    std::env::var("NEXT_CODE_GROK_CLI_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "grok".to_string())
}

/// Whether the Grok CLI is available to launch.
pub fn cli_available() -> bool {
    super::command_exists(&cli_path())
}
