//! Deferred / disabled legacy info-widget floats — **copy → wire**, not redesign.
//!
//! ## Active now (text-only interim)
//! - [`WorkspaceMapInfo`] + [`render_workspace_lines`] — list labels when rows exist
//! - [`DiagramsInfo`] + [`render_diagrams_lines`] — title list when diagrams exist
//!
//! Both stay **empty-gated** until Face/daemon supplies data (see TODOs below).
//! Buffer/image paint paths are pasted as **commented** copies so we do not lose them.
//!
//! ## Commented stubs only (do not enable yet)
//! - AmbientMode — legacy hard-disabled (`widget_disabled` + `has_data_for => false`)
//! - Tips — same hard-disable
//! - TeamView — `has_data_for(TeamView) => false` (dead)
//!
//! Citations: `next-code-tui/src/tui/info_widget.rs`, `info_widget_tips.rs`,
//! `info_widget_team.rs`, `next-code-tui-workspace/src/workspace_map_widget.rs`.

use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};

use super::{rgb, truncate_smart};

// ---------------------------------------------------------------------------
// WorkspaceMap — data + text interim (wired when row_labels non-empty)
// ---------------------------------------------------------------------------

/// Slim Face stand-in for legacy `VisibleWorkspaceRow` labels.
///
/// Full tile model lives in `next-code-tui-workspace::workspace_map::{
/// VisibleWorkspaceRow, WorkspaceSessionTile, WorkspaceSessionVisualState}`.
#[derive(Debug, Default, Clone)]
pub struct WorkspaceMapInfo {
    /// One label per workspace row (interim until tile buffer paint lands).
    pub row_labels: Vec<String>,
}

pub fn workspace_has_data(info: Option<&WorkspaceMapInfo>) -> bool {
    // Copied gate: `InfoWidgetData::has_data_for(WorkspaceMap)` →
    // `!self.workspace_rows.is_empty()` (`info_widget.rs`).
    info.map(|w| !w.row_labels.is_empty()).unwrap_or(false)
}

/// Text-list interim. Prefer buffer paint (commented below) when
/// `workspace_client` / `VisibleWorkspaceRow` exists on Face.
pub fn render_workspace_lines(info: &WorkspaceMapInfo, width: usize) -> Vec<Line<'static>> {
    if info.row_labels.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![Line::from(vec![Span::styled(
        "Workspace",
        Style::default().fg(rgb(180, 180, 190)).bold(),
    )])];
    for label in info.row_labels.iter().take(4) {
        lines.push(Line::from(vec![
            Span::styled("  · ", Style::default().fg(rgb(100, 100, 110))),
            Span::styled(
                truncate_smart(label, width.saturating_sub(4)),
                Style::default().fg(rgb(140, 140, 150)),
            ),
        ]));
    }
    lines
}

// TODO(face-floats): enable when Face has workspace_client + VisibleWorkspaceRow
// rows (legacy: `TuiState::workspace_map_rows` / `InfoWidgetData.workspace_rows`).
// Copied paint registration from `info_widget.rs::render_single_widget`
// (WorkspaceMap branch) + `workspace_map_widget::render_workspace_map`:
//
// ```
// // info_widget.rs — render_single_widget WorkspaceMap arm (~1359)
// if placement.kind == WidgetKind::WorkspaceMap {
//     if data.workspace_rows.is_empty() || inner.width == 0 || inner.height == 0 {
//         return;
//     }
//     frame.render_widget(block, rect);
//     super::workspace_map_widget::render_workspace_map(
//         frame.buffer_mut(),
//         inner,
//         &data.workspace_rows,
//         data.workspace_animation_tick,
//     );
//     return;
// }
//
// // workspace_map_widget.rs::render_workspace_map
// pub fn render_workspace_map(
//     buf: &mut Buffer,
//     area: Rect,
//     rows: &[VisibleWorkspaceRow],
//     tick: u64,
// ) {
//     clear_area(buf, area);
//     for placement in compute_workspace_tile_placements(area, rows) {
//         draw_workspace_tile(buf, placement, tick);
//     }
// }
//
// // height: info_widget.rs::calculate_widget_height WorkspaceMap arm (~1073)
// WidgetKind::WorkspaceMap => {
//     if data.workspace_rows.is_empty() { return 0; }
//     let (_preferred_w, preferred_h) =
//         super::workspace_map_widget::preferred_size(&data.workspace_rows);
//     preferred_h.min(max_height.saturating_sub(border_height))
// }
//
// // data fetch (commented call-site for pager_agent / build_info_float_data):
// // workspace_map: Some(WorkspaceMapInfo { row_labels: … from VisibleWorkspaceRow }),
// // or paint via render_workspace_map once Face owns the tile buffer path.
// ```

// ---------------------------------------------------------------------------
// Diagrams — data + text interim (wired when titles non-empty)
// ---------------------------------------------------------------------------

