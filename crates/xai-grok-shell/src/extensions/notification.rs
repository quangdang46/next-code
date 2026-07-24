//! Façade stub of upstream `xai-grok-shell::extensions::notification` — the
//! ACP session-notification DTOs the future pager renders (hook runs,
//! memory files, image compression stats, prompt usage, goal-tracker
//! verdicts). Field shapes are copied from upstream where cheap; `IndexMap`
//! (ordering-sensitive `modelUsage` map) is simplified to `HashMap` since
//! this façade layer has no serialization-order consumer yet.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum HookRunStatusDto {
    Success {
        elapsed_ms: u64,
    },
    Skipped,
    Failed {
        error: String,
        elapsed_ms: u64,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        blocked: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookRunEntryDto {
    pub name: String,
    pub status: HookRunStatusDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MemoryFileInfo {
    pub path: String,
    pub source: String,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_epoch_secs: Option<u64>,
    /// Claude memdir taxonomy (`user` / `feedback` / `project` / `reference`)
    /// or notepad tier (`priority` / `working` / `manual`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ImageCompressedEntry {
    pub index: usize,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub original_width: u32,
    pub original_height: u32,
    pub compressed_width: u32,
    pub compressed_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalClassifierVerdict {
    Achieved,
    NotAchieved,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PromptUsageModel {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cached_read_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
    #[serde(default)]
    pub model_calls: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PromptUsage {
    #[serde(flatten)]
    pub totals: PromptUsageModel,
    #[serde(default, rename = "modelUsage")]
    pub model_usage: HashMap<String, PromptUsageModel>,
    #[serde(default, rename = "numTurns")]
    pub num_turns: u64,
    #[serde(default, rename = "usageIsIncomplete")]
    pub usage_is_incomplete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RetryState {
    Retrying {
        attempt: u32,
        max_retries: u32,
        reason: String,
    },
    Exhausted {
        attempts: u32,
        reason: String,
        #[serde(default)]
        is_rate_limited: bool,
    },
    Failed {
        error_type: String,
        message: String,
    },
}

pub fn is_reauthable_failure(_error_type: Option<&str>, _message: &str) -> bool {
    false
}

pub fn attach_result_usage_fail_closed(
    _result: &mut serde_json::Value,
    _usage: &serde_json::Value,
) {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNotification {
    pub session_id: agent_client_protocol::SessionId,
    pub update: SessionUpdate,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "sessionUpdate")]
pub enum SessionUpdate {
    RetryState(RetryState),
    AutoCompactStarted {
        tokens_used: u64,
        context_window: u64,
        percentage: u8,
        reason: String,
    },
    AutoCompactCompleted {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tokens_before: Option<u64>,
        tokens_after: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        elapsed_ms: Option<i64>,
        summary_preview: Option<String>,
    },
    AutoCompactFailed { error: String },
    AutoCompactCancelled { reason: String },
    MemoryFlushStarted,
    MemoryFlushCompleted {
        result: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    MemoryDreamCompleted {
        result: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    MemorySessionSaved { path: String },
    MemoryFiles { files: Vec<MemoryFileInfo> },
    ModelAutoSwitched {
        previous_model_id: String,
        new_model_id: String,
        reason: String,
    },
    ModelChanged {
        model_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning_effort: Option<String>,
    },
    HookExecution {
        event_name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prompt_id: Option<String>,
        runs: Vec<HookRunEntryDto>,
    },
    HooksChanged {
        hooks: Vec<xai_hooks_plugins_types::HookInfo>,
        project_trusted: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        load_errors: Vec<String>,
    },
    PluginsChanged {
        plugins: Vec<xai_hooks_plugins_types::PluginInfo>,
    },
    HookAnnotation {
        #[serde(default)]
        message: String,
    },
    ScheduledTaskCreated {
        task_id: String,
        prompt: String,
        #[serde(default)]
        human_schedule: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        next_fire_at: Option<String>,
    },
    ScheduledTaskDeleted { task_id: String },
    ScheduledTaskFired {
        task_id: String,
        prompt: String,
        human_schedule: String,
        next_fire_at: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subagent_id: Option<String>,
    },
    MonitorEvent {
        task_id: String,
        description: String,
        event_text: String,
    },
    TaskBackgrounded {
        tool_call_id: String,
        task_id: String,
        command: String,
        cwd: String,
        output_file: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        monitor_description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    TaskCompleted {
        task_snapshot: xai_grok_tools::types::TaskSnapshot,
        #[serde(default)]
        will_wake: bool,
    },
    SubagentSpawned {
        subagent_id: String,
        parent_session_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_prompt_id: Option<String>,
        child_session_id: String,
        subagent_type: String,
        description: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effective_context_source: Option<String>,
        #[serde(default)]
        context_normalized: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        capability_mode: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        persona: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resumed_from: Option<String>,
    },
    SubagentFinished {
        subagent_id: String,
        child_session_id: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        tool_calls: u32,
        turns: u32,
        duration_ms: u64,
        #[serde(default)]
        tokens_used: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        #[serde(default)]
        will_wake: bool,
    },
    SubagentProgress {
        #[serde(default)]
        subagent_id: String,
        #[serde(default)]
        parent_session_id: String,
        child_session_id: String,
        duration_ms: u64,
        turn_count: u32,
        tool_call_count: u32,
        tokens_used: u64,
        context_window_tokens: u64,
        context_usage_pct: u8,
        #[serde(default)]
        tools_used: Vec<String>,
        error_count: u32,
    },
    TurnCompleted {
        prompt_id: String,
        stop_reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent_result: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<PromptUsage>,
    },
    SessionSummaryGenerated { session_summary: String },
    SessionRecap {
        summary: String,
        #[serde(default)]
        auto: bool,
    },
    SessionRecapUnavailable,
    GoalUpdated {
        goal_id: String,
        objective: String,
        status: String,
        phase: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        token_budget: Option<i64>,
        #[serde(default)]
        tokens_used: i64,
        elapsed_ms: u64,
        #[serde(default)]
        total_deliverables: u32,
        #[serde(default)]
        completed_deliverables: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        current_deliverable_id: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        current_deliverable_title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        current_subagent_role: Option<String>,
        #[serde(default)]
        total_worker_rounds: u32,
        #[serde(default)]
        total_verify_rounds: u32,
        #[serde(default)]
        token_baseline: i64,
        #[serde(default)]
        finished_subagent_tokens: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        live_subagent_tokens: Option<u64>,
        #[serde(default)]
        live_tokens_by_model: Vec<(String, u64)>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        live_context_pct: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        live_turn_count: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        live_tool_call_count: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_event: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_event_detail: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_event_timestamp: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pause_message: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        classifier_runs_attempted: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        classifier_max_runs: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_classifier_verdict: Option<GoalClassifierVerdict>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_classifier_details_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        verifying_completion: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        planning: Option<bool>,
    },
    InteractionResolved {
        tool_call_id: String,
    },
    ImageCompressed {
        #[serde(default)]
        images: Vec<ImageCompressedEntry>,
        #[serde(default)]
        message: String,
    },
    ImageDropped {
        #[serde(default)]
        notes: Vec<String>,
    },
    #[serde(other)]
    Unknown,
}
