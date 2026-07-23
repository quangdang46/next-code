//! Face-styled full-screen resume browser (bare `next-code --resume`).
//!
//! Left list (~40%) + right transcript preview (~60%), inspired by legacy TUI
//! session_picker layout and Face `MemoryBrowser` chrome. Distinct from the
//! expand-card `SessionPicker` used by `/resume`.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::actions::Action;
use crate::app::app_view::{InputOutcome, SessionPickerEntry};
use crate::input::line_editor::{LineEditOutcome, LineEditor};
use crate::theme::Theme;
use crate::views::modal_window::{
    self, ModalContentArea, ModalSizing, ModalWindowConfig, ModalWindowState, Shortcut,
};

const SPLIT_MIN_WIDTH: u16 = 80;
const LIST_WIDTH_RATIO: f64 = 0.40;
const PREVIEW_MAX_MESSAGES: usize = 20;
/// Visual lines per list entry (title · time; counts; prompt/cwd).
const LIST_ROW_LINES: u16 = 3;
const CWD_DISPLAY_MAX: usize = 36;
const PROMPT_DISPLAY_MAX: usize = 56;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeBrowserFocus {
    List,
    Preview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeBrowserMode {
    Browse,
    FilterFocused,
}

#[derive(Debug, Clone)]
pub struct ResumePreviewLine {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ResumeBrowserState {
    pub window: ModalWindowState,
    pub entries: Option<Vec<SessionPickerEntry>>,
    pub loading: bool,
    pub selected: usize,
    pub scroll_offset: usize,
    pub focus: ResumeBrowserFocus,
    pub mode: ResumeBrowserMode,
    pub preview_lines: Vec<ResumePreviewLine>,
    pub preview_loading: bool,
    pub preview_scroll: usize,
    pub preview_session_id: Option<String>,
    /// Bumped on selection change; stale preview loads are ignored.
    pub preview_seq: u64,
    query: LineEditor,
    filtered_cache: Vec<usize>,
    list_area: Rect,
    preview_area: Rect,
}

impl ResumeBrowserState {
    pub fn new_loading() -> Self {
        Self {
            window: ModalWindowState::new(),
            entries: None,
            loading: true,
            selected: 0,
            scroll_offset: 0,
            focus: ResumeBrowserFocus::List,
            mode: ResumeBrowserMode::Browse,
            preview_lines: Vec::new(),
            preview_loading: false,
            preview_scroll: 0,
            preview_session_id: None,
            preview_seq: 0,
            query: LineEditor::default(),
            filtered_cache: Vec::new(),
            list_area: Rect::default(),
            preview_area: Rect::default(),
        }
    }

    pub fn query(&self) -> &str {
        self.query.text()
    }

    pub fn set_entries(&mut self, entries: Vec<SessionPickerEntry>) {
        self.entries = Some(entries);
        self.loading = false;
        self.invalidate_filter();
        self.clamp_selected();
        self.bump_preview_request();
    }

    pub fn set_list_failed(&mut self, _error: &str) {
        self.loading = false;
        if self.entries.is_none() {
            self.entries = Some(Vec::new());
        }
        self.invalidate_filter();
    }

    pub fn apply_preview(
        &mut self,
        session_id: &str,
        seq: u64,
        lines: Vec<ResumePreviewLine>,
    ) -> bool {
        if seq != self.preview_seq {
            return false;
        }
        if self.preview_session_id.as_deref() != Some(session_id) {
            return false;
        }
        self.preview_lines = lines;
        self.preview_loading = false;
        self.preview_scroll = 0;
        true
    }

    pub fn selected_entry(&self) -> Option<&SessionPickerEntry> {
        let entries = self.entries.as_ref()?;
        self.filtered_cache
            .get(self.selected)
            .and_then(|&i| entries.get(i))
    }

    pub fn request_preview_effect(&self) -> Option<Action> {
        let entry = self.selected_entry()?;
        Some(Action::LoadResumePreview {
            session_id: entry.id.clone(),
            seq: self.preview_seq,
        })
    }

    fn invalidate_filter(&mut self) {
        let q = self.query().to_ascii_lowercase();
        self.filtered_cache = match self.entries.as_ref() {
            Some(entries) if q.is_empty() => (0..entries.len()).collect(),
            Some(entries) => entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.summary.to_ascii_lowercase().contains(&q)
                        || e.id.to_ascii_lowercase().contains(&q)
                        || e.cwd.to_ascii_lowercase().contains(&q)
                        || e.repo_name.to_ascii_lowercase().contains(&q)
                        || e.first_prompt
                            .as_deref()
                            .is_some_and(|fp| fp.to_ascii_lowercase().contains(&q))
                        || e.short_name
                            .as_deref()
                            .is_some_and(|sn| sn.to_ascii_lowercase().contains(&q))
                })
                .map(|(i, _)| i)
                .collect(),
            None => Vec::new(),
        };
    }

    fn clamp_selected(&mut self) {
        if self.filtered_cache.is_empty() {
            self.selected = 0;
            self.preview_lines.clear();
            self.preview_session_id = None;
            self.preview_loading = false;
            return;
        }
        if self.selected >= self.filtered_cache.len() {
            self.selected = self.filtered_cache.len() - 1;
        }
    }

    fn bump_preview_request(&mut self) {
        let Some(entry) = self.selected_entry().map(|e| e.id.clone()) else {
            self.preview_lines.clear();
            self.preview_session_id = None;
            self.preview_loading = false;
            return;
        };
        if self.preview_session_id.as_deref() == Some(entry.as_str()) && !self.preview_lines.is_empty()
        {
            return;
        }
        self.preview_seq += 1;
        self.preview_session_id = Some(entry);
        self.preview_lines.clear();
        self.preview_loading = true;
        self.preview_scroll = 0;
    }

    fn select_next(&mut self) {
        if self.selected + 1 < self.filtered_cache.len() {
            self.selected += 1;
            self.bump_preview_request();
        }
    }

    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.bump_preview_request();
        }
    }

    fn page_list(&mut self, down: bool) {
        let step = 10usize;
        if down {
            let max = self.filtered_cache.len().saturating_sub(1);
            self.selected = (self.selected + step).min(max);
        } else {
            self.selected = self.selected.saturating_sub(step);
        }
        self.bump_preview_request();
    }

    fn scroll_preview(&mut self, delta: isize) {
        if delta < 0 {
            self.preview_scroll = self.preview_scroll.saturating_sub((-delta) as usize);
        } else {
            self.preview_scroll = self.preview_scroll.saturating_add(delta as usize);
        }
    }
}

