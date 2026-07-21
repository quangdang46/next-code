//! Compile stubs for Face/pager imports (PR4). Not a tool runtime.
//! Provenance: xai-org/grok-build tools crate (SOURCE_REV ba69d70).

pub mod implementations;
pub mod registry;
pub mod reminders;
pub mod types;
pub mod util;

pub use util::detach_std_command;
