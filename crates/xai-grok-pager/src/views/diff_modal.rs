//! Face `/diff` modal — Claude DiffDialog parity over git HEAD and scrollback edits.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use similar::ChangeTag;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::app_view::InputOutcome;
use crate::diff::{DiffHunk, DiffLine, diff_hunks_from_strings, stitch_overlapping_hunks};
use crate::render::SafeBuf;
use crate::render::scrollbar::render_scrollbar;
use crate::scrollback::block::RenderBlock;
use crate::scrollback::blocks::tool::{DiffRenderConfig, EditToolCallBlock, ToolCallBlock};
use crate::scrollback::blocks::tool::render_diff_hunks_highlighted;
use crate::scrollback::state::ScrollbackState;
use crate::theme::Theme;
use crate::views::modal_window::{
    self, ModalContentArea, ModalSizing, ModalWindowConfig, ModalWindowState, Shortcut,
};

/// Outcome from key/mouse handling in the diff modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffModalOutcome {
    Changed,
    Unchanged,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffModalMode {
    FileList,
    FileDetail,
}

#[derive(Debug, Clone)]
pub struct DiffFileEntry {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub untracked: bool,
    pub hunks: Vec<DiffHunk>,
    /// Raw unified diff when hunks could not be parsed (git fallback).
    pub raw_unified: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffSourceSnapshot {
    pub label: String,
    pub files: Vec<DiffFileEntry>,
}

#[derive(Debug)]
pub struct DiffModalState {
    pub window: ModalWindowState,
    pub sources: Vec<DiffSourceSnapshot>,
    pub source_idx: usize,
    pub selected_file: usize,
    pub list_scroll: usize,
    pub mode: DiffModalMode,
    pub detail_scroll: usize,
    /// Cached rendered detail lines `(width, lines)` for the selected file.
    detail_cache: Option<(u16, Vec<Line<'static>>)>,
    detail_total_lines: usize,
    list_area: Rect,
    detail_area: Rect,
    list_scrollbar_area: Option<Rect>,
    detail_scrollbar_area: Option<Rect>,
}

impl DiffModalState {
    pub fn new(sources: Vec<DiffSourceSnapshot>) -> Self {
        Self {
            window: ModalWindowState::new(),
            sources,
            source_idx: 0,
            selected_file: 0,
            list_scroll: 0,
            mode: DiffModalMode::FileList,
            detail_scroll: 0,
            detail_cache: None,
            detail_total_lines: 0,
            list_area: Rect::default(),
            detail_area: Rect::default(),
            list_scrollbar_area: None,
            detail_scrollbar_area: None,
        }
    }

    pub fn build(scrollback: &ScrollbackState, cwd: &Path) -> Self {
        let mut sources = Vec::new();
        sources.push(collect_git_diff(cwd));
        sources.extend(collect_turn_diffs(scrollback));
        sources.retain(|s| !s.files.is_empty());
        if sources.is_empty() {
            sources.push(DiffSourceSnapshot {
                label: "Current".to_string(),
                files: Vec::new(),
            });
        }
        Self::new(sources)
    }

    fn current_source(&self) -> Option<&DiffSourceSnapshot> {
        self.sources.get(self.source_idx)
    }

    fn file_count(&self) -> usize {
        self.current_source().map(|s| s.files.len()).unwrap_or(0)
    }

    fn selected_entry(&self) -> Option<&DiffFileEntry> {
        self.current_source()
            .and_then(|s| s.files.get(self.selected_file))
    }

    fn clamp_file_selection(&mut self) {
        let count = self.file_count();
        if count == 0 {
            self.selected_file = 0;
            self.list_scroll = 0;
            return;
        }
        if self.selected_file >= count {
            self.selected_file = count - 1;
        }
    }

    fn switch_source(&mut self, delta: i32) {
        if self.sources.is_empty() {
            return;
        }
        let len = self.sources.len() as i32;
        let next = (self.source_idx as i32 + delta).rem_euclid(len) as usize;
        if next != self.source_idx {
            self.source_idx = next;
            self.clamp_file_selection();
            self.detail_scroll = 0;
            self.detail_cache = None;
        }
    }

