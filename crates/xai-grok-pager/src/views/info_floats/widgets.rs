//! Paste-copied compact float renderers for the remaining legacy WidgetKinds.
//!
//! Sources (do not redesign):
//! - `info_widget.rs` — types, `has_data_for`, compaction
//! - `info_widget_memory_render.rs` — `render_memory_compact` / count label
//! - `info_widget_usage.rs` — `render_usage_compact` / labeled bars
//! - `info_widget_git.rs` — `render_git_widget` / compact
//! - `info_widget_swarm_background.rs` — background + swarm stats
//! - `next_code_tui_render::swarm_gallery::render_swarm_compact` — dock tally
//! - `info_widget_todos.rs` — `render_todos_compact`
//! - WorkspaceMap / Diagrams / Ambient / Tips / TeamView → [`super::legacy_deferred`]

use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use super::{rgb, truncate_chars, truncate_smart};

pub use super::legacy_deferred::{
    DiagramsInfo, WorkspaceMapInfo, diagrams_has_data, render_diagrams_lines,
    render_workspace_lines, workspace_has_data,
};

// ---------------------------------------------------------------------------
// WidgetKind (Face float subset) — copied side/priority from info_widget.rs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatKind {
    Overview,
    /// Standalone model card — suppressed when Overview is placed (legacy merge).
    ModelInfo,
    /// Standalone context bar — suppressed when Overview is placed (legacy merge).
    ContextUsage,
    KvCache,
    MemoryActivity,
    UsageLimits,
    GitStatus,
    BackgroundTasks,
    Compaction,
    SwarmStatus,
    Todos,
    WorkspaceMap,
    Diagrams,
}

impl FloatKind {
    /// Legacy preferred dock side (Phase-2 scoring bias only).
    ///
    /// Face agent view is non-centered, so the placer seats everything on the
    /// **Right** — Left only exists when `margins.centered` in legacy layout.
    pub fn preferred_side(self) -> Side {
        match self {
            FloatKind::Diagrams
            | FloatKind::WorkspaceMap
            | FloatKind::Overview
            | FloatKind::Todos
            | FloatKind::ContextUsage
            | FloatKind::MemoryActivity => Side::Right,
            FloatKind::SwarmStatus
            | FloatKind::Compaction
            | FloatKind::BackgroundTasks
            | FloatKind::UsageLimits
            | FloatKind::KvCache
            | FloatKind::ModelInfo
            | FloatKind::GitStatus => Side::Left,
        }
    }

