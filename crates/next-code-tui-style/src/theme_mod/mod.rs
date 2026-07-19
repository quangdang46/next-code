//! Theme system for next-code — copied/adapted from grok-build.
//! All colors come from the `Theme` struct via Theme::current().

mod cache;
mod color_support;
mod grokday;
mod groknight;
pub mod md_style;
mod osc11;
mod oscura;
mod rosepine;
mod struct_def;
mod system_appearance;
mod terminal_default;
mod tokyonight;

pub use struct_def::Theme;

/// Available theme variants (matches grok-build's 5 themes + Auto).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeKind {
    DefaultGrokNight = 0,
    DefaultGrokDay = 1,
    TokyoNight = 2,
    RosePineMoon = 3,
    OscuraMidnight = 5,
    Auto = 4,
}

impl ThemeKind {
    /// All concrete theme kinds.
    pub const ALL: &[ThemeKind] = &[
        ThemeKind::DefaultGrokNight,
        ThemeKind::DefaultGrokDay,
        ThemeKind::TokyoNight,
        ThemeKind::RosePineMoon,
        ThemeKind::OscuraMidnight,
    ];

    /// Human-readable display name.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::DefaultGrokNight => "groknight",
            Self::DefaultGrokDay => "grokday",
            Self::TokyoNight => "tokyonight",
            Self::RosePineMoon => "rosepine",
            Self::OscuraMidnight => "oscura",
            Self::Auto => "auto",
        }
    }

    /// Parse a theme name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        match lower.as_str() {
            "auto" | "system" => Some(Self::Auto),
            "groknight" | "grok-night" | "dark" => Some(Self::DefaultGrokNight),
            "grokday" | "grok-day" | "light" | "day" => Some(Self::DefaultGrokDay),
            "tokyonight" | "tokyo-night" | "tokyo" => Some(Self::TokyoNight),
            "rosepine" | "rose-pine" | "rosepine-moon" => Some(Self::RosePineMoon),
            "oscura" | "oscura-midnight" => Some(Self::OscuraMidnight),
            _ => None,
        }
    }

    /// Whether this is the meta "auto" variant.
    pub fn is_auto(self) -> bool {
        self == Self::Auto
    }
}

impl Theme {
    /// Get the current theme, initialized from cache.
    pub fn current() -> Self {
        let kind = cache::current_kind();
        match kind {
            ThemeKind::DefaultGrokNight => Self::groknight(),
            ThemeKind::DefaultGrokDay => Self::grokday(),
            ThemeKind::TokyoNight => Self::tokyonight(),
            ThemeKind::RosePineMoon => Self::rosepine_moon(),
            ThemeKind::OscuraMidnight => Self::oscura_midnight(),
            ThemeKind::Auto => Self::groknight(),
        }
    }

    /// Get the currently active theme kind.
    pub fn current_kind() -> ThemeKind {
        cache::current_kind()
    }

    /// Apply a theme kind — updates the in-memory cache.
    pub fn apply_kind(kind: ThemeKind) {
        if kind.is_auto() {
            let resolved = resolve_auto_theme();
            cache::set(resolved);
        } else {
            cache::set(kind);
        }
    }

    /// Set theme kind directly.
    pub fn set_kind(kind: ThemeKind) {
        cache::set(kind);
    }
}

/// Resolve auto theme based on OS appearance.
fn resolve_auto_theme() -> ThemeKind {
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

/// Initialize theme from env (for startup).
pub fn init_theme_from_env() {
    let kind = cache::resolve_initial_theme();
    cache::set(kind);
}