pub fn render_resume_browser(buf: &mut Buffer, full_area: Rect, state: &mut ResumeBrowserState) {
    let theme = Theme::current();
    let shortcuts = build_shortcuts(&state.mode, state.focus);
    let modal_config = ModalWindowConfig {
        title: "Resume session",
        tabs: None,
        shortcuts: &shortcuts,
        sizing: ModalSizing {
            width_pct: 1.0,
            max_width: u16::MAX,
            min_width: 44,
            v_margin: 0,
            h_pad: 2,
            v_pad: 0,
            footer_lines: 2,
        },
        fold_info: None,
    };

    let Some(ModalContentArea {
        content: content_area,
        ..
    }) =
        modal_window::render_modal_window(buf, full_area, &mut state.window, &modal_config, &theme)
    else {
        return;
    };

    if content_area.height < 2 || content_area.width < 10 {
        return;
    }

    let show_preview = content_area.width >= SPLIT_MIN_WIDTH;
    let list_width = if show_preview {
        (content_area.width as f64 * LIST_WIDTH_RATIO) as u16
    } else {
        content_area.width
    };

    let list_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: list_width,
        height: content_area.height,
    };
    state.list_area = list_area;
    render_session_list(buf, list_area, state, &theme);

    if show_preview {
        let preview_x = content_area.x + list_width + 1;
        let preview_width = content_area.width.saturating_sub(list_width + 1);
        if preview_width > 2 {
            let sep_x = content_area.x + list_width;
            let sep_style = Style::default().fg(theme.gray_dim);
            for y in content_area.y..content_area.y + content_area.height {
                if let Some(cell) = buf.cell_mut((sep_x, y)) {
                    cell.set_symbol("\u{2502}");
                    cell.set_style(sep_style);
                }
            }
            let preview_area = Rect {
                x: preview_x,
                y: content_area.y,
                width: preview_width,
                height: content_area.height,
            };
            state.preview_area = preview_area;
            render_preview(buf, preview_area, state, &theme);
        } else {
            state.preview_area = Rect::default();
        }
    } else {
        state.preview_area = Rect::default();
    }
}

