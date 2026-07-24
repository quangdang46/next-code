//! Façade stub of upstream `xai-grok-shell::util::config` — the single
//! highest-frequency pager import prefix (135 hits across the upstream
//! pager crate). `RemoteSettings` is copied verbatim from
//! `xai-grok-config-types::RemoteSettings` (upstream `crates/codegen/
//! xai-grok-config-types/src/lib.rs`) since the pager consumes it as a
//! flat Default-derived DTO; the tiny PR3 `xai-grok-config` crate stub
//! (`{ folder_trust_enabled }`) is intentionally NOT touched or re-used
//! here — these are separate crates serving different layers.
//!
//! Disk read/write (`load`/`set_*`) is out of scope for this compile-stub
//! layer — no-op placeholders only.

use serde::{Deserialize, Serialize};
pub use xai_grok_shared::ui_config::DisplayRefreshSettings;

/// Simplified stand-in for upstream `DoomLoopRecoverySettings` (dropped:
/// upstream's tolerant-deserialize helpers below — not needed for a
/// Default-derived compile stub).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DoomLoopRecoverySettings {
    #[serde(default)]
    pub enabled: Option<bool>,
}

/// Simplified stand-in for upstream `GoalRoleModel` (a model-slug override
/// per goal-orchestrator role).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GoalRoleModel {
    #[serde(default)]
    pub model: Option<String>,
}

/// Re-export announcements crate type so pager `filter_expired` type-checks.
pub use xai_grok_announcements::RemoteAnnouncement;

/// Simplified stand-in for upstream `CampaignOverride`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CampaignOverride {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub enabled: bool,
}