    fn select_next_file(&mut self) {
        let count = self.file_count();
        if count == 0 {
            return;
        }
        if self.selected_file + 1 < count {
            self.selected_file += 1;
            self.detail_scroll = 0;
            self.detail_cache = None;
        }
    }

    fn select_prev_file(&mut self) {
        if self.selected_file > 0 {
            self.selected_file -= 1;
            self.detail_scroll = 0;
            self.detail_cache = None;
        }
    }

    pub fn enter_detail(&mut self) {
        if self.selected_entry().is_some() {
            self.mode = DiffModalMode::FileDetail;
            self.detail_scroll = 0;
            self.detail_cache = None;
        }
    }

    pub fn back_to_list(&mut self) {
        self.mode = DiffModalMode::FileList;
        self.detail_cache = None;
    }
}

/// Collect `git diff HEAD` plus untracked file names for the **Current** source.
pub fn collect_git_diff(cwd: &Path) -> DiffSourceSnapshot {
    let mut by_path: HashMap<String, DiffFileEntry> = HashMap::new();

    if let Ok(output) = Command::new("git")
        .args(["--no-optional-locks", "diff", "HEAD", "--numstat"])
        .current_dir(cwd)
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let Some(entry) = parse_numstat_line(line, cwd) else {
                continue;
            };
            by_path.insert(entry.path.clone(), entry);
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(cwd)
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for rel in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
            if by_path.contains_key(rel) {
                continue;
            }
            let full = cwd.join(rel);
            let content = std::fs::read_to_string(&full).unwrap_or_default();
            let hunks = diff_hunks_from_strings("", &content, 1);
            let (additions, deletions) = count_hunk_changes(&hunks);
            by_path.insert(
                rel.to_string(),
                DiffFileEntry {
                    path: rel.to_string(),
                    additions,
                    deletions,
                    untracked: true,
                    hunks,
                    raw_unified: None,
                },
            );
        }
    }

    let mut files: Vec<_> = by_path.into_values().collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    DiffSourceSnapshot {
        label: "Current".to_string(),
        files,
    }
}

fn parse_numstat_line(line: &str, cwd: &Path) -> Option<DiffFileEntry> {
    let mut parts = line.split('\t');
    let adds = parts.next()?.trim();
    let dels = parts.next()?.trim();
    let path = parts.next()?.trim();
    if path.is_empty() {
        return None;
    }

    let additions = adds.parse::<usize>().unwrap_or(0);
    let deletions = dels.parse::<usize>().unwrap_or(0);

    let unified = Command::new("git")
        .args(["--no-optional-locks", "diff", "HEAD", "--", path])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    let hunks = if unified.is_empty() {
        Vec::new()
    } else {
        parse_unified_diff_hunks(&unified)
    };
    let (additions, deletions) = if hunks.is_empty() {
        (additions, deletions)
    } else {
        count_hunk_changes(&hunks)
    };

    Some(DiffFileEntry {
        path: path.to_string(),
        additions,
        deletions,
        untracked: false,
        hunks: hunks.clone(),
        raw_unified: if hunks.is_empty() && !unified.is_empty() {
            Some(unified)
        } else {
            None
        },
    })
}

