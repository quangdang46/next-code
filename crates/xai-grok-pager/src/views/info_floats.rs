//! Face-local **paste-copy** of legacy compact info floats (no `next-code-tui` dep).
//!
//! ## Copied from (do not redesign)
//! - `info_widget_layout.rs` — `MIN_WIDGET_WIDTH` / `MAX_WIDGET_WIDTH`
//! - `info_widget.rs` — `AuthMethod`, `CacheHitInfo`, `effective_prompt_tokens`,
//!   `render_context_compact`, `render_kv_cache_summary_line`, border chrome
//!   from `render_single_widget`
//! - `info_widget_model.rs` — `render_model_info` + runtime metadata helpers
//! - `info_widget_usage.rs` — `render_usage_pill`, `render_context_usage_line`,
//!   `format_token_k`
//! - `info_widget_text.rs` — `truncate_smart`, `truncate_chars`
//! - `app/helpers.rs` — `pretty_model_display_name` (+ claude/title helpers)
//!
//! ## Face-only deltas (wire / overlay)
//! - Scroll-gated visibility ([`SCROLL_IDLE_HIDE_MS`])
//! - [`CacheHitInfo::apply_request_sample`] for ACP TokenUsage fold-in
//! - `Clear` under float boxes (legacy paints into empty margins)
//! - Slim [`InfoFloatData`] = fields those renderers need from `InfoWidgetData`

use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

/// Hide floats this long after the last scroll delta (mouse or keyboard).
/// Face product delta — legacy floats are always-on when data is present.
pub const SCROLL_IDLE_HIDE_MS: u64 = 1000;

// --- Copied from `info_widget_layout.rs` ---
const MIN_WIDGET_WIDTH: u16 = 24;
const MAX_WIDGET_WIDTH: u16 = 40;

// --- Copied from `next_code_provider_core::DEFAULT_CONTEXT_LIMIT` ---
const DEFAULT_CONTEXT_LIMIT: usize = 200_000;

const FLOAT_INSET: u16 = 1;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// Whether floats should paint given the last scroll-activity stamp.
pub fn floats_visible(last_scroll_at: Option<Instant>, now: Instant) -> bool {
    last_scroll_at.is_some_and(|at| {
        now.saturating_duration_since(at) < Duration::from_millis(SCROLL_IDLE_HIDE_MS)
    })
}

// ---------------------------------------------------------------------------
// Copied from `info_widget.rs` — AuthMethod
// ---------------------------------------------------------------------------

/// Authentication method used to access the model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMethod {
    #[default]
    Unknown,
    ApiKey,
    AnthropicOAuth,
    AnthropicApiKey,
    OpenAIOAuth,
    OpenAIApiKey,
    OpenRouterApiKey,
    OpenCodeApiKey,
    CopilotOAuth,
    GeminiOAuth,
}

// ---------------------------------------------------------------------------
// Copied from `info_widget.rs` — CacheHitInfo + effective_prompt_tokens
// ---------------------------------------------------------------------------

/// Session-level KV cache telemetry for providers that report cache usage.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CacheHitInfo {
    pub reported_input_tokens: u64,
    pub read_tokens: u64,
    pub creation_tokens: u64,
    pub optimal_input_tokens: u64,
    pub last_reported_input_tokens: Option<u64>,
    pub last_read_tokens: Option<u64>,
    pub last_creation_tokens: Option<u64>,
    pub last_optimal_input_tokens: Option<u64>,
    pub miss_attributions: Vec<CacheMissAttribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheMissAttribution {
    pub turn_number: usize,
    pub call_index: u16,
    pub missed_tokens: u64,
    pub reason: String,
}

/// Effective prompt size to use as the denominator for cache-hit ratios.
///
/// Copied from `info_widget.rs::effective_prompt_tokens`.
pub fn effective_prompt_tokens(input: u64, read: u64, creation: u64) -> u64 {
    if creation > 0 || read > input {
        input.saturating_add(read).saturating_add(creation)
    } else {
        input
    }
}

impl CacheHitInfo {
    fn effective_reported_tokens(&self) -> u64 {
        effective_prompt_tokens(
            self.reported_input_tokens,
            self.read_tokens,
            self.creation_tokens,
        )
    }

