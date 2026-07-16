use next_code_plugin_core::events::HandlerResult;
use std::sync::Arc;

pub use next_code_plugin_core::preflight::{PreflightResult, StaticAnalysis};

#[allow(clippy::type_complexity)]
pub enum HandlerSlot {
    Rust(
        Arc<
            dyn Fn(
                    next_code_plugin_core::events::EventInput,
                    Option<next_code_plugin_core::events::EventOutput>,
                )
                    -> std::pin::Pin<Box<dyn std::future::Future<Output = HandlerResult> + Send>>
                + Send
                + Sync,
        >,
    ),
}

impl Clone for HandlerSlot {
    fn clone(&self) -> Self {
        match self {
            Self::Rust(f) => Self::Rust(Arc::clone(f)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    pub path: std::path::PathBuf,
    pub manifest: next_code_plugin_core::manifest::PluginManifest,
}
