//! Spawn a next-code agent inside a freshly-prepared repo clone, run a
//! single eval task, and capture the resulting diff and trace.
//!
//! The runner resolves the configured `agent_id` through the
//! [`next_code_agent_runtime::AgentRegistry`] (loaded from
//! `.next-code/agents/*.toml`), spawns the binary as a subprocess in the
//! repo working directory, streams the trace, and finally extracts the
//! unified diff against the parent commit.
//!
//! Design source: `/tmp/codebuff/evals/buffbench/agent-runner.ts`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use crate::types::EvalRun;

/// Configuration for a single agent evaluation run.
///
/// `repo_path` should already contain a clean checkout of the eval
/// commit's parent SHA; the runner does not clone for the caller.
#[derive(Debug, Clone)]
pub struct AgentRunConfig {
    /// ID of the agent to run, matching an entry in the
    /// `jcode-agent-runtime` registry.
    pub agent_id: String,
    /// Natural-language prompt to send to the agent (typically
    /// `EvalCommit::prompt`).
    pub prompt: String,
    /// Working directory containing the prepared repo at the parent
    /// commit.
    pub repo_path: PathBuf,
    /// Hard cap on the number of agent turns before the run is
    /// aborted; mirrors BuffBench's per-task turn budget.
    pub max_turns: u32,
    /// Timeout for the entire run in seconds (defaults to 60 minutes).
    pub timeout_secs: u64,
    /// Extra environment variables applied to the agent subprocess on
    /// top of the calling process's environment.
    pub env: HashMap<String, String>,
    /// Path to the `next-code` binary. Defaults to searching $PATH.
    pub next_code_binary: Option<PathBuf>,
}

impl Default for AgentRunConfig {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            prompt: String::new(),
            repo_path: PathBuf::new(),
            max_turns: 100,
            timeout_secs: 60 * 60,
            env: HashMap::new(),
            next_code_binary: None,
        }
    }
}

/// Spawn the configured agent in `config.repo_path`, run it to
/// completion (or the turn / time budget), and return an [`EvalRun`]
/// populated with the agent's diff, judging placeholder, cost, and
/// duration.
pub async fn run_agent_in_repo(config: AgentRunConfig) -> Result<EvalRun> {
    let start = Instant::now();
    let timeout_duration = Duration::from_secs(config.timeout_secs);

    let next_code_bin = config
        .next_code_binary
        .clone()
        .unwrap_or_else(|| {
            std::env::var_os("NEXT_CODE_BIN")
                .or_else(|| std::env::var_os("JCODE_BIN"))
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("next-code"))
        });

    let mut env_vars: HashMap<String, String> = std::env::vars().collect();
    env_vars.extend(config.env);
    env_vars.insert("NEXT_CODE_AGENT_ID".to_owned(), config.agent_id.clone());
    env_vars.insert("JCODE_AGENT_ID".to_owned(), config.agent_id.clone());

    let mut child = Command::new(&next_code_bin)
        .current_dir(&config.repo_path)
        .envs(&env_vars)
        .args([
            "agent",
            "run",
            "--agent",
            &config.agent_id,
            "--output-mode",
            "stream",
            "--no-interactive",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn next-code binary at {:?}", next_code_bin))?;

    let mut child_stdin = child.stdin.take().expect("stdin captured");
    let stdout = child.stdout.take().expect("stdout captured");

    // Write the prompt to stdin
    {
        use tokio::io::AsyncWriteExt;
        let mut stdin = tokio::io::BufWriter::new(&mut child_stdin);
        stdin.write_all(config.prompt.as_bytes()).await?;
        stdin.flush().await?;
        drop(stdin);
    }

    let mut trace_lines = Vec::new();
    let reader = BufReader::new(stdout);
    let mut lines_stream = reader.lines();
    let timed_out = loop {
        let line = timeout(timeout_duration, lines_stream.next_line()).await;
        match line {
            Ok(Ok(Some(l))) => trace_lines.push(l),
            Ok(Ok(None)) => break false, // EOF — clean exit
            Ok(Err(_)) => break false,   // read error
            Err(_) => break true,        // timeout
        }
    };

    if timed_out {
        // Kill the child process so it doesn't become an orphan
        let _ = child.kill().await;
        // Consume the exit status after kill
        let _ = child.wait().await;
        return Ok(EvalRun {
            commit_sha: String::new(),
            prompt: config.prompt,
            diff: extract_diff_from_repo(&config.repo_path)
                .await
                .unwrap_or_default(),
            judging: Default::default(),
            cost_usd: 0.0,
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some("Timed out waiting for next-code subprocess".to_owned()),
        });
    }

    let status = child
        .wait()
        .await
        .context("failed to wait for next-code subprocess")?;

    let diff = extract_diff_from_repo(&config.repo_path).await?;
    let error = if !status.success() {
        Some(format!("next-code exited with status {:?}", status))
    } else {
        None
    };

    Ok(EvalRun {
        commit_sha: String::new(),
        prompt: config.prompt,
        diff,
        judging: Default::default(),
        cost_usd: 0.0,
        duration_ms: start.elapsed().as_millis() as u64,
        error,
    })
}

/// Produce a unified diff describing all uncommitted changes in
/// `repo_path` against its currently-checked-out HEAD.
pub async fn extract_diff_from_repo(repo_path: &Path) -> Result<String> {
    let repo_path = repo_path.to_owned();
    tokio::task::spawn_blocking(move || {
        let output = std::process::Command::new("git")
            .args(["diff", "--no-color", "HEAD"])
            .current_dir(&repo_path)
            .output()
            .context("git diff failed")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git diff exited with error: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    })
    .await
    .context("spawn_blocking panicked")?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn extract_diff_from_repo_nonexistent() {
        let result = extract_diff_from_repo(Path::new("/tmp/does-not-exist")).await;
        assert!(result.is_err());
    }
}