/// Remote settings fetched from cli-chat-proxy `GET /v1/settings`.
///
/// All fields are `Option` with `#[serde(default)]` so that:
/// - Missing fields from old servers are gracefully ignored
/// - New fields added in the future don't break existing clients
/// - Callers can distinguish "server said false" from "server didn't say"
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RemoteSettings {
    /// When `Some(true)`, the server recommends enabling leader mode.
    /// Used as a fallback when the user hasn't set `[cli] use_leader` locally.
    #[serde(default)]
    pub leader_mode: Option<bool>,
    #[serde(default)]
    pub max_upload_file_bytes: Option<u64>,
    #[serde(default)]
    pub max_upload_untracked_bytes: Option<u64>,
    /// When `Some(true)`, capture workspace files for non-git project dirs (client default: off).
    #[serde(default)]
    pub non_git_workspace_capture: Option<bool>,
    #[serde(default)]
    pub login_shell_capture: Option<bool>,
    /// When `Some(false)`, scheduled task fires run as main-conversation
    /// turns instead of background subagents.
    #[serde(default)]
    pub scheduler_background_loops: Option<bool>,
    /// Release channel: `"stable"` or `"alpha"`.
    /// Fallback when no local `[cli] channel` or `--alpha`/`--stable` flag is set.
    #[serde(default)]
    pub release_channel: Option<String>,
    /// When `Some(true)`, enable LOC attribution tracking for this session.
    #[serde(default)]
    pub loc_tracking: Option<bool>,
    /// Enable the experimental memory system remotely.
    #[serde(default)]
    pub memory_enabled: Option<bool>,
    #[serde(default)]
    pub memory_search_max_results: Option<u32>,
    #[serde(default)]
    pub memory_search_min_score: Option<f32>,
    #[serde(default)]
    pub memory_initial_injection_enabled: Option<bool>,
    #[serde(default)]
    pub memory_initial_injection_min_score: Option<f32>,
    #[serde(default)]
    pub memory_embedding_model: Option<String>,
    #[serde(default)]
    pub memory_embedding_dimensions: Option<u32>,
    #[serde(default)]
    pub pruning_enabled: Option<bool>,
    #[serde(default)]
    pub pruning_keep_last_n_turns: Option<u32>,
    #[serde(default)]
    pub pruning_soft_trim_threshold: Option<u32>,
    #[serde(default)]
    pub flush_enabled: Option<bool>,
    #[serde(default)]
    pub flush_soft_threshold_tokens: Option<u64>,
    #[serde(default)]
    pub flush_idle_timeout_secs: Option<u64>,
    #[serde(default)]
    pub flush_semantic_dedup_threshold: Option<f64>,
    #[serde(default)]
    pub memory_temporal_decay_enabled: Option<bool>,
    #[serde(default)]
    pub memory_temporal_decay_half_life_days: Option<f64>,
    #[serde(default)]
    pub memory_mmr_enabled: Option<bool>,
    #[serde(default)]
    pub memory_mmr_lambda: Option<f64>,
    #[serde(default)]
    pub memory_watcher_enabled: Option<bool>,
    #[serde(default)]
    pub dream_enabled: Option<bool>,
    #[serde(default)]
    pub dream_min_hours: Option<u64>,
    #[serde(default)]
    pub dream_min_sessions: Option<u64>,
    #[serde(default)]
    pub dream_check_interval_secs: Option<u64>,
    /// Cadence (seconds) of the pager's free→paid subscription watch.
    /// `0` disables it; the pager clamps and defaults (see its
    /// `app::subscription` module). Forwarded from the `grok_build_settings`
    /// remote settings flag via the CCP `/settings` flatten catch-all.
    #[serde(default)]
    pub subscription_watch_interval_secs: Option<u64>,
    #[serde(default)]
    pub writeback_enabled: Option<bool>,
    /// OAuth2 provider issuer URL (e.g., "https://auth.x.ai"). When present
    /// together with `oauth2_client_id`, the client uses OAuth2 authorization code
    /// flow. Controlled via remote settings for gradual rollout.
    #[serde(default)]
    pub oauth2_issuer: Option<String>,
    /// OAuth2 client_id for the CLI. Paired with `oauth2_issuer`.
    #[serde(default)]
    pub oauth2_client_id: Option<String>,
    /// When `Some(true)`, enable grok's default OAuth2 (xAI auth.x.ai).
    /// Enterprise OIDC (user's own IdP via `oidc` config) always wins.
    /// Controlled via remote settings; `--oauth` CLI flag overrides.
    #[serde(default)]
    pub grok_oauth_enabled: Option<bool>,
    #[serde(default)]
    pub lsp_tools_enabled: Option<bool>,
    /// Folder-trust gate kill-switch / remote default. Gates whether repo-local
    /// MCP/LSP servers (commands sourced from working-tree config files) require
    /// a per-folder trust decision before they are spawned. `Some(true)`
    /// enables, `Some(false)` is a kill-switch, `None` falls back to the client
    /// default (on). Sits below env `GROK_FOLDER_TRUST`, user
    /// `[folder_trust] enabled`, and managed config in the resolver chain. See
    /// `agent::folder_trust::feature_enabled`.
    #[serde(default)]
    pub folder_trust_enabled: Option<bool>,
    #[serde(default)]
    pub write_file_enabled: Option<bool>,
    /// File toolset: `"standard"` or `"hashline"`.
    /// Server-side default; local `[toolset] file_toolset` in config.toml
    /// takes precedence when set.
    #[serde(default)]
    pub file_toolset: Option<String>,
    /// Per-chunk idle timeout in seconds for inference streaming.
    /// Fallback when no per-model `inference_idle_timeout_secs` is set in config.toml.
    #[serde(default)]
    pub inference_idle_timeout_secs: Option<u64>,
    /// Global default MCP startup-handshake timeout (seconds); lowest-precedence
    /// fallback (per-server config, env, and requirements/managed override it).
    #[serde(default)]
    pub mcp_startup_timeout_secs: Option<u64>,
    /// remote settings `grok_build_settings.max_mcp_output_bytes` — global default
    /// MCP tool-result inline cap (bytes). Overridden by requirements, env,
    /// and `config.toml [mcp] max_output_bytes`. Built-in default 20_000.
    #[serde(default)]
    pub max_mcp_output_bytes: Option<u64>,
    /// When `Some(true)`, enable session registry hooks (register, update, finalize, memory upload).
    /// When absent or `Some(false)`, all hooks are disabled (default: disabled).
    #[serde(default)]
    pub session_registry_enabled: Option<bool>,
    /// The remote settings `doom_loop_recovery` JSON object; see
    /// [`DoomLoopRecoverySettings`]. Absent ⇒ every knob falls through to
    /// TOML/defaults; a partial object falls through per-field.
    #[serde(default)]
    pub doom_loop_recovery: Option<DoomLoopRecoverySettings>,
    /// Enable/disable the runtime turn-end TodoGate remotely.
    /// Precedence: CLI `--todo-gate` > this field > built-in default (`false`).
    /// The gate ships disabled; set this to `Some(true)` (via the
    /// `grok_build_settings` remote settings key) to enable it. See
    /// `session::acp_session::resolve_reminder_policy`.
    #[serde(default)]
    pub todo_gate_enabled: Option<bool>,
    /// Hard cap on TodoGate fires per user prompt.
    /// Precedence: this field > built-in default (`DEFAULT_TODO_GATE_MAX_FIRES`).
    /// No CLI override. See `session::acp_session::resolve_reminder_policy`.
    #[serde(default)]
    pub todo_gate_max_fires_per_prompt: Option<u32>,
    #[serde(default)]
    pub auto_wake_enabled: Option<bool>,
    #[serde(default)]
    pub cursor_skills_enabled: Option<bool>,
    #[serde(default)]
    pub cursor_rules_enabled: Option<bool>,
    #[serde(default)]
    pub cursor_agents_enabled: Option<bool>,
    #[serde(default)]
    pub claude_skills_enabled: Option<bool>,
    #[serde(default)]
    pub claude_rules_enabled: Option<bool>,
    #[serde(default)]
    pub claude_agents_enabled: Option<bool>,
    #[serde(default)]
    pub cursor_mcps_enabled: Option<bool>,
    #[serde(default)]
    pub cursor_hooks_enabled: Option<bool>,
    #[serde(default)]
    pub claude_mcps_enabled: Option<bool>,
    #[serde(default)]
    pub claude_hooks_enabled: Option<bool>,
    #[serde(default)]
    pub cursor_sessions_enabled: Option<bool>,
    #[serde(default)]
    pub claude_sessions_enabled: Option<bool>,
    #[serde(default)]
    pub codex_sessions_enabled: Option<bool>,
    /// When `Some(true)`, enable goal mode remotely.
    /// When `Some(false)`, force-disable it (kill-switch).
    /// Absent ⇒ client default (enabled).
    #[serde(default)]
    pub goal_enabled: Option<bool>,
    /// When `Some(true)`, enable the goal-completion classifier remotely.
    /// When `Some(false)`, force-disable it.
    /// Absent ⇒ default tracks goal mode (enabled iff goal mode is on).
    #[serde(default)]
    pub goal_classifier_enabled: Option<bool>,
    /// When `Some(true)`, enable the goal planner remotely.
    /// When `Some(false)`, force-disable it.
    /// Absent ⇒ default tracks goal mode (enabled iff goal mode is on).
    #[serde(default)]
    pub goal_planner_enabled: Option<bool>,
    /// When `Some(true)`, enable the goal summarizer remotely (the one-shot
    /// closing "what was accomplished" summary on a verified achievement).
    /// When `Some(false)`, force-disable it (kill-switch).
    /// Absent ⇒ default tracks goal mode (enabled iff goal mode is on).
    #[serde(default)]
    pub goal_summary_enabled: Option<bool>,
    /// Number of adversarial skeptics spawned per goal-verification
    /// attempt (step ② of the staged gate). Clamped to `1..=5` at the
    /// resolver. Absent ⇒ harness default of
    /// `goal_classifier::GOAL_VERIFIER_SKEPTIC_COUNT` (3 today).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_verifier_count: Option<u32>,
    /// Maximum per-goal classifier runs before the goal auto-pauses
    /// (BackOff). Clamped to `1..=10` at the resolver. Absent ⇒ harness
    /// default of `goal_classifier::GOAL_CLASSIFIER_MAX_RUNS_DEFAULT`
    /// (3 today).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_classifier_max_runs: Option<u32>,
    /// Fire the stall-triggered strategist every N consecutive
    /// `NotAchieved` verifications. Clamped to `>= 1` at the resolver.
    /// Absent ⇒ default of `max(1, goal_classifier_max_runs / 2)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_strategist_every: Option<u32>,
    /// Planner role model+toolset. Absent ⇒ inherit current model. A
    /// present-but-malformed value is tolerantly dropped to `None` (not a
    /// hard parse error) so it cannot nuke the whole `RemoteSettings`
    /// payload (see [`deserialize_tolerant_goal_role_model`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_planner_model: Option<GoalRoleModel>,
    /// Strategist role model+toolset. Absent ⇒ inherit current model. A
    /// present-but-malformed value is tolerantly dropped to `None`
    /// (see [`deserialize_tolerant_goal_role_model`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_strategist_model: Option<GoalRoleModel>,
    /// Ordered skeptic pool. `pool[0]` = skeptic-0's model; skeptics
    /// `1..N` are assigned round-robin over the pool. Empty/absent ⇒
    /// inherit the current model. A single malformed pool entry is
    /// dropped rather than discarding the whole pool (see
    /// [`deserialize_tolerant_goal_skeptic_models`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goal_skeptic_models: Vec<GoalRoleModel>,
    /// Remote fallback for managed MCP connector fetching.
    #[serde(default)]
    pub managed_mcps_enabled: Option<bool>,
    #[serde(default)]
    pub managed_mcp_gateway_tools_enabled: Option<bool>,
    /// Fleet kill switch for the **external OTEL** stream (customer
    /// collectors). Restrictive-only by construction: there is deliberately
    /// no `external_otel_enabled` remote field — remote settings are fetched
    /// per-run and never persisted, so a remote "enable" could never reach
    /// init; org-wide enable ships via managed config instead. Applied
    /// in-process (tighten-only) via
    /// `xai_grok_telemetry::external::apply_remote_policy`.
    #[serde(default)]
    pub external_otel_disabled: Option<bool>,
    /// Force the external stream's content gates (`OTEL_LOG_USER_PROMPTS`,
    /// `OTEL_LOG_TOOL_DETAILS`) off regardless of local env/config.
    /// Tighten-only, like `external_otel_disabled`.
    #[serde(default)]
    pub external_otel_content_gates_locked: Option<bool>,
    #[serde(default)]
    pub telemetry_enabled: Option<bool>,
    /// Telemetry mode override (string): `"session-metrics"`, `"full"`, `"off"`.
    /// Takes precedence over `telemetry_enabled` (bool) when present.
    #[serde(default)]
    pub telemetry_mode: Option<String>,
    #[serde(default)]
    pub trace_upload_enabled: Option<bool>,
    /// Enable user-facing feedback (heuristic popups, `/feedback` command).
    /// Session analytics (signal sync, turn deltas) are gated separately
    /// by `telemetry_enabled`.
    #[serde(default)]
    pub feedback_enabled: Option<bool>,
    /// Two-pass (prefire) compaction. When approaching the auto-compact
    /// threshold the shell speculatively summarizes the history prefix in the
    /// background (pass 1 → NOTE₁); at compaction it summarizes NOTE₁ + the
    /// recent tail (pass 2 → final summary), keeping summarizer latency off the
    /// critical path. `Some(true)` enables (remote rollout), `Some(false)` forces
    /// off, `None` falls back to `[features] two_pass_compaction` /
    /// `GROK_TWO_PASS_COMPACTION` / default (off).
    #[serde(default)]
    pub two_pass_compaction_enabled: Option<bool>,
    /// Dynamic tip list from remote settings. When present with non-empty entries,
    /// one tip is shown at startup (rotated daily by UTC day).
    /// `None` or `[]` = no tips shown.
    #[serde(default)]
    pub tips: Option<Vec<String>>,
    /// When present, controls the non-Git-repo warning at session start.
    /// Controlled via remote settings (`non_git_warning` in `grok_build_settings`).
    /// Takes precedence over `[features] non_git_warning` in config.toml:
    /// `Some(true)` enables, `Some(false)` acts as a kill-switch, `None` falls back to local config.
    #[serde(default)]
    pub non_git_warning: Option<bool>,
    /// remote settings gate for first-run auto-registration of the official xAI
    /// marketplace source. `Some(true)` enables, `Some(false)` is a kill-switch,
    /// `None` falls back to env/default (off).
    #[serde(default)]
    pub official_marketplace_auto_register: Option<bool>,
    /// remote settings gate for the inline plugin-install CTA (keyword-matched
    /// marketplace upsell above the prompt). `Some(true)` enables, `Some(false)`
    /// is a kill-switch, `None` falls back to env/default (off).
    #[serde(default)]
    pub plugin_cta: Option<bool>,
    /// Remote announcements list from proxy. Malformed items are skipped entirely.
    /// `None` or `[]` = no announcements to display.
    #[serde(default)]
    pub announcements: Option<Vec<RemoteAnnouncement>>,
    #[serde(default)]
    pub web_search_model: Option<String>,
    #[serde(default)]
    pub session_summary_model: Option<String>,
    #[serde(default)]
    pub image_description_model: Option<String>,
    /// Server-side pin for the next-prompt suggestion model (tab-autocomplete
    /// ghost text), from the `grok_build_settings` remote settings flag. Sits below
    /// env (`GROK_PROMPT_SUGGESTIONS_MODEL`) and `[models] prompt_suggestion`
    /// in config.toml, above the client hint and the built-in
    /// `grok-build-0.1` default. The effective model is catalog-guarded: when
    /// it is not in the shell's model catalog the suggestion request is
    /// skipped entirely (never the session model). See
    /// `ModelOverrideConfig::resolve` and `handle_suggest_prompt`.
    #[serde(default)]
    pub prompt_suggestion_model: Option<String>,
    /// Server-recommended default model ID for new sessions.
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub campaigns: Vec<CampaignOverride>,
    /// When `Some(true)`, foreground commands that hit the default timeout are
    /// auto-backgrounded instead of killed. Fallback when no local
    /// `[toolset.bash] auto_background_on_timeout` is set in config.toml.
    #[serde(default)]
    pub auto_background_on_timeout: Option<bool>,
    /// When `Some(false)`, foreground commands containing a background `&`
    /// operator are rejected. Fallback when no local `[toolset.bash]
    /// allow_background_operator` is set; absent → client default (allow).
    #[serde(default)]
    pub allow_background_operator: Option<bool>,
    /// remote settings fallback for `[toolset.ask_user_question] timeout_enabled`.
    /// When `Some(false)`, questionnaires wait forever unless a higher tier
    /// (requirements / env / user / managed config) sets otherwise.
    #[serde(default)]
    pub ask_user_question_timeout_enabled: Option<bool>,
    /// remote settings fallback for `[toolset.ask_user_question] timeout_secs`
    /// (positive seconds). Absent → client default (1800 / 30 minutes).
    #[serde(default)]
    pub ask_user_question_timeout_secs: Option<u64>,
    /// When `Some(true)`, a completed subagent's isolated worktree is snapshotted
    /// into a durable git ref and its directory deleted (resume rehydrates from
    /// the ref). Fallback when no local `[features] subagent_worktree_snapshot`
    /// is set in config.toml. Absent → default (**disabled** — ships dark).
    #[serde(default)]
    pub subagent_worktree_snapshot_enabled: Option<bool>,
    /// When `Some(true)`, enable the `image_gen` tool for session-based auth users.
    /// When `Some(false)` or absent, the tool is hidden regardless of credentials.
    #[serde(default)]
    pub image_gen_enabled: Option<bool>,
    /// remote settings flag: optional Imagine model override for `image_gen`.
    /// When present and non-empty, `image_gen` uses this model slug
    /// (e.g. `grok-imagine-image`) instead of the default quality model
    /// (`grok-imagine-image-quality`). Absent/empty → default model.
    #[serde(default)]
    pub image_gen_model_override: Option<String>,
    /// When `Some(true)`, enable the `video_gen` tool for session-based auth users.
    /// When `Some(false)` or absent, the tool is hidden regardless of credentials.
    #[serde(default)]
    pub video_gen_enabled: Option<bool>,
    /// When `Some(true)`, enable the process-wide image normalize cache that
    /// amortises decode + integrity-check + re-encode work across SessionActors.
    /// Default: disabled. See `session::normalize_cache`.
    #[serde(default)]
    pub image_normalize_cache_enabled: Option<bool>,
    /// When `Some(true)`, enrich path-not-found errors with CWD reminders,
    /// "did you mean?" corrections, and similar-name suggestions.
    /// When `Some(false)` or absent, error messages are unchanged.
    #[serde(default)]
    pub path_not_found_hints: Option<bool>,
    /// Remote enable tier for the per-tip contextual hints. Each field is a
    /// soft default for one tip: `Some(false)` disables, `Some(true)` enables,
    /// absent/null ⇒ client default (on). User config beats this tier.
    #[serde(default)]
    pub contextual_hints: Option<ContextualHintsRemote>,
    /// Server-recommended worktree creation type. Fallback when no local
    /// `[cli] worktree_type` is set in config.toml.
    #[serde(default)]
    pub worktree_type: Option<String>,
    /// Server-recommended default for `restore_code` in worktree resume.
    /// Fallback when no local `[cli] restore_code` is set in config.toml.
    #[serde(default)]
    pub restore_code: Option<bool>,
    /// When `Some(true)`, Ctrl+C before the first server activity rewinds
    /// the prompt back into the input box instead of cancelling the turn.
    #[serde(default)]
    pub cancel_rewind_enabled: Option<bool>,
    /// Enables the session recap feature (`/recap` + automatic return-from-away).
    /// Optional remote kill-switch; shell defaults ON when unset (set `false` to disable).
    #[serde(default)]
    pub session_recap: Option<bool>,
    /// Enables the `ask_user_question` tool. Optional remote kill-switch:
    /// `Some(false)` strips the tool; `Some(true)` or absent → the shell
    /// default (ON). Feature-flagged via remote settings.
    #[serde(default)]
    pub ask_user_question_enabled: Option<bool>,
    /// When `Some(true)`, enable the `web_fetch` tool.
    /// When `Some(false)` or absent, the tool is not registered.
    /// Feature-flagged via remote settings for gradual rollout.
    #[serde(default)]
    pub web_fetch_enabled: Option<bool>,
    /// Egress proxy endpoint for the web_fetch tool.
    /// Fallback when no local `[toolset.web_fetch] proxy_endpoint` is set.
    #[serde(default)]
    pub web_fetch_proxy: Option<String>,
    /// Domain allowlist for the web_fetch tool.
    /// Fallback when no local `[toolset.web_fetch] allowed_domains` is set.
    #[serde(default)]
    pub web_fetch_allowed_domains: Option<Vec<String>>,
    /// When `Some(false)`, hide the resolved model ID in /session-info.
    #[serde(default)]
    pub show_resolved_model: Option<bool>,
    /// When `Some(true)`, enable session sharing.
    /// When `Some(false)` or absent, sharing is disabled.
    #[serde(default)]
    pub sharing_enabled: Option<bool>,
    /// Voice mode (STT dictation). Client default is **on** when absent.
    /// `Some(false)` is a remote kill switch; `Some(true)` forces on.
    /// Overridable locally via `GROK_VOICE_MODE`. Free-tier SuperGrok upsell
    /// is a separate client tier gate.
    #[serde(default)]
    pub voice_mode_enabled: Option<bool>,
    /// Whether ZDR (Zero Data Retention) users are allowed to use the product.
    /// Controlled via remote settings. Default `false` (blocked) during beta.
    #[serde(default)]
    pub zdr_access_enabled: Option<bool>,
    /// remote settings tier of the `remember_tool_approvals` gate (whether per-tool
    /// "Always allow …" prompt options are shown). Lowest precedence; typically
    /// targeted per-org. Default `false`.
    #[serde(default)]
    pub remember_tool_approvals: Option<bool>,
    /// remote settings tier of the crash-handler install gate. Lowest precedence in
    /// `resolve_crash_handler_enabled`; default off. `Some(false)` is a kill-switch.
    #[serde(default)]
    pub crash_handler_enabled: Option<bool>,
    /// Whether the TUI shows agent thinking/reasoning blocks in scrollback.
    /// `None` defers to local config / env / default (`true`).
    /// `Some(false)` is a remote kill-switch. Resolved via
    /// `resolve_show_thinking_blocks` (requirements > env > user > managed >
    /// remote > default true).
    #[serde(default)]
    pub show_thinking_blocks: Option<bool>,
    /// Whether the TUI folds runs of consecutive non-destructive tool calls
    /// (reads/searches/lists) into one transcript row. `None` defers to local
    /// config / env / default (`true`). `Some(false)` is a remote
    /// kill-switch. Resolved via `resolve_group_tool_verbs` (requirements >
    /// env > user > managed > remote > default true).
    #[serde(default)]
    pub group_tool_verbs: Option<bool>,
    /// Whether the TUI shows Edit tool calls as a collapsed one-line `+N/-M`
    /// diffstat summary by default and merges back-to-back edits to the same
    /// file into one row (expand for the diffs). `None` defers to local
    /// config / env / default (`true` in next-code denser resting transcript);
    /// `Some(false)` is a remote kill switch. Resolved via
    /// `resolve_collapsed_edit_blocks` (requirements > env > user > managed >
    /// remote > default true). Explicit pager.toml `[scrollback.blocks.edit]`
    /// shape keys override the flag's fold shape client-side; merging always
    /// follows the flag.
    #[serde(default)]
    pub collapsed_edit_blocks: Option<bool>,
    /// Display-refresh probe + auto-cadence. See [`DisplayRefreshSettings`].
    /// Partial object falls through per-field; resolved via `resolve_display_refresh`.
    #[serde(default)]
    pub display_refresh: Option<DisplayRefreshSettings>,
    /// Raw remote settings JSON for the `[auto_mode]` table (gate `enabled`,
    /// `prompt_type`, `classifier_model`). Coerced into the shell's typed
    /// `AutoModeConfig` (config-types stays dependency-light). Lowest-precedence
    /// layer in `resolve_auto_permission_mode_enabled` (client default ON).
    #[serde(default)]
    pub auto_mode: Option<serde_json::Value>,
    /// Soft default permission mode (`"ask"` / `"auto"` / `"always-approve"` /
    /// `"default"`). Used only when no effective TOML permission key is set.
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// User's subscription tier from remote settings `grok_build_access_gate`.
    /// E.g. "free", "premium", "supergrok", "supergrok_heavy".
    /// Stamped on analytics events + user profile for filtering.
    #[serde(default)]
    pub subscription_tier: Option<String>,
    #[serde(default)]
    pub gate_message: Option<String>,
    #[serde(default)]
    pub gate_url: Option<String>,
    #[serde(default)]
    pub gate_label: Option<String>,
    /// Whether the session picker groups entries by repo name.
    /// When `None` or `Some(false)`, sessions are shown in a flat list.
    #[serde(default)]
    pub session_picker_grouped: Option<bool>,
    /// Whether the user is allowed to use Grok Build. Set by remote settings
    /// `grok_build_access_gate` targeting rules. `None` = no server response
    /// yet (client uses own fallback check). `Some(false)` = blocked.
    #[serde(default)]
    pub allow_access: Option<bool>,
    /// User-friendly display name for the current subscription tier
    /// (e.g. "SuperGrok", "X Premium+", "Free", "API Key"). Set by CCP
    /// from the JWT tier claim (OAuth) or credential kind (API key).
    /// Free/Invalid OAuth → `"Free"`; API keys → `"API Key"` (Mixpanel
    /// `api_key`, never free).
    #[serde(default)]
    pub subscription_tier_display: Option<String>,
    /// Whether on-demand credit usage is enabled. When `Some(false)`, the
    /// billing extension blocks on-demand cap changes.
    #[serde(default)]
    pub on_demand_enabled: Option<bool>,
    /// When set to a non-empty URL, the pager's `/usage` command shows a link
    /// to that URL instead of fetching billing data from the backend.
    /// Server-controlled via the remote settings `grok_build_usage_redirect_url`
    /// feature flag (target it at personal-team users). `None`/empty keeps the
    /// default behaviour of fetching usage from the backend.
    #[serde(default)]
    pub usage_billing_redirect_url: Option<String>,
    /// Enable the shell command suggestion pipeline remotely.
    #[serde(default)]
    pub suggestions_enabled: Option<bool>,
    /// Enable AI-powered shell command suggestions remotely.
    #[serde(default)]
    pub suggestions_ai_enabled: Option<bool>,
    /// Global auto-compact threshold percent (0-100) from remote settings
    /// `grok_build_settings`. Per-model override on `ModelInfo`
    /// (`grok_build_models`) takes precedence; user config and env var
    /// further override per the resolver chain.
    #[serde(default)]
    pub auto_compact_threshold_percent: Option<u8>,
    /// Global system-prompt identity label. Per-model override wins; see
    /// `resolve_system_prompt_label`.
    #[serde(default)]
    pub system_prompt_label: Option<String>,
    /// Global per-compaction wall-clock budget (seconds) from remote settings;
    /// `0` disables. Env (`GROK_COMPACTION_WALL_CLOCK_SECS`) overrides it.
    /// Resolved via `resolve_compaction_wall_clock_budget_secs`.
    #[serde(default)]
    pub compaction_wall_clock_budget_secs: Option<u64>,
    /// Compaction mode (`summary` | `transcript` | `segments`) from remote settings.
    /// Env (`GROK_COMPACTION_MODE`) and user config override it.
    #[serde(default)]
    pub compaction_mode: Option<String>,
    /// Segments verbatim detail (`none` | `minimal` | `balanced` | `verbose`)
    /// from remote settings. Env (`GROK_COMPACTION_DETAIL`) and config override it.
    #[serde(default)]
    pub compaction_detail: Option<String>,
    /// remote settings verbatim-input flag; env (`GROK_COMPACTION_VERBATIM_INPUT`) and config override it. `None` = default (true).
    #[serde(default)]
    pub compaction_verbatim_input: Option<bool>,
    #[serde(default)]
    pub compaction_tool_choice: Option<String>,
    /// remote settings denylist of optional imagine tools to disable
    /// (e.g. `["image_edit"]`). When a tool is listed it is authoritatively
    /// removed from the toolset and local env/config can't re-enable it.
    /// Absent or not listed → each tool keeps its own default.
    /// See `Config::resolve_image_edit`.
    #[serde(default)]
    pub imagine_tools_disabled: Option<Vec<String>>,
    /// remote settings gate for the `grok workspace` CLI command (Computer Hub
    /// workspace exposure), from `grok_build_settings.workspace_command_enabled`.
    /// `Some(true)` enables it; `None`/`Some(false)` (the default) keep it off.
    #[serde(default)]
    pub workspace_command_enabled: Option<bool>,
    /// Master switch for jemalloc heap sampling + threshold dumps.
    /// `Some(true)` enables, `Some(false)` kill-switch, `None` = client default off.
    #[serde(default)]
    pub jemalloc_heap_profile_enabled: Option<bool>,
    /// Resident-byte thresholds (e.g. 2G/5G/10G as byte counts).
    /// `None` and `[]` are distinct on the wire.
    #[serde(default)]
    pub jemalloc_heap_profile_thresholds_bytes: Option<Vec<u64>>,
    /// Stats poll interval in seconds when set.
    #[serde(default)]
    pub jemalloc_heap_profile_poll_interval_secs: Option<u64>,
}

