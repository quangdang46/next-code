# NOTICE — xai-file-utils

Facade of `xai-org/grok-build` `xai-file-utils` (Apache-2.0) for the next-code Grok
Face migration (PR6).

Upstream: https://github.com/xai-org/grok-build
Upstream path: crates/codegen/xai-file-utils (~15 files: per-turn event tracking,
upload queue, S3/GCS storage clients, circuit breaker, refresh-aware auth)

## Role in next-code

Upstream is a large local-data-collection crate: per-turn event tracking, an
upload queue, and S3/GCS-compatible blob storage backed by `aws-sdk-s3`,
`gcloud-storage`, and `xai-grok-auth` credential refresh/circuit-breaker
machinery. Vendoring it wholesale was explicitly out of scope for PR6 — this is a
facade covering only the three import sites the pager needs to compile:

- `workspace_classifier::is_project_dir` — vendored near-verbatim (pure logic:
  simple filesystem-existence checks + path string matching against known OS/tool
  directories, no I/O beyond `Path::exists`/`dirs::*`).
- `gcs::upload_bytes` — stub, always returns `Err(...)`; no GCS/S3 client, no
  network dependency, no `aws-sdk-s3`/`gcloud-storage`/auth.
- `trace_context::span_from_meta_traceparent` — no-op span (same call shape,
  no OpenTelemetry propagator, so no distributed trace parenting).

Not vendored: `queue.rs`, `storage_client.rs`, `s3.rs`, `circuit_breaker_observer.rs`,
`upload_config.rs`, `events/` — none of these are pager import sites, and per the
plan this crate intentionally does not carry S3/GCS upload machinery or
`xai-grok-auth`.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