    pub fn hit_ratio(&self) -> Option<f32> {
        let denominator = self.effective_reported_tokens();
        if denominator == 0 {
            None
        } else {
            Some((self.read_tokens as f32 / denominator as f32).clamp(0.0, 1.0))
        }
    }

    pub fn optimal_ratio(&self) -> Option<f32> {
        if self.optimal_input_tokens == 0 {
            None
        } else {
            Some((self.read_tokens as f32 / self.optimal_input_tokens as f32).clamp(0.0, 1.0))
        }
    }

    pub fn last_ratio(&self) -> Option<f32> {
        let input = self.last_reported_input_tokens?;
        let denominator = effective_prompt_tokens(
            input,
            self.last_read_tokens.unwrap_or(0),
            self.last_creation_tokens.unwrap_or(0),
        );
        if denominator == 0 {
            None
        } else {
            Some((self.last_read_tokens.unwrap_or(0) as f32 / denominator as f32).clamp(0.0, 1.0))
        }
    }

    pub fn last_optimal_ratio(&self) -> Option<f32> {
        let optimal = self.last_optimal_input_tokens?;
        if optimal == 0 {
            None
        } else {
            Some((self.last_read_tokens.unwrap_or(0) as f32 / optimal as f32).clamp(0.0, 1.0))
        }
    }

    /// Face wire helper (not in legacy `CacheHitInfo`) — fold ACP TokenUsage cache fields.
    pub fn apply_request_sample(
        &mut self,
        input: u64,
        cache_read: Option<u64>,
        cache_creation: Option<u64>,
    ) {
        let read = cache_read.unwrap_or(0);
        let creation = cache_creation.unwrap_or(0);
        if cache_read.is_none() && cache_creation.is_none() {
            return;
        }

        let prior_optimal = self
            .last_reported_input_tokens
            .map(|prev_input| {
                effective_prompt_tokens(
                    prev_input,
                    self.last_read_tokens.unwrap_or(0),
                    self.last_creation_tokens.unwrap_or(0),
                )
            })
            .filter(|n| *n > 0);

        self.reported_input_tokens = self.reported_input_tokens.saturating_add(input);
        self.read_tokens = self.read_tokens.saturating_add(read);
        self.creation_tokens = self.creation_tokens.saturating_add(creation);
        if let Some(optimal) = prior_optimal {
            self.optimal_input_tokens = self.optimal_input_tokens.saturating_add(optimal);
        }

        self.last_reported_input_tokens = Some(input);
        self.last_read_tokens = Some(read);
        self.last_creation_tokens = Some(creation);
        self.last_optimal_input_tokens = prior_optimal;
    }
}

// ---------------------------------------------------------------------------
// Slim input — fields used by copied renderers (from `InfoWidgetData`)
// ---------------------------------------------------------------------------

/// Subset of legacy `InfoWidgetData` needed by Overview compact + KV summary.
#[derive(Debug, Default, Clone)]
pub struct InfoFloatData {
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub service_tier: Option<String>,
    pub native_compaction_mode: Option<String>,
    pub native_compaction_threshold_tokens: Option<usize>,
    pub session_count: Option<usize>,
    pub session_name: Option<String>,
    pub provider_name: Option<String>,
    pub auth_method: AuthMethod,
    pub context_info_stale: bool,
    /// Legacy gate: `context_info.as_ref().map(|c| c.total_chars > 0)`.
    pub context_ready: bool,
    pub observed_context_tokens: Option<u64>,
    pub context_limit: Option<usize>,
    pub is_compacting: bool,
    pub cache_hit_info: Option<CacheHitInfo>,
}

// ---------------------------------------------------------------------------
// Copied from `info_widget_text.rs`
// ---------------------------------------------------------------------------

fn truncate_smart(s: &str, max_len: usize) -> String {
    let char_len = s.chars().count();
    if char_len <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return "...".to_string();
    }

    let target = max_len - 3;
    let prefix = truncate_chars(s, target);

    if let Some(pos) = prefix.rfind(' ') {
        let before = &prefix[..pos];
        let pos_chars = before.chars().count();
        if pos_chars > target / 2 {
            return format!("{}...", before);
        }
    }
    format!("{}...", prefix)
}

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

