//! Facade of `xai-org/grok-build` `xai-file-utils` (Apache-2.0) for the
//! next-code Grok Face migration (PR6).
//!
//! Upstream is a large crate (~15 files) covering per-turn event tracking,
//! an upload queue, and S3/GCS-compatible blob storage with auth/circuit-
//! breaker machinery. Vendoring it wholesale is explicitly out of scope here
//! — this facade only reproduces the surfaces the pager imports:
//! [`workspace_classifier::is_project_dir`], [`gcs::upload_bytes`] (stub,
//! always `Err`, correct `StorageConfig` signature), and
//! [`trace_context::span_from_meta_traceparent`] (no-op span).
//!
//! Also exports [`UploadMethod`] / [`TraceExportConfig`] so shell can
//! re-export them the same way upstream `session::repo_changes` does.

pub mod gcs;
pub mod trace_context;
pub mod upload_config;
pub mod workspace_classifier;

pub use upload_config::{TraceExportConfig, UploadMethod};