/// Per-turn edit snapshots from scrollback, most recent turn first.
pub fn collect_turn_diffs(scrollback: &ScrollbackState) -> Vec<DiffSourceSnapshot> {
    let turns = scrollback.turns();
    let mut out = Vec::new();

    for turn_idx in (0..turns.len()).rev() {
        let turn = &turns[turn_idx];
        let mut by_path: HashMap<String, Vec<DiffHunk>> = HashMap::new();

        for entry_idx in turn.range() {
            let Some(entry) = scrollback.entry(entry_idx) else {
                continue;
            };
            let RenderBlock::ToolCall(ToolCallBlock::Edit(edit)) = &entry.block else {
                continue;
            };
            if edit.hunks.is_empty() {
                continue;
            }
            by_path
                .entry(edit.path.clone())
                .or_default()
                .extend(edit.hunks.iter().cloned());
        }

        if by_path.is_empty() {
            continue;
        }

        let mut files = Vec::new();
        for (path, hunks) in by_path {
            let stitched = stitch_overlapping_hunks(hunks);
            let (additions, deletions) = count_hunk_changes(&stitched);
            files.push(DiffFileEntry {
                path,
                additions,
                deletions,
                untracked: false,
                hunks: stitched,
                raw_unified: None,
            });
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let turn_num = turn_idx + 1;
        let preview = scrollback
            .turn_preview(turn_idx)
            .map(|p| truncate_preview(&p, 40))
            .unwrap_or_default();
        let label = if preview.is_empty() {
            format!("T{turn_num}")
        } else {
            format!("T{turn_num} · {preview}")
        };

        out.push(DiffSourceSnapshot { label, files });
    }

    out
}

fn truncate_preview(text: &str, max_cols: usize) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.width() <= max_cols {
        return flat;
    }
    let mut out = String::new();
    for ch in flat.chars() {
        if out.width() + ch.width().unwrap_or(0) + 3 > max_cols {
            break;
        }
        out.push(ch);
    }
    out.push_str("...");
    out
}

pub fn count_hunk_changes(hunks: &[DiffHunk]) -> (usize, usize) {
    let mut additions = 0usize;
    let mut deletions = 0usize;
    for hunk in hunks {
        for line in hunk {
            match line.tag {
                ChangeTag::Insert => additions += 1,
                ChangeTag::Delete => deletions += 1,
                ChangeTag::Equal => {}
            }
        }
    }
    (additions, deletions)
}

/// Parse unified diff body lines into [`DiffHunk`]s.
pub fn parse_unified_diff_hunks(text: &str) -> Vec<DiffHunk> {
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut current: DiffHunk = Vec::new();
    let mut old_line = 1usize;
    let mut new_line = 1usize;

    for line in text.lines() {
        if let Some(header) = line.strip_prefix("@@ ") {
            if !current.is_empty() {
                hunks.push(current);
                current = Vec::new();
            }
            if let Some((o, n)) = parse_hunk_header(header) {
                old_line = o;
                new_line = n;
            }
            continue;
        }
        if line.starts_with("---")
            || line.starts_with("+++")
            || line.starts_with("diff ")
            || line.starts_with("index ")
        {
            continue;
        }
        let Some(first) = line.chars().next() else {
            continue;
        };
        let body = &line[1..];
        match first {
            ' ' => {
                current.push(DiffLine {
                    text: format!("{body}\n"),
                    lo: old_line,
                    ln: new_line,
                    tag: ChangeTag::Equal,
                });
                old_line += 1;
                new_line += 1;
            }
            '+' => {
                current.push(DiffLine {
                    text: format!("{body}\n"),
                    lo: old_line,
                    ln: new_line,
                    tag: ChangeTag::Insert,
                });
                new_line += 1;
            }
            '-' => {
                current.push(DiffLine {
                    text: format!("{body}\n"),
                    lo: old_line,
                    ln: new_line,
                    tag: ChangeTag::Delete,
                });
                old_line += 1;
            }
            _ => {}
        }
    }
    if !current.is_empty() {
        hunks.push(current);
    }
    hunks
}

fn parse_hunk_header(header: &str) -> Option<(usize, usize)> {
    // -old_start,old_count +new_start,new_count @@
    let rest = header.strip_suffix(" @@")?;
    let mut parts = rest.split_whitespace();
    let old_part = parts.next()?;
    let new_part = parts.next()?;
    let old_start = old_part
        .trim_start_matches('-')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    let new_start = new_part
        .trim_start_matches('+')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    Some((old_start, new_start))
}