    /// Lower = higher priority (legacy WidgetKind::priority).
    pub fn priority(self) -> u8 {
        match self {
            FloatKind::Diagrams => 0,
            FloatKind::WorkspaceMap => 1,
            FloatKind::Overview => 2,
            FloatKind::Todos => 3,
            FloatKind::ContextUsage => 4,
            FloatKind::UsageLimits => 5,
            FloatKind::KvCache => 6,
            // Face product: MemoryActivity elevated above Model/Context siblings.
            FloatKind::MemoryActivity => 0,
            FloatKind::ModelInfo => 8,
            FloatKind::Compaction => 9,
            FloatKind::BackgroundTasks => 10,
            FloatKind::GitStatus => 11,
            FloatKind::SwarmStatus => 12,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// Types — slim copies of legacy info_widget structs
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_count: usize,
    pub disabled: bool,
    /// Compact activity summary (e.g. "working", "idle", "done").
    pub activity_summary: Option<String>,
    pub show_activity: bool,
}

impl MemoryInfo {
    pub fn should_render(&self) -> bool {
        !self.disabled && (self.total_count > 0 || self.activity_summary.is_some())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UsageProvider {
    #[default]
    None,
    Anthropic,
    OpenAI,
    CostBased,
    Copilot,
    /// Face billing credits (maps onto UsageLimits bars).
    Credits,
}

impl UsageProvider {
    pub fn label(self) -> &'static str {
        match self {
            UsageProvider::None => "",
            UsageProvider::Anthropic => "Anthropic",
            UsageProvider::OpenAI => "OpenAI",
            UsageProvider::CostBased => "",
            UsageProvider::Copilot => "Copilot",
            UsageProvider::Credits => "Credits",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UsageInfo {
    pub provider: UsageProvider,
    pub primary_limit_label: Option<String>,
    pub five_hour: f32,
    pub five_hour_resets_at: Option<String>,
    pub secondary_limit_label: Option<String>,
    pub seven_day: f32,
    pub seven_day_resets_at: Option<String>,
    pub spark: Option<f32>,
    pub spark_resets_at: Option<String>,
    pub total_cost: f32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitInfo {
    pub branch: String,
    pub modified: usize,
    pub staged: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
    pub dirty_files: Vec<String>,
}

impl GitInfo {
    pub fn is_interesting(&self) -> bool {
        self.modified > 0
            || self.staged > 0
            || self.untracked > 0
            || self.ahead > 0
            || self.behind > 0
    }
}

#[derive(Debug, Default, Clone)]
pub struct BackgroundInfo {
    pub running_count: usize,
    pub running_tasks: Vec<String>,
    pub progress_detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompactionInfo {
    pub is_compacting: bool,
    pub compacted_messages: usize,
    pub active_messages: usize,
    pub summary_chars: usize,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct SwarmMemberFloat {
    pub session_id: String,
    pub friendly_name: Option<String>,
    pub status: String,
    pub detail: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct SwarmInfo {
    pub managed_members: Vec<SwarmMemberFloat>,
    /// (completed, running, total)
    pub plan_progress: Option<(u32, u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoFloatItem {
    pub content: String,
    pub status: String,
}

#[derive(Debug, Default, Clone)]
pub struct TodosInfo {
    pub items: Vec<TodoFloatItem>,
    pub are_swarm_plan: bool,
}

// ---------------------------------------------------------------------------
// has_data gates — copied from InfoWidgetData::has_data_for
// ---------------------------------------------------------------------------

pub fn memory_has_data(info: Option<&MemoryInfo>) -> bool {
    info.map(MemoryInfo::should_render).unwrap_or(false)
}

pub fn usage_has_data(info: Option<&UsageInfo>) -> bool {
    info.map(|u| u.available).unwrap_or(false)
}

pub fn git_has_data(info: Option<&GitInfo>) -> bool {
    info.map(|g| g.is_interesting()).unwrap_or(false)
}

pub fn background_has_data(info: Option<&BackgroundInfo>) -> bool {
    info.map(|b| b.running_count > 0).unwrap_or(false)
}

pub fn compaction_has_data(info: Option<&CompactionInfo>) -> bool {
    info.is_some()
}

pub fn swarm_has_data(info: Option<&SwarmInfo>) -> bool {
    info.map(|s| !s.managed_members.is_empty()).unwrap_or(false)
}

pub fn todos_has_data(info: Option<&TodosInfo>) -> bool {
    info.map(|t| !t.items.is_empty()).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Memory — copied from render_memory_compact
// ---------------------------------------------------------------------------

fn memory_count_label(total_count: usize) -> String {
    if total_count == 1 {
        "1 memory".to_string()
    } else {
        format!("{total_count} memories")
    }
}

fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }
    let keep = max_width.saturating_sub(1);
    format!("{}…", truncate_chars(s, keep))
}

pub fn render_memory_compact(info: &MemoryInfo, inner_width: u16) -> Vec<Line<'static>> {
    if !info.should_render() {
        return Vec::new();
    }

    let max_width = inner_width.saturating_sub(2) as usize;
    let title = memory_count_label(info.total_count);
    let title_width = UnicodeWidthStr::width(title.as_str());
    let summary_width = max_width.saturating_sub(title_width + 5);
    let accent = if info.show_activity {
        rgb(140, 200, 255)
    } else if info.total_count > 0 {
        rgb(160, 160, 170)
    } else {
        rgb(140, 200, 255)
    };

    let mut spans = vec![
        Span::styled("🧠 ", Style::default().fg(rgb(200, 150, 255))),
        Span::styled(title, Style::default().fg(rgb(180, 180, 190)).bold()),
    ];
    if info.show_activity {
        let summary = info
            .activity_summary
            .as_deref()
            .unwrap_or("working");
        spans.push(Span::styled(" · ", Style::default().fg(rgb(100, 100, 110))));
        spans.push(Span::styled(
            truncate_with_ellipsis(summary, summary_width.max(8)),
            Style::default().fg(accent),
        ));
    }

    vec![Line::from(spans)]
}

// ---------------------------------------------------------------------------
// UsageLimits — copied from render_usage_compact + render_labeled_bar
// ---------------------------------------------------------------------------

fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1000 {
        format!("{}k", n / 1000)
    } else {
        format!("{n}")
    }
}

fn render_labeled_bar(
    label: &str,
    used_pct: u8,
    left_pct: u8,
    reset_time: Option<&str>,
    width: u16,
) -> Line<'static> {
    let color = if left_pct <= 20 {
        rgb(255, 100, 100)
    } else if left_pct <= 50 {
        rgb(255, 200, 100)
    } else {
        rgb(100, 200, 100)
    };

    const LABEL_WIDTH: usize = 7;
    const MIN_BAR_WIDTH: usize = 4;

    let full_suffix = match reset_time {
        Some(reset) if left_pct == 0 => format!(" resets {reset}"),
        Some(reset) => format!(" {left_pct}% left · {reset}"),
        None => format!(" {left_pct}% left"),
    };
    let suffix = match reset_time {
        Some(reset) if left_pct > 0 => {
            let compact = format!(" {left_pct}% · {reset}");
            let reset_only = format!(" · {reset}");
            let budget = usize::from(width).saturating_sub(LABEL_WIDTH + MIN_BAR_WIDTH);
            if UnicodeWidthStr::width(full_suffix.as_str()) <= budget {
                full_suffix
            } else if UnicodeWidthStr::width(compact.as_str()) <= budget {
                compact
            } else {
                reset_only
            }
        }
        _ => full_suffix,
    };
    let suffix_width = UnicodeWidthStr::width(suffix.as_str());
    let label_width = LABEL_WIDTH.min(usize::from(width).saturating_sub(suffix_width));
    let bar_width = usize::from(width)
        .saturating_sub(label_width + suffix_width)
        .min(12);

    let filled = ((used_pct as f32 / 100.0) * bar_width as f32).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let visible_label: String = label.chars().take(label_width).collect();
    let padded_label = format!("{visible_label:<label_width$}");

    Line::from(vec![
        Span::styled(padded_label, Style::default().fg(rgb(140, 140, 150))),
        Span::styled("▰".repeat(filled), Style::default().fg(color)),
        Span::styled("▱".repeat(empty), Style::default().fg(rgb(50, 50, 60))),
        Span::styled(suffix, Style::default().fg(color)),
    ])
}

pub fn render_usage_compact(info: &UsageInfo, width: u16) -> Vec<Line<'static>> {
    if !info.available {
        return Vec::new();
    }

    if matches!(info.provider, UsageProvider::CostBased) {
        return vec![Line::from(vec![Span::styled(
            format!(
                "${:.4} · {} in + {} out",
                info.total_cost,
                format_tokens(info.input_tokens),
                format_tokens(info.output_tokens)
            ),
            Style::default().fg(rgb(140, 140, 150)),
        )])];
    }

    if matches!(info.provider, UsageProvider::Copilot) {
        return vec![Line::from(vec![Span::styled(
            format!(
                "{} in + {} out",
                format_tokens(info.input_tokens),
                format_tokens(info.output_tokens)
            ),
            Style::default().fg(rgb(140, 140, 150)),
        )])];
    }

    let five_hr_used = (info.five_hour * 100.0).round().clamp(0.0, 100.0) as u8;
    let seven_day_used = (info.seven_day * 100.0).round().clamp(0.0, 100.0) as u8;
    let five_hr_left = 100u8.saturating_sub(five_hr_used);
    let seven_day_left = 100u8.saturating_sub(seven_day_used);

    let mut lines = Vec::new();
    let label = info.provider.label();
    if !label.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            format!("{label} limits"),
            Style::default()
                .fg(rgb(140, 140, 150))
                .add_modifier(Modifier::DIM),
        )]));
    }
    if let Some(primary_label) = info.primary_limit_label.as_deref() {
        lines.push(render_labeled_bar(
            primary_label,
            five_hr_used,
            five_hr_left,
            info.five_hour_resets_at.as_deref(),
            width,
        ));
    }
    if let Some(secondary_label) = info.secondary_limit_label.as_deref() {
        lines.push(render_labeled_bar(
            secondary_label,
            seven_day_used,
            seven_day_left,
            info.seven_day_resets_at.as_deref(),
            width,
        ));
    }
    if let Some(spark_usage) = info.spark {
        let spark_used = (spark_usage * 100.0).round().clamp(0.0, 100.0) as u8;
        let spark_left = 100u8.saturating_sub(spark_used);
        lines.push(render_labeled_bar(
            "Spark",
            spark_used,
            spark_left,
            info.spark_resets_at.as_deref(),
            width,
        ));
    }
    lines
}

// ---------------------------------------------------------------------------
// Git — copied from render_git_widget / render_git_compact
// ---------------------------------------------------------------------------

pub fn render_git_widget(info: &GitInfo, inner_width: u16, max_lines: u16) -> Vec<Line<'static>> {
    if !info.is_interesting() {
        return Vec::new();
    }

    let w = inner_width as usize;
    let mut parts: Vec<Span> = Vec::new();
    parts.push(Span::styled("⎇ ", Style::default().fg(rgb(240, 160, 60))));

    let mut stats_len = 0usize;
    if info.ahead > 0 {
        stats_len += format!(" ↑{}", info.ahead).chars().count();
    }
    if info.behind > 0 {
        stats_len += format!(" ↓{}", info.behind).chars().count();
    }
    if info.modified > 0 {
        stats_len += format!(" ~{}", info.modified).chars().count();
    }
    if info.staged > 0 {
        stats_len += format!(" +{}", info.staged).chars().count();
    }
    if info.untracked > 0 {
        stats_len += format!(" ?{}", info.untracked).chars().count();
    }

    let branch_max = w.saturating_sub(2 + stats_len).max(4);
    parts.push(Span::styled(
        truncate_smart(&info.branch, branch_max),
        Style::default()
            .fg(rgb(200, 200, 210))
            .add_modifier(Modifier::BOLD),
    ));

    if info.modified > 0 {
        parts.push(Span::styled(
            format!(" ~{}", info.modified),
            Style::default().fg(rgb(240, 200, 80)),
        ));
    }
    if info.staged > 0 {
        parts.push(Span::styled(
            format!(" +{}", info.staged),
            Style::default().fg(rgb(100, 200, 100)),
        ));
    }
    if info.untracked > 0 {
        parts.push(Span::styled(
            format!(" ?{}", info.untracked),
            Style::default().fg(rgb(140, 140, 150)),
        ));
    }
    if info.ahead > 0 {
        parts.push(Span::styled(
            format!(" ↑{}", info.ahead),
            Style::default().fg(rgb(100, 200, 100)),
        ));
    }
    if info.behind > 0 {
        parts.push(Span::styled(
            format!(" ↓{}", info.behind),
            Style::default().fg(rgb(255, 140, 100)),
        ));
    }

    let mut lines = vec![Line::from(parts)];
    let max_files = max_lines.saturating_sub(lines.len() as u16).min(5) as usize;
    for file in info.dirty_files.iter().take(max_files) {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                truncate_smart(file, w.saturating_sub(4)),
                Style::default().fg(rgb(140, 140, 155)),
            ),
        ]));
    }
    if info.dirty_files.len() > max_files {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("+{} more", info.dirty_files.len() - max_files),
                Style::default().fg(rgb(100, 100, 115)),
            ),
        ]));
    }
    lines
}

