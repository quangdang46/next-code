//! Face `!` bash mode — parse ACP prompt meta and run a user shell command.
//!
//! Face already owns bang chrome (`PromptInputMode::Bash` → `SendBashCommand` →
//! `PromptBlockMeta.bash_command`). This module is the next-code brain wire:
//! honor that meta, run the shell locally, and let `pager_agent` emit execute
//! tool updates with `bash_mode: true` (no model turn).

use std::path::Path;
use std::time::Duration;

use agent_client_protocol as acp;
use tokio::process::Command as TokioCommand;

/// Soft cap so Face scrollback / later context stay bounded.
pub const BASH_MODE_MAX_OUTPUT: usize = 30_000;
/// Match the bash tool default timeout.
pub const BASH_MODE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BashRunResult {
    pub output: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

/// Extract `PromptBlockMeta.bash_command` from the first text block that carries it.
///
/// Presence of the key (including empty string) means Face sent a bash-mode
/// prompt. Returns `None` for ordinary chat prompts.
pub fn bash_command_from_prompt(args: &acp::PromptRequest) -> Option<String> {
    for block in &args.prompt {
        let acp::ContentBlock::Text(t) = block else {
            continue;
        };
        let Some(meta) = t.meta.as_ref() else {
            continue;
        };
        let Some(value) = meta.get("bash_command") else {
            continue;
        };
        // Face always serializes a string; tolerate null/other by falling back
        // to the block text (still bash mode).
        let cmd = value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| t.text.clone());
        return Some(cmd);
    }
    None
}

/// Run `command` in `cwd` (session working dir) with a wall-clock timeout.
pub async fn run_shell_command(
    command: &str,
    cwd: Option<&Path>,
    timeout: Duration,
) -> BashRunResult {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return BashRunResult {
            output: String::new(),
            exit_code: 0,
            timed_out: false,
        };
    }

    let mut child_cmd = build_shell_command(trimmed);
    if let Some(dir) = cwd {
        child_cmd.current_dir(dir);
    }
    child_cmd.stdout(std::process::Stdio::piped());
    child_cmd.stderr(std::process::Stdio::piped());
    child_cmd.stdin(std::process::Stdio::null());
    // Cancelled timeout drops the Child — kill so the process does not linger.
    child_cmd.kill_on_drop(true);
    // Avoid inheriting Face/pager TTY quirks; keep output capture stable.
    child_cmd.env("TERM", "dumb");

    let child = match child_cmd.spawn() {
        Ok(c) => c,
        Err(err) => {
            return BashRunResult {
                output: format!("Failed to start shell: {err}"),
                exit_code: 127,
                timed_out: false,
            };
        }
    };

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(out)) => {
            let mut text = String::new();
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.is_empty() {
                text.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                text.push_str(&stderr);
            }
            BashRunResult {
                output: truncate_output(text),
                exit_code: out.status.code().unwrap_or(1),
                timed_out: false,
            }
        }
        Ok(Err(err)) => BashRunResult {
            output: format!("Failed to wait for shell: {err}"),
            exit_code: 1,
            timed_out: false,
        },
        Err(_) => BashRunResult {
            output: format!(
                "Command timed out after {}ms ({:.1}s)",
                timeout.as_millis(),
                timeout.as_secs_f64()
            ),
            exit_code: 124,
            timed_out: true,
        },
    }
}

fn build_shell_command(cmd_str: &str) -> TokioCommand {
    #[cfg(windows)]
    {
        let mut cmd = TokioCommand::new("cmd.exe");
        cmd.arg("/C").arg(cmd_str);
        cmd
    }
    #[cfg(not(windows))]
    {
        let mut cmd = TokioCommand::new("bash");
        cmd.arg("-c").arg(cmd_str);
        cmd
    }
}

fn truncate_output(mut output: String) -> String {
    if output.len() <= BASH_MODE_MAX_OUTPUT {
        return output;
    }
    // Keep a UTF-8 boundary.
    let mut end = BASH_MODE_MAX_OUTPUT;
    while end > 0 && !output.is_char_boundary(end) {
        end -= 1;
    }
    output.truncate(end);
    output.push_str("\n... (output truncated)");
    output
}

