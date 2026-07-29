//! Claude-style session task graph (TaskCreate / TaskGet / TaskList / TaskUpdate).
//!
//! Separate from the flat `todo` tool and from team `team_task_*` boards.
//! Tasks form a dependency graph via bidirectional `blocks` / `blockedBy`,
//! with optional `owner` and `activeForm` for swarm claim + spinner UX.
//!
//! Storage: `{NEXT_CODE_HOME}/task-graphs/{list_id}.json` (session-scoped by
//! default; override with `NEXT_CODE_TASK_LIST_ID`).

use crate::storage;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Env override for the shared task-list id (Claude: `CLAUDE_CODE_TASK_LIST_ID`).
pub const TASK_LIST_ID_ENV: &str = "NEXT_CODE_TASK_LIST_ID";

const VALID_STATUSES: &[&str] = &["pending", "in_progress", "completed"];

/// Process-wide mutex so concurrent tool calls in one process do not race
/// read-modify-write on the same list file.
fn list_locks() -> &'static Mutex<()> {
    static LOCKS: OnceLock<Mutex<()>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(()))
}

/// Resolve which task list to use for this call.
///
/// Priority: `NEXT_CODE_TASK_LIST_ID` → `session_id`.
pub fn resolve_task_list_id(session_id: &str) -> String {
    std::env::var(TASK_LIST_ID_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| session_id.to_string())
}

fn sanitize_path_component(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn list_path(task_list_id: &str) -> Result<PathBuf> {
    let base = storage::next_code_dir()?;
    Ok(base
        .join("task-graphs")
        .join(format!("{}.json", sanitize_path_component(task_list_id))))
}

/// One node in the session task graph (Claude Task schema).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphTask {
    pub id: String,
    pub subject: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_form: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub status: String,
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TaskListFile {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    tasks: Vec<GraphTask>,
}

fn load_list(task_list_id: &str) -> Result<TaskListFile> {
    let path = list_path(task_list_id)?;
    if !path.exists() {
        return Ok(TaskListFile::default());
    }
    storage::read_json(&path).or_else(|_| Ok(TaskListFile::default()))
}

fn save_list(task_list_id: &str, list: &TaskListFile) -> Result<()> {
    let path = list_path(task_list_id)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    storage::write_json_fast(&path, list)
}

fn with_list_mut<T>(
    task_list_id: &str,
    f: impl FnOnce(&mut TaskListFile) -> Result<T>,
) -> Result<T> {
    let _guard = list_locks()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut list = load_list(task_list_id)?;
    let out = f(&mut list)?;
    save_list(task_list_id, &list)?;
    Ok(out)
}

fn validate_status(status: &str) -> Result<()> {
    if VALID_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(anyhow!(
            "Invalid status '{status}'. Expected one of: pending, in_progress, completed, deleted"
        ))
    }
}

/// Create a pending task; returns the assigned numeric id string.
pub fn create_task(
    task_list_id: &str,
    subject: String,
    description: String,
    active_form: Option<String>,
    metadata: Option<Map<String, Value>>,
) -> Result<GraphTask> {
    if subject.trim().is_empty() {
        return Err(anyhow!("subject must not be empty"));
    }
    if description.trim().is_empty() {
        return Err(anyhow!("description must not be empty"));
    }
    with_list_mut(task_list_id, |list| {
        list.next_id = list.next_id.max(
            list.tasks
                .iter()
                .filter_map(|t| t.id.parse::<u64>().ok())
                .max()
                .unwrap_or(0),
        ) + 1;
        let id = list.next_id.to_string();
        let task = GraphTask {
            id: id.clone(),
            subject,
            description,
            active_form,
            owner: None,
            status: "pending".to_string(),
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            metadata,
        };
        list.tasks.push(task.clone());
        Ok(task)
    })
}

/// Get a task by id, or `None` if missing.
pub fn get_task(task_list_id: &str, task_id: &str) -> Result<Option<GraphTask>> {
    let list = load_list(task_list_id)?;
    Ok(list.tasks.into_iter().find(|t| t.id == task_id))
}