// ---------------------------------------------------------------------------
// Background — copied from render_background_lines
// ---------------------------------------------------------------------------

pub fn render_background_lines(info: &BackgroundInfo, width: usize) -> Vec<Line<'static>> {
    if info.running_count == 0 {
        return Vec::new();
    }
    let summary = format!("Background · {} running", info.running_count);
    let mut lines = vec![Line::from(vec![
        Span::styled("⏳ ", Style::default().fg(rgb(180, 140, 255))),
        Span::styled(summary, Style::default().fg(rgb(160, 160, 170))),
    ])];

    let row_width = width.saturating_sub(4).max(12);
    for (index, task) in info.running_tasks.iter().take(3).enumerate() {
        let detail = if index == 0 {
            info.progress_detail.as_deref()
        } else {
            None
        };
        let row_text = if let Some(detail) = detail {
            truncate_smart(&format!("{task} · {detail}"), row_width)
        } else {
            truncate_smart(task, row_width)
        };
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(rgb(120, 120, 130))),
            Span::styled(row_text, Style::default().fg(rgb(180, 180, 190))),
        ]));
    }

    let hidden = info.running_tasks.len().saturating_sub(3);
    if hidden > 0 {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default().fg(rgb(100, 100, 110))),
            Span::styled(
                format!("+{hidden} more"),
                Style::default().fg(rgb(140, 140, 150)),
            ),
        ]));
    }
    lines
}

