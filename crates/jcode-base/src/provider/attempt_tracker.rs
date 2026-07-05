//! Re-export attempt_tracker from provider-core.
//!
//! jcode-base providers (anthropic, claude, copilot) reference
//! `super::attempt_tracker::retry_backoff_delay` and
//! `super::attempt_tracker::track_attempt_output`. These live in
//! `jcode_provider_core` starting from the v0.30 runtime-split; this
//! module is a re-export shim so the legacy `super::attempt_tracker::*`
//! paths continue to resolve.

pub use jcode_provider_core::attempt_tracker::{track_attempt_output, retry_backoff_delay};
