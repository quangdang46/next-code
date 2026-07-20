//! Re-export of upstream `xai-grok-shell::session::repo_changes` upload types.
//!
//! Upstream (`grok-build/.../session/repo_changes/mod.rs`) does:
//! `pub use xai_file_utils::{TraceExportConfig, UploadMethod, …}`.
//! Pager `trace_cmd` imports `xai_grok_shell::session::repo_changes::TraceExportConfig`
//! and passes it to `xai_file_utils::gcs::upload_bytes`.

pub use xai_file_utils::{TraceExportConfig, UploadMethod};