pub fn handle_diff_key(state: &mut DiffModalState, key: &KeyEvent) -> DiffModalOutcome {
    if key.kind == KeyEventKind::Release {
        return DiffModalOutcome::Unchanged;
    }
    if key.modifiers.is_empty() {
        match key.code {
            KeyCode::Esc => {
                if state.mode == DiffModalMode::FileDetail {
                    state.back_to_list();
                    return DiffModalOutcome::Changed;
                }
                return DiffModalOutcome::Close;
            }
            KeyCode::Enter if state.mode == DiffModalMode::FileList => {
                state.enter_detail();
                return DiffModalOutcome::Changed;
            }
            KeyCode::Left => {
                if state.mode == DiffModalMode::FileDetail {
                    state.back_to_list();
                } else {
                    state.switch_source(-1);
                }
                return DiffModalOutcome::Changed;
            }
            KeyCode::Right => {
                state.switch_source(1);
                return DiffModalOutcome::Changed;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.mode == DiffModalMode::FileList {
                    state.select_next_file();
                } else {
                    state.detail_scroll = state.detail_scroll.saturating_add(3);
                }
                return DiffModalOutcome::Changed;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if state.mode == DiffModalMode::FileList {
                    state.select_prev_file();
                } else {
                    state.detail_scroll = state.detail_scroll.saturating_sub(3);
                }
                return DiffModalOutcome::Changed;
            }
            KeyCode::PageDown => {
                if state.mode == DiffModalMode::FileDetail {
                    state.detail_scroll = state.detail_scroll.saturating_add(10);
                    return DiffModalOutcome::Changed;
                }
            }
            KeyCode::PageUp => {
                if state.mode == DiffModalMode::FileDetail {
                    state.detail_scroll = state.detail_scroll.saturating_sub(10);
                    return DiffModalOutcome::Changed;
                }
            }
            _ => {}
        }
    }
    DiffModalOutcome::Unchanged
}

pub fn handle_diff_mouse(
    state: &mut DiffModalState,
    kind: MouseEventKind,
    column: u16,
    row: u16,
) -> DiffModalOutcome {
    let in_rect = |r: Rect| {
        r.width > 0
            && r.height > 0
            && column >= r.x
            && column < r.x + r.width
            && row >= r.y
            && row < r.y + r.height
    };

    match kind {
        MouseEventKind::ScrollDown => {
            if state.mode == DiffModalMode::FileDetail && in_rect(state.detail_area) {
                state.detail_scroll = state.detail_scroll.saturating_add(3);
                return DiffModalOutcome::Changed;
            }
            if state.mode == DiffModalMode::FileList && in_rect(state.list_area) {
                state.select_next_file();
                return DiffModalOutcome::Changed;
            }
            DiffModalOutcome::Unchanged
        }
        MouseEventKind::ScrollUp => {
            if state.mode == DiffModalMode::FileDetail && in_rect(state.detail_area) {
                state.detail_scroll = state.detail_scroll.saturating_sub(3);
                return DiffModalOutcome::Changed;
            }
            if state.mode == DiffModalMode::FileList && in_rect(state.list_area) {
                state.select_prev_file();
                return DiffModalOutcome::Changed;
            }
            DiffModalOutcome::Unchanged
        }
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            if state.mode == DiffModalMode::FileList && in_rect(state.list_area) {
                let row_rel = row.saturating_sub(state.list_area.y) as usize;
                let idx = state.list_scroll + row_rel;
                if idx < state.file_count() {
                    state.selected_file = idx;
                    state.detail_cache = None;
                    return DiffModalOutcome::Changed;
                }
            }
            DiffModalOutcome::Unchanged
        }
        _ => DiffModalOutcome::Unchanged,
    }
}