use std::collections::HashMap;

/// Simplified stand-in for upstream `ContextualHintsRemote` (all-Option
/// server-controlled tip toggles).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ContextualHintsRemote {
    #[serde(default)]
    pub undo: Option<bool>,
    #[serde(default)]
    pub plan_mode: Option<bool>,
    #[serde(default)]
    pub image_input: Option<bool>,
    #[serde(default)]
    pub send_now: Option<bool>,
    #[serde(default)]
    pub small_screen: Option<bool>,
    #[serde(default)]
    pub word_select: Option<bool>,
    #[serde(default)]
    pub ssh_wrap: Option<bool>,
}

/// Simplified stand-in for upstream `McpServerTransportConfig` (Stdio /
/// StreamableHttp variants only — SSE and other legacy variants dropped).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum McpServerTransportConfig {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: Option<HashMap<String, String>>,
        #[serde(default)]
        cwd: Option<String>,
    },
    StreamableHttp {
        #[serde(default)]
        url: String,
        #[serde(default)]
        transport_type: Option<String>,
        #[serde(default)]
        bearer_token_env_var: Option<String>,
        #[serde(default)]
        headers: Option<HashMap<String, String>>,
        #[serde(default)]
        oauth_client_id: Option<String>,
        #[serde(default)]
        oauth_client_secret_env_var: Option<String>,
        #[serde(default)]
        oauth_scopes: Option<Vec<String>>,
    },
}

