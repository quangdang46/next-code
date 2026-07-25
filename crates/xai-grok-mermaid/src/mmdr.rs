//! [`MmdrEngine`]: Mermaid → PNG via `mermaid-rs-renderer` (mmdr).
//!
//! Uses mmdr's Face embed API (`render_png_bytes`, `Theme::face_light` /
//! `face_dark`, hardened bundled-font raster). The pager keeps crash isolation
//! via out-of-process `__mermaid-render` + [`crate::run_with_timeout`].

use mermaid_rs_renderer::{
    LayoutConfig, PngRenderParams, RenderError, RenderOptions, RenderedPng, Theme,
    render_png_bytes,
};

use crate::{MermaidEngine, MermaidError, MermaidTheme, RenderParams, RenderedDiagram, Rgba};

/// Default Face engine: mmdr (`mermaid-rs-renderer`) with the secure PNG path.
#[derive(Debug, Default, Clone, Copy)]
pub struct MmdrEngine;

impl MmdrEngine {
    /// Construct an [`MmdrEngine`].
    pub fn new() -> Self {
        Self
    }
}

impl MermaidEngine for MmdrEngine {
    fn render(&self, source: &str, params: &RenderParams) -> Result<RenderedDiagram, MermaidError> {
        let options = RenderOptions {
            theme: theme_for(params.theme),
            layout: LayoutConfig::default(),
        };
        let png_params = png_params_for(params);
        // Source-byte limits are enforced by [`crate::render_checked`] before
        // the engine runs; mmdr's default cap is the same 64 KiB, so pass
        // through defaults for a second defense-in-depth check.
        let limits = mermaid_rs_renderer::RenderLimits::default();
        let out = render_png_bytes(source, options, &png_params, &limits).map_err(map_render_error)?;
        Ok(to_rendered(out))
    }
}

fn theme_for(theme: MermaidTheme) -> Theme {
    match theme {
        MermaidTheme::Light => Theme::face_light(),
        MermaidTheme::Dark => Theme::face_dark(),
    }
}

fn png_params_for(params: &RenderParams) -> PngRenderParams {
    PngRenderParams {
        target_width_px: params.target_width_px,
        max_height_px: params.max_height_px,
        scale: params.scale,
        min_width_px: params.min_width_px,
        background: params.background.map(map_rgba),
    }
}

fn map_rgba(c: Rgba) -> mermaid_rs_renderer::Rgba {
    mermaid_rs_renderer::Rgba::new(c.r, c.g, c.b, c.a)
}

fn to_rendered(out: RenderedPng) -> RenderedDiagram {
    RenderedDiagram {
        png: out.png,
        width_px: out.width_px,
        height_px: out.height_px,
    }
}

fn map_render_error(err: RenderError) -> MermaidError {
    match err {
        RenderError::Parse(e) => MermaidError::Parse(e.to_string()),
        RenderError::Layout(msg) => MermaidError::Layout(msg),
        RenderError::Rasterize(msg) => MermaidError::Rasterize(msg),
        // Face treats resource caps as Unsupported so the pager falls back to
        // the source code block (same as oversized-source in render_checked).
        RenderError::Unsupported(msg) | RenderError::ResourceLimit(msg) => {
            MermaidError::Unsupported(msg)
        }
        // Non-exhaustive: map unknown future variants to Unsupported.
        other => MermaidError::Unsupported(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderLimits, render_checked};

    #[test]
    fn flowchart_renders_decodable_png() {
        let out = MmdrEngine::new()
            .render("flowchart LR\nA-->B-->C", &RenderParams::default())
            .expect("flowchart should render");
        assert!(out.width_px > 0 && out.height_px > 0);
        let img = image::load_from_memory(&out.png).expect("valid png");
        assert_eq!(img.width(), out.width_px);
        assert_eq!(img.height(), out.height_px);
    }

    #[test]
    fn sequence_renders() {
        let out = MmdrEngine::new()
            .render(
                "sequenceDiagram\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi",
                &RenderParams::default(),
            )
            .expect("sequence should render");
        assert!(out.width_px > 0 && out.height_px > 0);
    }

    #[test]
    fn light_and_dark_differ() {
        let engine = MmdrEngine::new();
        let light = engine
            .render(
                "flowchart LR\nA-->B",
                &RenderParams {
                    theme: MermaidTheme::Light,
                    background: Some(MermaidTheme::Light.surface_background()),
                    ..Default::default()
                },
            )
            .expect("light");
        let dark = engine
            .render(
                "flowchart LR\nA-->B",
                &RenderParams {
                    theme: MermaidTheme::Dark,
                    background: Some(MermaidTheme::Dark.surface_background()),
                    ..Default::default()
                },
            )
            .expect("dark");
        assert_ne!(light.png, dark.png, "themes must change pixels");
    }

    #[test]
    fn theme_presets_match_face_surfaces() {
        assert_eq!(Theme::face_light().background, crate::LIGHT_SURFACE.to_hex());
        assert_eq!(Theme::face_dark().background, crate::DARK_SURFACE.to_hex());
    }

    #[test]
    fn garbage_input_never_panics() {
        let engine = MmdrEngine::new();
        let limits = RenderLimits::default();
        let params = RenderParams::default();
        for garbage in [
            "",
            "@@@@",
            "%% only a comment",
            "flowchart\n\n\n",
            "????????",
            "\u{0}\u{1}\u{2}\u{3}",
            "flowchart LR\n  A[unterminated --> ",
            "pie\n  : :",
            "erDiagram\n  A ||",
            "sequenceDiagram\n  A->>",
        ] {
            let out = render_checked(&engine, garbage, &params, &limits);
            assert!(
                !matches!(out, Err(MermaidError::Panic(_))),
                "engine panicked on {garbage:?}: {out:?}"
            );
        }
    }

    #[test]
    fn map_render_error_taxonomy() {
        use mermaid_rs_renderer::ParseError;
        assert!(matches!(
            map_render_error(RenderError::Parse(ParseError::UnclosedSubgraph {
                opened_at: 1
            })),
            MermaidError::Parse(_)
        ));
        assert!(matches!(
            map_render_error(RenderError::Layout("x".into())),
            MermaidError::Layout(_)
        ));
        assert!(matches!(
            map_render_error(RenderError::Rasterize("x".into())),
            MermaidError::Rasterize(_)
        ));
        assert!(matches!(
            map_render_error(RenderError::Unsupported("x".into())),
            MermaidError::Unsupported(_)
        ));
        assert!(matches!(
            map_render_error(RenderError::ResourceLimit("x".into())),
            MermaidError::Unsupported(_)
        ));
    }
}
