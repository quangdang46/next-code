//! Stub of upstream `xai-grok-agent::plugins` — only the install-registry,
//! manifest, and git-install types/functions the future pager's
//! `plugin_cmd.rs` imports. Marketplace/trust/hooks_adapter/local_refresh
//! (upstream) are not stubbed — no known pager-render/pager import site yet.

pub mod git_install;
pub mod install_registry;
pub mod manifest;
