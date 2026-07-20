//! Re-export of upstream `xai-grok-shell::session::repo_changes` upload types.
//!
//! Upstream (`grok-build/.../session/repo_changes/mod.rs`) does:
//! `pub use xai_file_utils::{TraceExportConfig, UploadMethod, …}`.
//! Pager `trace_cmd` imports `xai_grok_shell::session::repo_changes::TraceExportConfig`
//! and passes it to `xai_file_utils::gcs::upload_bytes`.
//!
//! PR5 originally shipped a wrong Inline/Reference stand-in; corrected in the
//! PR6 grok-build fidelity review to re-export the real shapes from
//! `xai-file-utils`.

pub use xai_file_utils::{TraceExportConfig, UploadMethod};
