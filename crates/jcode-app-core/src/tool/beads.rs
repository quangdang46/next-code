//! Beads-rs issue tracker tools for jcode agents.
//!
//! Tools: `beads_list`, `beads_create`, `beads_ready`, `beads_claim`,
//! `beads_close`, `beads_dep`.

use super::{Tool, ToolContext, ToolOutput};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

// ─── BeadsListTool ─────────────────────────────────────────────────────────

pub struct BeadsListTool;

impl BeadsListTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BeadsListInput {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    assignee: Option<String>,
    #[serde(default)]
    sort: Option<String>,
}

#[async_trait]
impl Tool for BeadsListTool {
    fn name(&self) -> &str {
        "beads_list"
    }
    fn description(&self) -> &str {
        "List beads issues with optional filters (status, label, assignee, limit)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["open", "in_progress", "blocked", "closed", "all"],
                    "description": "Filter by status."
                },
                "limit": { "type": "integer", "description": "Max results (default 50)." },
                "label": { "type": "string", "description": "Filter by label." },
                "assignee": { "type": "string", "description": "Filter by assignee." },
                "sort": {
                    "type": "string",
                    "enum": ["priority", "created", "updated"],
                    "description": "Sort order."
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: BeadsListInput = serde_json::from_value(input)?;
        let wd = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let project = crate::beads::BeadsProject::open(wd)
            .map_err(|e| anyhow::anyhow!("Failed to open beads project: {e}"))?;

        let mut filters = crate::beads::ListFilters::default();
        match params.status.as_deref() {
            Some("open") => {
                filters.statuses = Some(vec![crate::beads::Status::Open]);
            }
            Some("in_progress") => {
                filters.statuses = Some(vec![crate::beads::Status::InProgress]);
            }
            Some("blocked") => {
                filters.statuses = Some(vec![crate::beads::Status::Blocked]);
            }
            Some("closed") => {
                filters.include_closed = true;
                filters.statuses = Some(vec![crate::beads::Status::Closed]);
            }
            Some("all") => {
                filters.include_closed = true;
                filters.include_deferred = true;
            }
            _ => {
                filters.statuses = Some(vec![
                    crate::beads::Status::Open,
                    crate::beads::Status::InProgress,
                    crate::beads::Status::Blocked,
                ]);
            }
        }
        if let Some(label) = &params.label {
            filters.labels = Some(vec![label.clone()]);
        }
        if let Some(assignee) = &params.assignee {
            filters.assignee = Some(assignee.clone());
        }
        if let Some(sort) = &params.sort {
            filters.sort = Some(sort.clone());
        }
        filters.limit = params.limit.or(Some(50));

        let issues = project.storage().list_issues(&filters)?;
        let items: Vec<Value> = issues
            .into_iter()
            .map(|i| {
                json!({
                    "id": i.id, "title": i.title, "status": i.status.as_str(),
                    "priority": i.priority.to_string(), "assignee": i.assignee,
                    "labels": i.labels,
                    "created_at": i.created_at.to_rfc3339(),
                    "updated_at": i.updated_at.to_rfc3339(),
                })
            })
            .collect();

        Ok(
            ToolOutput::new(serde_json::to_string_pretty(&json!({"issues": items}))?)
                .with_title(format!("Issues: {}", items.len())),
        )
    }
}

// ─── BeadsCreateTool ───────────────────────────────────────────────────────

pub struct BeadsCreateTool;

impl BeadsCreateTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BeadsCreateInput {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    assignee: Option<String>,
    #[serde(default)]
    issue_type: Option<String>,
}