impl Default for McpServerTransportConfig {
    fn default() -> Self {
        Self::Stdio {
            command: String::new(),
            args: Vec::new(),
            env: None,
            cwd: None,
        }
    }
}

/// Simplified stand-in for upstream `McpServerConfig`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServerConfig {
    #[serde(flatten)]
    pub transport: McpServerTransportConfig,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub oauth: Option<McpOAuthConfig>,
    #[serde(default)]
    pub setup: Option<serde_json::Value>,
    #[serde(default)]
    pub startup_timeout_sec: Option<u64>,
    #[serde(default)]
    pub tool_timeout_sec: Option<u64>,
    #[serde(default)]
    pub tool_timeouts: Option<HashMap<String, u64>>,
    #[serde(default)]
    pub expose_image_base64: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpOAuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

/// Upstream reads/writes `~/.grok/settings.json` (merged with project +
/// managed config); this compile-stub layer has no disk-backed config, so
/// `load`/`set_*` are no-ops over an in-memory default.
pub fn load() -> RemoteSettings {
    RemoteSettings::default()
}

/// Persisted worktree preference for `/new` and `/fork` (`[hints]` in config.toml).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeHintMode {
    Ask,
    Always,
    #[default]
    Never,
}

impl WorktreeHintMode {
    pub fn from_config_str(s: &str) -> Self {
        match s {
            "always" => Self::Always,
            "never" => Self::Never,
            "ask" => Self::Ask,
            _ => Self::Never,
        }
    }

