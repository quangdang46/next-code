//! `BeadsTaskManager` — higher-level task operations for jcode tools.

use beads_rust::model::{Issue, IssueType, Priority, Status};
use beads_rust::storage::sqlite::IssueUpdate;
use beads_rust::storage::{ListFilters, ReadyFilters, ReadySortPolicy};

use crate::mapping::{ToBeadsIssue, ToJcodeTodoItem, TodoItem};
use crate::project::BeadsProject;

use anyhow::{Context, Result};

/// High-level task operations wrapping a `BeadsProject`.
pub struct BeadsTaskManager<'a> {
    project: &'a BeadsProject,
}

impl<'a> BeadsTaskManager<'a> {
    pub fn new(project: &'a BeadsProject) -> Self {
        BeadsTaskManager { project }
    }

    // ─── Task CRUD ────────────────────────────────────────────────────────

    /// Create a new task from a `TodoItem`.
    pub fn create_todo(&self, item: &TodoItem) -> Result<Issue> {
        let issue = item.to_issue();
        let actor = item.assigned_to.as_deref().unwrap_or("jcode");
        self.project.storage_mut().create_issue(&issue, actor)?;
        self.project.flush()?;
        Ok(issue)
    }

    /// Create a task with explicit fields.
    pub fn create_task(&self, title: &str, priority: Priority, labels: &[String]) -> Result<Issue> {
        let id = format!("bead-{}", short_id());
        let issue = Issue {
            id,
            title: title.to_string(),
            status: Status::Open,
            priority,
            issue_type: IssueType::Task,
            labels: labels.to_vec(),
            ..default_issue()
        };
        self.project.storage_mut().create_issue(&issue, "jcode")?;
        self.project.flush()?;
        Ok(issue)
    }

    /// List all open tasks.
    pub fn list_open_tasks(&self) -> Result<Vec<Issue>> {
        let filters = ListFilters {
            statuses: Some(vec![Status::Open, Status::InProgress, Status::Blocked]),
            ..ListFilters::default()
        };
        self.project
            .storage()
            .list_issues(&filters)
            .context("Failed to list open tasks")
    }

    /// List all tasks as `TodoItem`s.
    pub fn list_todo_items(&self) -> Result<Vec<TodoItem>> {
        let issues = self.list_open_tasks()?;
        Ok(issues.into_iter().map(|i| i.to_todo_item()).collect())
    }

    /// Get a single task by ID.
    pub fn get_task(&self, id: &str) -> Result<Option<Issue>> {
        self.project
            .storage()
            .get_issue(id)
            .context("Failed to get task")
    }

    /// Update a task's status.
    pub fn set_status(&self, id: &str, status: Status, actor: &str) -> Result<Issue> {
        let update = IssueUpdate {
            status: Some(status),
            ..IssueUpdate::default()
        };
        let updated = self
            .project
            .storage_mut()
            .update_issue(id, &update, actor)
            .context("Failed to update task status")?;
        self.project.flush()?;
        Ok(updated)
    }

    /// Close a task.
    pub fn close_task(&self, id: &str, reason: &str, actor: &str) -> Result<Issue> {
        let update = IssueUpdate {
            status: Some(Status::Closed),
            close_reason: Some(Some(reason.to_string())),
            ..IssueUpdate::default()
        };
        let updated = self
            .project
            .storage_mut()
            .update_issue(id, &update, actor)
            .context("Failed to close task")?;
        self.project.flush()?;
        Ok(updated)
    }

    // ─── Ready / Blocked queries ──────────────────────────────────────────

    /// Get tasks that are ready to work on.
    pub fn ready_tasks(&self, limit: usize) -> Result<Vec<Issue>> {
        let filters = ReadyFilters {
            limit: Some(limit),
            ..ReadyFilters::default()
        };
        self.project
            .storage()
            .get_ready_issues(&filters, ReadySortPolicy::Hybrid)
            .context("Failed to get ready tasks")
    }

    /// Get blocked tasks with blocker IDs.
    pub fn blocked_tasks(&self) -> Result<Vec<(Issue, Vec<String>)>> {
        self.project
            .storage()
            .get_blocked_issues()
            .context("Failed to get blocked tasks")
    }

    // ─── Dependencies ──────────────────────────────────────────────────────

    /// Add a dependency: `from` blocks on `to`.
    pub fn add_dependency(&self, from: &str, to: &str, actor: &str) -> Result<()> {
        if self.project.storage().would_create_cycle(from, to, true)? {
            anyhow::bail!("Adding dependency {from} -> {to} would create a cycle");
        }
        self.project
            .storage_mut()
            .add_dependency(from, to, "blocks", actor)?;
        self.project.flush()?;
        Ok(())
    }

    /// Remove a dependency.
    pub fn remove_dependency(&self, from: &str, to: &str, actor: &str) -> Result<()> {
        self.project
            .storage_mut()
            .remove_dependency(from, to, actor)?;
        self.project.flush()?;
        Ok(())
    }

    /// Get blockers for a task.
    pub fn blockers(&self, id: &str) -> Result<Vec<String>> {
        self.project
            .storage()
            .get_blockers(id)
            .context("Failed to get blockers")
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn default_issue() -> Issue {
    use chrono::Utc;
    Issue {
        id: String::new(),
        content_hash: None,
        title: String::new(),
        description: None,
        design: None,
        acceptance_criteria: None,
        notes: None,
        status: Status::Open,
        priority: Priority::MEDIUM,
        issue_type: IssueType::Task,
        assignee: None,
        owner: None,
        estimated_minutes: None,
        created_at: Utc::now(),
        created_by: None,
        updated_at: Utc::now(),
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

fn short_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos & 0xFFFF_FFFF)
}
