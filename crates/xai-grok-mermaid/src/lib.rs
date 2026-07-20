//! Facade of `xai-org/grok-build` `xai-grok-mermaid` (Apache-2.0) for the
//! next-code Grok Face migration (PR7).
//!
//! Upstream is a full Mermaid → PNG pipeline. This stub only reproduces the
//! types and entry points the pager imports; [`render_checked`] and
//! [`run_with_timeout`] always fail.

use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

/// Why a diagram failed to render.
#[derive(thiserror::Error, Debug)]
pub enum MermaidError {
    /// The source could not be parsed into a diagram.
    #[error("mermaid parse error: {0}")]
    Parse(String),
    /// The diagram parsed but layout failed.
    #[error("mermaid layout error: {0}")]
    Layout(String),
    /// The SVG could not be rasterized to PNG.
    #[error("mermaid rasterize error: {0}")]
    Rasterize(String),
    /// An external engine exceeded its wall-clock budget.
    #[error("mermaid render timed out")]
    Timeout,
    /// The engine cannot render this input.
    #[error("mermaid render unsupported: {0}")]
    Unsupported(String),
    /// The engine panicked and [`render_checked`] caught it.
    #[error("mermaid engine panicked: {0}")]
    Panic(String),
}

/// Caps applied by [`render_checked`] before the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderLimits {
    /// Maximum accepted source length in bytes.
    pub max_source_bytes: usize,
}

impl Default for RenderLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 64 * 1024,
        }
    }
}

/// Which color scheme a diagram should be rendered for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MermaidTheme {
    /// Light surfaces with dark text.
    #[default]
    Light,
    /// Dark surfaces with light text.
    Dark,
}

/// A straight 8-bit-per-channel, non-premultiplied RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    /// Red channel, 0–255.
    pub r: u8,
    /// Green channel, 0–255.
    pub g: u8,
    /// Blue channel, 0–255.
    pub b: u8,
    /// Alpha channel, 0–255.
    pub a: u8,
}

impl Rgba {
    /// Construct an [`Rgba`] from its four channels.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl MermaidTheme {
    /// The default opaque surface color a diagram blends into for this theme.
    pub fn surface_background(self) -> Rgba {
        match self {
            MermaidTheme::Light => Rgba::new(0xFA, 0xFA, 0xFA, 0xFF),
            MermaidTheme::Dark => Rgba::new(0x18, 0x18, 0x1B, 0xFF),
        }
    }
}

/// Parameters controlling a single diagram render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderParams {
    /// Color scheme to render for.
    pub theme: MermaidTheme,
    /// Target output width in pixels.
    pub target_width_px: u32,
    /// Hard ceiling on output height in pixels.
    pub max_height_px: u32,
    /// Oversample factor when `target_width_px == 0`.
    pub scale: f32,
    /// Minimum output width in pixels.
    pub min_width_px: u32,
    /// Opaque background fill.
    pub background: Option<Rgba>,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            theme: MermaidTheme::Light,
            target_width_px: 1024,
            max_height_px: 4096,
            scale: 1.0,
            min_width_px: 0,
            background: None,
        }
    }
}

impl RenderParams {
    /// Sizing tuned for opening a PNG in an OS image viewer.
    pub fn for_os_viewer(theme: MermaidTheme, min_width_px: u32, max_height_px: u32) -> Self {
        Self {
            theme,
            target_width_px: 0,
            max_height_px,
            scale: 2.0,
            min_width_px,
            background: Some(theme.surface_background()),
        }
    }
}

/// A rendered diagram: PNG bytes plus the exact raster dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedDiagram {
    /// The encoded PNG image.
    pub png: Vec<u8>,
    /// Output width in pixels.
    pub width_px: u32,
    /// Output height in pixels.
    pub height_px: u32,
}

/// A pluggable Mermaid rendering backend.
pub trait MermaidEngine: Send + Sync {
    /// Render `source` to a PNG using `params`.
    fn render(&self, source: &str, params: &RenderParams) -> Result<RenderedDiagram, MermaidError>;
}

struct StubEngine;

impl MermaidEngine for StubEngine {
    fn render(
        &self,
        _source: &str,
        _params: &RenderParams,
    ) -> Result<RenderedDiagram, MermaidError> {
        Err(MermaidError::Unsupported(
            "xai-grok-mermaid stub: rendering not available".into(),
        ))
    }
}

/// Construct the default engine (stub — always fails on render).
pub fn default_engine() -> Arc<dyn MermaidEngine> {
    Arc::new(StubEngine)
}

/// Render entry point. Stub always returns [`MermaidError::Unsupported`].
pub fn render_checked(
    _engine: &dyn MermaidEngine,
    _source: &str,
    _params: &RenderParams,
    _limits: &RenderLimits,
) -> Result<RenderedDiagram, MermaidError> {
    Err(MermaidError::Unsupported(
        "xai-grok-mermaid stub: rendering not available".into(),
    ))
}

/// Why a child subprocess run did not complete successfully.
#[derive(thiserror::Error, Debug)]
pub enum SubprocessError {
    /// The child could not be spawned.
    #[error("could not spawn child process: {0}")]
    Spawn(std::io::Error),
    /// The child exceeded its wall-clock budget.
    #[error("child process timed out")]
    Timeout,
    /// The child exited non-zero.
    #[error("child process exited with {0}")]
    NonZeroExit(std::process::ExitStatus),
    /// Waiting on the child failed.
    #[error("waiting on child process failed: {0}")]
    Wait(std::io::Error),
}

/// Spawn + wait helper. Stub always returns [`SubprocessError::Timeout`].
pub fn run_with_timeout(
    _cmd: Command,
    _stdin_payload: Option<&[u8]>,
    _timeout: Duration,
) -> Result<(), SubprocessError> {
    Err(SubprocessError::Timeout)
}