pub fn render_diff_modal(
    buf: &mut Buffer,
    full_area: Rect,
    state: &mut DiffModalState,
    compact: bool,
) {
    let theme = Theme::current();
    let source_label = state
        .current_source()
        .map(|s| s.label.as_str())
        .unwrap_or("Current");
    let title = match state.mode {
        DiffModalMode::FileList => format!("Diff · {source_label}"),
        DiffModalMode::FileDetail => state
            .selected_entry()
            .map(|e| format!("Diff · {}", e.path))
            .unwrap_or_else(|| "Diff".to_string()),
    };

    let shortcuts = build_shortcuts(&state.mode);
    let modal_config = ModalWindowConfig {
        title: &title,
        tabs: None,
        shortcuts: &shortcuts,
        sizing: ModalSizing {
            width_pct: 0.80,
            max_width: 120,
            min_width: 48,
            v_margin: 3,
            h_pad: 2,
            v_pad: 1,
            footer_lines: 2,
        }
        .with_compact(compact),
        fold_info: None,
    };

    let Some(ModalContentArea {
        content: content_area,
        ..
    }) = modal_window::render_modal_window(buf, full_area, &mut state.window, &modal_config, &theme)
    else {
        return;
    };

    if content_area.height < 2 || content_area.width < 10 {
        return;
    }

    match state.mode {
        DiffModalMode::FileList => render_file_list(buf, content_area, state, &theme),
        DiffModalMode::FileDetail => render_file_detail(buf, content_area, state, &theme),
    }
}

fn build_shortcuts(mode: &DiffModalMode) -> Vec<Shortcut<'static>> {
    match mode {
        DiffModalMode::FileList => vec![
            Shortcut {
                label: "\u{2190}/\u{2192} source",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "\u{2191}/\u{2193} nav",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Enter detail",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Esc close",
                clickable: false,
                id: 0,
            },
        ],
        DiffModalMode::FileDetail => vec![
            Shortcut {
                label: "\u{2190} back",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "\u{2192} source",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "\u{2191}/\u{2193} scroll",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Esc back",
                clickable: false,
                id: 0,
            },
        ],
    }
}

fn render_file_list(buf: &mut Buffer, area: Rect, state: &mut DiffModalState, theme: &Theme) {
    state.list_area = area;
    state.detail_area = Rect::default();
    state.detail_scrollbar_area = None;

    let files = state
        .current_source()
        .map(|s| &s.files)
        .cloned()
        .unwrap_or_default();

    if files.is_empty() {
        let msg = "No file changes in this source";
        buf.set_span(
            area.x,
            area.y + area.height / 2,
            &Span::styled(msg, Style::default().fg(theme.gray_dim).bg(theme.bg_base)),
            area.width,
        );
        state.list_scrollbar_area = None;
        return;
    }

    let visible_h = area.height as usize;
    if state.selected_file < state.list_scroll {
        state.list_scroll = state.selected_file;
    }
    if state.selected_file >= state.list_scroll + visible_h {
        state.list_scroll = state.selected_file.saturating_sub(visible_h.saturating_sub(1));
    }

    let total = files.len();
    let sb_area = if total > visible_h && area.width > 4 {
        Some(Rect {
            x: area.x + area.width - 1,
            y: area.y,
            width: 1,
            height: area.height,
        })
    } else {
        None
    };
    state.list_scrollbar_area = sb_area;
    let content_w = area.width.saturating_sub(if sb_area.is_some() { 2 } else { 0 });

    let end = total.min(state.list_scroll + visible_h);
    for (row, file) in files[state.list_scroll..end].iter().enumerate() {
        let y = area.y + row as u16;
        let idx = state.list_scroll + row;
        let selected = idx == state.selected_file;
        let bg = if selected {
            theme.bg_visual
        } else {
            theme.bg_base
        };
        let marker = if selected { '›' } else { ' ' };
        let stats = format!(
            "+{} -{}",
            file.additions, file.deletions
        );
        let untagged = if file.untracked { " (new)" } else { "" };
        let label = format!("{marker} {}{untagged}", file.path);
        let label_style = Style::default().fg(theme.text_primary).bg(bg);
        let stats_style = Style::default().fg(theme.gray).bg(bg);
        buf.set_style(
            Rect {
                x: area.x,
                y,
                width: content_w,
                height: 1,
            },
            Style::default().bg(bg),
        );
        buf.set_span(area.x + 1, y, &Span::styled(label, label_style), content_w.saturating_sub(1));
        let stats_w = stats.width() as u16;
        let stats_x = area.x + content_w.saturating_sub(stats_w + 1);
        if stats_x > area.x + 2 {
            buf.set_span(stats_x, y, &Span::styled(stats, stats_style), stats_w);
        }
    }

    render_scrollbar(
        buf,
        sb_area,
        sat_u16(total),
        sat_u16(visible_h),
        sat_u16(state.list_scroll),
        false,
    );
}