/// Slim Face stand-in for legacy `DiagramInfo` list (`next_code_tui_mermaid`).
#[derive(Debug, Default, Clone)]
pub struct DiagramsInfo {
    pub titles: Vec<String>,
}

pub fn diagrams_has_data(info: Option<&DiagramsInfo>) -> bool {
    // Copied gate: `has_data_for(Diagrams)` → `!self.diagrams.is_empty()`.
    info.map(|d| !d.titles.is_empty()).unwrap_or(false)
}

/// Text-list interim. Prefer mermaid image paint (commented below) when the
/// float image pipeline is available on Face.
pub fn render_diagrams_lines(info: &DiagramsInfo, width: usize) -> Vec<Line<'static>> {
    if info.titles.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![Line::from(vec![Span::styled(
        "Diagrams",
        Style::default().fg(rgb(180, 180, 190)).bold(),
    )])];
    for title in info.titles.iter().take(3) {
        lines.push(Line::from(vec![Span::styled(
            truncate_smart(title, width.saturating_sub(2)),
            Style::default().fg(rgb(140, 140, 150)),
        )]));
    }
    lines
}

// TODO(face-floats): enable when diagrams image pipeline is on Face floats
// (scrollback already has mermaid; float needs `render_image_widget_scale` +
// `DiagramInfo.hash`). Copied from `info_widget.rs`:
//
// ```
// // render_single_widget Diagrams arm (~1341)
// if placement.kind == WidgetKind::Diagrams {
//     frame.render_widget(block, rect);
//     render_diagrams_widget(frame, inner, data);
//     return;
// }
//
// // render_diagrams_widget (~1382)
// fn render_diagrams_widget(frame: &mut Frame, inner: Rect, data: &InfoWidgetData) {
//     if data.diagrams.is_empty() {
//         return;
//     }
//     // For now, just render the first/most recent diagram
//     let diagram = &data.diagrams[0];
//     // Scale up as well as down so margin diagrams use the whole widget
//     super::mermaid::render_image_widget_scale(
//         diagram.hash,
//         inner,
//         frame.buffer_mut(),
//         false,
//     );
// }
//
// // height (~1092): full available height minus border when diagrams non-empty
// WidgetKind::Diagrams => {
//     if data.diagrams.is_empty() { return 0; }
//     max_height.saturating_sub(border_height)
// }
//
// // render_widget_content returns Vec::new() for Diagrams — paint is special-cased.
//
// // data fetch (build_info_float_data): fold active mermaid titles/hashes from
// // scrollback agent blocks once float image registration exists.
// ```

// ---------------------------------------------------------------------------
// AmbientMode — COMMENT ONLY (legacy hard-disabled)
// ---------------------------------------------------------------------------
//
// TODO(face-floats): AmbientMode stub — do not wire.
// Reason: legacy `InfoWidgetData::widget_disabled` matches AmbientMode|Tips
// (`info_widget.rs` ~680), and `has_data_for(AmbientMode) => false` (~785).
// Copied from `info_widget.rs::render_ambient_widget` (~1858) + types
// `AmbientWidgetData` / `AmbientStatus` (`info_widget.rs` ~583, `ambient.rs`):
//
// ```
// #[derive(Debug, Clone)]
// pub struct AmbientWidgetData {
//     pub show_widget: bool,
//     pub status: AmbientStatus,
//     pub queue_count: usize,
//     pub next_queue_preview: Option<String>,
//     pub reminder_count: usize,
//     pub next_reminder_preview: Option<String>,
//     pub last_run_ago: Option<String>,
//     pub last_summary: Option<String>,
//     pub next_wake: Option<String>,
//     pub next_reminder_wake: Option<String>,
//     pub budget_percent: Option<f32>,
// }
//
// fn render_ambient_widget(data: &InfoWidgetData, inner: Rect) -> Vec<Line<'static>> {
//     let Some(info) = &data.ambient_info else { return Vec::new(); };
//     if !info.show_widget { return Vec::new(); }
//     let mut lines: Vec<Line> = Vec::new();
//     let dim = rgb(100, 100, 110);
//     let label_color = rgb(140, 140, 150);
//     let max_w = inner.width.saturating_sub(2) as usize;
//     let (icon, status_text, status_color) = match &info.status {
//         AmbientStatus::Idle => ("○", "Idle".to_string(), rgb(120, 120, 130)),
//         AmbientStatus::Running { detail } =>
//             ("●", format!("Running: {}", detail), rgb(100, 200, 100)),
//         AmbientStatus::Scheduled { .. } =>
//             ("◐", "Waiting for next run".to_string(), rgb(140, 180, 255)),
//         AmbientStatus::Paused { reason } => (
//             "⏸",
//             format!("Paused: {}", truncate_smart(reason, inner.width.saturating_sub(12) as usize)),
//             rgb(255, 200, 100),
//         ),
//         AmbientStatus::Disabled if info.reminder_count > 0 =>
//             ("⏰", "Scheduled tasks active".to_string(), rgb(140, 180, 255)),
//         AmbientStatus::Disabled => ("○", "Not running".to_string(), dim),
//     };
//     lines.push(Line::from(vec![
//         Span::styled(format!("{} ", icon), Style::default().fg(status_color)),
//         Span::styled(
//             truncate_smart(&status_text, inner.width.saturating_sub(3) as usize),
//             Style::default().fg(rgb(180, 180, 190)),
//         ),
//     ]));
//     // … queue / last-run / next-wake / budget bar (see info_widget.rs ~1904–2004)
//     lines
// }
// ```