/// ACP `_meta` map Face uses to paint `(user)` execute chrome.
pub fn bash_mode_tool_meta() -> acp::Meta {
    let mut meta = acp::Meta::new();
    meta.insert("bash_mode".into(), serde_json::Value::Bool(true));
    meta
}

/// `raw_input` shape Face reads for the execute header (`command` field).
pub fn bash_mode_raw_input(command: &str) -> serde_json::Value {
    serde_json::json!({ "command": command })
}

/// `raw_output` ToolOutput::Bash shape Face `extract_bash_output_from_value` expects.
pub fn bash_mode_raw_output(
    command: &str,
    output: &str,
    exit_code: i32,
    timed_out: bool,
    cwd: &str,
) -> serde_json::Value {
    serde_json::json!({
        "type": "Bash",
        "output": output.as_bytes(),
        "exit_code": exit_code,
        "command": command,
        "description": null,
        "timed_out": timed_out,
        "signal": null,
        "current_dir": cwd,
        "output_file": "",
        "total_bytes": output.len(),
        "output_delta": null,
        "was_bare_echo": false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_shell::extensions::prompt_meta::PromptBlockMeta;

    fn prompt_with_bash_meta(command: &str) -> acp::PromptRequest {
        let meta = PromptBlockMeta::bash(command);
        let meta_map = serde_json::to_value(&meta)
            .expect("PromptBlockMeta serializes")
            .as_object()
            .cloned();
        let block = acp::ContentBlock::Text(
            acp::TextContent::new(command).meta(meta_map),
        );
        acp::PromptRequest::new(acp::SessionId::new("sess"), vec![block])
    }

    #[test]
    fn extracts_bash_command_from_prompt_meta() {
        let args = prompt_with_bash_meta("git status");
        assert_eq!(
            bash_command_from_prompt(&args).as_deref(),
            Some("git status")
        );
    }

    #[test]
    fn extracts_empty_bash_command() {
        let args = prompt_with_bash_meta("");
        assert_eq!(bash_command_from_prompt(&args).as_deref(), Some(""));
    }

    #[test]
    fn ordinary_prompt_has_no_bash_command() {
        let block = acp::ContentBlock::Text(acp::TextContent::new("hello"));
        let args = acp::PromptRequest::new(acp::SessionId::new("sess"), vec![block]);
        assert_eq!(bash_command_from_prompt(&args), None);
    }

    #[test]
    fn bang_prefixed_text_without_meta_is_not_bash_mode() {
        // Ordinary chat that happens to start with `!` must not enter bash path.
        let block = acp::ContentBlock::Text(acp::TextContent::new("!not-meta"));
        let args = acp::PromptRequest::new(acp::SessionId::new("sess"), vec![block]);
        assert_eq!(bash_command_from_prompt(&args), None);
    }

    #[test]
    fn truncate_output_respects_utf8_boundary() {
        let input = format!("{}é", "a".repeat(BASH_MODE_MAX_OUTPUT - 1));
        let out = truncate_output(input);
        assert!(out.ends_with("\n... (output truncated)"));
        assert!(out.is_char_boundary(out.len()));
    }

    #[tokio::test]
    async fn run_shell_echo_succeeds() {
        #[cfg(windows)]
        let cmd = "echo hello-bang";
        #[cfg(not(windows))]
        let cmd = "echo hello-bang";
        let result = run_shell_command(cmd, None, Duration::from_secs(10)).await;
        assert_eq!(result.exit_code, 0, "output={}", result.output);
        assert!(!result.timed_out);
        assert!(
            result.output.to_ascii_lowercase().contains("hello-bang"),
            "unexpected output: {}",
            result.output
        );
    }

    #[tokio::test]
    async fn run_shell_empty_is_noop() {
        let result = run_shell_command("   ", None, Duration::from_secs(5)).await;
        assert_eq!(result.exit_code, 0);
        assert!(result.output.is_empty());
    }

    #[test]
    fn bash_mode_meta_sets_flag() {
        let meta = bash_mode_tool_meta();
        assert_eq!(meta.get("bash_mode"), Some(&serde_json::Value::Bool(true)));
    }
}
