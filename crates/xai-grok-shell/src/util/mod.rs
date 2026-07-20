//! Façade stub of upstream `xai-grok-shell::util` — only the highest-
//! frequency import prefixes from the future pager (see plan doc
//! PR5). `stderr`/`changelog`/`tips` re-export or thin-stub what PR2/3
//! already vendored in `xai-grok-shared`/`xai-grok-config`.

pub mod changelog;
pub mod clipboard;
pub mod config;
pub mod grok_home;
pub mod tips;

pub use xai_grok_shared::stderr::with_locked_stderr;
