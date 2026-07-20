//! Stub of upstream `xai-grok-shell::util::tips` — rotates contextual
//! "did you know" tip strings. This stub has no tip pool and always
//! returns `None` (no-op advance).

pub fn pick_and_advance(_seen: &[String]) -> Option<String> {
    None
}