// ---------------------------------------------------------------------------
// Compaction — copied from render_compaction_widget
// ---------------------------------------------------------------------------

/// Rough chars→tokens (legacy uses compaction::CHARS_PER_TOKEN ≈ 4).
const CHARS_PER_TOKEN: usize = 4;

pub fn render_compaction_widget(info: &CompactionInfo, inner_width: u16) -> Vec<Line<'static>> {
    let title_color = if info.is_compacting {
        rgb(255, 220, 140)
    } else {
        rgb(110, 210, 140)
    };
    let label_color = rgb(140, 140, 150);
    let status = if info.is_compacting {
        "compacting"
    } else {
        "compacted"
    };
    let summary_tokens = (info.summary_chars / CHARS_PER_TOKEN)
        .max(usize::from(info.summary_chars > 0));
    let detail = format!(
        "{} old · {} active · ~{} summary tok",
        info.compacted_messages, info.active_messages, summary_tokens
    );
    vec![
        Line::from(vec![
            Span::styled("Compaction ", Style::default().fg(label_color)),
            Span::styled(status, Style::default().fg(title_color).bold()),
            Span::styled(
                format!(" · {}", info.mode),
                Style::default().fg(label_color),
            ),
        ]),
        Line::from(Span::styled(
            truncate_smart(&detail, inner_width as usize),
            Style::default().fg(rgb(180, 180, 190)),
        )),
    ]
}

