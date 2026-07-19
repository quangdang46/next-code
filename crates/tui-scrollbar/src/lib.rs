/// Scrollbar widget stub for Grok compatibility.
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

/// Scroll direction enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Top,
    Bottom,
}

/// Subcell constant — Grok uses usize arithmetic.
pub const SUBCELL: usize = 1;

/// Scrollbar type alias.
pub type ScrollBar = Scrollbar;

/// Represents a scrollbar instance.
#[derive(Debug, Clone)]
pub struct Scrollbar {
    pub orientation: ScrollDirection,
    pub thumb_style: Style,
    pub track_style: Option<Style>,
}

impl Scrollbar {
    pub fn new(_orientation: ScrollDirection) -> Self {
        Self { orientation: ScrollDirection::Up, thumb_style: Style::default(), track_style: None }
    }
    pub fn vertical(lengths: ScrollLengths) -> Self {
        let _ = lengths;
        Self { orientation: ScrollDirection::Up, thumb_style: Style::default(), track_style: None }
    }
    pub fn offset(mut self, _offset: usize) -> Self { self }
    pub fn thumb_style(mut self, style: Style) -> Self { self.thumb_style = style; self }
    pub fn track_style(mut self, style: Option<Style>) -> Self { self.track_style = style; self }
    pub fn render(&self, _area: Rect, _buf: &mut Buffer) {}
}

/// Scrollbar state.
#[derive(Debug, Clone)]
pub struct ScrollbarState {
    pub position: usize,
    pub content_length: usize,
    pub viewport_length: usize,
}

impl ScrollbarState {
    pub fn new() -> Self {
        Self { position: 0, content_length: 0, viewport_length: 0 }
    }
    pub fn position(mut self, pos: usize) -> Self { self.position = pos; self }
    pub fn content_length(mut self, len: usize) -> Self { self.content_length = len; self }
    pub fn viewport_length(mut self, len: usize) -> Self { self.viewport_length = len; self }
    pub fn next(&mut self) {}
}

/// Scroll lengths (Grok compatibility).
#[derive(Debug, Clone, Copy)]
pub struct ScrollLengths {
    pub content_len: usize,
    pub viewport_len: usize,
}

impl ScrollLengths {
    pub fn new(content_len: usize, viewport_len: usize) -> Self {
        Self { content_len, viewport_len }
    }
}

/// Scroll metrics (Grok compatibility).
#[derive(Debug, Clone, Copy)]
pub struct ScrollMetrics {
    pub scroll_offset: f64,
    pub thumb_size: f64,
    pub content_len: usize,
    pub viewport_len: usize,
    pub position: usize,
    pub thumb_len: usize,
}

impl ScrollMetrics {
    pub fn new(lengths: ScrollLengths, position: usize, viewport: u16) -> Self {
        Self {
            scroll_offset: 0.0,
            thumb_size: 1.0,
            content_len: lengths.content_len,
            viewport_len: lengths.viewport_len,
            position,
            thumb_len: 1,
        }
    }
    pub fn thumb_len(&self) -> usize { self.thumb_len }
    pub fn offset_for_thumb_start(&self, _thumb_start: usize) -> usize { self.position }
}