// ---------------------------------------------------------------------------
// Copied from `app/helpers.rs` — pretty_model_display_name
// ---------------------------------------------------------------------------

fn pretty_model(model: &str) -> String {
    pretty_model_display_name(model)
}

fn pretty_model_display_name(model: &str) -> String {
    let model = model.trim();
    if model.is_empty() {
        return "your default model".to_string();
    }

    let (core, long_context) = match model.strip_suffix("[1m]") {
        Some(stripped) => (stripped, true),
        None => (model, false),
    };

    let lower = core.to_ascii_lowercase();
    let mut pretty = if let Some(rest) = lower.strip_prefix("gpt-") {
        format!("GPT-{}", rest)
    } else if lower.starts_with("claude-") {
        prettify_claude(core)
    } else {
        title_case_dashed(core)
    };

    if long_context {
        pretty.push_str(" (1M)");
    }
    pretty
}

fn prettify_claude(core: &str) -> String {
    let parts: Vec<&str> = core.split('-').collect();
    let mut words: Vec<String> = Vec::new();
    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];
        if part.chars().all(|c| c.is_ascii_digit())
            && i + 1 < parts.len()
            && parts[i + 1].chars().all(|c| c.is_ascii_digit())
        {
            words.push(format!("{}.{}", part, parts[i + 1]));
            i += 2;
            continue;
        }
        words.push(title_case_word(part));
        i += 1;
    }
    words.join(" ")
}

fn title_case_dashed(core: &str) -> String {
    core.split('-')
        .map(title_case_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }
    if word.chars().any(|c| c.is_ascii_digit()) {
        return word.to_string();
    }
    let mut chars = word.chars();
    let first = chars.next().unwrap().to_ascii_uppercase();
    format!("{}{}", first, chars.as_str())
}

// ---------------------------------------------------------------------------
// Copied from `info_widget_usage.rs`
// ---------------------------------------------------------------------------