    pub fn as_config_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    pub fn resolve_pair(hints: Option<&toml::Value>) -> (Self, Self) {
        let get_str = |key: &str| -> Option<Self> {
            hints
                .and_then(|h| h.get(key))
                .and_then(|v| v.as_str())
                .map(Self::from_config_str)
        };
        let legacy = get_str("worktree_mode");
        let new_session = get_str("new_session_worktree_mode")
            .or(legacy)
            .unwrap_or(Self::Never);
        let fork = get_str("fork_worktree_mode")
            .or(legacy)
            .unwrap_or(Self::Ask);
        (new_session, fork)
    }
}

pub fn env_bool(name: &str) -> Option<bool> {
    match std::env::var(name).ok()?.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub const DISPLAY_REFRESH_DEFAULT_CADENCE_MS: u64 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionCadence {
    pub min_draw_ms: u64,
    pub scroll_ms: u64,
    pub auto_applied: bool,
    pub reason: &'static str,
}

impl Default for MotionCadence {
    fn default() -> Self {
        Self {
            min_draw_ms: DISPLAY_REFRESH_DEFAULT_CADENCE_MS,
            scroll_ms: DISPLAY_REFRESH_DEFAULT_CADENCE_MS,
            auto_applied: false,
            reason: "default",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedContextualHints {
    pub undo: bool,
    pub plan_mode: bool,
    pub image_input: bool,
    pub send_now: bool,
    pub small_screen: bool,
    pub word_select: bool,
    pub ssh_wrap: bool,
}

impl Default for ResolvedContextualHints {
    fn default() -> Self {
        Self {
            undo: true,
            plan_mode: true,
            image_input: true,
            send_now: true,
            small_screen: true,
            word_select: true,
            ssh_wrap: true,
        }
    }
}

/// Effective display-refresh policy after layered resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayRefreshPolicy {
    pub probe_enabled: bool,
    pub auto_cadence_enabled: bool,
    pub floor_ms: u32,
    pub ceiling_ms: u32,
    pub min_hz: u32,
    pub max_hz: u32,
}

impl Default for DisplayRefreshPolicy {
    fn default() -> Self {
        Self {
            probe_enabled: true,
            auto_cadence_enabled: false,
            floor_ms: 8,
            ceiling_ms: 16,
            min_hz: 55,
            max_hz: 165,
        }
    }
}

pub fn resolve_display_refresh(
    _requirements: Option<&toml::Value>,
    _user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote: Option<&RemoteSettings>,
) -> DisplayRefreshPolicy {
    DisplayRefreshPolicy::default()
}

pub fn resolve_motion_cadence(
    _policy: &DisplayRefreshPolicy,
    _probe_hz: Option<u32>,
    _min_draw_env: Option<u64>,
    _scroll_env: Option<u64>,
) -> MotionCadence {
    MotionCadence::default()
}

pub fn resolve_contextual_hints(
    _ui: &crate::agent::config::ContextualHints,
    _remote: Option<&ContextualHintsRemote>,
) -> ResolvedContextualHints {
    ResolvedContextualHints::default()
}

pub fn use_leader_from_toml_opt(_root: &toml::Value) -> Option<bool> {
    None
}

pub fn use_leader_from_toml(root: &toml::Value) -> bool {
    use_leader_from_toml_opt(root).unwrap_or(false)
}

pub fn load_mcp_servers(
    _cwd: &std::path::Path,
    _compat: &xai_grok_tools::types::compat::CompatConfig,
) -> Vec<agent_client_protocol::McpServer> {
    Vec::new()
}

// --- PR10: real disk setters → ~/.next-code/config.toml ---
use std::path::{Path, PathBuf};

fn serialize_to_toml_value(v: impl serde::Serialize) -> anyhow::Result<toml_edit::Value> {
    let json = serde_json::to_value(v)?;
    Ok(match json {
        serde_json::Value::Bool(b) => toml_edit::Value::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml_edit::Value::from(i)
            } else if let Some(u) = n.as_u64() {
                toml_edit::Value::from(u as i64)
            } else if let Some(f) = n.as_f64() {
                toml_edit::Value::from(f)
            } else {
                anyhow::bail!("unsupported number for config write")
            }
        }
        serde_json::Value::String(s) => toml_edit::Value::from(s),
        other => anyhow::bail!("unsupported setting value kind: {other}"),
    })
}

async fn set_ui_key(key: &str, v: impl serde::Serialize) -> anyhow::Result<()> {
    let value = serialize_to_toml_value(v)?;
    let key = key.to_string();
    tokio::task::spawn_blocking(move || {
        xai_grok_config::set_toml_key("ui", &key, value).map_err(anyhow::Error::from)
    })
    .await??;
    Ok(())
}

async fn set_ui_nested(sub: &str, key: &str, v: impl serde::Serialize) -> anyhow::Result<()> {
    let value = serialize_to_toml_value(v)?;
    let sub = sub.to_string();
    let key = key.to_string();
    tokio::task::spawn_blocking(move || {
        xai_grok_config::set_toml_nested_key("ui", &sub, &key, value).map_err(anyhow::Error::from)
    })
    .await??;
    Ok(())
}

pub async fn set_ask_user_question_timeout_enabled(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("ask_user_question_timeout_enabled", v).await
}
pub async fn set_auto_dark_theme(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("auto_dark_theme", v).await
}
pub async fn set_auto_light_theme(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("auto_light_theme", v).await
}
pub async fn set_auto_update(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("auto_update", v).await
}
pub async fn set_cancel_subagents_on_turn_cancel(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("cancel_subagents_on_turn_cancel", v).await
}
pub async fn set_collapsed_edit_blocks(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("collapsed_edit_blocks", v).await
}
pub async fn set_compact_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("compact_mode", v).await
}
pub async fn set_contextual_hint_image_input(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "image_input", v).await
}
pub async fn set_contextual_hint_plan_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "plan_mode", v).await
}
pub async fn set_contextual_hint_send_now(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "send_now", v).await
}
pub async fn set_contextual_hint_small_screen(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "small_screen", v).await
}
pub async fn set_contextual_hint_ssh_wrap(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "ssh_wrap", v).await
}
pub async fn set_contextual_hint_undo(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "undo", v).await
}
pub async fn set_contextual_hint_word_select(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("contextual_hints", "word_select", v).await
}
// Info float visibility helpers
pub async fn set_info_float_model_info(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "model_info", v).await
}
pub async fn set_info_float_context_usage(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "context_usage", v).await
}
pub async fn set_info_float_kv_cache(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "kv_cache", v).await
}
pub async fn set_info_float_memory_activity(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "memory_activity", v).await
}
pub async fn set_info_float_usage_limits(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "usage_limits", v).await
}
pub async fn set_info_float_git_status(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "git_status", v).await
}
pub async fn set_info_float_background_tasks(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "background_tasks", v).await
}
pub async fn set_info_float_compaction(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "compaction", v).await
}
pub async fn set_info_float_swarm_status(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "swarm_status", v).await
}
pub async fn set_info_float_todos(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "todos", v).await
}
pub async fn set_info_float_workspace_map(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "workspace_map", v).await
}
pub async fn set_info_float_diagrams(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("info_floats", "diagrams", v).await
}
pub async fn set_default_model(v: impl serde::Serialize) -> anyhow::Result<()> {
    // Shared with next-code brain: `[provider].default_model` (+ optional provider).
    // Prefer [`set_default_model_and_provider`] when the provider pin is known.
    let value = serialize_to_toml_value(v)?;
    let Some(model) = value.as_str().map(str::to_string) else {
        anyhow::bail!("default_model must be a string");
    };
    set_default_model_and_provider(model, None).await
}

