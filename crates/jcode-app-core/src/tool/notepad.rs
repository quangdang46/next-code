use super::{Tool, ToolContext, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

/// A notepad tool that reads or writes a single tier.
pub struct NotepadTool {
    name: &'static str,
    description: &'static str,
    tier: crate::notepad::NotepadTier,
    is_write: bool,
}

impl NotepadTool {
    fn notepad_from_ctx(ctx: &ToolContext) -> Option<crate::notepad::Notepad> {
        let cfg = &crate::config::config().notepad;
        crate::notepad::Notepad::new(
            ctx.working_dir.as_deref(),
            cfg,
        )
    }

    // -- Priority tier -------------------------------------------------------

    pub fn read_priority() -> Self {
        Self {
            name: "read_priority",
            description: "Read the priority notes — critical context that is always injected into the system prompt. This tier is intended for the model to store short notes that must survive compaction and be visible every turn (current goal, key constraints, pinned decisions).",
            tier: crate::notepad::NotepadTier::Priority,
            is_write: false,
        }
    }

    pub fn write_priority() -> Self {
        Self {
            name: "write_priority",
            description: "Overwrite the priority notes with the given content. The priority tier is automatically injected into the system prompt at the start of every turn, so it survives context compaction. Use this to persist critical context the model must not forget (current goal, blocking decisions, key constraints).",
            tier: crate::notepad::NotepadTier::Priority,
            is_write: true,
        }
    }

    // -- Working tier --------------------------------------------------------

    pub fn read_working() -> Self {
        Self {
            name: "read_working",
            description: "Read the working-notes scratchpad for the current session. Content persists across turns but is not injected automatically.",
            tier: crate::notepad::NotepadTier::Working,
            is_write: false,
        }
    }

    pub fn write_working() -> Self {
        Self {
            name: "write_working",
            description: "Overwrite the working-notes scratchpad with the given content. Use this as a session-scoped scratchpad (context summary, partial plans, notes to self).",
            tier: crate::notepad::NotepadTier::Working,
            is_write: true,
        }
    }

    // -- Manual tier ---------------------------------------------------------

    pub fn read_manual() -> Self {
        Self {
            name: "read_manual",
            description: "Read the manual notes — user-authored notes that persist across sessions. Content is not injected automatically.",
            tier: crate::notepad::NotepadTier::Manual,
            is_write: false,
        }
    }

    pub fn write_manual() -> Self {
        Self {
            name: "write_manual",
            description: "Overwrite the manual notes with the given content. Use this to persist user-authored notes across sessions.",
            tier: crate::notepad::NotepadTier::Manual,
            is_write: true,
        }
    }
}

#[async_trait]
impl Tool for NotepadTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn parameters_schema(&self) -> Value {
        if self.is_write {
            json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The content to write to the notepad tier."
                    }
                },
                "required": ["content"]
            })
        } else {
            json!({
                "type": "object",
                "properties": {}
            })
        }
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let Some(notepad) = Self::notepad_from_ctx(&ctx) else {
            return Ok(ToolOutput::new(
                "Notepad is disabled. Enable it in your config (notepad.enabled: true)."
                    .to_string(),
            ));
        };

        if self.is_write {
            let content = input
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("");
            notepad.write(self.tier, content)?;
            Ok(ToolOutput::new(format!(
                "Wrote {} notepad ({} chars).",
                self.tier.as_str(),
                content.len()
            )))
        } else {
            let content = notepad.read(self.tier);
            if content.is_empty() {
                Ok(ToolOutput::new(format!(
                    "{} notepad is empty.",
                    capitalize(self.tier.as_str())
                )))
            } else {
                Ok(ToolOutput::new(format!(
                    "# {} Notepad\n\n{}",
                    capitalize(self.tier.as_str()),
                    content
                )))
            }
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ---------------------------------------------------------------------------
// NotepadPruneTool — clear the working tier (session-scoped scratchpad)
// ---------------------------------------------------------------------------

/// Tool that prunes (clears) the working tier.
pub struct NotepadPruneTool;

impl NotepadPruneTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotepadPruneTool {
    fn name(&self) -> &str {
        "prune"
    }

    fn description(&self) -> &str {
        "Clear the working-notes tier (session-scoped scratchpad). Use this between sessions or when the working notes are no longer relevant."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }

    async fn execute(&self, _input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let cfg = &crate::config::config().notepad;
        let Some(notepad) = crate::notepad::Notepad::new(ctx.working_dir.as_deref(), cfg) else {
            return Ok(ToolOutput::new(
                "Notepad is disabled. Enable it in your config (notepad.enabled: true)."
                    .to_string(),
            ));
        };
        notepad.prune()?;
        Ok(ToolOutput::new("Working notepad cleared.".to_string()))
    }
}

// ---------------------------------------------------------------------------
// NotepadStatsTool — report file statistics for all tiers
// ---------------------------------------------------------------------------

/// Tool that reports file statistics for all three notepad tiers.
pub struct NotepadStatsTool;

impl NotepadStatsTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotepadStatsTool {
    fn name(&self) -> &str {
        "stats"
    }

    fn description(&self) -> &str {
        "Show notepad file statistics for all three tiers (priority, working, manual) — file sizes and whether each tier has content."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }

    async fn execute(&self, _input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let cfg = &crate::config::config().notepad;
        let Some(notepad) = crate::notepad::Notepad::new(ctx.working_dir.as_deref(), cfg) else {
            return Ok(ToolOutput::new(
                "Notepad is disabled. Enable it in your config (notepad.enabled: true)."
                    .to_string(),
            ));
        };
        let stats = notepad.stats();
        let mut lines = vec![format!(
            "Total size: {} bytes",
            stats.total_size_bytes
        )];
        for t in &stats.tiers {
            lines.push(format!(
                "- {}: {} bytes {}",
                t.name,
                t.file_size_bytes,
                if t.has_content { "(has content)" } else { "(empty)" }
            ));
        }
        Ok(ToolOutput::new(lines.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_notepad() -> (tempfile::TempDir, crate::notepad::Notepad) {
        let dir = tempfile::tempdir().unwrap();
        let config = crate::notepad::NotepadConfig {
            enabled: true,
            dir: ".notepad-test".to_string(),
            max_chars_per_tier: 4096,
        };
        let np = crate::notepad::Notepad::new(Some(dir.path()), &config).unwrap();
        (dir, np)
    }

    fn test_ctx(dir: &std::path::Path) -> ToolContext {
        ToolContext {
            session_id: "test".to_string(),
            message_id: "msg1".to_string(),
            tool_call_id: "tc1".to_string(),
            working_dir: Some(dir.to_path_buf()),
            stdin_request_tx: None,
            graceful_shutdown_signal: None,
            execution_mode: crate::tool::ToolExecutionMode::Direct,
        }
    }

    #[tokio::test]
    async fn test_read_priority_tool_empty() {
        let (dir, np) = temp_notepad();
        np.clear(crate::notepad::NotepadTier::Priority).unwrap();

        let tool = NotepadTool::read_priority();
        let output = tool.execute(json!({}), test_ctx(dir.path())).await.unwrap();
        assert!(output.output.contains("Priority notepad is empty"));
    }

    #[tokio::test]
    async fn test_write_then_read_priority() {
        let (dir, np) = temp_notepad();
        np.clear(crate::notepad::NotepadTier::Priority).unwrap();

        let write_tool = NotepadTool::write_priority();
        let output = write_tool
            .execute(json!({"content": "test content"}), test_ctx(dir.path()))
            .await
            .unwrap();
        assert!(output.output.contains("Wrote priority notepad"));

        let read_tool = NotepadTool::read_priority();
        let output = read_tool.execute(json!({}), test_ctx(dir.path())).await.unwrap();
        assert!(output.output.contains("test content"));
    }

    #[tokio::test]
    async fn test_working_and_manual_tiers() {
        let (dir, np) = temp_notepad();
        np.clear(crate::notepad::NotepadTier::Working).unwrap();
        np.clear(crate::notepad::NotepadTier::Manual).unwrap();

        for (write_tool, content, tier_name) in [
            (NotepadTool::write_working(), "working data", "working"),
            (NotepadTool::write_manual(), "manual data", "manual"),
        ] {
            let output = write_tool
                .execute(json!({"content": content}), test_ctx(dir.path()))
                .await
                .unwrap();
            assert!(output.output.contains(&format!("Wrote {}", tier_name)));

            let read_tool = match tier_name {
                "working" => NotepadTool::read_working(),
                _ => NotepadTool::read_manual(),
            };
            let output = read_tool.execute(json!({}), test_ctx(dir.path())).await.unwrap();
            assert!(output.output.contains(content));
        }
    }

    #[tokio::test]
    async fn test_disabled_notepad_returns_empty_read() {
        let dir = tempfile::tempdir().unwrap();
        let tool = NotepadTool::read_priority();
        let output = tool.execute(json!({}), test_ctx(dir.path())).await.unwrap();
        // Should be empty since we never wrote anything
        assert!(output.output.contains("Priority notepad is empty"));
    }
}
