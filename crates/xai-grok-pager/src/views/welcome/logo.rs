//! Logo component — next-code animated idle logo (donut / orbit rings).
//!
//! Replaces Grok braille `assets/logo/*.txt`. Uses pure math from
//! `next-code-tui-anim` and blits into the Face ratatui 0.28 buffer.

use std::cell::RefCell;
use std::time::Instant;

use next_code_tui_anim::{hsv_to_rgb, sample_donut, sample_orbit_rings, shape_char_3x3};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::theme::Theme;

/// Height at or above which the compact logo is shown (below it, no logo).
const SMALL_LOGO_MIN_HEIGHT: u16 = 18;
/// Height at or above which the full logo is shown.
const FULL_LOGO_MIN_HEIGHT: u16 = 24;

const FULL_ROWS: u16 = 12;
const FULL_COLS: u16 = 40;
const COMPACT_ROWS: u16 = 8;
const COMPACT_COLS: u16 = 28;

/// Animation phase in seconds since the first render.
fn anim_phase_secs() -> f32 {
    use std::sync::OnceLock;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

const ANIM_FPS: f32 = 20.0;

/// Quantized animation frame for welcome redraw throttling.
pub fn shimmer_frame() -> u64 {
    (anim_phase_secs() * ANIM_FPS) as u64
}

fn logo_tier(window_height: u16) -> Option<(u16, u16)> {
    if window_height < SMALL_LOGO_MIN_HEIGHT {
        None
    } else if window_height < FULL_LOGO_MIN_HEIGHT {
        Some((COMPACT_ROWS, COMPACT_COLS))
    } else {
        Some((FULL_ROWS, FULL_COLS))
    }
}

pub fn logo_line_count(window_height: u16) -> u16 {
    logo_tier(window_height).map(|(rows, _)| rows).unwrap_or(0)
}

pub fn logo_visual_width(window_height: u16) -> u16 {
    logo_tier(window_height).map(|(_, cols)| cols).unwrap_or(24)
}

pub fn full_logo_line_count() -> u16 {
    FULL_ROWS
}

pub fn full_logo_visual_width() -> u16 {
    FULL_COLS
}

pub fn compact_logo_line_count() -> u16 {
    COMPACT_ROWS
}

pub fn render_logo(area: Rect, buf: &mut Buffer, _theme: &Theme, window_height: u16) {
    if let Some((rows, cols)) = logo_tier(window_height) {
        render_anim(area, buf, rows, cols);
    }
}

pub fn render_full_logo(area: Rect, buf: &mut Buffer, _theme: &Theme) {
    render_anim(area, buf, FULL_ROWS, FULL_COLS);
}

pub fn render_compact_logo(area: Rect, buf: &mut Buffer, _theme: &Theme) {
    render_anim(area, buf, COMPACT_ROWS, COMPACT_COLS);
}

struct IdleBuffers {
    hit: Vec<bool>,
    lum_map: Vec<f32>,
    z_buf: Vec<f32>,
    size: usize,
}

impl IdleBuffers {
    fn resize_and_clear(&mut self, len: usize) {
        if self.size != len {
            self.hit.resize(len, false);
            self.lum_map.resize(len, 0.0);
            self.z_buf.resize(len, 0.0);
            self.size = len;
        }
        self.hit.fill(false);
        self.lum_map.fill(0.0);
        self.z_buf.fill(0.0);
    }
}

thread_local! {
    static IDLE_BUF: RefCell<IdleBuffers> = RefCell::new(IdleBuffers {
        hit: Vec::new(),
        lum_map: Vec::new(),
        z_buf: Vec::new(),
        size: 0,
    });
}

fn render_anim(area: Rect, buf: &mut Buffer, prefer_rows: u16, prefer_cols: u16) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let rows = area.height.min(prefer_rows);
    let cols = area.width.min(prefer_cols);
    let x0 = area.x + area.width.saturating_sub(cols) / 2;
    let y0 = area.y + area.height.saturating_sub(rows) / 2;
    let draw = Rect::new(x0, y0, cols, rows);

    let elapsed = anim_phase_secs();
    let cw = draw.width as usize;
    let ch = draw.height as usize;
    const SUB_X: usize = 3;
    const SUB_Y: usize = 3;
    let sw = cw * SUB_X;
    let sh = ch * SUB_Y;

    IDLE_BUF.with(|cell| {
        let mut bufs = cell.borrow_mut();
        bufs.resize_and_clear(sw * sh);
        let bufs = &mut *bufs;

        // Alternate donut / orbit for visual variety across launches.
        if (elapsed as u64 / 30) % 2 == 0 {
            sample_donut(
                elapsed,
                sw,
                sh,
                &mut bufs.hit,
                &mut bufs.lum_map,
                &mut bufs.z_buf,
            );
        } else {
            sample_orbit_rings(
                elapsed,
                sw,
                sh,
                &mut bufs.hit,
                &mut bufs.lum_map,
                &mut bufs.z_buf,
            );
        }

        blit_idle(buf, draw, &bufs.hit, &bufs.lum_map, sw, elapsed * 40.0);
    });
}

