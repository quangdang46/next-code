//! Stub of upstream `xai-grok-shell::builtin`.

/// Bundled check-work skill markdown (empty Face stub).
pub const CHECK_SKILL_MD: &str = "# check-work\n";

/// Bundled best-of-n skill markdown (empty Face stub).
pub const BEST_OF_N_SKILL_MD: &str = "# best-of-n\n";

pub fn bundled_skill_names() -> Vec<String> {
    vec!["check-work".into(), "best-of-n".into()]
}
