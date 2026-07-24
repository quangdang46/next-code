//! Claude-compatible unified `Agent` tool — thin façade over `swarm` spawn/DM.
//!
//! Maps Claude Code `AgentTool` args (`subagent_type`, `isolation: worktree`,
//! `model`, `resume`, …) onto next-code swarm primitives without rewriting
//! swarm internals. See `LOOK-20260724-claude-code-tools-gaps.md`.

use super::communicate::CommunicateTool;
use super::{Tool, ToolContext, ToolOutput};
use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Canonical next-code role after normalizing Claude / Face / oh-my names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Explore,
    Plan,
    General,
    Verification,
    CodeReviewer,
    Custom,
}

impl AgentRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Explore => "explore",
            Self::Plan => "plan",
            Self::General => "general-purpose",
            Self::Verification => "verification",
            Self::CodeReviewer => "code-reviewer",
            Self::Custom => "custom",
        }
    }
}

/// Normalize Claude / Face / oh-my `subagent_type` strings to a role + optional
/// custom label retained for the prompt.
pub fn map_subagent_type(raw: Option<&str>) -> (AgentRole, String) {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return (AgentRole::General, AgentRole::General.as_str().to_string());
    };

    let key: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();

    let role = match key.as_str() {
        "explore" => AgentRole::Explore,
        "plan" => AgentRole::Plan,
        "general" | "generalpurpose" | "sisyphus" | "sisyphusjunior" | "atlas" => {
            AgentRole::General
        }
        "verification" | "verify" => AgentRole::Verification,
        "codereviewer" | "review" | "reviewer" => AgentRole::CodeReviewer,
        // Read-only analysis personas from oh-my / team eligibility map to explore.
        "oracle" | "librarian" | "multimodallooker" | "metis" | "momus" => AgentRole::Explore,
        _ => AgentRole::Custom,
    };

    let label = if role == AgentRole::Custom {
        raw.to_string()
    } else {
        role.as_str().to_string()
    };
    (role, label)
}

fn role_preamble(role: AgentRole, label: &str) -> String {
    match role {
        AgentRole::Explore => {
            "You are a fast read-only explore agent. Search and analyze the codebase; \
             do not create, edit, or delete files. Prefer ffs_glob/ffs_grep/read/ls and \
             read-only bash. Report findings clearly and finish when the search is done."
                .to_string()
        }
        AgentRole::Plan => {
            "You are a planning agent. Design an approach, list steps and risks, and do \
             not implement code changes unless explicitly asked. Prefer read-only tools \
             while gathering context."
                .to_string()
        }
        AgentRole::General => String::new(),
        AgentRole::Verification => {
            "You are a verification agent. Validate claims with concrete evidence \
             (tests, file reads, command output). Do not claim success without proof."
                .to_string()
        }
        AgentRole::CodeReviewer => {
            "You are a code-review agent. Focus on correctness, regressions, security, \
             and clarity. Prefer reading diffs and related code over rewriting files."
                .to_string()
        }
        AgentRole::Custom => format!(
            "You are operating as specialized agent type `{label}`. Follow that role's \
             intent while completing the task."
        ),
    }
}

fn build_spawn_prompt(
    role: AgentRole,
    label: &str,
    description: &str,
    prompt: &str,
    name: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    let preamble = role_preamble(role, label);
    if !preamble.is_empty() {
        parts.push(preamble);
    }
    if let Some(name) = name.map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(format!("Agent name: {name}"));
    }
    parts.push(format!("Task summary: {description}"));
    parts.push(format!("subagent_type: {label}"));
    parts.push(prompt.trim().to_string());
    parts.join("\n\n")
}

/// Create an isolated git worktree for `isolation: "worktree"`.
///
/// Returns `(worktree_path, branch_name)`. Uses a dedicated branch under
/// `agent/` so the parent checkout is untouched.
pub fn create_isolation_worktree(repo_cwd: &Path, slug: &str) -> Result<(PathBuf, String)> {
    let repo_root = git_rev_parse_show_toplevel(repo_cwd)
        .with_context(|| format!("isolation=worktree requires a git repo (cwd {})", repo_cwd.display()))?;

    let safe_slug: String = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(48)
        .collect();
    let safe_slug = if safe_slug.is_empty() {
        "agent".to_string()
    } else {
        safe_slug
    };

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let branch = format!("agent/{safe_slug}-{stamp}");
    let worktree_dir = repo_root
        .join(".next-code")
        .join("agent-worktrees")
        .join(format!("{safe_slug}-{stamp}"));

    if let Some(parent) = worktree_dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let output = Command::new("git")
        .args([
            "-C",
            repo_root.to_str().ok_or_else(|| anyhow!("repo path is not UTF-8"))?,
            "worktree",
            "add",
            "-b",
            &branch,
            worktree_dir
                .to_str()
                .ok_or_else(|| anyhow!("worktree path is not UTF-8"))?,
        ])
        .output()
        .context("failed to run git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree add failed: {}", stderr.trim());
    }

    Ok((worktree_dir, branch))
}