fn render_session_list(
    buf: &mut Buffer,
    area: Rect,
    state: &mut ResumeBrowserState,
    theme: &Theme,
) {
    let search_y = area.y;
    let filter_focused = matches!(state.mode, ResumeBrowserMode::FilterFocused);
    let list_focused = state.focus == ResumeBrowserFocus::List;
    let viewport = state.query.viewport(area.width as usize);
    if state.query().is_empty() {
        let placeholder = if filter_focused {
            "type to filter..."
        } else {
            "/ to filter..."
        };
        buf.set_span(
            area.x,
            search_y,
            &Span::styled(
                placeholder,
                Style::default().fg(theme.gray_dim).bg(theme.bg_base),
            ),
            area.width,
        );
    } else {
        let leading;
        let visible = if filter_focused {
            &state.query()[viewport.visible_byte_range.clone()]
        } else {
            leading = crate::render::line_utils::truncate_str(state.query(), area.width as usize);
            &leading
        };
        buf.set_span(
            area.x,
            search_y,
            &Span::styled(
                visible,
                Style::default().fg(theme.text_primary).bg(theme.bg_base),
            ),
            area.width,
        );
    }
    if filter_focused {
        let cursor_x = area.x + viewport.cursor_display_column as u16;
        if cursor_x < area.x + area.width
            && let Some(cell) = buf.cell_mut((cursor_x, search_y))
        {
            cell.set_style(Style::default().fg(theme.bg_base).bg(theme.text_primary));
        }
    }

    let entries_start_y = search_y + 1;
    let available_height = area.height.saturating_sub(1) as usize;
    let visible_slots = (available_height / LIST_ROW_LINES as usize).max(1);
    if state.selected < state.scroll_offset {
        state.scroll_offset = state.selected;
    }
    if state.selected >= state.scroll_offset + visible_slots {
        state.scroll_offset = state.selected.saturating_sub(visible_slots.saturating_sub(1));
    }

    if state.loading && state.entries.is_none() {
        buf.set_span(
            area.x,
            entries_start_y,
            &Span::styled(
                "Loading sessions…",
                Style::default().fg(theme.gray).bg(theme.bg_base),
            ),
            area.width,
        );
        return;
    }

    let filtered = state.filtered_cache.clone();
    if filtered.is_empty() {
        let msg = if state.query().is_empty() {
            "No sessions found"
        } else {
            "No matches"
        };
        buf.set_span(
            area.x,
            entries_start_y,
            &Span::styled(msg, Style::default().fg(theme.gray_dim).bg(theme.bg_base)),
            area.width,
        );
        return;
    }

    let end = filtered.len().min(state.scroll_offset + visible_slots);
    let entries = state.entries.as_ref().map(|e| e.as_slice()).unwrap_or(&[]);
    for (row, &orig_idx) in filtered[state.scroll_offset..end].iter().enumerate() {
        let y0 = entries_start_y + (row as u16) * LIST_ROW_LINES;
        if y0 >= area.y + area.height {
            break;
        }
        let Some(entry) = entries.get(orig_idx) else {
            continue;
        };
        let filt_idx = state.scroll_offset + row;
        let is_selected = filt_idx == state.selected && list_focused;
        let bg = if is_selected {
            theme.bg_visual
        } else {
            theme.bg_base
        };
        let row_h = LIST_ROW_LINES.min(area.y + area.height - y0);
        let row_rect = Rect {
            x: area.x,
            y: y0,
            width: area.width,
            height: row_h,
        };
        buf.set_style(row_rect, Style::default().bg(bg));

        // Leading status + selection bar (Face glyphs, not animal emoji titles).
        let has_content = entry.num_messages > 0
            || entry.user_message_count > 0
            || entry.assistant_message_count > 0
            || entry.first_prompt.as_ref().is_some_and(|s| !s.trim().is_empty());
        let status_glyph = if has_content {
            crate::glyphs::filled_dot()
        } else {
            "\u{25cb}" // ○
        };
        let status_color = if has_content {
            theme.accent_success
        } else {
            theme.gray_dim
        };
        let lead_x = area.x;
        if is_selected {
            buf.set_span(
                lead_x,
                y0,
                &Span::styled(
                    crate::glyphs::selection_bar(),
                    Style::default().fg(theme.accent_user).bg(bg),
                ),
                1,
            );
        }
        buf.set_span(
            lead_x + 1,
            y0,
            &Span::styled(
                status_glyph,
                Style::default().fg(status_color).bg(bg),
            ),
            1,
        );

        let title = primary_title(entry);
        let ago = relative_time(entry.updated_at);
        let text_x = lead_x + 3;
        let max_w = area.width.saturating_sub(3) as usize;
        let title_budget = max_w.saturating_sub(ago.width() + 3);
        let title_disp = truncate_ellipsis(&title, title_budget);
        let line1 = format!("{title_disp} · {ago}");
        buf.set_span(
            text_x,
            y0,
            &Span::styled(
                truncate_ellipsis(&line1, max_w),
                Style::default()
                    .fg(theme.text_primary)
                    .bg(bg)
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            area.width.saturating_sub(3),
        );

        if row_h >= 2 {
            let counts = format_entry_counts(entry);
            let counts_style = if counts.is_empty() {
                Style::default().fg(theme.gray_dim).bg(bg)
            } else {
                Style::default().fg(theme.accent_user).bg(bg)
            };
            let meta = if entry.repo_name.is_empty() {
                counts
            } else if counts.is_empty() {
                entry.repo_name.clone()
            } else {
                format!("{} · {}", entry.repo_name, counts)
            };
            buf.set_span(
                text_x,
                y0 + 1,
                &Span::styled(truncate_ellipsis(&meta, max_w), counts_style),
                area.width.saturating_sub(3),
            );
        }

        if row_h >= 3 {
            let line3 = format_entry_secondary(entry);
            buf.set_span(
                text_x,
                y0 + 2,
                &Span::styled(
                    truncate_ellipsis(&line3, max_w),
                    Style::default().fg(theme.gray_dim).bg(bg),
                ),
                area.width.saturating_sub(3),
            );
        }
    }
}

fn format_entry_counts(entry: &SessionPickerEntry) -> String {
    if entry.user_message_count > 0 || entry.assistant_message_count > 0 {
        format!(
            "{} user · {} asst",
            entry.user_message_count, entry.assistant_message_count
        )
    } else if entry.num_messages > 0 {
        format!("{} msgs", entry.num_messages)
    } else {
        String::new()
    }
}

/// Scannable list/header title: chat brief first, animal short_name last.
///
/// Order: `summary` when it is a real title/brief → `first_prompt` →
/// `short_name` → truncated `id`. (List API already prefers customTitle /
/// generated title / firstPrompt into `summary`.)
fn primary_title(entry: &SessionPickerEntry) -> String {
    let summary = entry.summary.trim();
    let short = entry.short_name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let prompt = entry
        .first_prompt
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let summary_is_short = short.is_some_and(|sn| summary == sn)
        || (short.is_none()
            && !summary.is_empty()
            && !summary.contains(' ')
            && summary.chars().count() <= 24);

    if !summary.is_empty() && !summary_is_short {
        return truncate_ellipsis(summary, PROMPT_DISPLAY_MAX);
    }
    if let Some(fp) = prompt {
        return truncate_ellipsis(fp, PROMPT_DISPLAY_MAX);
    }
    if !summary.is_empty() {
        return summary.to_string();
    }
    if let Some(sn) = short {
        return sn.to_string();
    }
    entry.id.chars().take(12).collect()
}

fn format_entry_secondary(entry: &SessionPickerEntry) -> String {
    let mut parts: Vec<String> = Vec::new();
    let title = primary_title(entry);
    // Tiny dim badge for memorable name when title is the chat brief.
    if let Some(sn) = entry
        .short_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if sn != title.as_str() {
            parts.push(format!("@{sn}"));
        }
    }
    if !entry.cwd.is_empty() {
        parts.push(format!(
            "{} {}",
            crate::glyphs::diamond_hollow(),
            truncate_cwd(&entry.cwd, CWD_DISPLAY_MAX)
        ));
    }
    if parts.is_empty() {
        entry.id.chars().take(12).collect()
    } else {
        parts.join(" · ")
    }
}

