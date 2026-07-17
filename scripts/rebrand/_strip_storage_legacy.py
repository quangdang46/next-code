#!/usr/bin/env python3
"""One-shot: strip dual-read / migrate / deprecated aliases from storage lib."""
from __future__ import annotations

import re
from pathlib import Path

path = Path("crates/next-code-storage/src/lib.rs")
text = path.read_text(encoding="utf-8")

text = text.replace("use std::sync::Once;\n", "")
text = text.replace(
    """    pub fn clear_home_env() {
        next_code_core::env::remove_var("NEXT_CODE_HOME");
        next_code_core::env::remove_var("NEXT_CODE_HOME");
    }
""",
    """    pub fn clear_home_env() {
        next_code_core::env::remove_var("NEXT_CODE_HOME");
    }
""",
)
text = text.replace(
    'const MIGRATE_MARKER: &str = ".migrated-from-next-code";\n'
    "static MIGRATE_LOG_ONCE: Once = Once::new();\n\n",
    "",
)
text = text.replace(
    """/// Can be overridden with `$NEXT_CODE_RUNTIME_DIR` (canonical) or legacy
/// `$NEXT_CODE_RUNTIME_DIR`.
""",
    """/// Can be overridden with `$NEXT_CODE_RUNTIME_DIR`.
""",
)

old_next = """/// Resolve the next-code home directory.
///
/// Resolution order:
/// 1. `$NEXT_CODE_HOME` (canonical)
/// 2. `$JCODE_HOME` (legacy dual-read)
/// 3. `~/.next-code`, migrating from legacy `~/.jcode` when the new dir is missing
///    and the legacy dir exists.
///
/// On a successful one-shot migrate, a `.migrated-from-jcode` marker is written
/// and a single log line is emitted.
pub fn next_code_dir() -> Result<PathBuf> {
    if let Ok(path) = next_code_core::env::product_env("HOME") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory"))?;
    let new_dir = home.join(".next-code");
    let legacy_dir = home.join(".jcode"); // legacy dual-read home

    if !new_dir.exists() && legacy_dir.exists() {
        migrate_legacy_home(&legacy_dir, &new_dir);
    }

    Ok(new_dir)
}

/// Deprecated alias for [`next_code_dir`]. Prefer the new name.
#[deprecated(note = "use next_code_dir instead")]
pub fn jcode_dir() -> Result<PathBuf> {
    next_code_dir()
}

/// Project-local product directory names, newest first.
///
/// Prefer `.next-code/`, fall back to legacy `.next-code/`. Competitor paths such as
/// `.claude/` are intentionally **not** included here — callers that dual-read
/// those keep their own explicit fallbacks.
pub const PROJECT_DIR_CANDIDATES: &[&str] = &[".next-code", ".next-code"];
"""

new_next = """/// Resolve the next-code home directory.
///
/// Resolution order:
/// 1. `$NEXT_CODE_HOME`
/// 2. `~/.next-code`
pub fn next_code_dir() -> Result<PathBuf> {
    if let Ok(path) = next_code_core::env::product_env("HOME") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory"))?;
    Ok(home.join(".next-code"))
}

/// Project-local product directory name.
pub const PROJECT_DIR_CANDIDATES: &[&str] = &[".next-code"];
"""

if old_next not in text:
    raise SystemExit("next_code_dir block not found")
text = text.replace(old_next, new_next)

# Drop migrate helpers between project_product_dir and logs_dir.
m = re.search(
    r"(pub fn project_product_dir\(root: &Path\) -> PathBuf \{.*?\n\}\n\n)"
    r"/// Best-effort migrate.*?^pub fn logs_dir",
    text,
    flags=re.S | re.M,
)
if not m:
    raise SystemExit("migrate block not found")
text = text[: m.start()] + m.group(1) + "pub fn logs_dir" + text[m.end() :]

# Simplify project_product_dir docs (already rewritten by keeping fn body).
text = text.replace(
    """/// Resolve the project-local product root directory itself
/// (`.next-code` preferred, `.next-code` fallback).
///
/// Returns the first existing candidate, or the canonical `.next-code` path
/// when neither exists.
""",
    """/// Resolve the project-local product root directory (`.next-code`).
""",
)

text = text.replace(
    """/// Tries `<root>/.next-code/<relative>` first, then `<root>/.next-code/<relative>`.
/// Returns the first candidate that exists. When neither exists, returns the
/// canonical `.next-code` path so callers that create the path write to the
/// new name.
""",
    """/// Returns `<root>/.next-code/<relative>` when present, otherwise the canonical
/// `.next-code` path so callers that create the path write there.
""",
)

# Remove legacy_app_config_dir if present (duplicate of app_config_dir after rewrite).
text = re.sub(
    r"\n/// Legacy app config dir.*?pub fn legacy_app_config_dir\(\) -> Result<PathBuf> \{.*?\n\}\n",
    "\n",
    text,
    count=1,
    flags=re.S,
)

