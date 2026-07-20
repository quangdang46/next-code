//! Compile stub of `xai-org/grok-build` `xai-grok-agent` (PR5).
//!
//! Upstream is a ~30-file crate covering agent builder/discovery/plugin
//! marketplace machinery. Only the handful of types/functions the future
//! pager actually imports (`agents_modal.rs`, `plugin_cmd.rs` — agent
//! discovery + plugin install types) are stubbed here, matching upstream
//! signatures with empty/no-op bodies. Full agent-builder logic
//! (`builder.rs`, `compaction.rs`, `system_reminder.rs`, …) is
//! intentionally NOT vendored — it belongs to the runtime side of the
//! adapter (PR8 / `GrokHost`), not this Face compile-stub layer.

pub mod config;
pub mod discovery;
pub mod plugins;

pub use config::AgentDefinition;