// ---------------------------------------------------------------------------
// Swarm — paste of render_swarm_compact tally (without tui_render dep)
// ---------------------------------------------------------------------------

fn is_active_status(status: &str) -> bool {
    matches!(
        status,
        "running" | "spawned" | "ready" | "blocked" | "waiting_network"
    )
}

fn plan_progress_bar(done: u32, running: u32, total: u32, width: usize) -> Line<'static> {
    const CELL: &str = "▁";
    let cells = width.max(1);
    let total = total.max(1) as usize;
    let done = (done as usize).min(total);
    let running = (running as usize).min(total.saturating_sub(done));
    let mut done_cells = ((done as f64 / total as f64) * cells as f64).round() as usize;
    let mut run_cells = ((running as f64 / total as f64) * cells as f64).round() as usize;
    if done > 0 && done_cells == 0 {
        done_cells = 1;
    }
    if running > 0 && run_cells == 0 {
        run_cells = 1;
    }
    while done_cells + run_cells > cells {
        if run_cells > done_cells {
            run_cells -= 1;
        } else if done_cells > 0 {
            done_cells -= 1;
        } else {
            break;
        }
    }
    let rest = cells.saturating_sub(done_cells + run_cells);
    Line::from(vec![
        Span::styled(
            CELL.repeat(done_cells),
            Style::default().fg(rgb(100, 200, 100)),
        ),
        Span::styled(
            CELL.repeat(run_cells),
            Style::default().fg(rgb(255, 200, 100)),
        ),
        Span::styled(CELL.repeat(rest), Style::default().fg(rgb(60, 60, 72))),
    ])
}