text = text.replace(
    """/// Dual-reads the legacy `~/.config/jcode` path only for existence-sensitive
/// callers via [`legacy_app_config_dir`]; new writes go to `next-code`.
""",
    "",
)
text = text.replace(
    """/// Default location is the platform config dir + `next-code` (for example
/// `~/.config/next-code` on Linux). When `NEXT_CODE_HOME` / `NEXT_CODE_HOME` is
/// set, sandbox this under `$HOME/config/next-code` so self-dev/tests do not
/// leak into the user's real config directory.
""",
    """/// Default location is the platform config dir + `next-code` (for example
/// `~/.config/next-code` on Linux). When `NEXT_CODE_HOME` is set, sandbox this
/// under `$HOME/config/next-code` so self-dev/tests do not leak into the user's
/// real config directory.
""",
)
text = text.replace(
    """/// to `~/.next-code/state` (respecting `NEXT_CODE_HOME` / legacy `NEXT_CODE_HOME`).
///
/// When `NEXT_CODE_RUNTIME_DIR` / `NEXT_CODE_RUNTIME_DIR` is set (tests and
""",
    """/// to `~/.next-code/state` (respecting `NEXT_CODE_HOME`).
///
/// When `NEXT_CODE_RUNTIME_DIR` is set (tests and
""",
)
text = text.replace(
    """/// `$NEXT_CODE_HOME/external/` (or legacy `$NEXT_CODE_HOME/external/`) when a
""",
    """/// `$NEXT_CODE_HOME/external/` when a
""",
)
text = text.replace(
    """/// the new `next-code` and legacy `next-code` config segments when both exist.
pub fn harden_user_config_permissions() {
    if let Some(config_dir) = dirs::config_dir() {
        for segment in ["next-code", "next-code"] {
""",
    """/// the `next-code` config segment when it exists.
pub fn harden_user_config_permissions() {
    if let Some(config_dir) = dirs::config_dir() {
        for segment in ["next-code"] {
""",
)

# Rewrite tests: drop dual-read / migrate cases.
old_tests = """    #[test]
    fn prefers_next_code_home() {
        let _g = lock_env();
        clear_home_env();
        let temp = tempfile::tempdir().expect("tempdir");
        let preferred = temp.path().join("preferred");
        std::fs::create_dir_all(&preferred).unwrap();
        next_code_core::env::set_var("NEXT_CODE_HOME", &preferred);
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path().join("legacy"));
        let got = next_code_dir().unwrap();
        assert_eq!(got, preferred);
        clear_home_env();
    }

    #[test]
    fn falls_back_next_code_home() {
        let _g = lock_env();
        clear_home_env();
        let temp = tempfile::tempdir().expect("tempdir");
        let legacy = temp.path().join("legacy-home");
        std::fs::create_dir_all(&legacy).unwrap();
        next_code_core::env::set_var("NEXT_CODE_HOME", &legacy);
        let got = next_code_dir().unwrap();
        assert_eq!(got, legacy);
        clear_home_env();
    }

    #[test]
    fn migrates_legacy_home() {
        let _g = lock_env();
        clear_home_env();
        // Point HOME at a temp sandbox so we never touch the real user home.
        let sandbox = tempfile::tempdir().expect("sandbox");
        let prev_home = std::env::var_os("HOME");
        next_code_core::env::set_var("HOME", sandbox.path());

        let legacy = sandbox.path().join(".next-code");
        let new_dir = sandbox.path().join(".next-code");
        std::fs::create_dir_all(legacy.join("sessions")).unwrap();
        std::fs::write(legacy.join("sessions").join("a.json"), b"{}").unwrap();

        assert!(!new_dir.exists());
        let got = next_code_dir().unwrap();
        assert_eq!(got, new_dir);
        assert!(new_dir.exists(), "migrated home must exist");
        assert!(
            new_dir.join("sessions").join("a.json").exists(),
            "session data must migrate"
        );
        assert!(
            new_dir.join(MIGRATE_MARKER).exists(),
            "marker must be written"
        );

        match prev_home {
            Some(h) => next_code_core::env::set_var("HOME", h),
            None => next_code_core::env::remove_var("HOME"),
        }
        clear_home_env();
    }

    #[test]
    fn fresh_user_gets_next_code() {
"""

new_tests = """    #[test]
    fn prefers_next_code_home() {
        let _g = lock_env();
        clear_home_env();
        let temp = tempfile::tempdir().expect("tempdir");
        let preferred = temp.path().join("preferred");
        std::fs::create_dir_all(&preferred).unwrap();
        next_code_core::env::set_var("NEXT_CODE_HOME", &preferred);
        let got = next_code_dir().unwrap();
        assert_eq!(got, preferred);
        clear_home_env();
    }

    #[test]
    fn fresh_user_gets_next_code() {
"""

if old_tests not in text:
    raise SystemExit("tests block not found")
text = text.replace(old_tests, new_tests)

text = text.replace(
    """    fn runtime_dir_dual_reads_legacy() {
        let _g = lock_env();
        next_code_core::env::remove_var("NEXT_CODE_RUNTIME_DIR");
        let temp = tempfile::tempdir().expect("tempdir");
        next_code_core::env::set_var("NEXT_CODE_RUNTIME_DIR", temp.path());
        assert_eq!(runtime_dir(), temp.path());
        next_code_core::env::remove_var("NEXT_CODE_RUNTIME_DIR");
    }
""",
    """    fn runtime_dir_reads_next_code() {
        let _g = lock_env();
        next_code_core::env::remove_var("NEXT_CODE_RUNTIME_DIR");
        let temp = tempfile::tempdir().expect("tempdir");
        next_code_core::env::set_var("NEXT_CODE_RUNTIME_DIR", temp.path());
        assert_eq!(runtime_dir(), temp.path());
        next_code_core::env::remove_var("NEXT_CODE_RUNTIME_DIR");
    }
""",
)

path.write_text(text, encoding="utf-8")
print("wrote", path)