/// Persist `[provider].default_model` and optionally `[provider].default_provider`
/// in one toml_edit write (preserves `[ui]` and other sibling tables).
pub async fn set_default_model_and_provider(
    model: String,
    provider: Option<String>,
) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        xai_grok_config::set_provider_defaults(Some(model.as_str()), provider.as_deref())
            .map_err(anyhow::Error::from)
    })
    .await??;
    Ok(())
}
pub async fn set_default_selected_permission(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("default_selected_permission", v).await
}
pub async fn set_display_refresh_auto_cadence(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_nested("display_refresh", "auto_cadence_enabled", v).await
}
pub async fn set_fork_secondary_model(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("fork_secondary_model", v).await
}
pub async fn set_group_tool_verbs(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("group_tool_verbs", v).await
}
pub async fn set_hunk_tracker_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("hunk_tracker_mode", v).await
}
pub async fn set_invert_scroll(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("invert_scroll", v).await
}
pub async fn set_keep_text_selection(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("keep_text_selection", v).await
}
pub async fn set_max_thoughts_width(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("max_thoughts_width", v).await
}
pub async fn set_page_flip_on_send(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("page_flip_on_send", v).await
}
pub async fn set_prompt_suggestions(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("prompt_suggestions", v).await
}
pub async fn set_remember_tool_approvals(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("remember_tool_approvals", v).await
}
pub async fn set_render_mermaid(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("render_mermaid", v).await
}
pub async fn set_screen_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("screen_mode", v).await
}
pub async fn set_btw_output_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("btw_output_mode", v).await
}
pub async fn set_btw_sidebar_width(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("btw_sidebar_width", v).await
}
pub async fn set_scroll_lines(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("scroll_lines", v).await
}
pub async fn set_scroll_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("scroll_mode", v).await
}
pub async fn set_scroll_speed(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("scroll_speed", v).await
}
pub async fn set_show_thinking_blocks(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("show_thinking_blocks", v).await
}
pub async fn set_show_timeline(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("show_timeline", v).await
}
pub async fn set_show_timestamps(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("show_timestamps", v).await
}
pub async fn set_show_tips(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("show_tips", v).await
}
pub async fn set_simple_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("simple_mode", v).await
}
pub async fn set_theme(v: impl serde::Serialize) -> anyhow::Result<()> {
    // Face ThemeKind display name (e.g. "Grok Night") — NOT origin dark/light.
    set_ui_key("theme", v).await
}
pub async fn set_vim_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("vim_mode", v).await
}
pub async fn set_voice_capture_mode(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("voice_capture_mode", v).await
}
pub async fn set_voice_stt_language(v: impl serde::Serialize) -> anyhow::Result<()> {
    set_ui_key("voice_stt_language", v).await
}
pub async fn update_config(
    f: impl FnOnce(&mut crate::agent::config::Config),
) -> anyhow::Result<()> {
    let mut cfg = crate::agent::config::Config::default();
    f(&mut cfg);
    // Face PersistPermissionMode writes through this path.
    if let Some(ref mode) = cfg.ui.permission_mode {
        set_ui_key("permission_mode", mode.clone()).await?;
    }
    Ok(())
}
pub fn user_config_path() -> PathBuf {
    xai_grok_config::user_config_toml_path()
}
pub fn project_config_path(cwd: &Path) -> PathBuf {
    cwd.join(".next-code").join("config.toml")
}
pub fn worktree_type() -> WorktreeHintMode {
    WorktreeHintMode::Never
}