pub fn render_swarm_compact(
    info: &SwarmInfo,
    width: usize,
    max_height: usize,
) -> Vec<Line<'static>> {
    let members = &info.managed_members;
    if members.is_empty() || width < 8 || max_height == 0 {
        return Vec::new();
    }
    let active = members
        .iter()
        .filter(|m| is_active_status(&m.status))
        .count();
    let attention = members
        .iter()
        .filter(|m| {
            matches!(
                m.status.as_str(),
                "blocked" | "failed" | "crashed" | "waiting_network"
            )
        })
        .count();

    let sep_style = Style::default().fg(rgb(80, 80, 90));
    let mut spans: Vec<Span<'static>> = vec![
        Span::styled("🐝 ", Style::default().fg(rgb(255, 200, 100))),
        Span::styled(
            format!("{active}/{} agents", members.len()),
            Style::default().fg(if active > 0 {
                rgb(255, 200, 100)
            } else {
                rgb(120, 120, 130)
            }),
        ),
    ];
    let mut used: usize = spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
    if let Some((done, _running, total)) = info.plan_progress {
        let text = format!("nodes {done}/{total}");
        let tw = UnicodeWidthStr::width(text.as_str());
        if used + 3 + tw <= width {
            used += 3 + tw;
            spans.push(Span::styled(" · ", sep_style));
            spans.push(Span::styled(text, Style::default().fg(rgb(160, 160, 170))));
        }
    }
    if attention > 0 {
        let text = format!("⚠{attention}");
        if used + 3 + UnicodeWidthStr::width(text.as_str()) <= width {
            spans.push(Span::styled(" · ", sep_style));
            spans.push(Span::styled(text, Style::default().fg(rgb(255, 170, 80))));
        }
    }
    let mut out = vec![Line::from(spans)];

    if let Some((done, running, total)) = info.plan_progress
        && total > 0
        && max_height >= 2
    {
        out.push(plan_progress_bar(done, running, total, width));
    }
    out.truncate(max_height);
    out
}

// ---------------------------------------------------------------------------
// Todos — copied from render_todos_compact (no goals/confidence)
// ---------------------------------------------------------------------------