fn render_file_detail(buf: &mut Buffer, area: Rect, state: &mut DiffModalState, theme: &Theme) {
    state.detail_area = area;
    state.list_area = Rect::default();
    state.list_scrollbar_area = None;
    buf.set_style(area, Style::default().bg(theme.bg_base));

    let Some(file) = state.selected_entry().cloned() else {
        let msg = "No file selected";
        buf.set_span(
            area.x,
            area.y + area.height / 2,
            &Span::styled(msg, Style::default().fg(theme.gray_dim).bg(theme.bg_base)),
            area.width,
        );
        return;
    };

    let width = area.width;
    let needs_rebuild = state
        .detail_cache
        .as_ref()
        .is_none_or(|(w, _)| *w != width);
    if needs_rebuild {
        state.detail_cache = Some((width, build_detail_lines(&file, width, theme)));
    }

    let lines = state.detail_cache.as_ref().unwrap().1.clone();
    state.detail_total_lines = lines.len();
    let visible = area.height as usize;
    let max_scroll = lines.len().saturating_sub(visible);
    state.detail_scroll = state.detail_scroll.min(max_scroll);
    let scroll = state.detail_scroll;

    let sb_area = if lines.len() > visible && area.width > 4 {
        Some(Rect {
            x: area.x + area.width - 1,
            y: area.y,
            width: 1,
            height: area.height,
        })
    } else {
        None
    };
    state.detail_scrollbar_area = sb_area;
    let content_w = area.width.saturating_sub(if sb_area.is_some() { 2 } else { 0 });

    for (row, line) in lines.iter().skip(scroll).take(visible).enumerate() {
        let y = area.y + row as u16;
        buf.set_line_safe(area.x, y, line, content_w);
    }

    render_scrollbar(
        buf,
        sb_area,
        sat_u16(lines.len()),
        sat_u16(visible),
        sat_u16(scroll),
        false,
    );
}

fn build_detail_lines(file: &DiffFileEntry, width: u16, theme: &Theme) -> Vec<Line<'static>> {
    if !file.hunks.is_empty() {
        let config = DiffRenderConfig::default();
        let path = PathBuf::from(&file.path);
        let rendered = render_diff_hunks_highlighted(&file.hunks, &path, theme, width, &config);
        return rendered.into_iter().map(|dl| dl.line).collect();
    }

    if let Some(raw) = &file.raw_unified {
        return raw
            .lines()
            .map(|line| {
                let style = if line.starts_with('+') && !line.starts_with("+++") {
                    Style::default().fg(theme.diff_insert_fg)
                } else if line.starts_with('-') && !line.starts_with("---") {
                    Style::default().fg(theme.diff_delete_fg)
                } else if line.starts_with("@@") {
                    Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_primary)
                };
                Line::from(Span::styled(line.to_string(), style))
            })
            .collect();
    }

    vec![Line::from(Span::styled(
        "(No diff content)",
        Style::default().fg(theme.gray_dim),
    ))]
}

fn sat_u16(v: usize) -> u16 {
    v.min(u16::MAX as usize) as u16
}