fn truncate_cwd(cwd: &str, max: usize) -> String {
    if cwd.chars().count() <= max {
        return cwd.to_string();
    }
    let chars: Vec<char> = cwd.chars().collect();
    let take = max.saturating_sub(3);
    let suffix: String = chars
        .iter()
        .rev()
        .take(take)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("...{suffix}")
}

fn truncate_ellipsis(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if s.width() <= max_width {
        return s.to_string();
    }
    let trunc = truncate_to_width(s, max_width.saturating_sub(3));
    format!("{trunc}...")
}

fn relative_time(ts: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let delta = now.signed_duration_since(ts);
    if delta.num_days() >= 1 {
        format!("{}d", delta.num_days())
    } else if delta.num_hours() >= 1 {
        format!("{}h", delta.num_hours())
    } else if delta.num_minutes() >= 1 {
        format!("{}m", delta.num_minutes())
    } else {
        "now".into()
    }
}

fn render_preview(buf: &mut Buffer, area: Rect, state: &mut ResumeBrowserState, theme: &Theme) {
    buf.set_style(area, Style::default().bg(theme.bg_base));
    let preview_focused = state.focus == ResumeBrowserFocus::Preview;
    let selected = state.selected_entry();

    let mut header_lines: Vec<(String, Style)> = Vec::new();
    if let Some(entry) = selected {
        let title = primary_title(entry);
        let focus_mark = if preview_focused { " ▸" } else { "" };
        header_lines.push((
            format!("{title}{focus_mark}"),
            Style::default()
                .fg(theme.accent_user)
                .bg(theme.bg_base)
                .add_modifier(Modifier::BOLD),
        ));
        if let Some(sn) = entry
            .short_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != title.as_str())
        {
            header_lines.push((
                format!("@{sn}"),
                Style::default().fg(theme.gray_dim).bg(theme.bg_base),
            ));
        }
        if !entry.cwd.is_empty() {
            header_lines.push((
                format!(
                    "{} {}",
                    crate::glyphs::diamond_hollow(),
                    truncate_cwd(&entry.cwd, area.width.saturating_sub(4) as usize)
                ),
                Style::default().fg(theme.gray).bg(theme.bg_base),
            ));
        }
        if let Some(model) = entry.model_id.as_deref().filter(|s| !s.is_empty()) {
            header_lines.push((
                model.to_string(),
                Style::default().fg(theme.gray_dim).bg(theme.bg_base),
            ));
        }
    } else {
        header_lines.push((
            if preview_focused {
                "Preview (focused)".into()
            } else {
                "Preview".into()
            },
            Style::default()
                .fg(theme.accent_user)
                .bg(theme.bg_base)
                .add_modifier(Modifier::BOLD),
        ));
    }

    for (i, (text, style)) in header_lines.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            return;
        }
        buf.set_span(
            area.x,
            y,
            &Span::styled(truncate_ellipsis(text, area.width as usize), *style),
            area.width,
        );
    }

    let header_h = header_lines.len() as u16;
    let body_y = area.y + header_h;
    let body_h = area.height.saturating_sub(header_h);
    if body_h == 0 {
        return;
    }

    if state.preview_loading {
        buf.set_span(
            area.x,
            body_y,
            &Span::styled(
                "Loading transcript…",
                Style::default().fg(theme.gray).bg(theme.bg_base),
            ),
            area.width,
        );
        return;
    }

    if state.preview_lines.is_empty() {
        let (icon_line, why) = match selected {
            Some(entry)
                if entry.num_messages == 0
                    && entry.user_message_count == 0
                    && entry.assistant_message_count == 0 =>
            {
                (
                    format!("{} No transcript preview", crate::glyphs::diamond_hollow()),
                    "Empty session — no visible user/assistant turns yet",
                )
            }
            Some(_) => (
                format!("{} No transcript preview", crate::glyphs::diamond_hollow()),
                "Journal/snapshot had no previewable turns (system-only or unloadable)",
            ),
            None => (
                format!("{} Select a session", crate::glyphs::diamond_hollow()),
                "Pick a row on the left to load the transcript",
            ),
        };
        buf.set_span(
            area.x,
            body_y,
            &Span::styled(
                truncate_ellipsis(&icon_line, area.width as usize),
                Style::default().fg(theme.gray).bg(theme.bg_base),
            ),
            area.width,
        );
        if body_h >= 2 {
            buf.set_span(
                area.x,
                body_y + 1,
                &Span::styled(
                    truncate_ellipsis(why, area.width as usize),
                    Style::default().fg(theme.gray_dim).bg(theme.bg_base),
                ),
                area.width,
            );
        }
        return;
    }

    let mut display_lines: Vec<Line<'static>> = Vec::new();
    for line in &state.preview_lines {
        let role_style = match line.role.as_str() {
            "user" => Style::default().fg(theme.accent_user).bg(theme.bg_base),
            "assistant" => Style::default().fg(theme.accent_assistant).bg(theme.bg_base),
            _ => Style::default().fg(theme.gray).bg(theme.bg_base),
        };
        let role_label = format!("{}: ", line.role);
        let wrap_width = area.width.saturating_sub(1) as usize;
        let body = wrap_text(&line.text, wrap_width.saturating_sub(role_label.width().min(12)));
        for (i, chunk) in body.into_iter().enumerate() {
            if i == 0 {
                display_lines.push(Line::from(vec![
                    Span::styled(role_label.clone(), role_style),
                    Span::styled(chunk, Style::default().fg(theme.text_primary).bg(theme.bg_base)),
                ]));
            } else {
                display_lines.push(Line::from(Span::styled(
                    format!("  {chunk}"),
                    Style::default().fg(theme.text_primary).bg(theme.bg_base),
                )));
            }
        }
        display_lines.push(Line::from(""));
    }

    let total = display_lines.len();
    let visible = body_h as usize;
    state.preview_scroll = state
        .preview_scroll
        .min(total.saturating_sub(visible));
    let scroll = state.preview_scroll;
    for (row, line_idx) in (scroll..total.min(scroll + visible)).enumerate() {
        let y = body_y + row as u16;
        buf.set_line(area.x, y, &display_lines[line_idx], area.width);
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut rest = paragraph;
        while !rest.is_empty() {
            if rest.width() <= width {
                out.push(rest.to_string());
                break;
            }
            let mut end = rest.len();
            let mut best = 0;
            for (i, _) in rest.char_indices() {
                if rest[..i].width() > width {
                    break;
                }
                best = i;
                end = i;
            }
            if best == 0 {
                // Single wide grapheme — take one char.
                end = rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            }
            // Prefer breaking on whitespace when possible.
            if let Some(space) = rest[..end].rfind(char::is_whitespace) {
                if space > 0 {
                    end = space;
                }
            }
            out.push(rest[..end].trim_end().to_string());
            rest = rest[end..].trim_start();
        }
    }
    out
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        return s.to_string();
    }
    let mut out = String::new();
    for ch in s.chars() {
        if out.width() + ch.width().unwrap_or(0) > max_width {
            break;
        }
        out.push(ch);
    }
    out
}