pub fn render_todos_compact(info: &TodosInfo) -> Vec<Line<'static>> {
    if info.items.is_empty() {
        return Vec::new();
    }
    let total = info.items.len();
    let mut completed = 0usize;
    let mut in_progress = 0usize;
    for todo in &info.items {
        match todo.status.as_str() {
            "completed" => completed += 1,
            "in_progress" => in_progress += 1,
            _ => {}
        }
    }
    let pending = total.saturating_sub(completed);
    let label = if info.are_swarm_plan { "Plan" } else { "Todos" };
    let _ = completed;
    vec![
        Line::from(vec![Span::styled(
            label,
            Style::default().fg(rgb(180, 180, 190)).bold(),
        )]),
        Line::from(vec![
            Span::styled(
                format!("{total} total"),
                Style::default().fg(rgb(160, 160, 170)),
            ),
            Span::styled(" · ", Style::default().fg(rgb(100, 100, 110))),
            Span::styled(
                format!("{in_progress} active"),
                Style::default().fg(rgb(255, 200, 100)),
            ),
            Span::styled(" · ", Style::default().fg(rgb(100, 100, 110))),
            Span::styled(
                format!("{pending} open"),
                Style::default().fg(rgb(140, 140, 150)),
            ),
        ]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect()
    }

    #[test]
    fn memory_compact_shows_count() {
        let info = MemoryInfo {
            total_count: 3,
            disabled: false,
            activity_summary: None,
            show_activity: false,
        };
        let lines = render_memory_compact(&info, 36);
        assert_eq!(lines.len(), 1);
        assert!(line_text(&lines[0]).contains("3 memories"));
        assert!(memory_has_data(Some(&info)));
        assert!(!memory_has_data(Some(&MemoryInfo {
            disabled: true,
            total_count: 3,
            ..Default::default()
        })));
    }

    #[test]
    fn git_interesting_gate() {
        let clean = GitInfo {
            branch: "main".into(),
            modified: 0,
            staged: 0,
            untracked: 0,
            ahead: 0,
            behind: 0,
            dirty_files: vec![],
        };
        assert!(!git_has_data(Some(&clean)));
        let dirty = GitInfo {
            modified: 2,
            dirty_files: vec!["a.rs".into()],
            ..clean.clone()
        };
        assert!(git_has_data(Some(&dirty)));
        let lines = render_git_widget(&dirty, 36, 6);
        assert!(!lines.is_empty());
        assert!(line_text(&lines[0]).contains("~2"));
    }

    #[test]
    fn background_empty_when_idle() {
        assert!(!background_has_data(Some(&BackgroundInfo::default())));
        let info = BackgroundInfo {
            running_count: 2,
            running_tasks: vec!["bash".into(), "task".into()],
            progress_detail: Some("compiling".into()),
        };
        let lines = render_background_lines(&info, 40);
        assert!(line_text(&lines[0]).contains("2 running"));
    }

    #[test]
    fn todos_compact_counts() {
        let info = TodosInfo {
            items: vec![
                TodoFloatItem {
                    content: "a".into(),
                    status: "completed".into(),
                },
                TodoFloatItem {
                    content: "b".into(),
                    status: "in_progress".into(),
                },
                TodoFloatItem {
                    content: "c".into(),
                    status: "pending".into(),
                },
            ],
            are_swarm_plan: false,
        };
        let lines = render_todos_compact(&info);
        assert_eq!(line_text(&lines[0]), "Todos");
        let summary = line_text(&lines[1]);
        assert!(summary.contains("3 total"));
        assert!(summary.contains("1 active"));
        assert!(summary.contains("2 open"));
    }

    #[test]
    fn swarm_requires_managed_members() {
        assert!(!swarm_has_data(Some(&SwarmInfo::default())));
        let info = SwarmInfo {
            managed_members: vec![SwarmMemberFloat {
                session_id: "s1".into(),
                friendly_name: Some("fox".into()),
                status: "running".into(),
                detail: None,
                role: None,
            }],
            plan_progress: Some((1, 1, 3)),
        };
        let lines = render_swarm_compact(&info, 34, 3);
        assert!(line_text(&lines[0]).contains("1/1 agents"));
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn usage_credits_bar() {
        let info = UsageInfo {
            provider: UsageProvider::Credits,
            primary_limit_label: Some("Weekly".into()),
            five_hour: 0.42,
            available: true,
            ..Default::default()
        };
        let lines = render_usage_compact(&info, 36);
        assert!(line_text(&lines[0]).contains("Credits"));
        assert!(line_text(&lines[1]).contains("Weekly"));
    }
}