// ---------------------------------------------------------------------------
// Tips — COMMENT ONLY (legacy hard-disabled)
// ---------------------------------------------------------------------------
//
// TODO(face-floats): Tips stub — do not wire.
// Reason: `widget_disabled(Tips)` + `has_data_for(Tips) => false`
// (`info_widget.rs` ~680 / ~793). Copied from `info_widget_tips.rs`:
//
// ```
// pub(super) fn tips_widget_height(inner_width: usize) -> u16 {
//     let effective_w = inner_width.saturating_sub(2);
//     let tip = current_tip(effective_w);
//     let lines = wrap_tip_text(&tip.text, effective_w);
//     1 + lines.len() as u16
// }
//
// pub(super) fn render_tips_widget(inner: Rect) -> Vec<Line<'static>> {
//     let w = inner.width.saturating_sub(2) as usize;
//     let tip = current_tip(w);
//     let wrapped = wrap_tip_text(&tip.text, w);
//     let mut lines: Vec<Line<'static>> = Vec::new();
//     lines.push(Line::from(vec![
//         Span::styled("💡 ", Style::default().fg(rgb(255, 210, 80))),
//         Span::styled(
//             "Did you know?",
//             Style::default().fg(rgb(200, 200, 210)).add_modifier(Modifier::BOLD),
//         ),
//     ]));
//     for line_text in wrapped {
//         lines.push(Line::from(vec![
//             Span::raw("  "),
//             Span::styled(line_text, Style::default().fg(rgb(160, 160, 175))),
//         ]));
//     }
//     lines
// }
// // tip cycle: TIP_CYCLE_SECONDS=15, all_tips() list in info_widget_tips.rs
// ```

// ---------------------------------------------------------------------------
// TeamView — COMMENT ONLY (legacy dead has_data_for)
// ---------------------------------------------------------------------------
//
// TODO(face-floats): TeamView stub — do not wire.
// Reason: `has_data_for(TeamView) => false` always (`info_widget.rs` ~778)
// even when `team_info` is populated. Copied from `info_widget_team.rs`:
//
// ```
// #[derive(Debug, Default, Clone)]
// pub struct TeamInfo {
//     pub team_name: String,
//     pub member_total: usize,
//     pub members: Vec<TeamMemberView>,
//     pub tasks: Vec<TeamTaskView>,
// }
//
// pub(super) fn render_team_widget(data: &InfoWidgetData, inner: Rect) -> Vec<Line<'static>> {
//     let Some(info) = &data.team_info else { return Vec::new(); };
//     let mut lines = Vec::new();
//     let active = info.members.iter().filter(|m| m.status == "running").count();
//     lines.push(Line::from(vec![
//         Span::styled("👥 ", Style::default().fg(rgb(255, 200, 100))),
//         Span::styled(
//             truncate_smart(&info.team_name, inner.width.saturating_sub(20) as usize),
//             Style::default().fg(rgb(220, 220, 230)).bold(),
//         ),
//         Span::styled(
//             format!(
//                 " {}/{} · {} active · {} tasks",
//                 info.members.len(), info.member_total, active, info.tasks.len()
//             ),
//             Style::default().fg(rgb(140, 140, 150)),
//         ),
//     ]));
//     // … member rows + task DAG (info_widget_team.rs ~107–159)
//     lines
// }
// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_empty_without_data() {
        assert!(render_workspace_lines(&WorkspaceMapInfo::default(), 30).is_empty());
        assert!(render_diagrams_lines(&DiagramsInfo::default(), 30).is_empty());
        assert!(!workspace_has_data(None));
        assert!(!diagrams_has_data(None));
    }

    #[test]
    fn workspace_text_lists_labels() {
        let info = WorkspaceMapInfo {
            row_labels: vec!["ws0".into(), "ws1".into()],
        };
        assert!(workspace_has_data(Some(&info)));
        let lines = render_workspace_lines(&info, 40);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn diagrams_text_lists_titles() {
        let info = DiagramsInfo {
            titles: vec!["flow".into()],
        };
        assert!(diagrams_has_data(Some(&info)));
        let lines = render_diagrams_lines(&info, 40);
        assert_eq!(lines.len(), 2);
    }
}