/// List all tasks, excluding those with `metadata._internal == true`.
pub fn list_tasks(task_list_id: &str) -> Result<Vec<GraphTask>> {
    let list = load_list(task_list_id)?;
    Ok(list
        .tasks
        .into_iter()
        .filter(|t| {
            !t.metadata
                .as_ref()
                .and_then(|m| m.get("_internal"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .collect())
}

/// Summary view for TaskList: open blockers only (completed blockers omitted).
pub fn list_task_summaries(task_list_id: &str) -> Result<Vec<TaskSummary>> {
    let tasks = list_tasks(task_list_id)?;
    let completed: std::collections::HashSet<String> = tasks
        .iter()
        .filter(|t| t.status == "completed")
        .map(|t| t.id.clone())
        .collect();
    Ok(tasks
        .into_iter()
        .map(|t| TaskSummary {
            id: t.id,
            subject: t.subject,
            status: t.status,
            owner: t.owner,
            blocked_by: t
                .blocked_by
                .into_iter()
                .filter(|id| !completed.contains(id))
                .collect(),
        })
        .collect())
}

/// Compact TaskList row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummary {
    pub id: String,
    pub subject: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub blocked_by: Vec<String>,
}

/// Fields that may be patched by TaskUpdate.
#[derive(Debug, Clone, Default)]
pub struct TaskUpdateFields {
    pub subject: Option<String>,
    pub description: Option<String>,
    pub active_form: Option<String>,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub metadata: Option<Map<String, Value>>,
    pub add_blocks: Vec<String>,
    pub add_blocked_by: Vec<String>,
}

/// Result of a TaskUpdate call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateResult {
    pub success: bool,
    pub task_id: String,
    pub updated_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_change: Option<StatusChange>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusChange {
    pub from: String,
    pub to: String,
}

/// Bidirectional edge: `from` blocks `to` (and `to` is blockedBy `from`).
fn link_blocks(list: &mut TaskListFile, from_id: &str, to_id: &str) -> Result<bool> {
    if from_id == to_id {
        return Err(anyhow!("a task cannot block itself"));
    }
    let from_idx = list
        .tasks
        .iter()
        .position(|t| t.id == from_id)
        .ok_or_else(|| anyhow!("Task '{from_id}' not found"))?;
    let to_idx = list
        .tasks
        .iter()
        .position(|t| t.id == to_id)
        .ok_or_else(|| anyhow!("Task '{to_id}' not found"))?;
    let mut changed = false;
    if !list.tasks[from_idx].blocks.contains(&to_id.to_string()) {
        list.tasks[from_idx].blocks.push(to_id.to_string());
        changed = true;
    }
    if !list.tasks[to_idx].blocked_by.contains(&from_id.to_string()) {
        list.tasks[to_idx].blocked_by.push(from_id.to_string());
        changed = true;
    }
    Ok(changed)
}

fn strip_task_refs(list: &mut TaskListFile, task_id: &str) {
    for task in &mut list.tasks {
        task.blocks.retain(|id| id != task_id);
        task.blocked_by.retain(|id| id != task_id);
    }
}

/// Update or delete a task. `status: "deleted"` removes it permanently.
pub fn update_task(
    task_list_id: &str,
    task_id: &str,
    fields: TaskUpdateFields,
) -> Result<TaskUpdateResult> {
    with_list_mut(task_list_id, |list| {
        let Some(idx) = list.tasks.iter().position(|t| t.id == task_id) else {
            return Ok(TaskUpdateResult {
                success: false,
                task_id: task_id.to_string(),
                updated_fields: Vec::new(),
                error: Some("Task not found".to_string()),
                status_change: None,
            });
        };

        if fields.status.as_deref() == Some("deleted") {
            let from = list.tasks[idx].status.clone();
            list.tasks.remove(idx);
            strip_task_refs(list, task_id);
            return Ok(TaskUpdateResult {
                success: true,
                task_id: task_id.to_string(),
                updated_fields: vec!["deleted".to_string()],
                error: None,
                status_change: Some(StatusChange {
                    from,
                    to: "deleted".to_string(),
                }),
            });
        }

        let mut updated_fields = Vec::new();
        let mut status_change = None;

        {
            let task = &mut list.tasks[idx];
            if let Some(subject) = fields.subject {
                if subject != task.subject {
                    task.subject = subject;
                    updated_fields.push("subject".to_string());
                }
            }
            if let Some(description) = fields.description {
                if description != task.description {
                    task.description = description;
                    updated_fields.push("description".to_string());
                }
            }
            if let Some(active_form) = fields.active_form {
                if task.active_form.as_deref() != Some(active_form.as_str()) {
                    task.active_form = Some(active_form);
                    updated_fields.push("activeForm".to_string());
                }
            }
            if let Some(owner) = fields.owner {
                if task.owner.as_deref() != Some(owner.as_str()) {
                    task.owner = Some(owner);
                    updated_fields.push("owner".to_string());
                }
            }
            if let Some(patch) = fields.metadata {
                let mut merged = task.metadata.clone().unwrap_or_default();
                for (key, value) in patch {
                    if value.is_null() {
                        merged.remove(&key);
                    } else {
                        merged.insert(key, value);
                    }
                }
                task.metadata = if merged.is_empty() {
                    None
                } else {
                    Some(merged)
                };
                updated_fields.push("metadata".to_string());
            }
            if let Some(status) = fields.status {
                validate_status(&status)?;
                if status != task.status {
                    status_change = Some(StatusChange {
                        from: task.status.clone(),
                        to: status.clone(),
                    });
                    task.status = status;
                    updated_fields.push("status".to_string());
                }
            }
        }

        if !fields.add_blocks.is_empty() {
            let mut changed = false;
            for to_id in &fields.add_blocks {
                if link_blocks(list, task_id, to_id)? {
                    changed = true;
                }
            }
            if changed {
                updated_fields.push("blocks".to_string());
            }
        }
        if !fields.add_blocked_by.is_empty() {
            let mut changed = false;
            for from_id in &fields.add_blocked_by {
                if link_blocks(list, from_id, task_id)? {
                    changed = true;
                }
            }
            if changed {
                updated_fields.push("blockedBy".to_string());
            }
        }

        Ok(TaskUpdateResult {
            success: true,
            task_id: task_id.to_string(),
            updated_fields,
            error: None,
            status_change,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn with_temp_home<T>(f: impl FnOnce() -> T) -> T {
        let _guard = crate::storage::lock_test_env();
        let previous = std::env::var_os("NEXT_CODE_HOME");
        let dir = tempfile::TempDir::new().expect("tempdir");
        crate::env::set_var("NEXT_CODE_HOME", dir.path());
        let out = f();
        match previous {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
        out
    }

    #[test]
    fn create_get_list_update_and_dependencies() {
        with_temp_home(|| {
            let list = "sess-1";
            let a = create_task(
                list,
                "Design schema".into(),
                "Define auth tables".into(),
                Some("Designing schema".into()),
                None,
            )
            .unwrap();
            assert_eq!(a.id, "1");
            assert_eq!(a.status, "pending");

            let b = create_task(
                list,
                "Implement API".into(),
                "Wire JWT middleware".into(),
                None,
                None,
            )
            .unwrap();
            assert_eq!(b.id, "2");

            let result = update_task(
                list,
                "2",
                TaskUpdateFields {
                    add_blocked_by: vec!["1".into()],
                    status: Some("in_progress".into()),
                    owner: Some("worker-1".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(result.success);
            assert!(result.updated_fields.contains(&"blockedBy".to_string()));
            assert!(result.updated_fields.contains(&"status".to_string()));
            assert!(result.updated_fields.contains(&"owner".to_string()));

            let got = get_task(list, "2").unwrap().unwrap();
            assert_eq!(got.blocked_by, vec!["1"]);
            assert_eq!(got.owner.as_deref(), Some("worker-1"));
            assert_eq!(got.status, "in_progress");

            let blocker = get_task(list, "1").unwrap().unwrap();
            assert_eq!(blocker.blocks, vec!["2"]);

            let summaries = list_task_summaries(list).unwrap();
            assert_eq!(summaries.len(), 2);
            let two = summaries.iter().find(|s| s.id == "2").unwrap();
            assert_eq!(two.blocked_by, vec!["1"]);

            update_task(
                list,
                "1",
                TaskUpdateFields {
                    status: Some("completed".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            let summaries = list_task_summaries(list).unwrap();
            let two = summaries.iter().find(|s| s.id == "2").unwrap();
            assert!(
                two.blocked_by.is_empty(),
                "completed blockers filtered from TaskList"
            );
        });
    }

    #[test]
    fn delete_removes_refs_and_internal_hidden_from_list() {
        with_temp_home(|| {
            let list = "sess-del";
            create_task(list, "A".into(), "a".into(), None, None).unwrap();
            create_task(list, "B".into(), "b".into(), None, None).unwrap();
            update_task(
                list,
                "1",
                TaskUpdateFields {
                    add_blocks: vec!["2".into()],
                    ..Default::default()
                },
            )
            .unwrap();

            let deleted = update_task(
                list,
                "1",
                TaskUpdateFields {
                    status: Some("deleted".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(deleted.updated_fields, vec!["deleted"]);
            assert!(get_task(list, "1").unwrap().is_none());
            let b = get_task(list, "2").unwrap().unwrap();
            assert!(b.blocked_by.is_empty());

            let mut meta = Map::new();
            meta.insert("_internal".into(), json!(true));
            create_task(list, "Hidden".into(), "x".into(), None, Some(meta)).unwrap();
            let visible = list_tasks(list).unwrap();
            assert!(visible.iter().all(|t| t.subject != "Hidden"));
        });
    }

    #[test]
    fn resolve_task_list_id_prefers_env() {
        let _guard = crate::storage::lock_test_env();
        let previous = std::env::var_os(TASK_LIST_ID_ENV);
        crate::env::set_var(TASK_LIST_ID_ENV, "shared-team");
        assert_eq!(resolve_task_list_id("session-xyz"), "shared-team");
        match previous {
            Some(v) => crate::env::set_var(TASK_LIST_ID_ENV, v),
            None => crate::env::remove_var(TASK_LIST_ID_ENV),
        }
    }
}
