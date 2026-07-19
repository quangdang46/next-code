//! Theme backwards-compatibility wrapper.
//! All free functions now delegate to Theme::current() from the theme module.
//! Spinner/utility functions are kept in-place (they don't depend on Theme).

use crate::theme_mod::Theme;
use crate::color;
use crate::color::rgb;
use ratatui::prelude::*;

// ─── Backward-compatible color functions ───────────────────────────────
// These delegate to Theme::current() so existing callers compile unchanged.

pub fn user_color() -> Color { Theme::current().accent_user }
pub fn ai_color() -> Color { Theme::current().accent_assistant }
pub fn tool_color() -> Color { Theme::current().accent_tool }
pub fn file_link_color() -> Color { Theme::current().link_fg }
pub fn dim_color() -> Color { Theme::current().gray }
pub fn accent_color() -> Color { Theme::current().accent_assistant }
pub fn system_message_color() -> Color { Theme::current().accent_system }
pub fn queued_color() -> Color { Theme::current().warning }
pub fn asap_color() -> Color { Theme::current().running }
pub fn pending_color() -> Color { Theme::current().gray_dim }
pub fn user_text() -> Color { Theme::current().text_secondary }
pub fn user_bg() -> Color { Theme::current().bg_highlight }
pub fn ai_text() -> Color { Theme::current().text_primary }
pub fn header_icon_color() -> Color { rgb(120, 210, 230) }
pub fn header_name_color() -> Color { Theme::current().text_primary }
pub fn header_session_color() -> Color { Theme::current().text_primary }

// ─── Spinner / animation utilities (unchanged) ────────────────────────

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const LIVENESS_INDICATOR_FPS: f32 = 1.5;
pub const LIVENESS_SPINNER_FPS: f32 = 12.5;

pub fn spinner_frame_index(elapsed: f32, fps: f32) -> usize {
    ((elapsed * fps) as usize) % SPINNER_FRAMES.len()
}

pub fn spinner_frame(elapsed: f32, fps: f32) -> &'static str {
    SPINNER_FRAMES[spinner_frame_index(elapsed, fps)]
}

pub fn activity_indicator_frame_index(elapsed: f32, fps: f32, enable_decorative_animations: bool) -> usize {
    if enable_decorative_animations {
        spinner_frame_index(elapsed, fps)
    } else {
        spinner_frame_index(elapsed, LIVENESS_SPINNER_FPS)
    }
}

pub fn activity_indicator(elapsed: f32, fps: f32, enable_decorative_animations: bool) -> &'static str {
    SPINNER_FRAMES[activity_indicator_frame_index(elapsed, fps, enable_decorative_animations)]
}

pub fn color_to_floats(c: Color, fallback: (f32, f32, f32)) -> (f32, f32, f32) {
    match c {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        Color::Indexed(n) => {
            let (r, g, b) = color::indexed_to_rgb(n);
            (r as f32, g as f32, b as f32)
        }
        _ => fallback,
    }
}

pub fn blend_color(from: Color, to: Color, t: f32) -> Color {
    let (fr, fg, fb) = color_to_floats(from, (80.0, 80.0, 80.0));
    let (tr, tg, tb) = color_to_floats(to, (200.0, 200.0, 200.0));
    let r = fr + (tr - fr) * t;
    let g = fg + (tg - fg) * t;
    let b = fb + (tb - fb) * t;
    rgb(r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8)
}

pub fn rainbow_prompt_color(distance: usize) -> Color {
    const RAINBOW: [(u8, u8, u8); 7] = [
        (255, 80, 80), (255, 160, 80), (255, 230, 80),
        (80, 220, 100), (80, 200, 220), (100, 140, 255), (180, 100, 255),
    ];
    const GRAY: (u8, u8, u8) = (80, 80, 80);
    let decay = (-0.4 * distance as f32).exp();
    let idx = distance.min(RAINBOW.len() - 1);
    let (r, g, b) = RAINBOW[idx];
    let blend_fn = |c: u8, g: u8| -> u8 { (c as f32 * decay + g as f32 * (1.0 - decay)) as u8 };
    rgb(blend_fn(r, GRAY.0), blend_fn(g, GRAY.1), blend_fn(b, GRAY.2))
}

pub fn prompt_entry_color(base: Color, t: f32) -> Color {
    let peak = rgb(255, 230, 120);
    let phase = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
    blend_color(base, peak, phase.clamp(0.0, 1.0) * 0.7)
}

pub fn prompt_entry_bg_color(base: Color, t: f32) -> Color {
    let spotlight = rgb(58, 66, 82);
    let ease_in = 1.0 - (1.0 - t).powi(3);
    let ease_out = (1.0 - t).powi(2);
    let phase = (ease_in * ease_out * 1.65).clamp(0.0, 1.0);
    blend_color(base, spotlight, phase * 0.85)
}

pub fn prompt_entry_shimmer_color(base: Color, pos: f32, t: f32) -> Color {
    let travel = (t * 1.15).clamp(0.0, 1.0);
    let width = 0.18;
    let dist = (pos - travel).abs();
    let shimmer = (1.0 - (dist / width).clamp(0.0, 1.0)).powf(2.2);
    let pulse = (1.0 - t).powf(0.55);
    let highlight = rgb(255, 248, 210);
    blend_color(base, highlight, shimmer * pulse * 0.7)
}

pub fn animated_tool_color(elapsed: f32, enable_decorative_animations: bool) -> Color {
    if !enable_decorative_animations { return tool_color(); }
    let t = (elapsed * 2.0).sin() * 0.5 + 0.5;
    let r = (80.0 + t * 106.0) as u8;
    let g = (200.0 - t * 61.0) as u8;
    let b = (220.0 + t * 35.0) as u8;
    rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_frames_are_circular_braille_sequence() {
        assert_eq!(SPINNER_FRAMES, &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);
    }

    #[test]
    fn spinner_frame_wraps_at_sequence_length() {
        assert_eq!(spinner_frame(0.0, 10.0), "⠋");
        assert_eq!(spinner_frame(0.9, 10.0), "⠏");
        assert_eq!(spinner_frame(1.0, 10.0), "⠋");
    }

    #[test]
    fn activity_indicator_still_advances_without_decorative_animations() {
        let first = activity_indicator(0.0, 12.5, false);
        let later = activity_indicator(1.0, 12.5, false);
        assert!(SPINNER_FRAMES.contains(&first));
        assert_ne!(first, later);
    }
}
