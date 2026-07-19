//! System appearance detection for auto day/night theming.
//! Copied/adapted from grok-build system_appearance.rs.
//! Uses `dark-light` crate for cross-platform detection.

use super::ThemeKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAppearance {
    Light,
    Dark,
}

/// Detect the current system appearance using dark-light crate.
pub fn detect() -> Option<SystemAppearance> {
    match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => Some(SystemAppearance::Dark),
        Ok(dark_light::Mode::Light) => Some(SystemAppearance::Light),
        _ => None,
    }
}

/// Convert a SystemAppearance + optional dark_theme/light_theme → ThemeKind.
pub fn to_theme_kind(appearance: SystemAppearance) -> ThemeKind {
    match appearance {
        SystemAppearance::Dark => ThemeKind::DefaultGrokNight,
        SystemAppearance::Light => ThemeKind::DefaultGrokDay,
    }
}
