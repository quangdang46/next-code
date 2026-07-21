//! Hooks module — lifecycle hooks for next-code events.

pub mod cli;
pub mod config;
pub mod dispatch;
pub mod execute;
pub mod matcher;
pub mod registry;
pub mod types;

pub use config::{
    env_hooks_config_path, face_hook_name, legacy_v1_to_v2_handlers, list_hook_layer_entries,
    load_hooks_config, load_hooks_config_from_path, load_hooks_config_from_path_detailed,
    parse_face_hook_name, project_hooks_config_path, set_hook_enabled_by_face_name,
    set_hook_enabled_in_file, user_hooks_config_path, AgentHandlerConfig, CommandHandlerConfig,
    HookEvent, HookHandlerConfig, HookLayerEntry, HookSettings, HooksConfig, HooksConfigScope,
    HttpHandlerConfig, PluginHandlerConfig,
};
pub use dispatch::{
    dispatch_hooks, get_hook_metrics, get_hook_metrics_for_event, ClassifiedOutcome,
    ClassifiedResult, DispatchConfig, DispatchStats,
};
pub use execute::{execute_command_hook, execute_hook, execute_http_hook};
pub use matcher::{matches, parse_multi_pattern, HookMatcher, MatcherContext};
pub use registry::{HookContext, HookRegistry};
pub use types::*;

#[cfg(test)]
mod tests;