#[async_trait]
impl Tool for BeadsCreateTool {
    fn name(&self) -> &str {
        "beads_create"
    }
    fn description(&self) -> &str {
        "Create a new beads issue/task."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object", "required": ["title"],
            "properties": {
                "title": { "type": "string", "description": "Issue title." },
                "description": { "type": "string" },
                "priority": {
                    "type": "integer",
                    "description": "0=critical, 1=high, 2=medium, 3=low, 4=backlog",
                    "default": 2
                },
                "labels": { "type": "array", "items": { "type": "string" } },
                "assignee": { "type": "string" },
                "issue_type": {
                    "type": "string", "enum": ["task", "bug", "feature", "epic", "chore"]
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: BeadsCreateInput = serde_json::from_value(input)?;
        let wd = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let project = crate::beads::BeadsProject::open_or_init(wd, "bead")
            .map_err(|e| anyhow::anyhow!("Failed to open beads project: {e}"))?;

        let priority = crate::beads::Priority(params.priority.unwrap_or(2).clamp(0, 4));
        let issue_type = match params.issue_type.as_deref() {
            Some("bug") => crate::beads::IssueType::Bug,
            Some("feature") => crate::beads::IssueType::Feature,
            Some("epic") => crate::beads::IssueType::Epic,
            Some("chore") => crate::beads::IssueType::Chore,
            _ => crate::beads::IssueType::Task,
        };

        let id = format!("bead-{}", short_id());
        let now = chrono::Utc::now();
        let issue = crate::beads::Issue {
            id,
            title: params.title,
            description: params.description,
            priority,
            issue_type,
            labels: params.labels,
            assignee: params.assignee,
            status: crate::beads::Status::Open,
            created_at: now,
            updated_at: now,
            ..default_issue()
        };
        project
            .storage_mut()
            .create_issue(&issue, &ctx.session_id)
            .context("Failed to create issue")?;
        project.flush()?;
        Ok(ToolOutput::new(format!("Created issue `{}`", issue.id))
            .with_title(issue.id.clone())
            .with_metadata(json!({"id": issue.id})))
    }
}

// ─── BeadsReadyTool ────────────────────────────────────────────────────────

pub struct BeadsReadyTool;

impl BeadsReadyTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BeadsReadyInput {
    #[serde(default = "default_ready_limit")]
    limit: usize,
}
fn default_ready_limit() -> usize {
    10
}

#[async_trait]
impl Tool for BeadsReadyTool {
    fn name(&self) -> &str {
        "beads_ready"
    }
    fn description(&self) -> &str {
        "Show beads issues ready-to-work (no blockers, highest priority first)."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "properties": {
            "limit": { "type": "integer", "description": "Max results (default 10)." }
        }})
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: BeadsReadyInput = serde_json::from_value(input)?;
        let wd = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let project = crate::beads::BeadsProject::open(wd)
            .map_err(|e| anyhow::anyhow!("Failed to open beads project: {e}"))?;

        let manager = crate::beads::BeadsTaskManager::new(&project);
        let ready = manager.ready_tasks(params.limit)?;
        let items: Vec<Value> = ready
            .into_iter()
            .map(|i| {
                json!({
                    "id": i.id, "title": i.title, "priority": i.priority.to_string(),
                    "assignee": i.assignee, "labels": i.labels,
                    "created_at": i.created_at.to_rfc3339(),
                })
            })
            .collect();

        Ok(
            ToolOutput::new(serde_json::to_string_pretty(&json!({"ready": items}))?)
                .with_title(format!("Ready: {} items", items.len())),
        )
    }
}

// ─── BeadsClaimTool ────────────────────────────────────────────────────────

pub struct BeadsClaimTool;

impl BeadsClaimTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BeadsClaimInput {
    id: String,
}

#[async_trait]
impl Tool for BeadsClaimTool {
    fn name(&self) -> &str {
        "beads_claim"
    }
    fn description(&self) -> &str {
        "Claim a beads issue (set status to in_progress)."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "required": ["id"], "properties": {
            "id": { "type": "string", "description": "Issue ID to claim." }
        }})
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: BeadsClaimInput = serde_json::from_value(input)?;
        let wd = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let project = crate::beads::BeadsProject::open(wd)
            .map_err(|e| anyhow::anyhow!("Failed to open beads project: {e}"))?;

        let manager = crate::beads::BeadsTaskManager::new(&project);
        let issue = manager
            .set_status(
                &params.id,
                crate::beads::Status::InProgress,
                &ctx.session_id,
            )
            .map_err(|e| anyhow::anyhow!("Failed to claim issue: {e}"))?;

        Ok(ToolOutput::new(format!("Claimed issue `{}`", issue.id))
            .with_title(issue.title)
            .with_metadata(json!({"id": issue.id, "status": "in_progress"})))
    }
}