fn render_usage_pill(used_tokens: usize, limit_tokens: usize, width: u16) -> Line<'static> {
    let safe_limit = limit_tokens.max(1);
    let bar_width = (width as usize).min(24);
    if bar_width == 0 {
        return Line::default();
    }

    let mut used_cells = ((used_tokens as f64 / safe_limit as f64) * bar_width as f64)
        .round()
        .max(0.0) as usize;
    if used_cells > bar_width {
        used_cells = bar_width;
    }

    let used_pct = ((used_tokens as f64 / safe_limit as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    let left_pct = 100u8.saturating_sub(used_pct);
    let used_color = if left_pct <= 20 {
        rgb(255, 100, 100)
    } else if left_pct <= 50 {
        rgb(255, 200, 100)
    } else {
        rgb(100, 200, 100)
    };

    let empty_cells = bar_width.saturating_sub(used_cells);
    let mut spans = Vec::new();
    spans.push(Span::styled(
        "▰".repeat(used_cells),
        Style::default().fg(used_color),
    ));
    if empty_cells > 0 {
        spans.push(Span::styled(
            "▱".repeat(empty_cells),
            Style::default().fg(rgb(50, 50, 60)),
        ));
    }
    Line::from(spans)
}

/// Context usage line (label + tokens + optional pill).
/// Copied from `info_widget_usage.rs::render_context_usage_line`.
pub fn render_context_usage_line(
    label: &str,
    used_tokens: usize,
    limit_tokens: usize,
    width: u16,
) -> Line<'static> {
    let tokens = format!(
        "{}/{}",
        format_token_k(used_tokens),
        format_token_k(limit_tokens)
    );
    let used_pct = ((used_tokens as f64 / limit_tokens.max(1) as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    let left_pct = 100u8.saturating_sub(used_pct);
    let token_color = if left_pct <= 20 {
        rgb(255, 100, 100)
    } else if left_pct <= 50 {
        rgb(255, 200, 100)
    } else {
        rgb(100, 200, 100)
    };

    let label_width = UnicodeWidthStr::width(label);
    let tokens_width = UnicodeWidthStr::width(tokens.as_str());
    // label + space + tokens + space + bar
    let bar_width = width.saturating_sub((label_width + 1 + tokens_width + 1) as u16);

    let mut spans = vec![
        Span::styled(format!("{label} "), Style::default().fg(rgb(140, 140, 150))),
        Span::styled(
            format!("{tokens} "),
            Style::default().fg(token_color).bold(),
        ),
    ];

    if bar_width >= 3 {
        spans.extend(render_usage_pill(used_tokens, limit_tokens, bar_width).spans);
    }
    Line::from(spans)
}

fn format_token_k(tokens: usize) -> String {
    if tokens >= 1000 {
        format!("{}k", tokens / 1000)
    } else {
        format!("{}", tokens)
    }
}

// ---------------------------------------------------------------------------
// Copied from `info_widget_model.rs` — render_model_info
// ---------------------------------------------------------------------------

fn render_model_info(data: &InfoFloatData, inner: Rect) -> Vec<Line<'static>> {
    let Some(model) = &data.model else {
        return Vec::new();
    };

    let short_name = pretty_model(model);
    let max_len = inner.width.saturating_sub(2) as usize;

    let mut spans = vec![Span::styled(
        if short_name.chars().count() > max_len {
            format!(
                "{}...",
                truncate_chars(&short_name, max_len.saturating_sub(3))
            )
        } else {
            short_name
        },
        Style::default().fg(rgb(180, 180, 190)).bold(),
    )];

    append_model_runtime_metadata(&mut spans, data);

    if let Some(mode) = &data.native_compaction_mode {
        let label = if let Some(tokens) = data.native_compaction_threshold_tokens {
            format!("native {} @ {}k", mode, tokens / 1000)
        } else {
            format!("native {}", mode)
        };
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(label, Style::default().fg(rgb(120, 210, 230))));
    }

    let mut lines = vec![Line::from(spans)];

    let has_provider = data
        .provider_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some();
    let has_auth = data.auth_method != AuthMethod::Unknown;

    if has_provider || has_auth {
        let mut detail_spans: Vec<Span> = Vec::new();

        if let Some(provider) = data
            .provider_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            detail_spans.push(Span::styled(
                provider.to_lowercase(),
                Style::default().fg(rgb(140, 180, 255)),
            ));
        }

        if has_auth {
            let (icon, label, _color) = match data.auth_method {
                AuthMethod::ApiKey => ("🔑", "API Key", rgb(180, 180, 190)),
                AuthMethod::AnthropicOAuth => ("🔐", "OAuth", rgb(255, 160, 100)),
                AuthMethod::AnthropicApiKey => ("🔑", "API Key", rgb(180, 180, 190)),
                AuthMethod::OpenAIOAuth => ("🔐", "OAuth", rgb(100, 200, 180)),
                AuthMethod::OpenAIApiKey => ("🔑", "API Key", rgb(180, 180, 190)),
                AuthMethod::OpenRouterApiKey => ("🔑", "API Key", rgb(140, 180, 255)),
                AuthMethod::OpenCodeApiKey => ("🔑", "API Key", rgb(140, 180, 255)),
                AuthMethod::CopilotOAuth => ("🔐", "OAuth", rgb(110, 200, 140)),
                AuthMethod::GeminiOAuth => ("🔐", "OAuth", rgb(120, 190, 255)),
                AuthMethod::Unknown => unreachable!(),
            };
            if !detail_spans.is_empty() {
                detail_spans.push(Span::styled(" · ", Style::default().fg(rgb(80, 80, 90))));
            }
            detail_spans.push(Span::styled(
                format!("{} {}", icon, label),
                Style::default().fg(rgb(140, 140, 150)),
            ));
        }

        if !detail_spans.is_empty() {
            lines.push(Line::from(detail_spans));
        }
    }

    if data.session_count.is_some() || data.session_name.is_some() {
        let mut parts = Vec::new();

        if let Some(sessions) = data.session_count {
            parts.push(format!(
                "{} session{}",
                sessions,
                if sessions == 1 { "" } else { "s" }
            ));
        }

        if let Some(name) = data.session_name.as_deref()
            && !name.trim().is_empty()
        {
            parts.push(name.to_string());
        }

        if !parts.is_empty() {
            let detail = truncate_smart(&parts.join(" · "), max_len.saturating_sub(2));
            lines.push(Line::from(vec![Span::styled(
                detail,
                Style::default().fg(rgb(140, 140, 150)),
            )]));
        }
    }

    lines
}

