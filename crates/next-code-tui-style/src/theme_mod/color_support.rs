//! Terminal color support detection and quantization.
//! Copied/adapted from grok-build color_support.rs.

use std::sync::OnceLock;

use ratatui::style::Color;

use crate::color::indexed_to_rgb;

/// Terminal color support level (ordered low → high).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorLevel {
    None,
    Basic,
    Ansi256,
    TrueColor,
}

impl ColorLevel {
    pub fn has_color(self) -> bool { self >= Self::Basic }
    pub fn has_256(self) -> bool { self >= Self::Ansi256 }
    pub fn has_truecolor(self) -> bool { self >= Self::TrueColor }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Basic => "basic",
            Self::Ansi256 => "256",
            Self::TrueColor => "truecolor",
        }
    }
}

impl std::fmt::Display for ColorLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

static COLOR_LEVEL: OnceLock<ColorLevel> = OnceLock::new();

pub fn detect() -> ColorLevel {
    *COLOR_LEVEL.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            return ColorLevel::None;
        }
        // Use next-code's existing color capability detection
        let cap = crate::color::color_capability();
        match cap {
            crate::color::ColorCapability::TrueColor => ColorLevel::TrueColor,
            crate::color::ColorCapability::Color256 => ColorLevel::Ansi256,
        }
    })
}

pub fn get() -> ColorLevel { detect() }

pub fn set(level: ColorLevel) -> Result<(), ColorLevel> {
    COLOR_LEVEL.set(level)
}

/// Quantize a Color to the terminal's supported level.
pub fn quantize_color(color: Color, level: ColorLevel) -> Color {
    match level {
        ColorLevel::TrueColor => color,
        ColorLevel::Ansi256 => match color {
            Color::Rgb(r, g, b) => Color::Indexed(nearest_indexed(r, g, b)),
            other => other,
        },
        ColorLevel::Basic => match color {
            Color::Rgb(r, g, b) => indexed_to_ansi16(nearest_indexed(r, g, b)),
            Color::Indexed(n) => indexed_to_ansi16(n),
            other => other,
        },
        ColorLevel::None => Color::Reset,
    }
}

pub fn quantize(color: Color) -> Color {
    quantize_color(color, get())
}

/// Find the nearest 256-color index for an RGB triplet.
fn nearest_indexed(r: u8, g: u8, b: u8) -> u8 {
    let mut best = 0u8;
    let mut best_dist = u32::MAX;

    // 6x6x6 color cube (indices 16–231)
    for ri in 0..6 {
        for gi in 0..6 {
            for bi in 0..6 {
                let pr = if ri == 0 { 0 } else { 95 + (ri - 1) * 40 };
                let pg = if gi == 0 { 0 } else { 95 + (gi - 1) * 40 };
                let pb = if bi == 0 { 0 } else { 95 + (bi - 1) * 40 };
                let idx = 16 + (ri * 36 + gi * 6 + bi);
                let dr = r as i32 - pr;
                let dg = g as i32 - pg;
                let db = b as i32 - pb;
                let dist = (dr * dr + dg * dg + db * db) as u32;
                if dist < best_dist {
                    best_dist = dist;
                    best = idx as u8;
                }
            }
        }
    }

    // Grayscale ramp (indices 232–255)
    for gi in 0..24 {
        let pg = 8 + gi * 10;
        let idx = 232 + gi;
        let dr = r as i32 - pg;
        let dg = g as i32 - pg;
        let db = b as i32 - pg;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = idx as u8;
        }
    }

    best
}

/// Map a 256-color index to the nearest basic ANSI 16 color.
fn indexed_to_ansi16(n: u8) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        _ => {
            let (rr, gg, bb) = indexed_to_rgb(n);
            rgb_to_ansi16(rr, gg, bb)
        }
    }
}

fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> Color {
    const PALETTE: [(u8, u8, u8, Color); 16] = [
        (0, 0, 0, Color::Black),
        (128, 0, 0, Color::Red),
        (0, 128, 0, Color::Green),
        (128, 128, 0, Color::Yellow),
        (0, 0, 128, Color::Blue),
        (128, 0, 128, Color::Magenta),
        (0, 128, 128, Color::Cyan),
        (192, 192, 192, Color::White),
        (128, 128, 128, Color::DarkGray),
        (255, 0, 0, Color::LightRed),
        (0, 255, 0, Color::LightGreen),
        (255, 255, 0, Color::LightYellow),
        (0, 0, 255, Color::LightBlue),
        (255, 0, 255, Color::LightMagenta),
        (0, 255, 255, Color::LightCyan),
        (255, 255, 255, Color::White),
    ];
    let mut best = Color::White;
    let mut best_dist = u32::MAX;
    for &(pr, pg, pb, color) in &PALETTE {
        let dr = r as i32 - pr as i32;
        let dg = g as i32 - pg as i32;
        let db = b as i32 - pb as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = color;
        }
    }
    best
}