fn build_shortcuts(mode: &ResumeBrowserMode, focus: ResumeBrowserFocus) -> Vec<Shortcut<'static>> {
    match mode {
        ResumeBrowserMode::FilterFocused => vec![
            Shortcut {
                label: "type to filter",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Esc exit filter",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Enter resume",
                clickable: false,
                id: 0,
            },
        ],
        ResumeBrowserMode::Browse => {
            let focus_hint = match focus {
                ResumeBrowserFocus::List => "Tab preview",
                ResumeBrowserFocus::Preview => "Tab list",
            };
            vec![
                Shortcut {
                    label: "j/k move",
                    clickable: false,
                    id: 0,
                },
                Shortcut {
                    label: focus_hint,
                    clickable: false,
                    id: 0,
                },
                Shortcut {
                    label: "/ filter",
                    clickable: false,
                    id: 0,
                },
                Shortcut {
                    label: "Enter resume",
                    clickable: false,
                    id: 0,
                },
                Shortcut {
                    label: "Esc back",
                    clickable: false,
                    id: 0,
                },
            ]
        }
    }
}

pub fn handle_resume_browser_key(
    state: &mut ResumeBrowserState,
    key: &KeyEvent,
) -> InputOutcome {
    if key.kind == KeyEventKind::Release {
        return InputOutcome::Unchanged;
    }

    // Modal chrome close (Esc when not filtering) handled by caller via window,
    // but we also treat Esc here for filter exit / dismiss.
    match state.mode {
        ResumeBrowserMode::FilterFocused => handle_filter_focused(state, key),
        ResumeBrowserMode::Browse => handle_browse(state, key),
    }
}