fn append_model_runtime_metadata(spans: &mut Vec<Span<'static>>, data: &InfoFloatData) {
    if let Some(effort) = data
        .reasoning_effort
        .as_deref()
        .and_then(short_reasoning_effort)
    {
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(
            format!("({effort})"),
            Style::default().fg(rgb(255, 200, 100)),
        ));
    }

    if let Some(tier) = data.service_tier.as_deref().and_then(short_service_tier) {
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(
            format!("[{tier}]"),
            Style::default().fg(rgb(200, 140, 255)).bold(),
        ));
    }
}

fn short_reasoning_effort(effort: &str) -> Option<&str> {
    let effort = effort.trim();
    if effort.is_empty() {
        return None;
    }
    Some(match effort {
        "xhigh" => "xhi",
        "high" => "hi",
        "medium" => "med",
        "low" => "lo",
        "none" => "∅",
        "swarm" => "swarm",
        "swarm-deep" => "swarm+",
        other => other,
    })
}

fn short_service_tier(service_tier: &str) -> Option<&str> {
    let service_tier = service_tier.trim();
    if service_tier.is_empty() || service_tier == "off" || service_tier == "default" {
        return None;
    }
    Some(match service_tier {
        "priority" => "fast",
        "flex" => "flex",
        other => other,
    })
}

// ---------------------------------------------------------------------------
// Copied from `info_widget.rs` — render_context_compact
// ---------------------------------------------------------------------------

fn render_context_compact(data: &InfoFloatData, inner: Rect) -> Vec<Line<'static>> {
    if data.context_info_stale {
        return vec![Line::from(vec![
            Span::styled("Context ", Style::default().fg(rgb(140, 140, 150))),
            Span::styled("updating...", Style::default().fg(rgb(220, 180, 80))),
        ])];
    }
    // Legacy: requires context_info with total_chars > 0, or observed tokens.
    if !data.context_ready && data.observed_context_tokens.is_none() {
        return Vec::new();
    }

    let used_tokens = data
        .observed_context_tokens
        .map(|t| t as usize)
        .unwrap_or(0);
    if used_tokens == 0 && !data.context_ready {
        return Vec::new();
    }
    let limit_tokens = data.context_limit.unwrap_or(DEFAULT_CONTEXT_LIMIT).max(1);
    let label = if data.is_compacting {
        "Context📦"
    } else {
        "Context"
    };

    vec![render_context_usage_line(
        label,
        used_tokens,
        limit_tokens,
        inner.width,
    )]
}

// ---------------------------------------------------------------------------
// Copied from `info_widget.rs` — KV summary (compact Overview / KvCache line)
// ---------------------------------------------------------------------------

fn ratio_pct(ratio: f32) -> u8 {
    (ratio * 100.0).round().clamp(0.0, 100.0) as u8
}

fn kv_cache_optimal_color(pct: u8) -> Color {
    match pct {
        0..=24 => rgb(255, 110, 110),
        25..=59 => rgb(255, 200, 100),
        60..=84 => rgb(140, 180, 255),
        _ => rgb(110, 210, 140),
    }
}

