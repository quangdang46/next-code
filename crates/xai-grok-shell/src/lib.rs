//! Compile-stub façade of `xai-org/grok-build` `xai-grok-shell` (PR5).
//!
//! Upstream is a ~434-file, ~14MB crate (terminal/session/relay/auth/
//! extensions/tools/…). This is deliberately NOT a wholesale vendor —
//! only the highest-frequency import prefixes the future pager actually
//! uses are stubbed, per the approved narrow plan
//! (`docs/plans/PLAN-20260720-grok-pr5-agent-shell-acp.md`):
//!
//! - `util::config` (135 hits) — full `RemoteSettings` DTO + MCP config shapes
//! - `agent::config` (51 hits) — `Config`/`AgentDefinition`/`AgentMode`/…
//! - `util::with_locked_stderr` (37 hits) — re-exported from `xai-grok-shared`
//! - `sampling::types` / `sampling::error` (30/11 hits)
//! - `auth::{AuthMeta, GateInfo, AuthManager}` (30/17/5 hits)
//! - `extensions::{notification, session_search, mcp, task, billing}`
//! - `config::{load_effective_config, load_from_disk, load_managed_config,
//!   load_merged_requirements, find_project_configs, plugin toggles}`
//! - `util::{grok_home, clipboard, changelog, tips}`
//! - `session::{persistence, ContextInfo, PromptOrigin, …}` + thin
//!   sub-modules (worktree, storage, merge, restore, repo_changes,
//!   prompt_queue, info)
//! - `models::default_model`, `tier::is_restricted_tier_name`,
//!   `active_sessions`
//!
//! Function bodies are empty/no-op placeholders and DTOs are
//! `Default`-derived — not real logic — matching the PR3/PR4 stub
//! convention (see `xai-grok-workspace/src/file_system/fuzzy.rs`).
//! No real disk I/O, no real git/MCP/auth-provider network calls.
//!
//! This PR does **not** wire anything into `next-code-agent-runtime` or
//! `next-code-app-core`'s Registry — that remains PR8 (`GrokHost`).

pub mod active_sessions;
pub mod agent;
pub mod auth;
pub mod config;
pub mod extensions;
pub mod models;
pub mod sampling;
pub mod session;
pub mod tier;
pub mod util;