fn handle_filter_focused(state: &mut ResumeBrowserState, key: &KeyEvent) -> InputOutcome {
    match key.code {
        KeyCode::Esc => {
            state.mode = ResumeBrowserMode::Browse;
            InputOutcome::Changed
        }
        KeyCode::Enter => pick_selected(state),
        KeyCode::Down => {
            state.select_next();
            preview_or_changed(state)
        }
        KeyCode::Up => {
            state.select_prev();
            preview_or_changed(state)
        }
        _ => {
            let outcome = state.query.handle_key(key);
            finish_filter_edit(state, outcome)
        }
    }
}

fn finish_filter_edit(state: &mut ResumeBrowserState, outcome: LineEditOutcome) -> InputOutcome {
    match outcome {
        LineEditOutcome::TextChanged => {
            state.invalidate_filter();
            state.clamp_selected();
            state.bump_preview_request();
            preview_or_changed(state)
        }
        LineEditOutcome::CursorChanged | LineEditOutcome::HandledNoChange => InputOutcome::Changed,
        LineEditOutcome::Unhandled => InputOutcome::Unchanged,
    }
}

fn handle_browse(state: &mut ResumeBrowserState, key: &KeyEvent) -> InputOutcome {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return InputOutcome::Unchanged;
    }
    match key.code {
        KeyCode::Esc => InputOutcome::Action(Action::CloseResumeBrowser),
        KeyCode::Char('/') => {
            state.mode = ResumeBrowserMode::FilterFocused;
            state.focus = ResumeBrowserFocus::List;
            InputOutcome::Changed
        }
        KeyCode::Tab => {
            state.focus = match state.focus {
                ResumeBrowserFocus::List => ResumeBrowserFocus::Preview,
                ResumeBrowserFocus::Preview => ResumeBrowserFocus::List,
            };
            InputOutcome::Changed
        }
        KeyCode::Char('h') | KeyCode::Left => {
            state.focus = ResumeBrowserFocus::List;
            InputOutcome::Changed
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.focus = ResumeBrowserFocus::Preview;
            InputOutcome::Changed
        }
        KeyCode::Enter => pick_selected(state),
        KeyCode::Down | KeyCode::Char('j') => match state.focus {
            ResumeBrowserFocus::List => {
                state.select_next();
                preview_or_changed(state)
            }
            ResumeBrowserFocus::Preview => {
                state.scroll_preview(3);
                InputOutcome::Changed
            }
        },
        KeyCode::Up | KeyCode::Char('k') => match state.focus {
            ResumeBrowserFocus::List => {
                state.select_prev();
                preview_or_changed(state)
            }
            ResumeBrowserFocus::Preview => {
                state.scroll_preview(-3);
                InputOutcome::Changed
            }
        },
        KeyCode::PageDown | KeyCode::Char('J') => match state.focus {
            ResumeBrowserFocus::List => {
                state.page_list(true);
                preview_or_changed(state)
            }
            ResumeBrowserFocus::Preview => {
                state.scroll_preview(10);
                InputOutcome::Changed
            }
        },
        KeyCode::PageUp | KeyCode::Char('K') => match state.focus {
            ResumeBrowserFocus::List => {
                state.page_list(false);
                preview_or_changed(state)
            }
            ResumeBrowserFocus::Preview => {
                state.scroll_preview(-10);
                InputOutcome::Changed
            }
        },
        _ => InputOutcome::Unchanged,
    }
}