/// Result of [`effective_yolo_for_launch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EffectiveYolo {
    pub yolo: bool,
    pub blocked_warning: Option<&'static str>,
    pub policy_block: Option<&'static str>,
}

/// Where a resolved config value came from (Face stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigSource {
    Requirement,
    Cli,
    Env,
    ManagedConfig,
    UserConfig,
    Remote,
    #[default]
    Default,
}

impl std::fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Requirement => "requirement",
            Self::Cli => "cli",
            Self::Env => "env",
            Self::ManagedConfig => "managed",
            Self::UserConfig => "user",
            Self::Remote => "remote",
            Self::Default => "default",
        };
        f.write_str(s)
    }
}

/// A resolved config value with its source.
#[derive(Debug, Clone)]
pub struct Resolved<T> {
    pub value: T,
    pub source: ConfigSource,
}

impl<T> Resolved<T> {
    pub fn new(value: T, source: ConfigSource) -> Self {
        Self { value, source }
    }
}

fn resolved_bool(value: bool) -> Resolved<bool> {
    Resolved::new(value, ConfigSource::Default)
}

/// Launch permission-mode discriminant used by display helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PermissionMode {
    #[default]
    Ask,
    AlwaysApprove,
    Auto,
    Default,
}

impl PermissionMode {
    pub fn is_always_approve(self) -> bool {
        matches!(self, Self::AlwaysApprove)
    }
    pub fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedHints {
    pub project_picker_disabled: bool,
    pub new_session_worktree_mode: WorktreeHintMode,
    pub fork_worktree_mode: WorktreeHintMode,
}

pub fn effective_auto_for_launch(
    _cli_always_approve: bool,
    _cli_permission_mode: Option<&str>,
    _remote_permission_mode: Option<&str>,
) -> bool {
    false
}

pub fn effective_yolo_for_launch(
    _cli_always_approve: bool,
    _cli_permission_mode: Option<&str>,
    _remote_permission_mode: Option<&str>,
) -> EffectiveYolo {
    EffectiveYolo::default()
}

pub fn load_mcp_server_configs_with_project(
    _cwd: &Path,
) -> Vec<(String, (McpServerConfig, &'static str))> {
    vec![]
}
pub async fn save_mcp_server_config_at(
    _path: &Path,
    _name: &str,
    _cfg: &McpServerConfig,
) -> anyhow::Result<()> {
    Ok(())
}
pub async fn delete_mcp_server_config_at(_path: &Path, _name: &str) -> anyhow::Result<bool> {
    Ok(false)
}
pub fn mcp_server_defined_at(_path: &Path, _name: &str) -> bool { false }
pub fn cache_remote_auto_mode(_v: Option<serde_json::Value>) {}
pub fn cache_remote_auto_permission_mode_enabled(_v: Option<bool>) {}
pub fn auto_permission_mode_enabled_from_disk() -> bool { true }

pub fn clamped_display_permission_mode(mode: PermissionMode) -> &'static str {
    if mode.is_always_approve() || mode.is_auto() {
        "ask"
    } else {
        match mode {
            PermissionMode::Ask => "ask",
            PermissionMode::Default => "default",
            PermissionMode::AlwaysApprove => "always-approve",
            PermissionMode::Auto => "auto",
        }
    }
}

pub fn parse_permission_mode_canonical(s: &str) -> PermissionMode {
    match s.trim().to_ascii_lowercase().as_str() {
        "always-approve" | "yolo" | "bypasspermissions" => PermissionMode::AlwaysApprove,
        "auto" => PermissionMode::Auto,
        "default" => PermissionMode::Default,
        _ => PermissionMode::Ask,
    }
}

pub fn permission_mode_from_ui_if_set(ui: &toml::Value) -> Option<PermissionMode> {
    ui.get("permission_mode")
        .and_then(|v| v.as_str())
        .map(parse_permission_mode_canonical)
}

pub fn resolved_display_permission_mode(
    effective_ui: Option<&toml::Value>,
    remote_permission_mode: Option<&str>,
) -> &'static str {
    clamped_display_permission_mode(resolve_permission_mode(
        effective_ui,
        remote_permission_mode,
    ))
}