fn git_rev_parse_show_toplevel(cwd: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .context("failed to run git rev-parse")?;
    if !output.status.success() {
        bail!(
            "not a git repository: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let path = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(path))
}

pub struct AgentTool {
    swarm: CommunicateTool,
}

impl AgentTool {
    pub fn new() -> Self {
        Self {
            swarm: CommunicateTool::new(),
        }
    }
}

impl Default for AgentTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct AgentInput {
    description: String,
    prompt: String,
    #[serde(default)]
    subagent_type: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    isolation: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    /// Prior swarm/Agent session id. Continues via `swarm` DM (existing resume path).
    #[serde(default)]
    resume: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    run_in_background: Option<bool>,
    #[serde(default)]
    effort: Option<String>,
}

#[async_trait]
impl Tool for AgentTool {
    fn name(&self) -> &str {
        "Agent"
    }

    fn description(&self) -> &str {
        "Launch a specialized subagent (Claude-compatible Agent façade). \
         Use subagent_type for Explore / Plan / general-purpose / verification / code-reviewer \
         (or a custom label). Optional isolation=\"worktree\" runs in a temporary git worktree. \
         Optional model overrides the spawn model. Optional resume continues an existing \
         swarm agent session via DM. Thin wrapper over swarm spawn/message — use swarm \
         directly for multi-agent plans, teams, and task graphs."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["description", "prompt"],
            "properties": {
                "intent": super::intent_schema_property(),
                "description": {
                    "type": "string",
                    "description": "Short (3-5 word) description of the task."
                },
                "prompt": {
                    "type": "string",
                    "description": "Full task instructions for the agent."
                },
                "subagent_type": {
                    "type": "string",
                    "description": "Specialized agent type. Common values: Explore, Plan, general-purpose, verification, code-reviewer. Case/separator insensitive; custom values are passed through as a role label."
                },
                "model": {
                    "type": "string",
                    "description": "Optional model override for this agent (same semantics as swarm spawn model)."
                },
                "isolation": {
                    "type": "string",
                    "enum": ["worktree"],
                    "description": "Isolation mode. \"worktree\" creates a temporary git worktree so the agent works on an isolated copy of the repo. Mutually exclusive with cwd."
                },
                "cwd": {
                    "type": "string",
                    "description": "Absolute working directory for the agent. Mutually exclusive with isolation=\"worktree\"."
                },
                "resume": {
                    "type": "string",
                    "description": "Existing swarm/Agent session id to continue. Sends prompt as a DM instead of spawning. Prefer this over inventing a second resume protocol."
                },
                "name": {
                    "type": "string",
                    "description": "Optional display name for the spawned agent."
                },
                "run_in_background": {
                    "type": "boolean",
                    "description": "When true (default), spawn in headless/background style. When false, prefer inline swarm spawn mode."
                },
                "effort": {
                    "type": "string",
                    "enum": ["none", "low", "medium", "high", "xhigh", "max"],
                    "description": "Optional reasoning effort for the spawned agent."
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let parsed: AgentInput = serde_json::from_value(input)
            .context("Invalid Agent tool input")?;

        let description = parsed.description.trim();
        let prompt = parsed.prompt.trim();
        if description.is_empty() {
            bail!("'description' must be a non-empty string");
        }
        if prompt.is_empty() {
            bail!("'prompt' must be a non-empty string");
        }

        let (role, label) = map_subagent_type(parsed.subagent_type.as_deref());
        let spawn_prompt = build_spawn_prompt(
            role,
            &label,
            description,
            prompt,
            parsed.name.as_deref(),
        );

        // Resume path: wire through existing swarm DM — do not invent half-broken resume.
        if let Some(resume_id) = parsed
            .resume
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if parsed.isolation.is_some() || parsed.cwd.is_some() {
                bail!(
                    "resume cannot be combined with isolation/cwd; the existing agent keeps its working directory"
                );
            }
            // swarm DM requires tldr when message body is long (>240 chars).
            let tldr: String = description.chars().take(200).collect();
            let swarm_input = json!({
                "action": "dm",
                "to_session": resume_id,
                "message": spawn_prompt,
                "tldr": tldr,
            });
            let mut output = self.swarm.execute(swarm_input, ctx).await?;
            let suffix = format!(
                "\n[Agent resume] session={resume_id} subagent_type={label}"
            );
            output.output.push_str(&suffix);
            return Ok(output);
        }

        let isolation = parsed
            .isolation
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let cwd = parsed
            .cwd
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if isolation.is_some() && cwd.is_some() {
            bail!("isolation and cwd are mutually exclusive");
        }

        let mut worktree_meta: Option<(PathBuf, String)> = None;
        let working_dir = if isolation == Some("worktree") {
            let base = ctx
                .working_dir
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let (path, branch) = create_isolation_worktree(&base, description)?;
            let path_str = path.display().to_string();
            worktree_meta = Some((path, branch));
            Some(path_str)
        } else {
            cwd.map(|s| s.to_string())
        };

        let background = parsed.run_in_background.unwrap_or(true);
        let spawn_mode = if background { "headless" } else { "inline" };

        let mut swarm_input = json!({
            "action": "spawn",
            "prompt": spawn_prompt,
            "spawn_mode": spawn_mode,
        });
        if let Some(dir) = &working_dir {
            swarm_input["working_dir"] = json!(dir);
        }
        if let Some(model) = parsed
            .model
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            swarm_input["model"] = json!(model);
        }
        if let Some(effort) = parsed
            .effort
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            swarm_input["effort"] = json!(effort);
        }

        let mut output = self.swarm.execute(swarm_input, ctx).await?;

        let mut trailer = format!("\n[Agent] subagent_type={label}");
        if let Some((path, branch)) = worktree_meta {
            trailer.push_str(&format!(
                " isolation=worktree path={} branch={}",
                path.display(),
                branch
            ));
        }
        if let Some(model) = parsed.model.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            trailer.push_str(&format!(" model={model}"));
        }
        output.output.push_str(&trailer);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn map_subagent_type_defaults_to_general() {
        let (role, label) = map_subagent_type(None);
        assert_eq!(role, AgentRole::General);
        assert_eq!(label, "general-purpose");
    }

    #[test]
    fn map_subagent_type_normalizes_claude_names() {
        assert_eq!(map_subagent_type(Some("Explore")).0, AgentRole::Explore);
        assert_eq!(map_subagent_type(Some("Plan")).0, AgentRole::Plan);
        assert_eq!(
            map_subagent_type(Some("general-purpose")).0,
            AgentRole::General
        );
        assert_eq!(
            map_subagent_type(Some("Code Reviewer")).0,
            AgentRole::CodeReviewer
        );
        assert_eq!(
            map_subagent_type(Some("verification")).0,
            AgentRole::Verification
        );
    }

    #[test]
    fn map_subagent_type_maps_oh_my_readonly_to_explore() {
        assert_eq!(map_subagent_type(Some("oracle")).0, AgentRole::Explore);
        assert_eq!(map_subagent_type(Some("librarian")).0, AgentRole::Explore);
    }

    #[test]
    fn map_subagent_type_keeps_custom_label() {
        let (role, label) = map_subagent_type(Some("my-specialist"));
        assert_eq!(role, AgentRole::Custom);
        assert_eq!(label, "my-specialist");
    }

    #[test]
    fn build_spawn_prompt_includes_role_and_description() {
        let text = build_spawn_prompt(
            AgentRole::Explore,
            "explore",
            "find auth",
            "Where is JWT validated?",
            Some("scout"),
        );
        assert!(text.contains("read-only explore"));
        assert!(text.contains("Task summary: find auth"));
        assert!(text.contains("Agent name: scout"));
        assert!(text.contains("Where is JWT validated?"));
    }

    #[test]
    fn agent_tool_schema_exposes_claude_fields() {
        let schema = AgentTool::new().parameters_schema();
        let props = schema["properties"].as_object().unwrap();
        for key in [
            "description",
            "prompt",
            "subagent_type",
            "model",
            "isolation",
            "cwd",
            "resume",
            "name",
            "run_in_background",
        ] {
            assert!(props.contains_key(key), "missing {key}");
        }
        assert_eq!(AgentTool::new().name(), "Agent");
    }

    #[test]
    fn create_isolation_worktree_adds_git_worktree() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        assert!(
            Command::new("git")
                .args(["init"])
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["config", "user.email", "test@example.com"])
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["config", "user.name", "test"])
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );
        std::fs::write(repo.join("README"), "hi").unwrap();
        assert!(
            Command::new("git")
                .args(["add", "README"])
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["commit", "-m", "init"])
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );

        let (wt, branch) = create_isolation_worktree(repo, "find auth").unwrap();
        assert!(wt.join("README").exists());
        assert!(branch.starts_with("agent/"));
        assert!(wt.to_string_lossy().contains("agent-worktrees"));
    }
}