/// KV summary line. Copied from `info_widget.rs::render_kv_cache_summary_line`.
pub fn render_kv_cache_summary_line(cache: &CacheHitInfo) -> Option<Line<'static>> {
    let lifetime_ratio = cache.hit_ratio()?;
    let lifetime_pct = ratio_pct(lifetime_ratio);
    let warm_pct = cache.optimal_ratio().map(ratio_pct);
    let last_pct = cache.last_ratio().map(ratio_pct);
    let last_optimal_pct = cache.last_optimal_ratio().map(ratio_pct);
    let health_pct = last_optimal_pct
        .or(last_pct)
        .or(warm_pct)
        .unwrap_or(lifetime_pct);
    let color = kv_cache_optimal_color(health_pct);

    let mut spans = vec![Span::styled(
        "KV cache: ",
        Style::default().fg(rgb(180, 180, 190)).bold(),
    )];

    if let Some(warm_pct) = warm_pct {
        spans.push(Span::styled(
            "yield ",
            Style::default().fg(rgb(140, 140, 150)),
        ));
        spans.push(Span::styled(
            format!("{}%", warm_pct),
            Style::default().fg(color).bold(),
        ));
    } else {
        spans.push(Span::styled(
            "priming",
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(last_pct) = last_pct {
        spans.push(Span::styled(" · ", Style::default().fg(rgb(80, 80, 90))));
        spans.push(Span::styled(
            "last ",
            Style::default().fg(rgb(140, 140, 150)),
        ));
        spans.push(Span::styled(
            format!("{}%", last_pct),
            Style::default().fg(color).bold(),
        ));
    }

    spans.push(Span::styled(" · ", Style::default().fg(rgb(80, 80, 90))));
    spans.push(Span::styled(
        "session ",
        Style::default().fg(rgb(140, 140, 150)),
    ));
    spans.push(Span::styled(
        format!("{}%", lifetime_pct),
        Style::default().fg(color).bold(),
    ));

    Some(Line::from(spans))
}

// ---------------------------------------------------------------------------
// Compact Overview lines — copied from `render_sections` CompactOnly path
// (model + context only; KV stays on the Left float, not merged)
// ---------------------------------------------------------------------------

fn render_overview_compact_lines(data: &InfoFloatData, inner: Rect) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if data.model.is_some() {
        lines.extend(render_model_info(data, inner));
    }

    if data.context_ready || data.observed_context_tokens.is_some() || data.context_info_stale {
        lines.extend(render_context_compact(data, inner));
    }

    lines
}

fn render_kv_compact_lines(cache: &CacheHitInfo) -> Vec<Line<'static>> {
    match render_kv_cache_summary_line(cache) {
        Some(line) => vec![line],
        None => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Border chrome — copied from `render_single_widget` Block setup
// ---------------------------------------------------------------------------

fn float_border_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(rgb(70, 70, 80)).dim())
}

fn paint_bordered_float(buf: &mut Buffer, rect: Rect, lines: Vec<Line<'static>>) {
    if rect.width < MIN_WIDGET_WIDTH || rect.height < 3 || lines.is_empty() {
        return;
    }
    // Face overlay adaptation: clear under the box (legacy paints into free margin).
    Clear.render(rect, buf);
    let block = float_border_block();
    let inner = block.inner(rect);
    block.render(rect, buf);
    let mut clipped = lines;
    clipped.truncate(inner.height as usize);
    Paragraph::new(clipped).render(inner, buf);
}

fn place_right(area: Rect, content_h: u16) -> Option<Rect> {
    let width = MAX_WIDGET_WIDTH.min(area.width.saturating_sub(FLOAT_INSET * 2));
    if width < MIN_WIDGET_WIDTH {
        return None;
    }
    let height = content_h
        .saturating_add(2) // borders
        .min(area.height.saturating_sub(FLOAT_INSET));
    if height < 3 {
        return None;
    }
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(width).saturating_sub(FLOAT_INSET));
    let y = area.y.saturating_add(FLOAT_INSET.min(area.height.saturating_sub(1)));
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

