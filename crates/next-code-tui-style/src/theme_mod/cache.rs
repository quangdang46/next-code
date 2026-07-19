//! In-memory theme cache + resolution.
//! Copied/adapted from grok-build cache.rs.
//! Simplified for Phase A: only AtomicU8 for CURRENT + resolve_initial_theme().

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use super::ThemeKind;

/// In-memory theme kind, encoded as a u8.
static CURRENT: AtomicU8 = AtomicU8::new(ThemeKind::DefaultGrokNight as u8);
static LOADED: AtomicBool = AtomicBool::new(false);

/// Decode u8 → ThemeKind.
fn theme_kind_from_u8(byte: u8) -> ThemeKind {
    match byte {
        x if x == ThemeKind::DefaultGrokNight as u8 => ThemeKind::DefaultGrokNight,
        x if x == ThemeKind::DefaultGrokDay as u8 => ThemeKind::DefaultGrokDay,
        x if x == ThemeKind::Auto as u8 => ThemeKind::Auto,
        _ => ThemeKind::DefaultGrokNight,
    }
}

/// Get the current theme kind, seeding from config on first call.
pub fn current_kind() -> ThemeKind {
    if !LOADED.load(Ordering::Acquire) {
        let kind = resolve_initial_theme();
        set(kind);
    }
    theme_kind_from_u8(CURRENT.load(Ordering::Relaxed))
}

/// Set the current theme kind.
pub fn set(kind: ThemeKind) {
    CURRENT.store(kind as u8, Ordering::Relaxed);
    LOADED.store(true, Ordering::Release);
}

/// Resolve the effective theme on startup.
/// Precedence:
/// 1. NEXT_CODE_THEME environment variable
/// 2. Auto detect (macOS AppleInterfaceStyle)
/// 3. Default: DefaultGrokNight
pub fn resolve_initial_theme() -> ThemeKind {
    // 1) Env var
    if let Ok(val) = std::env::var("NEXT_CODE_THEME") {
        let val = val.trim().to_ascii_lowercase();
        if let Some(kind) = ThemeKind::from_name(&val) {
            if kind != ThemeKind::Auto {
                return kind;
            }
        }
    }
    // Auto mode: detect OS appearance (simple env var check for now)
    #[cfg(target_os = "macos")]
    {
        if let Ok(val) = std::env::var("AppleInterfaceStyle") {
            return if val.to_ascii_lowercase().contains("dark") {
                ThemeKind::DefaultGrokNight
            } else {
                ThemeKind::DefaultGrokDay
            };
        }
    }
    ThemeKind::DefaultGrokNight
}
