//! Beads-rs integration for jcode.
//!
//! This module re-exports the `jcode-beads-bridge` crate so that jcode code can
//! access beads_rust functionality via `crate::beads::*` instead of importing
//! `jcode_beads_bridge` directly.

pub use jcode_beads_bridge::*;