fn place_left(area: Rect, content_h: u16) -> Option<Rect> {
    let width = MAX_WIDGET_WIDTH.min(area.width.saturating_sub(FLOAT_INSET * 2));
    if width < MIN_WIDGET_WIDTH {
        return None;
    }
    let height = content_h
        .saturating_add(2)
        .min(area.height.saturating_sub(FLOAT_INSET));
    if height < 3 {
        return None;
    }
    let x = area.x.saturating_add(FLOAT_INSET);
    let y = area.y.saturating_add(FLOAT_INSET.min(area.height.saturating_sub(1)));
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

/// Paint Right Overview compact + Left KV floats into the scrollback content rect.
///
/// Placement mirrors legacy `preferred_side`: Overview/Context → Right, KvCache → Left.
pub fn render_info_floats(buf: &mut Buffer, area: Rect, data: &InfoFloatData) {
    if area.width < MIN_WIDGET_WIDTH || area.height < 3 {
        return;
    }

    // Measure Overview with provisional inner width (outer - borders).
    let provisional_inner_w = MAX_WIDGET_WIDTH
        .min(area.width.saturating_sub(FLOAT_INSET * 2))
        .saturating_sub(2);
    let overview_inner = Rect {
        x: 0,
        y: 0,
        width: provisional_inner_w.max(1),
        height: area.height.saturating_sub(2).max(1),
    };
    let overview_lines = render_overview_compact_lines(data, overview_inner);
    if !overview_lines.is_empty()
        && let Some(rect) = place_right(area, overview_lines.len() as u16)
    {
        let inner_w = rect.width.saturating_sub(2).max(1);
        let lines = render_overview_compact_lines(
            data,
            Rect {
                x: 0,
                y: 0,
                width: inner_w,
                height: rect.height.saturating_sub(2).max(1),
            },
        );
        paint_bordered_float(buf, rect, lines);
    }

    if let Some(cache) = data.cache_hit_info.as_ref() {
        let kv_lines = render_kv_compact_lines(cache);
        if !kv_lines.is_empty()
            && let Some(rect) = place_left(area, kv_lines.len() as u16)
        {
            paint_bordered_float(buf, rect, kv_lines);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_visible_only_within_idle_window() {
        let t0 = Instant::now();
        assert!(!floats_visible(None, t0));
        assert!(floats_visible(Some(t0), t0));
        assert!(floats_visible(
            Some(t0),
            t0 + Duration::from_millis(SCROLL_IDLE_HIDE_MS - 1)
        ));
        assert!(!floats_visible(
            Some(t0),
            t0 + Duration::from_millis(SCROLL_IDLE_HIDE_MS)
        ));
    }

    #[test]
    fn effective_prompt_tokens_split_vs_subset() {
        assert_eq!(effective_prompt_tokens(100, 50, 0), 100);
        assert_eq!(effective_prompt_tokens(100, 50, 10), 160);
        assert_eq!(effective_prompt_tokens(40, 80, 0), 120);
    }

    #[test]
    fn overview_compact_matches_legacy_line_order() {
        let data = InfoFloatData {
            model: Some("deepseek-v4-flash".into()),
            provider_name: Some("opencode go".into()),
            session_count: Some(2),
            session_name: Some("beach Retriever".into()),
            context_ready: true,
            observed_context_tokens: Some(3000),
            context_limit: Some(1_000_000),
            ..Default::default()
        };
        let lines = render_overview_compact_lines(&data, Rect::new(0, 0, 36, 10));
        assert!(lines.len() >= 4, "model + provider + sessions + context");
        let texts: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect();
        assert!(texts[0].to_lowercase().contains("deepseek"));
        assert!(texts[1].contains("opencode go"));
        assert!(texts[2].contains("2 sessions"));
        assert!(texts[3].contains("Context"));
        assert!(texts[3].contains("3k"));
        assert!(texts[3].contains("1000k"));
    }

    #[test]
    fn context_line_includes_label_and_tokens() {
        let line = render_context_usage_line("Context", 50_000, 200_000, 40);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("Context"));
        assert!(text.contains("50k"));
        assert!(text.contains("200k"));
    }

    #[test]
    fn kv_summary_shows_priming_then_yield() {
        let mut cache = CacheHitInfo::default();
        cache.apply_request_sample(1000, Some(0), Some(1000));
        let priming = render_kv_cache_summary_line(&cache).expect("summary");
        let priming_text: String = priming
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(priming_text.contains("priming"));
        assert!(priming_text.contains("session"));

        cache.apply_request_sample(1000, Some(800), Some(0));
        let warm = render_kv_cache_summary_line(&cache).expect("summary");
        let warm_text: String = warm.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(warm_text.contains("yield"));
        assert!(warm_text.contains("last"));
        assert!(warm_text.contains("session"));
    }

    #[test]
    fn kv_summary_none_without_hits_denominator() {
        let cache = CacheHitInfo::default();
        assert!(render_kv_cache_summary_line(&cache).is_none());
    }

    #[test]
    fn pretty_model_title_cases_dashed_ids() {
        assert_eq!(pretty_model("deepseek-v4-flash"), "Deepseek v4 Flash");
    }
}
