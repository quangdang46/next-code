//! Façade re-export of upstream `xai-grok-shell::util::clipboard`. The real
//! implementation already lives in `xai-grok-shared::clipboard` (vendored
//! in PR2); this module just re-exports the pager's import surface so
//! `use xai_grok_shell::util::clipboard::*` keeps working without a second
//! copy of ~2000 lines of platform clipboard code.
pub use xai_grok_shared::clipboard::*;
