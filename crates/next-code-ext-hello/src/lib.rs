/// Example workspace-crate plugin for next_code.
/// Demonstrates how a crate inside the workspace can act as a plugin
/// by referencing next-code-plugin-core types.

use next_code_plugin_core::{PluginManager, PluginSource};

/// Returns a greeting manifest intended for registration.
pub fn manifest() -> &'static str {
    "next-code-ext-hello 0.1.0 — example workspace-crate plugin"
}

/// Placeholder showing that PluginManager/PluginSource can be
/// constructed by an extension crate.
#[allow(dead_code)]
fn example_usage() {
    let _src = PluginSource::WorkspaceCrate {
        crate_name: "next-code-ext-hello".into(),
    };
    let _mgr: Option<PluginManager> = None;
}
