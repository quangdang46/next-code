//! Claude-style TaskCreate / TaskGet / TaskList / TaskUpdate tools.
//!
//! Session-scoped task graph with `blocks` / `blockedBy`, `owner`, and
//! `activeForm`. Coexists with the flat `todo` tool and team `team_task_*`
//! tools — does not replace either.

use super::{Tool, ToolContext, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::task_graph::{
    TaskUpdateFields, create_task, get_task, list_task_summaries, resolve_task_list_id, update_task,
};

fn list_id(ctx: &ToolContext) -> String {
    resolve_task_list_id(&ctx.session_id)
}

fn repair_task_id(input: &mut Value) {
    // Claude Code repairs id / task_id → taskId before execution.
    let obj = match input.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    if obj.contains_key("taskId") {
        return;
    }
    if let Some(v) = obj.remove("task_id").or_else(|| obj.remove("id")) {
        obj.insert("taskId".to_string(), v);
    }
    if let Some(v) = obj.remove("active_form") {
        obj.insert("activeForm".to_string(), v);
    }
}

// ---------------------------------------------------------------------------
// TaskCreate
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct TaskCreateTool;

impl TaskCreateTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskCreateInput {
    subject: String,
    description: String,
    #[serde(default)]
    active_form: Option<String>,
    #[serde(default)]
    metadata: Option<Map<String, Value>>,
}

#[async_trait]
impl Tool for TaskCreateTool {
    fn name(&self) -> &str {
        "TaskCreate"
    }

    fn description(&self) -> &str {
        "Create a new task in the session task graph. Tasks start as pending \
         with no owner. Use TaskUpdate to set status, owner, and dependencies \
         (blocks / blockedBy). Prefer this over rewriting the full todo list \
         when tracking multi-step work with dependencies."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["subject", "description"],
            "properties": {
                "intent": super::intent_schema_property(),
                "subject": {
                    "type": "string",
                    "description": "Brief imperative title (e.g. \"Fix authentication bug\")."
                },
                "description": {
                    "type": "string",
                    "description": "What needs to be done."
                },
                "activeForm": {
                    "type": "string",
                    "description": "Present continuous form shown in spinner when in_progress (e.g. \"Fixing authentication bug\")."
                },
                "metadata": {
                    "type": "object",
                    "description": "Arbitrary metadata to attach to the task."
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: TaskCreateInput = serde_json::from_value(input)?;
        let task = create_task(
            &list_id(&ctx),
            params.subject,
            params.description,
            params.active_form,
            params.metadata,
        )?;
        let payload = json!({ "task": { "id": task.id, "subject": task.subject } });
        Ok(ToolOutput::new(format!(
            "Task #{} created successfully: {}",
            task.id,
            payload["task"]["subject"].as_str().unwrap_or("")
        ))
        .with_title(format!("Task #{} created", task.id))
        .with_metadata(payload))
    }
}

// ---------------------------------------------------------------------------
// TaskGet
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct TaskGetTool;

impl TaskGetTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskGetInput {
    task_id: String,
}

#[async_trait]
impl Tool for TaskGetTool {
    fn name(&self) -> &str {
        "TaskGet"
    }

    fn description(&self) -> &str {
        "Retrieve full details for a task by id, including description, status, \
         blocks, and blockedBy."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["taskId"],
            "properties": {
                "intent": super::intent_schema_property(),
                "taskId": {
                    "type": "string",
                    "description": "The ID of the task to retrieve."
                }
            }
        })
    }

    async fn execute(&self, mut input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        repair_task_id(&mut input);
        let params: TaskGetInput = serde_json::from_value(input)?;
        match get_task(&list_id(&ctx), &params.task_id)? {
            None => Ok(ToolOutput::new("Task not found")
                .with_title(format!("Task #{} missing", params.task_id))
                .with_metadata(json!({ "task": null }))),
            Some(task) => {
                let mut lines = vec![
                    format!("Task #{}: {}", task.id, task.subject),
                    format!("Status: {}", task.status),
                    format!("Description: {}", task.description),
                ];
                if let Some(owner) = &task.owner {
                    lines.push(format!("Owner: {owner}"));
                }
                if let Some(active) = &task.active_form {
                    lines.push(format!("Active form: {active}"));
                }
                if !task.blocked_by.is_empty() {
                    lines.push(format!(
                        "Blocked by: {}",
                        task.blocked_by
                            .iter()
                            .map(|id| format!("#{id}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !task.blocks.is_empty() {
                    lines.push(format!(
                        "Blocks: {}",
                        task.blocks
                            .iter()
                            .map(|id| format!("#{id}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                Ok(ToolOutput::new(lines.join("\n"))
                    .with_title(format!("Task #{}", task.id))
                    .with_metadata(json!({ "task": task })))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// TaskList
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct TaskListTool;

impl TaskListTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TaskListTool {
    fn name(&self) -> &str {
        "TaskList"
    }

    fn description(&self) -> &str {
        "List all tasks in the session task graph with status, owner, and open \
         blockers. Use after finishing work to find the next claimable task \
         (pending, no owner, empty blockedBy)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property()
            }
        })
    }

    async fn execute(&self, _input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let tasks = list_task_summaries(&list_id(&ctx))?;
        if tasks.is_empty() {
            return Ok(ToolOutput::new("No tasks found")
                .with_title("No tasks")
                .with_metadata(json!({ "tasks": [] })));
        }
        let lines: Vec<String> = tasks
            .iter()
            .map(|task| {
                let owner = task
                    .owner
                    .as_ref()
                    .map(|o| format!(" ({o})"))
                    .unwrap_or_default();
                let blocked = if task.blocked_by.is_empty() {
                    String::new()
                } else {
                    format!(
                        " [blocked by {}]",
                        task.blocked_by
                            .iter()
                            .map(|id| format!("#{id}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!(
                    "#{} [{}] {}{}{}",
                    task.id, task.status, task.subject, owner, blocked
                )
            })
            .collect();
        Ok(ToolOutput::new(lines.join("\n"))
            .with_title(format!("{} task(s)", tasks.len()))
            .with_metadata(json!({ "tasks": tasks })))
    }
}

// ---------------------------------------------------------------------------
// TaskUpdate
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct TaskUpdateTool;

impl TaskUpdateTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskUpdateInput {
    task_id: String,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    active_form: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    add_blocks: Option<Vec<String>>,
    #[serde(default)]
    add_blocked_by: Option<Vec<String>>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    metadata: Option<Map<String, Value>>,
}

#[async_trait]
impl Tool for TaskUpdateTool {
    fn name(&self) -> &str {
        "TaskUpdate"
    }

    fn description(&self) -> &str {
        "Update a task's status, owner, description, activeForm, metadata, or \
         dependencies. Set status to \"deleted\" to remove a task. Use \
         addBlocks / addBlockedBy to wire the dependency graph."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["taskId"],
            "properties": {
                "intent": super::intent_schema_property(),
                "taskId": {
                    "type": "string",
                    "description": "The ID of the task to update."
                },
                "subject": {
                    "type": "string",
                    "description": "New subject for the task."
                },
                "description": {
                    "type": "string",
                    "description": "New description for the task."
                },
                "activeForm": {
                    "type": "string",
                    "description": "Present continuous form shown in spinner when in_progress."
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "deleted"],
                    "description": "New status. Use \"deleted\" to permanently remove the task."
                },
                "addBlocks": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Task IDs that this task blocks."
                },
                "addBlockedBy": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Task IDs that must complete before this task can start."
                },
                "owner": {
                    "type": "string",
                    "description": "Agent name to assign / claim the task."
                },
                "metadata": {
                    "type": "object",
                    "description": "Metadata keys to merge. Set a key to null to delete it."
                }
            }
        })
    }

    async fn execute(&self, mut input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        repair_task_id(&mut input);
        let params: TaskUpdateInput = serde_json::from_value(input)?;
        let result = update_task(
            &list_id(&ctx),
            &params.task_id,
            TaskUpdateFields {
                subject: params.subject,
                description: params.description,
                active_form: params.active_form,
                status: params.status,
                owner: params.owner,
                metadata: params.metadata,
                add_blocks: params.add_blocks.unwrap_or_default(),
                add_blocked_by: params.add_blocked_by.unwrap_or_default(),
            },
        )?;

        let content = if !result.success {
            result
                .error
                .clone()
                .unwrap_or_else(|| format!("Task #{} not found", result.task_id))
        } else if result.updated_fields.is_empty() {
            format!("Task #{} unchanged", result.task_id)
        } else {
            format!(
                "Updated task #{} {}",
                result.task_id,
                result.updated_fields.join(", ")
            )
        };

        Ok(ToolOutput::new(content)
            .with_title(format!("Task #{}", result.task_id))
            .with_metadata(serde_json::to_value(&result)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_code_tool_core::ToolExecutionMode;

    fn ctx(session: &str) -> ToolContext {
        ToolContext {
            session_id: session.to_string(),
            message_id: "m".into(),
            tool_call_id: "t".into(),
            working_dir: Some(std::env::temp_dir()),
            stdin_request_tx: None,
            ask_user_question_tx: None,
            graceful_shutdown_signal: None,
            execution_mode: ToolExecutionMode::Direct,
            best_of_n_run_id: None,
            best_of_n_candidate_id: None,
        }
    }

    #[tokio::test]
    async fn create_list_get_update_round_trip() {
        let _guard = crate::storage::lock_test_env();
        let previous = std::env::var_os("NEXT_CODE_HOME");
        let dir = tempfile::TempDir::new().expect("tempdir");
        crate::env::set_var("NEXT_CODE_HOME", dir.path());

        let session = "tool-round-trip";
        let create = TaskCreateTool::new()
            .execute(
                json!({
                    "subject": "Write tests",
                    "description": "Cover task graph CRUD",
                    "activeForm": "Writing tests"
                }),
                ctx(session),
            )
            .await
            .unwrap();
        assert!(create.output.contains("Task #1 created"));
        let id = create
            .metadata
            .as_ref()
            .and_then(|m| m["task"]["id"].as_str())
            .unwrap()
            .to_string();

        TaskCreateTool::new()
            .execute(
                json!({
                    "subject": "Wire registry",
                    "description": "Register Task* tools"
                }),
                ctx(session),
            )
            .await
            .unwrap();

        TaskUpdateTool::new()
            .execute(
                json!({
                    "task_id": id,
                    "status": "in_progress",
                    "addBlocks": ["2"]
                }),
                ctx(session),
            )
            .await
            .unwrap();

        let listed = TaskListTool::new()
            .execute(json!({}), ctx(session))
            .await
            .unwrap();
        assert!(listed.output.contains("#1 [in_progress] Write tests"));
        assert!(listed.output.contains("#2 [pending] Wire registry"));
        assert!(listed.output.contains("blocked by #1"));

        let got = TaskGetTool::new()
            .execute(json!({ "id": "2" }), ctx(session))
            .await
            .unwrap();
        assert!(got.output.contains("Blocked by: #1"));
        assert!(got.output.contains("Wire registry"));

        match previous {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
    }

    #[test]
    fn tool_names_match_claude() {
        assert_eq!(TaskCreateTool::new().name(), "TaskCreate");
        assert_eq!(TaskGetTool::new().name(), "TaskGet");
        assert_eq!(TaskListTool::new().name(), "TaskList");
        assert_eq!(TaskUpdateTool::new().name(), "TaskUpdate");
    }
}
