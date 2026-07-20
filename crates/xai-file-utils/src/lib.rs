//! Facade of `xai-org/grok-build` `xai-file-utils` (Apache-2.0) for the
//! next-code Grok Face migration (PR6).
//!
//! Upstream is a large crate (~15 files) covering per-turn event tracking,
//! an upload queue, and S3/GCS-compatible blob storage with auth/circuit-
//! breaker machinery. Vendoring it wholesale is explicitly out of scope here
//! — this facade only reproduces the three surfaces the pager imports:
//! [`workspace_classifier::is_project_dir`] (ported faithfully — it's pure,
//! filesystem-existence-check-only logic), [`gcs::upload_bytes`] (stub,
//! always `Err`, no GCS/S3 client or network dependency), and
//! [`trace_context::span_from_meta_traceparent`] (no-op span, no
//! OpenTelemetry dependency). Everything else upstream (auth, S3, the upload
//! queue, circuit breaker, event tracking) is intentionally not present.

pub mod gcs;
pub mod trace_context;
pub mod workspace_classifier;