pub fn resolve_permission_mode(
    effective_ui: Option<&toml::Value>,
    remote_permission_mode: Option<&str>,
) -> PermissionMode {
    if let Some(ui) = effective_ui
        && let Some(mode) = permission_mode_from_ui_if_set(ui)
    {
        return mode;
    }
    if let Some(mode_str) = remote_permission_mode {
        return parse_permission_mode_canonical(mode_str);
    }
    PermissionMode::Ask
}

pub fn resolve_auto_permission_mode_enabled() -> bool { true }

pub fn resolve_announcements(
    _requirements: Option<&toml::Value>,
    _user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote: Option<&[RemoteAnnouncement]>,
) -> Vec<RemoteAnnouncement> {
    vec![]
}

pub fn resolve_collapsed_edit_blocks(
    _requirements: Option<&toml::Value>,
    user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote: Option<&RemoteSettings>,
) -> Resolved<bool> {
    // Prefer explicit `[ui].collapsed_edit_blocks`; else next-code product
    // default true (denser resting transcript). Do not regress to grok false.
    if let Some(v) = user
        .and_then(|u| u.get("ui"))
        .and_then(|ui| ui.get("collapsed_edit_blocks"))
        .and_then(|v| v.as_bool())
    {
        return Resolved::new(v, ConfigSource::UserConfig);
    }
    resolved_bool(true)
}

pub fn resolve_group_tool_verbs(
    _requirements: Option<&toml::Value>,
    user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote: Option<&RemoteSettings>,
) -> Resolved<bool> {
    if let Some(v) = user
        .and_then(|u| u.get("ui"))
        .and_then(|ui| ui.get("group_tool_verbs"))
        .and_then(|v| v.as_bool())
    {
        return Resolved::new(v, ConfigSource::UserConfig);
    }
    resolved_bool(true)
}

pub fn resolve_hints(
    _effective_config: Option<&toml::Value>,
    _requirements: Option<&toml::Value>,
    _user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
) -> ResolvedHints {
    ResolvedHints::default()
}

pub fn resolve_mcp_push_server_status(
    _requirements: Option<&toml::Value>,
    _user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
) -> bool {
    false
}

pub fn resolve_mouse_reporting_toggle(
    _effective: Option<&toml::Value>,
    _ui: &crate::agent::config::UiConfig,
) -> Resolved<bool> {
    resolved_bool(false)
}

pub fn resolve_remote_fetch_enabled() -> bool { false }

pub fn resolve_show_thinking_blocks(
    _requirements: Option<&toml::Value>,
    user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote: Option<&RemoteSettings>,
) -> Resolved<bool> {
    if let Some(v) = user
        .and_then(|u| u.get("ui"))
        .and_then(|ui| ui.get("show_thinking_blocks"))
        .and_then(|v| v.as_bool())
    {
        return Resolved::new(v, ConfigSource::UserConfig);
    }
    resolved_bool(true)
}

pub fn resolve_tips(
    _requirements: Option<&toml::Value>,
    _user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote_tips: Option<&[String]>,
) -> Vec<String> {
    vec![]
}

pub fn resolve_zdr_access_enabled(
    _requirements: Option<&toml::Value>,
    _user: Option<&toml::Value>,
    _managed: Option<&toml::Value>,
    _remote: Option<&RemoteSettings>,
) -> bool {
    false
}

pub fn load_require_plan_approval() -> bool { false }

/// Persist preferred model for next cold start.
///
/// Stock Grok writes `[models].default`; next-code remaps to
/// `[provider].default_model` (and optional `default_provider` via
/// [`persist_models_default_with_provider`]).
pub async fn persist_models_default(
    model: Option<String>,
    reasoning_effort: Option<crate::sampling::types::ReasoningEffort>,
) -> anyhow::Result<()> {
    persist_models_default_with_provider(model, None, reasoning_effort).await
}

/// Like [`persist_models_default`], but also pins `[provider].default_provider`
/// when `provider` is `Some` (atomic pair write).
pub async fn persist_models_default_with_provider(
    model: Option<String>,
    provider: Option<String>,
    _reasoning_effort: Option<crate::sampling::types::ReasoningEffort>,
) -> anyhow::Result<()> {
    let Some(model) = model else {
        return Ok(());
    };
    set_default_model_and_provider(model, provider).await
}

pub fn set_remote_campaigns_from_settings(_s: Option<&RemoteSettings>) {}