fn pick_selected(state: &mut ResumeBrowserState) -> InputOutcome {
    let Some(entry) = state.selected_entry() else {
        return InputOutcome::Changed;
    };
    InputOutcome::Action(Action::PickResumeBrowserSession {
        session_id: entry.id.clone(),
        cwd: entry.cwd.clone(),
        source: entry.source.clone(),
    })
}

fn preview_or_changed(state: &mut ResumeBrowserState) -> InputOutcome {
    if let Some(action) = state.request_preview_effect() {
        InputOutcome::Action(action)
    } else {
        InputOutcome::Changed
    }
}

pub const fn preview_max_messages() -> usize {
    PREVIEW_MAX_MESSAGES
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, summary: &str) -> SessionPickerEntry {
        SessionPickerEntry {
            id: id.into(),
            summary: summary.into(),
            updated_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            cwd: "/repo".into(),
            hostname: None,
            source: "local".into(),
            model_id: None,
            num_messages: 1,
            user_message_count: 0,
            assistant_message_count: 0,
            first_prompt: None,
            short_name: None,
            last_active_at: None,
            branch: None,
            repo_name: "repo".into(),
            worktree_label: None,
            card_detail: None,
        }
    }

    #[test]
    fn filter_narrows_list_and_bumps_preview_seq() {
        let mut state = ResumeBrowserState::new_loading();
        state.set_entries(vec![entry("a", "alpha"), entry("b", "beta")]);
        let seq_before = state.preview_seq;
        state.query.set_text("bet");
        state.invalidate_filter();
        state.clamp_selected();
        state.bump_preview_request();
        assert_eq!(state.filtered_cache.len(), 1);
        assert_eq!(state.selected_entry().map(|e| e.id.as_str()), Some("b"));
        assert!(state.preview_seq >= seq_before);
    }

    #[test]
    fn primary_title_prefers_first_prompt_over_animal_short_name() {
        let mut e = entry("sess-rooster", "rooster");
        e.short_name = Some("rooster".into());
        e.first_prompt = Some("Fix the resume list density for Face".into());
        assert_eq!(
            primary_title(&e),
            "Fix the resume list density for Face"
        );
    }

    #[test]
    fn primary_title_keeps_real_summary() {
        let mut e = entry("sess-rooster", "Resume list enrichment");
        e.short_name = Some("rooster".into());
        e.first_prompt = Some("Fix the resume list density".into());
        assert_eq!(primary_title(&e), "Resume list enrichment");
    }

    #[test]
    fn secondary_line_badges_short_name_when_title_is_brief() {
        let mut e = entry("sess-blazing", "blazing");
        e.short_name = Some("blazing".into());
        e.first_prompt = Some("Fix the resume list density".into());
        e.cwd = "/Users/me/Projects/next-code".into();
        let secondary = format_entry_secondary(&e);
        assert!(secondary.contains("@blazing"), "{secondary}");
        assert!(secondary.contains("next-code"), "{secondary}");
        assert!(!secondary.contains("prompt:"), "{secondary}");
    }

    #[test]
    fn secondary_line_skips_short_badge_when_title_is_short_name() {
        let mut e = entry("sess-blazing", "blazing");
        e.short_name = Some("blazing".into());
        e.cwd = "/repo".into();
        let secondary = format_entry_secondary(&e);
        assert!(!secondary.contains("@blazing"), "{secondary}");
        assert!(secondary.contains("/repo"), "{secondary}");
    }

    #[test]
    fn enter_emits_pick_action() {
        let mut state = ResumeBrowserState::new_loading();
        state.set_entries(vec![entry("s1", "one")]);
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let out = handle_resume_browser_key(&mut state, &key);
        assert!(matches!(
            out,
            InputOutcome::Action(Action::PickResumeBrowserSession { session_id, .. })
                if session_id == "s1"
        ));
    }
}
