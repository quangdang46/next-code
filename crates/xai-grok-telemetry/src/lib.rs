//! No-op telemetry shim for Face render substrate (grown for PR7 pager).

pub mod client;
pub mod debug_log;
pub mod events;
pub mod external;
pub mod hooks_log;
pub mod instrumentation;
pub mod otel_layer;
pub mod sampling_log;
pub mod sentry;
pub mod session_ctx;
pub mod unified_log;

pub use client::is_enabled;
pub use session_ctx::log_event;
