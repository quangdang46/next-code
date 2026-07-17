//! Logging infrastructure for next-code
//!
//! Logs to ~/.next-code/logs/ with automatic rotation
//!
//! Supports thread-local context for server, session, provider, and model info.
//!
//! The implementation lives in the `next-code-logging` workspace crate so that this
//! very-high-fanout, low-churn subsystem forms a stable compile-cache boundary
//! and does not pull the root crate into rebuilds. This module is a thin facade
//! that preserves the existing `crate::logging::*` API for all call sites.

pub use next_code_logging::*;