pub fn outcome_to_input(outcome: DiffModalOutcome) -> InputOutcome {
    match outcome {
        DiffModalOutcome::Changed => InputOutcome::Changed,
        DiffModalOutcome::Unchanged => InputOutcome::Unchanged,
        DiffModalOutcome::Close => InputOutcome::Changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrollback::block::RenderBlock;
    use crate::scrollback::state::ScrollbackState;
    use crossterm::event::KeyModifiers;

    fn sample_hunk() -> DiffHunk {
        diff_hunks_from_strings("old\n", "new\n", 1)
            .into_iter()
            .next()
            .unwrap()
    }

    #[test]
    fn count_hunk_changes_tags() {
        let hunks = diff_hunks_from_strings("a\nb\n", "a\nc\n", 1);
        let (add, del) = count_hunk_changes(&hunks);
        assert!(add >= 1);
        assert!(del >= 1);
    }

    #[test]
    fn parse_unified_diff_basic() {
        let unified = "\
--- a/file.rs
+++ b/file.rs
@@ -1,2 +1,2 @@
 old
-new
+newer
";
        let hunks = parse_unified_diff_hunks(unified);
        assert_eq!(hunks.len(), 1);
        let (add, del) = count_hunk_changes(&hunks);
        assert_eq!(add, 1);
        assert_eq!(del, 1);
    }

    #[test]
    fn collect_turn_diffs_groups_by_turn_and_path() {
        let mut scrollback = ScrollbackState::new();
        scrollback.push_block(RenderBlock::user_prompt("fix bug"));
        scrollback.push_block(RenderBlock::ToolCall(ToolCallBlock::Edit(
            EditToolCallBlock::new("src/a.rs", vec![sample_hunk()]),
        )));
        scrollback.push_block(RenderBlock::user_prompt("second"));
        scrollback.push_block(RenderBlock::ToolCall(ToolCallBlock::Edit(
            EditToolCallBlock::new("src/b.rs", vec![sample_hunk()]),
        )));

        let sources = collect_turn_diffs(&scrollback);
        assert_eq!(sources.len(), 2);
        assert!(sources[0].label.starts_with("T2"));
        assert_eq!(sources[0].files[0].path, "src/b.rs");
        assert!(sources[1].label.starts_with("T1"));
        assert_eq!(sources[1].files[0].path, "src/a.rs");
    }

    #[test]
    fn collect_turn_diffs_skips_empty_hunks() {
        let mut scrollback = ScrollbackState::new();
        scrollback.push_block(RenderBlock::user_prompt("q"));
        scrollback.push_block(RenderBlock::ToolCall(ToolCallBlock::Edit(
            EditToolCallBlock::new("empty.rs", Vec::new()),
        )));
        assert!(collect_turn_diffs(&scrollback).is_empty());
    }

    #[test]
    fn key_nav_list_and_detail() {
        let mut state = DiffModalState::new(vec![
            DiffSourceSnapshot {
                label: "Current".into(),
                files: vec![DiffFileEntry {
                    path: "a.rs".into(),
                    additions: 1,
                    deletions: 0,
                    untracked: false,
                    hunks: vec![sample_hunk()],
                    raw_unified: None,
                }],
            },
            DiffSourceSnapshot {
                label: "T1".into(),
                files: vec![DiffFileEntry {
                    path: "b.rs".into(),
                    additions: 2,
                    deletions: 1,
                    untracked: false,
                    hunks: vec![sample_hunk()],
                    raw_unified: None,
                }],
            },
        ]);

        assert_eq!(
            handle_diff_key(
                &mut state,
                &KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)
            ),
            DiffModalOutcome::Changed
        );
        assert_eq!(state.source_idx, 1);

        assert_eq!(
            handle_diff_key(
                &mut state,
                &KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
            ),
            DiffModalOutcome::Changed
        );
        assert_eq!(state.mode, DiffModalMode::FileDetail);

        assert_eq!(
            handle_diff_key(
                &mut state,
                &KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)
            ),
            DiffModalOutcome::Changed
        );
        assert_eq!(state.mode, DiffModalMode::FileList);

        assert_eq!(
            handle_diff_key(
                &mut state,
                &KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
            ),
            DiffModalOutcome::Close
        );
    }
}