// ─── BeadsCloseTool ────────────────────────────────────────────────────────

pub struct BeadsCloseTool;

impl BeadsCloseTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BeadsCloseInput {
    id: String,
    #[serde(default)]
    reason: String,
}

#[async_trait]
impl Tool for BeadsCloseTool {
    fn name(&self) -> &str {
        "beads_close"
    }
    fn description(&self) -> &str {
        "Close a beads issue with an optional reason."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "required": ["id"], "properties": {
            "id": { "type": "string", "description": "Issue ID to close." },
            "reason": { "type": "string", "description": "Reason for closing." }
        }})
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: BeadsCloseInput = serde_json::from_value(input)?;
        let wd = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let project = crate::beads::BeadsProject::open(wd)
            .map_err(|e| anyhow::anyhow!("Failed to open beads project: {e}"))?;

        let manager = crate::beads::BeadsTaskManager::new(&project);
        let issue = manager.close_task(&params.id, &params.reason, &ctx.session_id)?;

        Ok(ToolOutput::new(format!("Closed issue `{}`", issue.id))
            .with_title(issue.title)
            .with_metadata(json!({"id": issue.id, "status": "closed"})))
    }
}

// ─── BeadsDepTool ──────────────────────────────────────────────────────────

pub struct BeadsDepTool;

impl BeadsDepTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BeadsDepInput {
    action: String,
    issue: String,
    depends_on: String,
}

#[async_trait]
impl Tool for BeadsDepTool {
    fn name(&self) -> &str {
        "beads_dep"
    }
    fn description(&self) -> &str {
        "Add or remove a beads dependency. 'issue' blocks on 'depends_on'."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "required": ["action", "issue", "depends_on"], "properties": {
            "action": { "type": "string", "enum": ["add", "remove"] },
            "issue": { "type": "string", "description": "Issue that blocks." },
            "depends_on": { "type": "string", "description": "Issue that is blocked by." }
        }})
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: BeadsDepInput = serde_json::from_value(input)?;
        let wd = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new("."));
        let project = crate::beads::BeadsProject::open(wd)
            .map_err(|e| anyhow::anyhow!("Failed to open beads project: {e}"))?;

        let manager = crate::beads::BeadsTaskManager::new(&project);
        match params.action.as_str() {
            "add" => {
                manager.add_dependency(&params.issue, &params.depends_on, &ctx.session_id)?;
                Ok(ToolOutput::new(format!(
                    "Added dep: {} blocks on {}",
                    params.issue, params.depends_on
                )))
            }
            "remove" => {
                manager.remove_dependency(&params.issue, &params.depends_on, &ctx.session_id)?;
                Ok(ToolOutput::new(format!(
                    "Removed dep: {} → {}",
                    params.issue, params.depends_on
                )))
            }
            _ => Err(anyhow::anyhow!("Unknown action: {}", params.action)),
        }
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn short_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos & 0xFFFF_FFFF)
}

fn default_issue() -> crate::beads::Issue {
    let now = chrono::Utc::now();
    crate::beads::Issue {
        id: String::new(),
        content_hash: None,
        title: String::new(),
        description: None,
        design: None,
        acceptance_criteria: None,
        notes: None,
        status: crate::beads::Status::Open,
        priority: crate::beads::Priority(2),
        issue_type: crate::beads::IssueType::Task,
        assignee: None,
        owner: None,
        estimated_minutes: None,
        created_at: now,
        created_by: None,
        updated_at: now,
        closed_at: None,
        close_reason: None,
        closed_by_session: None,
        due_at: None,
        defer_until: None,
        external_ref: None,
        source_system: Some("jcode".to_string()),
        source_repo: None,
        source_repo_path: None,
        agent_context: None,
        labels: Vec::new(),
        deleted_at: None,
        deleted_by: None,
        delete_reason: None,
        original_type: None,
        compaction_level: None,
        compacted_at: None,
        compacted_at_commit: None,
        original_size: None,
        sender: None,
        ephemeral: false,
        pinned: false,
        is_template: false,
        dependencies: Vec::new(),
        comments: Vec::new(),
    }
}