fn blit_idle(
    buf: &mut Buffer,
    area: Rect,
    hit: &[bool],
    lum_map: &[f32],
    sw: usize,
    time_hue: f32,
) {
    const SUB_X: usize = 3;
    const SUB_Y: usize = 3;
    let cw = area.width as usize;
    let ch = area.height as usize;

    for row in 0..ch {
        let y = area.y + row as u16;
        for col in 0..cw {
            let x = area.x + col as u16;

            let mut pattern = 0u16;
            let mut total_lum = 0.0f32;
            let mut hit_count = 0u32;

            for sy in 0..SUB_Y {
                for sx in 0..SUB_X {
                    let px = col * SUB_X + sx;
                    let py = row * SUB_Y + sy;
                    let idx = py * sw + px;
                    if hit[idx] {
                        pattern |= 1 << (sy * SUB_X + sx);
                        total_lum += lum_map[idx];
                        hit_count += 1;
                    }
                }
            }

            let cell = &mut buf[(x, y)];
            if hit_count == 0 {
                cell.set_char(' ');
            } else {
                let avg_lum = total_lum / hit_count as f32;
                let coverage = hit_count as f32 / (SUB_X * SUB_Y) as f32;
                let t = (avg_lum + 1.0) * 0.5;
                let glyph = shape_char_3x3(pattern, t);

                let mut hue = (time_hue + t * 160.0) % 360.0;
                if hue < 0.0 {
                    hue += 360.0;
                }
                let sat = 0.5 + t * 0.4;
                let val = (0.10 + t * t * 0.90) * (0.55 + coverage * 0.45);
                let (r, g, b) = hsv_to_rgb(hue, sat, val);
                cell.set_char(glyph)
                    .set_fg(Color::Rgb(r, g, b));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_sizes_by_height() {
        assert_eq!(logo_line_count(SMALL_LOGO_MIN_HEIGHT - 1), 0);
        assert_eq!(logo_line_count(SMALL_LOGO_MIN_HEIGHT), COMPACT_ROWS);
        assert_eq!(logo_line_count(FULL_LOGO_MIN_HEIGHT), FULL_ROWS);
    }

    #[test]
    fn full_logo_helpers_report_fixed_budget() {
        assert_eq!(full_logo_line_count(), FULL_ROWS);
        assert_eq!(full_logo_visual_width(), FULL_COLS);
        assert!(full_logo_line_count() > compact_logo_line_count());
    }

    #[test]
    fn shimmer_frame_advances() {
        let a = shimmer_frame();
        std::thread::sleep(std::time::Duration::from_millis(60));
        let b = shimmer_frame();
        assert!(b >= a);
    }
}
