# MASTER_UI.md — jcode TUI Redesign Specification
> Full UI/UX spec with ASCII mockups + code for every feature
> Based on research: Claude Code + OpenCode + Codex
> Stack: Rust + ratatui + crossterm

---

## Table of Contents

1. [Overall Layout](#1-overall-layout)
2. [Status Bar](#2-status-bar)
3. [Chat Viewport](#3-chat-viewport)
4. [User Message](#4-user-message)
5. [Assistant Message (Text)](#5-assistant-message-text)
6. [Thinking/Reasoning Block](#6-thinkingreasoning-block)
7. [Tool Call — Bash](#7-tool-call--bash)
8. [Tool Call — Edit](#8-tool-call--edit)
9. [Tool Call — Read](#9-tool-call--read)
10. [Tool Call — Glob/Grep](#10-tool-call--globgrep)
11. [Tool Call — Agent (Sub-agent)](#11-tool-call--agent-sub-agent)
12. [Permission Dialog — Bash](#12-permission-dialog--bash)
13. [Permission Dialog — Edit](#13-permission-dialog--edit)
14. [Permission Dialog — Read](#14-permission-dialog--read)
15. [Chat Composer (Input)](#15-chat-composer-input)
16. [Spinner States](#16-spinner-states)
17. [Unseen Divider](#17-unseen-divider)
18. [Transcript Overlay (Full-screen)](#18-transcript-overlay-full-screen)
19. [Keybinding Which-Key Panel](#19-keybinding-which-key-panel)
20. [Session Picker](#20-session-picker)
21. [Footer / Hints Bar](#21-footer--hints-bar)
22. [System Messages](#22-system-messages)
23. [Mermaid Diagram Pane](#23-mermaid-diagram-pane)
24. [Swarm Gallery (Multi-Agent)](#24-swarm-gallery-multi-agent)
25. [Theme Switching](#25-theme-switching)
26. [Error State](#26-error-state)
27. [Splash / Empty State](#27-splash--empty-state)
28. [Onboarding Flow](#28-onboarding-flow)

---

## 1. Overall Layout

The main TUI layout follows a vertical column structure.

```
┌─────────────────────────────────────────────────────────────────────┐
│ STATUS BAR (1 row)                                                 │
│ claude-sonnet-4-20250514  ctx:42%  $0.12  cache:78%  ▌auto        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ CHAT VIEWPORT (scrollable, auto-pinned to bottom)                  │
│                                                                     │
│  ┌─ User Message ─────────────────────────────────────────────────┐ │
│  │ > Fix the bug in auth.rs                                      │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─ Assistant ────────────────────────────────────────────────────┐ │
│  │ I'll look at the auth module to find the bug.                 │ │
│  │                                                               │ │
│  │ ┌─ Bash ────────────────────────────────────────────────────┐ │ │
│  │ │ $ grep -n "validate" src/auth.rs                          │ │ │
│  │ │ ✓ exit: 0                                                 │ │ │
│  │ │   12: fn validate_token(token: &str) -> bool {            │ │ │
│  │ │   45:   if !validate_expiry(expiry) {                     │ │ │
│  │ └───────────────────────────────────────────────────────────┘ │ │
│  │                                                               │ │
│  │ ┌─ Edit ────────────────────────────────────────────────────┐ │ │
│  │ │ → Update src/auth.rs                                      │ │ │
│  │ │   -   if !validate_expiry(expiry) {                       │ │ │
│  │ │   +   if !validate_expiry(expiry, now) {                  │ │ │
│  │ └───────────────────────────────────────────────────────────┘ │ │
│  │                                                               │ │
│  │ Fixed the bug — `validate_expiry` was missing the current     │ │
│  │ time parameter.                                               │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ ⠋ Thinking...                                       (spinner row)  │
├─────────────────────────────────────────────────────────────────────┤
│ ▌                                                                  │
│ (input area — grows with content)                                  │
│                                                                    │
├─────────────────────────────────────────────────────────────────────┤
│ Tab:autocomplete  Ctrl+X:leader  Ctrl+O:transcript  /:commands    │
└─────────────────────────────────────────────────────────────────────┘
```

### Layout Types (Rust)

```rust
// crates/jcode-tui/src/layout.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout computation.
pub fn compute_main_layout(area: Rect) -> MainLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Status bar
            Constraint::Min(1),     // Chat viewport (flexible)
            Constraint::Length(1),   // Spinner (hidden when idle)
            Constraint::Length(3),   // Composer / permission dialog
            Constraint::Length(1),   // Footer hints
        ])
        .split(area);

    MainLayout {
        status_bar: chunks[0],
        viewport: chunks[1],
        spinner: chunks[2],
        composer: chunks[3],
        footer: chunks[4],
    }
}

pub struct MainLayout {
    pub status_bar: Rect,
    pub viewport: Rect,
    pub spinner: Rect,
    pub composer: Rect,
    pub footer: Rect,
}

/// With side panel (optional, toggled):
///
/// ┌────────────────────────────┬──────────────┐
/// │ Main Column                │ Side Panel   │
/// │ (status+viewport+composer) │ (pinned/     │
/// │                            │  mermaid/    │
/// │                            │  workspace)  │
/// └────────────────────────────┴──────────────┘
pub fn compute_layout_with_panel(area: Rect, panel_width: u16) -> (MainLayout, Rect) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),            // Main column
            Constraint::Length(panel_width), // Side panel
        ])
        .split(area);

    let main = compute_main_layout(horizontal[0]);
    let panel = horizontal[1];
    (main, panel)
}
```

---

## 2. Status Bar

```
┌─────────────────────────────────────────────────────────────────────┐
│ claude-sonnet-4-20250514  ctx:42%  $0.12  cache:78%  ▌auto        │
└─────────────────────────────────────────────────────────────────────┘

  ↑ model name          ↑ context %   ↑ cost   ↑ cache  ↑ mode
  (dimmed)              (green if     (white)  (cyan)   (purple)
                         <80%, red
                         if >90%)
```

### Responsive variants

```
Narrow terminal (<60 cols):

│ sonnet-4  42%  $0.12  auto │

Very narrow (<40 cols):

│ 42% │
```

### Code

```rust
// crates/jcode-tui/src/status_bar.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct StatusBarState {
    pub model_name: String,
    pub context_percent: f64,      // 0.0 - 1.0
    pub cost_dollars: f64,
    pub cache_hit_percent: f64,    // 0.0 - 1.0
    pub permission_mode: PermissionMode,
    pub fast_mode: bool,
}

#[derive(Debug, Clone)]
pub enum PermissionMode {
    Auto,
    Default,
    Plan,
    BypassPermissions,
}

pub fn render_status_bar(state: &StatusBarState, theme: &Theme, area: Rect, buf: &mut Buffer) {
    let width = area.width as usize;
    let mut spans = Vec::new();

    // Model name (always shown)
    let model_display = truncate(&state.model_name, 20);
    spans.push(Span::styled(
        model_display,
        Style::default().fg(theme.text_muted.into()),
    ));

    if width > 40 {
        // Context percentage with color coding
        let ctx_color = if state.context_percent > 0.9 {
            theme.error
        } else if state.context_percent > 0.7 {
            theme.warning
        } else {
            theme.success
        };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("ctx:{:.0}%", state.context_percent * 100.0),
            Style::default().fg(ctx_color.into()),
        ));
    }

    if width > 55 {
        // Cost
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("${:.2}", state.cost_dollars),
            Style::default().fg(theme.text.into()),
        ));
    }

    if width > 65 {
        // Cache hit rate
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("cache:{:.0}%", state.cache_hit_percent * 100.0),
            Style::default().fg(theme.info.into()),
        ));
    }

    if width > 80 {
        // Permission mode
        let (mode_icon, mode_color) = match state.permission_mode {
            PermissionMode::Auto => ("▌auto", theme.accent),
            PermissionMode::Default => ("▌default", theme.text_muted),
            PermissionMode::Plan => ("▌plan", theme.info),
            PermissionMode::BypassPermissions => ("▌bypass", theme.warning),
        };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(mode_icon, Style::default().fg(mode_color.into())));
    }

    if state.fast_mode && width > 90 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("⚡fast", Style::default().fg(theme.warning.into())));
    }

    let line = Line::from(spans);
    buf.set_line(area.x, area.y, &line, area.width);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
```

---

## 3. Chat Viewport

The viewport is a scrollable area showing the conversation transcript.

### Scroll behaviors

```
User scrolls up:
┌─────────────────────────────────────────────────────────────┐
│ > What is the error?                                        │
│ The error occurs in main.rs:42 because...                   │
│─────────── 3 new messages ──────────────────────────────── ←│
│ > Fix it please                                             │
│ I've fixed the issue by...                                  │
│ (current content, pinned to bottom when not scrolling)      │
└─────────────────────────────────────────────────────────────┘
                ↑ "3 new messages" divider appears here

User scrolls down to bottom:
→ Divider disappears, auto-pins to bottom

Mouse wheel:
→ 3-line incremental scroll (smooth)
→ Velocity-based acceleration (configurable)
```

### Code

```rust
// crates/jcode-tui/src/viewport.rs

pub struct Viewport {
    /// Committed history cells (finalized).
    cells: Vec<Arc<dyn HistoryCell>>,
    /// Active cell (in-progress, mutating).
    active_cell: Option<Box<dyn HistoryCell>>,
    /// Scroll offset from bottom (0 = pinned to bottom).
    scroll_offset: u16,
    /// Whether we're pinned to bottom.
    pinned: bool,
    /// Unseen message divider state.
    unseen_divider: Option<UnseenDivider>,
    /// Viewport height in rows.
    height: u16,
}

impl Viewport {
    /// Scroll up by N lines.
    pub fn scroll_up(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
        self.pinned = false;
        self.update_unseen_divider();
    }

    /// Scroll down by N lines.
    pub fn scroll_down(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        if self.scroll_offset == 0 {
            self.pinned = true;
            self.unseen_divider = None;
        }
    }

    /// Jump to bottom (pin).
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.pinned = true;
        self.unseen_divider = None;
    }

    /// Page up.
    pub fn page_up(&mut self) {
        self.scroll_up(self.height.saturating_sub(2));
    }

    /// Page down.
    pub fn page_down(&mut self) {
        self.scroll_down(self.height.saturating_sub(2));
    }

    /// Jump to previous user prompt.
    pub fn jump_to_previous_prompt(&mut self) {
        // Find previous UserMessage cell and scroll to it
        let target = self.cells.iter().rposition(|c| c.is_user_message());
        if let Some(idx) = target {
            self.scroll_offset = self.compute_offset_for_cell(idx);
            self.pinned = false;
        }
    }

    /// Jump to next user prompt.
    pub fn jump_to_next_prompt(&mut self) {
        let current = self.cell_at_offset(self.scroll_offset);
        let target = self.cells.iter()
            .skip(current + 1)
            .position(|c| c.is_user_message());
        if let Some(idx) = target {
            self.scroll_offset = self.compute_offset_for_cell(current + 1 + idx);
        }
    }

    /// Update unseen divider when new messages arrive while scrolled up.
    fn update_unseen_divider(&mut self) {
        if !self.pinned && self.active_cell.is_some() {
            self.unseen_divider = Some(UnseenDivider {
                message_count: self.unseen_count(),
                scroll_height: self.total_height(),
            });
        }
    }
}
```

---

## 4. User Message

```
┌─ User ──────────────────────────────────────────────────────────────┐
│ > Fix the bug in auth.rs                                           │
└─────────────────────────────────────────────────────────────────────┘

  ↑ left border is colored per-agent (7 colors for sub-agents)
  ↑ "User" label is dimmed
  ↑ text is wrapped to terminal width
  ↑ if message has images, they render inline below the text
```

### With image attachment

```
┌─ User ──────────────────────────────────────────────────────────────┐
│ > What's wrong with this code?                                     │
│                                                                    │
│ ┌──────────────────────────────────┐                               │
│ │  [screenshot.png]                │                               │
│ │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                               │
│ │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                               │
│ │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                               │
│ └──────────────────────────────────┘                               │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/history_cell/user_message.rs

use crate::history_cell::HistoryCell;

pub struct UserMessageCell {
    pub text: String,
    pub images: Vec<ImageAttachment>,
    pub agent_color: Option<ratatui::style::Color>, // for sub-agents
    pub timestamp: Option<chrono::NaiveDateTime>,
    pub queued: bool, // "QUEUED" badge
}

pub struct ImageAttachment {
    pub path: String,
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl HistoryCell for UserMessageCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;

        // Left border (agent color)
        let border_color = self.agent_color.unwrap_or(theme.user_message_border);
        for row in 0..area.height {
            buf[(area.x, area.y + row)]
                .set_symbol("│")
                .set_fg(border_color);
        }

        let content_area = Rect {
            x: area.x + 2,
            y: area.y,
            width: area.width.saturating_sub(3),
            height: area.height,
        };

        // Header: "User" label + optional timestamp
        let header = Line::from(vec![
            Span::styled("User ", Style::default().fg(theme.text_muted.into())),
            if let Some(ts) = &self.timestamp {
                Span::styled(
                    ts.format("%H:%M").to_string(),
                    Style::default().fg(theme.text_subtle.into()),
                )
            } else {
                Span::raw("")
            },
            if self.queued {
                Span::styled(" QUEUED", Style::default().fg(theme.warning.into())
                    .add_modifier(Modifier::BOLD))
            } else {
                Span::raw("")
            },
        ]);
        buf.set_line(content_area.x, y, &header, content_area.width);
        y += 1;

        // Message text (wrapped)
        for line in wrap_text(&self.text, content_area.width as usize) {
            if y >= area.y + area.height { break; }
            let display = Line::from(Span::styled(
                format!("> {}", line),
                Style::default().fg(theme.text.into()),
            ));
            buf.set_line(content_area.x, y, &display, content_area.width);
            y += 1;
        }

        // Images
        for img in &self.images {
            if y + 5 > area.y + area.height { break; }
            render_inline_image(img, content_area, buf);
            y += 5;
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        let text_height = wrap_text(&self.text, width.saturating_sub(3) as usize).len() as u16;
        let img_height = if self.images.is_empty() { 0 } else { 5 };
        1 + text_height + img_height // header + text + images
    }

    fn is_user_message(&self) -> bool { true }
}
```

---

## 5. Assistant Message (Text)

```
┌─ Assistant ─────────────────────────────────────────────────────────┐
│ I'll analyze the auth module. Here's what I found:                 │
│                                                                    │
│ The bug is on line 42 — `validate_expiry` is called without the   │
│ current timestamp, so it always uses `None` as the default.        │
│                                                                    │
│ I'll fix this by adding a `now` parameter:                         │
│                                                                    │
│ ```rust                                                             │
│ fn validate_expiry(expiry: i64, now: i64) -> bool {               │
│     expiry > now                                                    │
│ }                                                                   │
│ ```                                                                 │
└─────────────────────────────────────────────────────────────────────┘

  ↑ assistant label is green (theme.ai_message)
  ↑ text is syntax-highlighted markdown
  ↑ code blocks have syntax highlighting
  ↑ tables render as formatted text
```

### Code

```rust
// crates/jcode-tui/src/history_cell/assistant_message.rs

pub struct AssistantMessageCell {
    pub parts: Vec<MessagePart>,
    pub agent_color: Option<Color>,
}

pub enum MessagePart {
    Text(String),
    Code { language: String, code: String },
    ToolUse(String, Box<dyn HistoryCell>),  // tool name + ExecCell
    Thinking(String),                        // collapsed reasoning
}

impl HistoryCell for AssistantMessageCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        let content_area = Rect {
            x: area.x + 2,
            width: area.width.saturating_sub(3),
            ..area
        };

        for part in &self.parts {
            match part {
                MessagePart::Text(text) => {
                    for line in wrap_markdown(text, content_area.width as usize) {
                        if y >= area.y + area.height { break; }
                        buf.set_line(content_area.x, y, &line, content_area.width);
                        y += 1;
                    }
                    y += 1; // paragraph spacing
                }
                MessagePart::Code { language, code } => {
                    // Render code block with syntax highlighting
                    let code_lines = render_code_block(language, code, theme);
                    for line in &code_lines {
                        if y >= area.y + area.height { break; }
                        buf.set_line(content_area.x, y, &line, content_area.width);
                        y += 1;
                    }
                    y += 1;
                }
                MessagePart::ToolUse(name, cell) => {
                    cell.render(content_area, buf, theme);
                    y += cell.desired_height(content_area.width);
                }
                MessagePart::Thinking(text) => {
                    // Render as collapsed "Thinking..." with toggle
                    let line = Line::from(vec![
                        Span::styled("  💭 Thinking: ", Style::default().fg(theme.text_subtle.into())),
                        Span::styled(truncate(text, 60), Style::default().fg(theme.text_subtle.into())
                            .add_modifier(Modifier::ITALIC)),
                    ]);
                    buf.set_line(content_area.x, y, &line, content_area.width);
                    y += 1;
                }
            }
        }
    }
}
```

---

## 6. Thinking/Reasoning Block

```
Collapsed (default):

┌─ Assistant ─────────────────────────────────────────────────────────┐
│  💭 Thinking: Let me analyze the auth module... (2.3s)             │
│                                                                    │
│ I'll fix the bug in auth.rs...                                     │
└─────────────────────────────────────────────────────────────────────┘

Expanded (toggled with Ctrl+E or click):

┌─ Assistant ─────────────────────────────────────────────────────────┐
│  💭 Thinking (2.3s)                                                │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Let me analyze the auth module. I need to find where         │  │
│  │ validate_expiry is called and check if it has the right      │  │
│  │ parameters. Looking at line 42...                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                    │
│ I'll fix the bug in auth.rs...                                     │
└─────────────────────────────────────────────────────────────────────┘

Hidden (toggle again):

┌─ Assistant ─────────────────────────────────────────────────────────┐
│ I'll fix the bug in auth.rs...                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/history_cell/thinking_block.rs

pub struct ThinkingBlockCell {
    pub text: String,
    pub duration: std::time::Duration,
    pub display_mode: ThinkingDisplayMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkingDisplayMode {
    Collapsed,   // one line preview
    Expanded,    // full text in bordered box
    Hidden,      // not shown at all
}

impl ThinkingBlockCell {
    pub fn toggle(&mut self) {
        self.display_mode = match self.display_mode {
            ThinkingDisplayMode::Collapsed => ThinkingDisplayMode::Expanded,
            ThinkingDisplayMode::Expanded => ThinkingDisplayMode::Hidden,
            ThinkingDisplayMode::Hidden => ThinkingDisplayMode::Collapsed,
        };
    }
}

impl HistoryCell for ThinkingBlockCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        match self.display_mode {
            ThinkingDisplayMode::Hidden => {},
            ThinkingDisplayMode::Collapsed => {
                let preview = truncate(&self.text, 60);
                let line = Line::from(vec![
                    Span::styled("  💭 Thinking: ", Style::default().fg(theme.text_subtle.into())),
                    Span::styled(
                        format!("{} ({:.1}s)", preview, self.duration.as_secs_f64()),
                        Style::default().fg(theme.text_subtle.into())
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]);
                buf.set_line(area.x, area.y, &line, area.width);
            }
            ThinkingDisplayMode::Expanded => {
                let mut y = area.y;
                // Header
                let header = Line::from(vec![
                    Span::styled(
                        format!("  💭 Thinking ({:.1}s)", self.duration.as_secs_f64()),
                        Style::default().fg(theme.text_subtle.into()),
                    ),
                ]);
                buf.set_line(area.x, y, &header, area.width);
                y += 1;

                // Bordered box with text
                let box_area = Rect {
                    x: area.x + 2,
                    y,
                    width: area.width.saturating_sub(4),
                    height: area.height.saturating_sub(2).min(10),
                };
                render_bordered_text(&self.text, box_area, buf, theme);
            }
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        match self.display_mode {
            ThinkingDisplayMode::Hidden => 0,
            ThinkingDisplayMode::Collapsed => 1,
            ThinkingDisplayMode::Expanded => {
                2 + (self.text.len() as u16 / (width.saturating_sub(4))).min(10)
            }
        }
    }
}
```

---

## 7. Tool Call — Bash

```
Running:

┌─ Bash ──────────────────────────────────────────────────────────────┐
│ $ cargo test --lib jcode-tui                                       │
│ ⠋ running... 2.3s                                                  │
└─────────────────────────────────────────────────────────────────────┘

Completed (success):

┌─ Bash ──────────────────────────────────────────────────────────────┐
│ $ cargo test --lib jcode-tui                                       │
│ ✓ exit: 0                                                          │
│   test result: ok. 42 passed; 0 failed; 0 ignored                  │
│                                                                    │
│   running 42 tests                                                  │
│   test theme::tests::test_cie76 ... ok                             │
│   test keymap::tests::test_context ... ok                          │
│   ...                                                              │
└─────────────────────────────────────────────────────────────────────┘

Completed (failure):

┌─ Bash ──────────────────────────────────────────────────────────────┐
│ $ cargo build                                                      │
│ ✗ exit: 1                                                          │
│   error[E0596]: cannot borrow `buf` as mutable                     │
│     --> src/render.rs:42:5                                          │
│      |                                                              │
│   42 |     buf.set_string(x, y, "hello", style);                   │
│      |     ^^^ mutable borrow occurs here                          │
└─────────────────────────────────────────────────────────────────────┘

Truncated (verbose=false):

┌─ Bash ──────────────────────────────────────────────────────────────┐
│ $ ls -la src/                                                      │
│ ✓ exit: 0                                                          │
│   drwxr-xr-x  5 user  staff  160 Jun 25  .                        │
│   -rw-r--r--  1 user  staff  420 Jun 25  main.rs                  │
│   -rw-r--r--  1 user  staff  890 Jun 25  auth.rs                  │
│   ... 3 more lines                                                 │
└─────────────────────────────────────────────────────────────────────┘
                ↑ "3 more lines" when output > 5 lines
```

### Code

```rust
// crates/jcode-tui/src/tool_ui/bash.rs

use crate::tool_ui::ToolUi;
use crate::history_cell::exec_cell::{ExecCell, ExecCall, ExecStatus, ExecGroupKind};
use jcode_tui_style::Theme;

const TOOL_CALL_MAX_LINES: usize = 5;
const USER_SHELL_MAX_LINES: usize = 50;

pub struct BashToolUi;

impl ToolUi for BashToolUi {
    fn tool_name(&self) -> &str { "Bash" }
    fn icon(&self) -> &str { "$" }

    fn render_use(&self, call: &ExecCall, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;

        // Command line
        let cmd_line = Line::from(vec![
            Span::styled("$ ", Style::default().fg(theme.tool_bash.into())),
            Span::styled(&call.input_summary, Style::default().fg(theme.text.into())),
        ]);
        buf.set_line(area.x, y, &cmd_line, area.width);
        y += 1;

        // Status
        match &call.status {
            ExecStatus::Running { spinner_tick } => {
                let spinner = SPINNER_FRAMES[*spinner_tick as usize % SPINNER_FRAMES.len()];
                let elapsed = call.start_time.elapsed().as_secs_f64();
                let line = Line::from(vec![
                    Span::styled(
                        format!("{} running...", spinner),
                        Style::default().fg(theme.spinner.into()),
                    ),
                    Span::styled(
                        format!(" {:.1}s", elapsed),
                        Style::default().fg(theme.text_subtle.into()),
                    ),
                ]);
                buf.set_line(area.x, y, &line, area.width);
            }
            ExecStatus::Completed { exit_code } => {
                let (icon, color) = if *exit_code == 0 {
                    ("✓", theme.success)
                } else {
                    ("✗", theme.error)
                };
                let line = Line::from(vec![
                    Span::styled(
                        format!("{} exit: {}", icon, exit_code),
                        Style::default().fg(color.into()),
                    ),
                ]);
                buf.set_line(area.x, y, &line, area.width);
            }
            ExecStatus::Failed { error } => {
                let line = Line::from(vec![
                    Span::styled("✗ ", Style::default().fg(theme.error.into())),
                    Span::styled(error, Style::default().fg(theme.error.into())),
                ]);
                buf.set_line(area.x, y, &line, area.width);
            }
            _ => {}
        }
    }

    fn render_result(&self, output: &ExecOutput, area: Rect, buf: &mut Buffer, theme: &Theme) -> u16 {
        let mut y = area.y;
        let max_lines = TOOL_CALL_MAX_LINES;

        let lines: Vec<&str> = output.stdout.lines().collect();
        let display_count = lines.len().min(max_lines);

        for line in &lines[..display_count] {
            if y >= area.y + area.height { break; }
            // Syntax-highlight bash output
            let highlighted = highlight_bash_line(line, theme);
            buf.set_line(area.x, y, &highlighted, area.width);
            y += 1;
        }

        // Truncation indicator
        if lines.len() > max_lines {
            let trunc = Line::from(vec![
                Span::styled(
                    format!("  ... {} more lines", lines.len() - max_lines),
                    Style::default().fg(theme.text_subtle.into()),
                ),
            ]);
            buf.set_line(area.x, y, &trunc, area.width);
            y += 1;
        }

        // Stderr in red
        for line in output.stderr.lines().take(3) {
            if y >= area.y + area.height { break; }
            let err = Line::from(vec![
                Span::styled(line, Style::default().fg(theme.error.into())),
            ]);
            buf.set_line(area.x, y, &err, area.width);
            y += 1;
        }

        y - area.y
    }

    fn summary(&self, call: &ExecCall) -> String {
        format!("$ {}", truncate(&call.input_summary, 80))
    }
}

/// Syntax highlight a single bash output line.
fn highlight_bash_line(line: &str, theme: &Theme) -> Line<'static> {
    // Simple highlighting: numbers in yellow, paths in cyan, errors in red
    let mut spans = Vec::new();
    for token in line.split_whitespace() {
        let style = if token.parse::<f64>().is_ok() {
            Style::default().fg(theme.warning.into())
        } else if token.starts_with('/') || token.contains('/') {
            Style::default().fg(theme.info.into())
        } else if token.starts_with("error") || token.starts_with("Error") {
            Style::default().fg(theme.error.into())
        } else {
            Style::default().fg(theme.text.into())
        };
        spans.push(Span::styled(token.to_string(), style));
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}
```

---

## 8. Tool Call — Edit

```
Running:

┌─ Edit ──────────────────────────────────────────────────────────────┐
│ → Update src/auth.rs                                               │
│ ⠋ applying...                                                      │
└─────────────────────────────────────────────────────────────────────┘

Completed:

┌─ Edit ──────────────────────────────────────────────────────────────┐
│ → Update src/auth.rs                                               │
│  12   fn validate_expiry(expiry: i64) -> bool {                   │
│  12   fn validate_expiry(expiry: i64, now: i64) -> bool {         │
│  13       expiry > 0                                               │
│  13       expiry > now                                             │
└─────────────────────────────────────────────────────────────────────┘

Create (new file):

┌─ Edit ──────────────────────────────────────────────────────────────┐
│ ★ Create src/new_module.rs                                         │
│  + use std::collections::HashMap;                                  │
│  +                                                                 │
│  + pub struct NewModule {                                          │
│  +     data: HashMap<String, String>,                              │
│  + }                                                               │
└─────────────────────────────────────────────────────────────────────┘

  ↑ green for additions (+), red for deletions (-)
  ↑ line numbers shown
```

### Code

```rust
// crates/jcode-tui/src/tool_ui/edit.rs

pub struct EditToolUi;

impl ToolUi for EditToolUi {
    fn tool_name(&self) -> &str { "Edit" }
    fn icon(&self) -> &str { "→" }

    fn render_use(&self, call: &ExecCall, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        let is_create = call.parsed.as_ref()
            .and_then(|p| p.get("old_str"))
            .map(|s| s.is_null() || s.as_str() == Some(""))
            .unwrap_or(false);

        let (action, color) = if is_create {
            ("★ Create", theme.success)
        } else {
            ("→ Update", theme.tool_edit)
        };

        let file_path = call.parsed.as_ref()
            .and_then(|p| p.get("file_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Header
        let header = Line::from(vec![
            Span::styled(format!("{} ", action), Style::default().fg(color.into())),
            Span::styled(file_path, Style::default().fg(theme.text.into())
                .add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(area.x, y, &header, area.width);
        y += 1;

        // Inline diff preview
        if let Some(diff) = extract_diff(call) {
            for diff_line in diff.lines().take(8) {
                if y >= area.y + area.height { break; }
                let (prefix, style) = match diff_line.chars().next() {
                    Some('+') => ("+", Style::default().fg(theme.diff_added.into())),
                    Some('-') => ("-", Style::default().fg(theme.diff_removed.into())),
                    _ => (" ", Style::default().fg(theme.text_muted.into())),
                };
                let line = Line::from(vec![
                    Span::styled(
                        format!("  {}{}", prefix, &diff_line[1..]),
                        style,
                    ),
                ]);
                buf.set_line(area.x, y, &line, area.width);
                y += 1;
            }

            // Show "more" indicator if diff is long
            if diff.lines().count() > 8 {
                let more = Line::from(vec![
                    Span::styled(
                        format!("  ... {} more diff lines", diff.lines().count() - 8),
                        Style::default().fg(theme.text_subtle.into()),
                    ),
                ]);
                buf.set_line(area.x, y, &more, area.width);
                y += 1;
            }
        }
    }

    fn render_result(&self, output: &ExecOutput, area: Rect, buf: &mut Buffer, theme: &Theme) -> u16 {
        let line = if output.exit_code == Some(0) {
            Line::from(vec![
                Span::styled("✓ ", Style::default().fg(theme.success.into())),
                Span::styled(
                    format!("Updated {}", output.file_path),
                    Style::default().fg(theme.text_muted.into()),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("✗ ", Style::default().fg(theme.error.into())),
                Span::styled(
                    output.stderr.lines().next().unwrap_or("Edit failed"),
                    Style::default().fg(theme.error.into()),
                ),
            ])
        };
        buf.set_line(area.x, area.y, &line, area.width);
        1
    }

    fn summary(&self, call: &ExecCall) -> String {
        let is_create = /* ... */;
        let action = if is_create { "Create" } else { "Edit" };
        let file = /* extract file_path */;
        format!("{} {}", action, file)
    }
}
```

---

## 9. Tool Call — Read

```
Running:

┌─ Read ──────────────────────────────────────────────────────────────┐
│ → Read src/auth.rs                                                 │
│ ⠋ reading...                                                       │
└─────────────────────────────────────────────────────────────────────┘

Completed (verbose):

┌─ Read ──────────────────────────────────────────────────────────────┐
│ → Read src/auth.rs                                                 │
│   1  │ use crate::token::validate_token;                           │
│   2  │                                                             │
│   3  │ pub fn validate_expiry(expiry: i64, now: i64) -> bool {    │
│   4  │     expiry > now                                            │
│   5  │ }                                                           │
│   6  │                                                             │
│   7  │ pub fn check_permissions(token: &str) -> Result<(), Error> {│
└─────────────────────────────────────────────────────────────────────┘

Completed (non-verbose, default):

┌─ Read ──────────────────────────────────────────────────────────────┐
│ → Read src/auth.rs (7 lines)                                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/tool_ui/read.rs

pub struct ReadToolUi;

impl ToolUi for ReadToolUi {
    fn tool_name(&self) -> &str { "Read" }
    fn icon(&self) -> &str { "→" }

    fn render_use(&self, call: &ExecCall, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let file_path = extract_file_path(call);
        let line = Line::from(vec![
            Span::styled("→ Read ", Style::default().fg(theme.tool_read.into())),
            Span::styled(&file_path, Style::default().fg(theme.text.into())
                .add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);
    }

    fn render_result(&self, output: &ExecOutput, verbose: bool, area: Rect, buf: &mut Buffer, theme: &Theme) -> u16 {
        if verbose {
            // Show syntax-highlighted file content with line numbers
            render_file_content(output, area, buf, theme)
        } else {
            // Just show summary
            let line_count = output.stdout.lines().count();
            let line = Line::from(vec![
                Span::styled("→ Read ", Style::default().fg(theme.tool_read.into())),
                Span::styled(&output.file_path, Style::default().fg(theme.text.into())),
                Span::styled(
                    format!(" ({} lines)", line_count),
                    Style::default().fg(theme.text_muted.into()),
                ),
            ]);
            buf.set_line(area.x, area.y, &line, area.width);
            1
        }
    }
}

fn render_file_content(output: &ExecOutput, area: Rect, buf: &mut Buffer, theme: &Theme) -> u16 {
    let mut y = area.y;
    let line_num_width = output.stdout.lines().count().to_string().len();

    for (i, line) in output.stdout.lines().enumerate() {
        if y >= area.y + area.height { break; }
        let num = format!("{:>width$} │ ", i + 1, width = line_num_width);
        let highlighted = highlight_syntax(line, &output.file_path, theme);

        let display = Line::from(vec![
            Span::styled(num, Style::default().fg(theme.text_subtle.into())),
            highlighted,
        ]);
        buf.set_line(area.x, y, &display, area.width);
        y += 1;
    }

    y - area.y
}
```

---

## 10. Tool Call — Glob/Grep

```
Glob — inline (compact):

  ☆ glob **/*.rs → 42 matches

Glob — expanded (verbose):

┌─ Glob ──────────────────────────────────────────────────────────────┐
│ ☆ glob **/*.rs → 42 matches                                        │
│   src/main.rs                                                      │
│   src/auth.rs                                                      │
│   src/lib.rs                                                       │
│   ... 39 more                                                      │
└─────────────────────────────────────────────────────────────────────┘

Grep — inline (compact):

  ◆ grep "validate" src/ → 7 matches in 3 files

Grep — expanded (verbose):

┌─ Grep ──────────────────────────────────────────────────────────────┐
│ ◆ grep "validate" src/ → 7 matches in 3 files                      │
│   src/auth.rs:12: fn validate_expiry(...)                           │
│   src/auth.rs:45: if !validate_expiry(...)                         │
│   src/token.rs:8: pub fn validate_token(...)                       │
│   ... 4 more                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/tool_ui/glob.rs

pub struct GlobToolUi;

impl ToolUi for GlobToolUi {
    fn tool_name(&self) -> &str { "Glob" }
    fn icon(&self) -> &str { "☆" }
    fn color(&self, theme: &Theme) -> Color { theme.info }

    fn render_use(&self, call: &ExecCall, verbose: bool, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let pattern = extract_pattern(call);
        let match_count = extract_match_count(call);

        if verbose {
            // Expanded view with file list
            let header = Line::from(vec![
                Span::styled("☆ ", Style::default().fg(theme.info.into())),
                Span::styled(
                    format!("glob {} → {} matches", pattern, match_count),
                    Style::default().fg(theme.text.into()),
                ),
            ]);
            buf.set_line(area.x, area.y, &header, area.width);
            // ... render file list
        } else {
            // Inline view
            let line = Line::from(vec![
                Span::styled("☆ ", Style::default().fg(theme.info.into())),
                Span::styled(
                    format!("glob {} → {} matches", pattern, match_count),
                    Style::default().fg(theme.text_muted.into()),
                ),
            ]);
            buf.set_line(area.x, area.y, &line, area.width);
        }
    }
}

// crates/jcode-tui/src/tool_ui/grep.rs

pub struct GrepToolUi;

impl ToolUi for GrepToolUi {
    fn tool_name(&self) -> &str { "Grep" }
    fn icon(&self) -> &str { "◆" }
    fn color(&self, theme: &Theme) -> Color { theme.info }

    fn render_use(&self, call: &ExecCall, verbose: bool, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let pattern = extract_pattern(call);
        let match_count = extract_match_count(call);
        let file_count = extract_file_count(call);

        let line = Line::from(vec![
            Span::styled("◆ ", Style::default().fg(theme.info.into())),
            Span::styled(
                format!("grep \"{}\" → {} matches in {} files", pattern, match_count, file_count),
                if verbose { Style::default().fg(theme.text.into()) }
                else { Style::default().fg(theme.text_muted.into()) },
            ),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
```

---

## 11. Tool Call — Agent (Sub-agent)

```
Running:

┌─ Agent ─────────────────────────────────────────────────────────────┐
│ 🔱 Sub-agent: "research auth patterns"                            │
│ ⠋ running... 12.3s                                                 │
│   tools: 3 read, 2 grep, 1 bash                                   │
└─────────────────────────────────────────────────────────────────────┘

Completed:

┌─ Agent ─────────────────────────────────────────────────────────────┐
│ ✓ Sub-agent: "research auth patterns" (15.2s)                      │
│   tools: 5 read, 2 grep, 1 bash                                   │
│                                                                    │
│   Found 3 common auth patterns in the codebase:                    │
│   1. JWT token validation                                          │
│   2. Session-based auth                                            │
│   3. OAuth2 flow                                                   │
└─────────────────────────────────────────────────────────────────────┘

Delegating (when spawning):

┌─ Agent ─────────────────────────────────────────────────────────────┐
│ 📤 Delegating to sub-agent...                                      │
│   task: "implement the fix"                                        │
│   model: claude-sonnet-4-20250514                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/tool_ui/agent.rs

pub struct AgentToolUi;

impl ToolUi for AgentToolUi {
    fn tool_name(&self) -> &str { "Agent" }
    fn icon(&self) -> &str { "🔱" }
    fn color(&self, theme: &Theme) -> Color { theme.info }

    fn render_use(&self, call: &ExecCall, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let task = extract_task(call);
        let model = extract_model(call);

        match &call.status {
            ExecStatus::Running { .. } => {
                let header = Line::from(vec![
                    Span::styled("📤 Delegating to sub-agent...", Style::default().fg(theme.info.into())),
                ]);
                buf.set_line(area.x, area.y, &header, area.width);
                if let Some(task) = &task {
                    let line = Line::from(vec![
                        Span::styled("  task: ", Style::default().fg(theme.text_subtle.into())),
                        Span::styled(format!("\"{}\"", task), Style::default().fg(theme.text.into())),
                    ]);
                    buf.set_line(area.x, area.y + 1, &line, area.width);
                }
                if let Some(model) = &model {
                    let line = Line::from(vec![
                        Span::styled("  model: ", Style::default().fg(theme.text_subtle.into())),
                        Span::styled(model, Style::default().fg(theme.text_muted.into())),
                    ]);
                    buf.set_line(area.x, area.y + 2, &line, area.width);
                }
            }
            ExecStatus::Completed { .. } => {
                let duration = call.duration.unwrap_or_default().as_secs_f64();
                let header = Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(theme.success.into())),
                    Span::styled(
                        format!("Sub-agent: \"{}\" ({:.1}s)", task.as_deref().unwrap_or(""), duration),
                        Style::default().fg(theme.text.into()),
                    ),
                ]);
                buf.set_line(area.x, area.y, &header, area.width);

                // Tool usage summary
                let tools_used = extract_tools_used(call);
                let line = Line::from(vec![
                    Span::styled("  tools: ", Style::default().fg(theme.text_subtle.into())),
                    Span::styled(tools_used, Style::default().fg(theme.text_muted.into())),
                ]);
                buf.set_line(area.x, area.y + 1, &line, area.width);
            }
            _ => {}
        }
    }
}
```

---

## 12. Permission Dialog — Bash

```
┌─────────────────────────────────────────────────────────────────────┐
│ 🔐 Permission required                                             │
│                                                                    │
│ Bash wants to run:                                                 │
│                                                                    │
│ $ rm -rf /tmp/test                                                 │
│                                                                    │
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ ⚠ This command will delete files permanently.                  │ │
│ └────────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  [y] Allow    [Y] Always    [n] Deny    [Esc] Abort               │
│                                                                    │
│  Ctrl+D: debug  Ctrl+E: explanation                                │
└─────────────────────────────────────────────────────────────────────┘

  ↑ replaces the composer area at the bottom
  ↑ yellow border for warning
  ↑ keyboard shortcuts shown as hints
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/permission_dialog/bash_permission.rs

use crate::bottom_pane::{BottomPaneResult, BottomPaneView};
use jcode_tui_core::keymap::KeyCombo;

pub struct BashPermissionDialog {
    command: String,
    reason: Option<String>,
    selected: usize,
    choices: Vec<PermissionChoice>,
    show_debug: bool,
    show_explanation: bool,
}

struct PermissionChoice {
    label: String,
    key: String,
    action: PermissionAction,
}

enum PermissionAction {
    AllowOnce,
    AllowForSession,
    AllowAlways,
    Deny,
    Abort,
}

impl BashPermissionDialog {
    pub fn new(command: String, reason: Option<String>) -> Self {
        Self {
            command,
            reason,
            selected: 0,
            choices: vec![
                PermissionChoice { label: "Allow".into(), key: "y".into(), action: PermissionAction::AllowOnce },
                PermissionChoice { label: "Always".into(), key: "Y".into(), action: PermissionAction::AllowForSession },
                PermissionChoice { label: "Deny".into(), key: "n".into(), action: PermissionAction::Deny },
                PermissionChoice { label: "Abort".into(), key: "Esc".into(), action: PermissionAction::Abort },
            ],
            show_debug: false,
            show_explanation: false,
        }
    }
}

impl BottomPaneView for BashPermissionDialog {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;

        // Header
        let header = Line::from(vec![
            Span::styled("🔐 ", Style::default().fg(theme.warning.into())),
            Span::styled("Permission required", Style::default().fg(theme.text.into())
                .add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(area.x, y, &header, area.width);
        y += 2;

        // Command
        let cmd_label = Line::from(vec![
            Span::styled("Bash wants to run:", Style::default().fg(theme.text_muted.into())),
        ]);
        buf.set_line(area.x, y, &cmd_label, area.width);
        y += 1;

        let cmd_line = Line::from(vec![
            Span::styled("$ ", Style::default().fg(theme.tool_bash.into())),
            Span::styled(&self.command, Style::default().fg(theme.text.into())),
        ]);
        buf.set_line(area.x + 2, y, &cmd_line, area.width.saturating_sub(4));
        y += 2;

        // Warning (if destructive)
        if self.is_destructive() {
            render_warning_box("This command may be destructive.", area, &mut y, theme);
        }

        // Explanation (if shown)
        if self.show_explanation {
            if let Some(reason) = &self.reason {
                let line = Line::from(vec![
                    Span::styled("💡 ", Style::default().fg(theme.info.into())),
                    Span::styled(reason, Style::default().fg(theme.text.into())
                        .add_modifier(Modifier::ITALIC)),
                ]);
                buf.set_line(area.x + 2, y, &line, area.width.saturating_sub(4));
                y += 1;
            }
        }

        // Debug info (if shown)
        if self.show_debug {
            render_debug_info(self, area, &mut y, theme);
        }

        // Choices
        y += 1;
        let mut spans = Vec::new();
        for (i, choice) in self.choices.iter().enumerate() {
            let style = if i == self.selected {
                Style::default().fg(theme.background.into()).bg(theme.accent.into())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_muted.into())
            };
            spans.push(Span::styled(
                format!(" [{}] {} ", choice.key, choice.label),
                style,
            ));
        }
        let choices_line = Line::from(spans);
        buf.set_line(area.x, y, &choices_line, area.width);
        y += 1;

        // Hints
        let hints = Line::from(vec![
            Span::styled("Ctrl+D: debug  Ctrl+E: explanation",
                Style::default().fg(theme.text_subtle.into())),
        ]);
        buf.set_line(area.x, y, &hints, area.width);
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match (key.key.as_str(), key.ctrl, key.alt) {
            ("y", false, false) => { self.selected = 0; true }
            ("Y", false, false) => { self.selected = 1; true }
            ("n", false, false) => { self.selected = 2; true }
            ("d", true, false) => { self.show_debug = !self.show_debug; true }
            ("e", true, false) => { self.show_explanation = !self.show_explanation; true }
            ("left" | "h", false, false) => {
                self.selected = (self.selected + self.choices.len() - 1) % self.choices.len();
                true
            }
            ("right" | "l", false, false) => {
                self.selected = (self.selected + 1) % self.choices.len();
                true
            }
            _ => false,
        }
    }

    fn completion(&self) -> Option<BottomPaneResult> {
        Some(BottomPaneResult::Approved {
            choice: self.choices[self.selected].action.clone(),
        })
    }

    fn handle_ctrl_c(&mut self) -> BottomPaneResult {
        BottomPaneResult::Cancelled
    }
}
```

---

## 13. Permission Dialog — Edit

```
┌─────────────────────────────────────────────────────────────────────┐
│ 🔐 Permission required                                             │
│                                                                    │
│ Edit wants to modify:                                              │
│                                                                    │
│ → src/auth.rs                                                      │
│                                                                    │
│ ┌─ diff ────────────────────────────────────────────────────────┐  │
│ │  12 │ fn validate_expiry(expiry: i64) -> bool {              │  │
│ │  12 │ fn validate_expiry(expiry: i64, now: i64) -> bool {    │  │
│ │  13 │     expiry > 0                                          │  │
│ │  13 │     expiry > now                                        │  │
│ └───────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  [y] Allow    [a] Always Allow    [n] Deny                        │
└─────────────────────────────────────────────────────────────────────┘

  ↑ diff is syntax-highlighted
  ↑ green for additions, red for deletions
```

---

## 14. Permission Dialog — Read

```
┌─────────────────────────────────────────────────────────────────────┐
│ 🔐 Permission required                                             │
│                                                                    │
│ Read wants to access:                                              │
│                                                                    │
│ → /etc/passwd                                                      │
│                                                                    │
│ [y] Allow    [a] Always Allow    [n] Deny                         │
└─────────────────────────────────────────────────────────────────────┘

  ↑ simpler than Bash/Edit — just shows file path
  ↑ no diff preview needed
```

---

## 15. Chat Composer (Input)

```
Normal state:

┌─────────────────────────────────────────────────────────────────────┐
│ ▌                                                                  │
│                                                                    │
└─────────────────────────────────────────────────────────────────────┘

With input:

┌─────────────────────────────────────────────────────────────────────┐
│ Fix the bug in auth.rs and add tests                               │
│ ▌                                                                  │
└─────────────────────────────────────────────────────────────────────┘

With autocomplete popup:

┌─────────────────────────────────────────────────────────────────────┐
│ Read src/aut                                                       │
│ ┌──────────────────────┐                                           │
│ │ auth.rs              │                                           │
│ │ auth_test.rs         │                                           │
│ │ auto_complete.rs     │                                           │
│ └──────────────────────┘                                           │
└─────────────────────────────────────────────────────────────────────┘

  ↑ autocomplete shows file matches from ffs
  ↑ cursor is a blinking block
  ↑ input grows vertically with content (max 10 lines)
  ↑ Ctrl+G opens external editor
  ↑ Ctrl+S stashes current input
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/chat_composer.rs

use crate::bottom_pane::BottomPaneView;
use jcode_tui_core::keymap::KeyCombo;

pub struct ChatComposer {
    /// Current input text.
    text: String,
    /// Cursor position (byte offset).
    cursor: usize,
    /// Autocomplete state.
    autocomplete: Option<AutocompleteState>,
    /// Input history for arrow-key navigation.
    history: Vec<String>,
    /// Current history index (-1 = current input).
    history_index: isize,
    /// Stashed input (Ctrl+S).
    stash: Option<String>,
    /// External editor active flag.
    external_editor_active: bool,
}

struct AutocompleteState {
    query: String,
    candidates: Vec<String>,
    selected: usize,
}

impl BottomPaneView for ChatComposer {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        let input_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(2), // leave room for autocomplete
        };

        // Render input text
        let lines: Vec<&str> = self.text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if y >= input_area.y + input_area.height { break; }
            let display = Line::from(vec![
                Span::styled(*line, Style::default().fg(theme.text.into())),
            ]);
            buf.set_line(input_area.x, y, &display, input_area.width);
            y += 1;
        }

        // Render cursor
        if y < input_area.y + input_area.height {
            buf.set_string(input_area.x, y, "▌", Style::default().fg(theme.accent.into()));
        }

        // Render autocomplete popup
        if let Some(ref ac) = self.autocomplete {
            let popup_area = Rect {
                x: input_area.x,
                y: y + 1,
                width: 30.min(input_area.width),
                height: (ac.candidates.len() as u16 + 2).min(8),
            };
            render_autocomplete_popup(ac, popup_area, buf, theme);
        }
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match (key.key.as_str(), key.ctrl, key.shift, key.alt) {
            // Submit
            ("enter", false, false, false) => {
                if self.text.trim().is_empty() { return false; }
                // Will trigger completion
                false
            }

            // Queue message (Tab)
            ("tab", false, false, false) => {
                // Queue current input for next turn
                self.queue_input();
                true
            }

            // History navigation
            ("up", false, false, false) => {
                if self.autocomplete.is_some() {
                    self.autocomplete.as_mut().unwrap().prev();
                } else {
                    self.history_prev();
                }
                true
            }
            ("down", false, false, false) => {
                if self.autocomplete.is_some() {
                    self.autocomplete.as_mut().unwrap().next();
                } else {
                    self.history_next();
                }
                true
            }

            // Stash (Ctrl+S)
            ("s", true, false, false) => {
                self.stash = Some(self.text.clone());
                self.text.clear();
                self.cursor = 0;
                true
            }

            // Unstash (Ctrl+Shift+S or just start typing)
            // External editor (Ctrl+G or Ctrl+X Ctrl+E)
            ("g", true, false, false) | ("e", true, true, false) => {
                self.open_external_editor();
                true
            }

            // Autocomplete accept (Tab when popup is open)
            ("tab", false, false, false) if self.autocomplete.is_some() => {
                self.accept_autocomplete();
                true
            }

            // Autocomplete dismiss (Esc)
            ("escape", false, false, false) if self.autocomplete.is_some() => {
                self.autocomplete = None;
                true
            }

            // Regular text input
            _ => {
                self.insert_char(key);
                self.update_autocomplete();
                true
            }
        }
    }

    fn completion(&self) -> Option<BottomPaneResult> {
        if self.text.trim().is_empty() {
            None
        } else {
            Some(BottomPaneResult::Submitted {
                text: self.text.clone(),
            })
        }
    }
}

impl ChatComposer {
    fn update_autocomplete(&mut self) {
        // Trigger autocomplete if input looks like a file path
        let last_word = self.text.rsplit_whitespace().next().unwrap_or("");
        if last_word.contains('/') || last_word.starts_with('@') {
            let query = last_word.trim_start_matches('@');
            let candidates = ffs_search(query); // search via ffs MCP
            if !candidates.is_empty() {
                self.autocomplete = Some(AutocompleteState {
                    query: query.to_string(),
                    candidates,
                    selected: 0,
                });
            }
        } else {
            self.autocomplete = None;
        }
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() { return; }
        if self.history_index == -1 {
            self.stash = Some(self.text.clone());
        }
        self.history_index = (self.history_index + 1)
            .min(self.history.len() as isize - 1);
        self.text = self.history[self.history_index as usize].clone();
        self.cursor = self.text.len();
    }

    fn history_next(&mut self) {
        if self.history_index < 0 { return; }
        self.history_index -= 1;
        if self.history_index < 0 {
            self.text = self.stash.take().unwrap_or_default();
            self.history_index = -1;
        } else {
            self.text = self.history[self.history_index as usize].clone();
        }
        self.cursor = self.text.len();
    }
}
```

---

## 16. Spinner States

```
Idle:
(empty)

Thinking:
⠋ Thinking...

Tool running:
⚙ Running Bash...

Agent spawning:
🔱 Spawning sub-agent...

Waiting for network:
⏳ Waiting for network...

Permission pending:
🔐 Awaiting permission...

Streaming:
⚙ Streaming response...

Compact (during tool output):
·

Hook executing:
⚡ Running hook...

Agent delegating:
📤 Delegating...

Brief (quick operations):
✨

Speculation:
🔮 Speculating...
```

### Code

```rust
// crates/jcode-tui-style/src/spinner.rs

/// Animated braille spinner frames.
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// All spinner modes with their visual representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerMode {
    Idle,
    Thinking,
    Tool,
    Compact,
    Hook,
    Waiting,
    Permission,
    Speculation,
    AgentFork,
    AgentDelegating,
    AgentRunning,
    BriefWaiting,
    BriefGenerating,
}

impl SpinnerMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Thinking => "Thinking...",
            Self::Tool => "Running...",
            Self::Compact => "",
            Self::Hook => "Running hook...",
            Self::Waiting => "Waiting for network...",
            Self::Permission => "Awaiting permission...",
            Self::Speculation => "Speculating...",
            Self::AgentFork => "Spawning sub-agent...",
            Self::AgentDelegating => "Delegating...",
            Self::AgentRunning => "Sub-agent running...",
            Self::BriefWaiting => "",
            Self::BriefGenerating => "",
        }
    }

    pub fn icon(&self, tick: u8) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Thinking | Self::Tool | Self::AgentRunning => {
                SPINNER_FRAMES[tick as usize % SPINNER_FRAMES.len()]
            }
            Self::Compact => "·",
            Self::Hook => "⚡",
            Self::Waiting => "⏳",
            Self::Permission => "🔐",
            Self::Speculation => "🔮",
            Self::AgentFork | Self::AgentDelegating => "📤",
            Self::BriefWaiting => "💤",
            Self::BriefGenerating => "✨",
        }
    }

    pub fn color(&self, theme: &Theme) -> ratatui::style::Color {
        match self {
            Self::Idle => theme.text_subtle.into(),
            Self::Thinking => theme.accent.into(),
            Self::Tool => theme.text_muted.into(),
            Self::Permission => theme.warning.into(),
            Self::AgentFork | Self::AgentDelegating | Self::AgentRunning => theme.info.into(),
            Self::Waiting => theme.warning.into(),
            _ => theme.text_subtle.into(),
        }
    }
}

/// Render the spinner in a 1-row area.
pub fn render_spinner(mode: SpinnerMode, tick: u8, area: Rect, buf: &mut Buffer, theme: &Theme) {
    if mode == SpinnerMode::Idle { return; }

    let icon = mode.icon(tick);
    let label = mode.label();
    let color = mode.color(theme);

    let line = Line::from(vec![
        Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(label, Style::default().fg(color)),
    ]);
    buf.set_line(area.x, area.y, &line, area.width);
}
```

---

## 17. Unseen Divider

```
When user scrolls up and new messages arrive:

┌─────────────────────────────────────────────────────────────────────┐
│ > older message                                                    │
│                                                                    │
│────────────── 3 new messages ───────────────────────────────────── │
│                                                                    │
│ > newer message (pinned content)                                   │
└─────────────────────────────────────────────────────────────────────┘
                ↑ clickable to jump to bottom

When at bottom, divider is hidden.
```

### Code

```rust
// crates/jcode-tui/src/history_cell/unseen_divider.rs

pub struct UnseenDivider {
    pub message_count: usize,
}

impl HistoryCell for UnseenDivider {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let text = format!(" {} new messages ", self.message_count);
        let padding = (area.width as usize).saturating_sub(text.len()) / 2;
        let left_pad = "─".repeat(padding.saturating_sub(1));
        let right_pad = "─".repeat(area.width as usize - padding - text.len() + 1);

        let line = Line::from(vec![
            Span::styled(left_pad, Style::default().fg(theme.border.into())),
            Span::styled(&text, Style::default().fg(theme.accent.into())
                .add_modifier(Modifier::BOLD)),
            Span::styled(right_pad, Style::default().fg(theme.border.into())),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);
    }

    fn desired_height(&self, _width: u16) -> u16 { 1 }
}
```

---

## 18. Transcript Overlay (Full-screen)

```
Toggle with Ctrl+O — shows full transcript in alternate screen:

┌─────────────────────────────────────────────────────────────────────┐
│ TRANSCRIPT                                          ↑/↓ scroll    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ [User]                                                              │
│ > Fix the bug in auth.rs                                            │
│                                                                     │
│ [Assistant]                                                         │
│ I'll analyze the auth module.                                       │
│                                                                     │
│ [Bash] $ grep -n "validate" src/auth.rs                            │
│        ✓ exit: 0                                                    │
│        12: fn validate_token(token: &str) -> bool {                 │
│                                                                     │
│ [Edit] → Update src/auth.rs                                         │
│                                                                     │
│ [Assistant]                                                          │
│ Fixed the bug.                                                      │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ /search  ↑↓ navigate  q: close  Ctrl+R: reverse search             │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/transcript_overlay.rs

pub struct TranscriptOverlay {
    cells: Vec<Arc<dyn HistoryCell>>,
    scroll: usize,
    search_mode: bool,
    search_query: String,
    search_matches: Vec<usize>,
    current_match: usize,
}

impl BottomPaneView for TranscriptOverlay {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let header_area = Rect { height: 1, ..area };
        let content_area = Rect { y: area.y + 1, height: area.height - 2, ..area };
        let footer_area = Rect { y: area.y + area.height - 1, height: 1, ..area };

        // Header
        let header = Line::from(vec![
            Span::styled("TRANSCRIPT", Style::default().fg(theme.text.into())
                .add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("↑/↓ scroll", Style::default().fg(theme.text_subtle.into())),
        ]);
        buf.set_line(header_area.x, header_area.y, &header, header_area.width);

        // Content (scrollable)
        let mut y = content_area.y;
        for cell in &self.cells {
            if y >= content_area.y + content_area.height { break; }
            let height = cell.desired_height(content_area.width);
            let cell_area = Rect { y, height: height.min(content_area.height - (y - content_area.y)), ..content_area };
            cell.render(cell_area, buf, theme);
            y += height;
        }

        // Footer
        let footer = if self.search_mode {
            Line::from(vec![
                Span::styled(format!("/{}", self.search_query), Style::default().fg(theme.accent.into())),
                Span::raw("  "),
                Span::styled(
                    format!("{}/{}", self.current_match + 1, self.search_matches.len()),
                    Style::default().fg(theme.text_muted.into()),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("/search  ", Style::default().fg(theme.text_subtle.into())),
                Span::styled("↑↓ navigate  ", Style::default().fg(theme.text_subtle.into())),
                Span::styled("q: close  ", Style::default().fg(theme.text_subtle.into())),
                Span::styled("Ctrl+R: reverse search", Style::default().fg(theme.text_subtle.into())),
            ])
        };
        buf.set_line(footer_area.x, footer_area.y, &footer, footer_area.width);
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match (key.key.as_str(), key.ctrl) {
            ("up" | "k", false) => { self.scroll_up(); true }
            ("down" | "j", false) => { self.scroll_down(); true }
            ("pageup", false) => { self.page_up(); true }
            ("pagedown", false) => { self.page_down(); true }
            ("g", false) => { self.scroll_to_top(); true }
            ("G", false) => { self.scroll_to_bottom(); true }
            ("/", false) => { self.search_mode = true; true }
            ("r", true) => { self.reverse_search(); true }
            ("escape" | "q", false) => { self.close(); true }
            _ => false,
        }
    }

    fn is_complete(&self) -> bool { self.closed }
    fn view_id(&self) -> &str { "transcript_overlay" }
}
```

---

## 19. Keybinding Which-Key Panel

```
Toggle with Ctrl+Alt+K:

┌─────────────────────────────────────────────────────────────────────┐
│ Keybindings                                            ↑/↓ scroll│
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ Global                                                              │
│   Ctrl+C        Interrupt                                           │
│   Ctrl+D        Exit                                                │
│   Ctrl+L        Redraw                                              │
│   Ctrl+O        Toggle transcript                                   │
│   Ctrl+T        Toggle todos                                        │
│   Ctrl+X        Leader key                                          │
│   Ctrl+Alt+K    This panel                                         │
│                                                                     │
│ Chat                                                                │
│   Enter         Submit message                                      │
│   Tab           Queue message                                       │
│   Ctrl+G        External editor                                     │
│   Ctrl+S        Stash input                                         │
│   Shift+↑       Message actions                                     │
│   ↑/↓           History navigation                                  │
│                                                                     │
│ Composer                                                            │
│   Ctrl+Enter    Submit with context                                 │
│   Ctrl+X Ctrl+E External editor                                     │
│                                                                     │
│ [filter: ]                                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/which_key.rs

pub struct WhichKeyPanel {
    bindings: Vec<KeybindingGroup>,
    scroll: usize,
    filter: String,
}

struct KeybindingGroup {
    name: String,
    bindings: Vec<KeyBindingDisplay>,
}

struct KeyBindingDisplay {
    key: String,
    description: String,
}

impl BottomPaneView for WhichKeyPanel {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;

        // Header
        let header = Line::from(vec![
            Span::styled("Keybindings", Style::default().fg(theme.text.into())
                .add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(area.x, y, &header, area.width);
        y += 2;

        // Filtered groups
        for group in &self.bindings {
            if y >= area.y + area.height { break; }

            // Group header
            let group_header = Line::from(vec![
                Span::styled(&group.name, Style::default().fg(theme.accent.into())
                    .add_modifier(Modifier::BOLD)),
            ]);
            buf.set_line(area.x, y, &group_header, area.width);
            y += 1;

            // Bindings in group
            for binding in &group.bindings {
                if y >= area.y + area.height { break; }
                if !self.filter.is_empty() && !binding.description.to_lowercase()
                    .contains(&self.filter.to_lowercase()) {
                    continue;
                }

                let line = Line::from(vec![
                    Span::styled(
                        format!("  {:<20}", binding.key),
                        Style::default().fg(theme.warning.into()),
                    ),
                    Span::styled(
                        &binding.description,
                        Style::default().fg(theme.text_muted.into()),
                    ),
                ]);
                buf.set_line(area.x, y, &line, area.width);
                y += 1;
            }

            y += 1; // spacing between groups
        }

        // Footer
        let footer = Line::from(vec![
            Span::styled("type to filter  ↑/↓ scroll  q/esc: close",
                Style::default().fg(theme.text_subtle.into())),
        ]);
        buf.set_line(area.x, area.y + area.height - 1, &footer, area.width);
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match key.key.as_str() {
            "escape" | "q" => { self.close(); true }
            "up" | "k" => { self.scroll_up(); true }
            "down" | "j" => { self.scroll_down(); true }
            _ => {
                // Add character to filter
                if key.key.len() == 1 && !key.ctrl && !key.alt {
                    self.filter.push_str(&key.key);
                    true
                } else if key.key == "backspace" {
                    self.filter.pop();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view_id(&self) -> &str { "which_key" }
}
```

---

## 20. Session Picker

```
Toggle with --resume or from slash command:

┌─────────────────────────────────────────────────────────────────────┐
│ Sessions (12)                              type to search: auth    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ ▸ auth bug fix                    2h ago  main  claude-sonnet-4     │
│   Fix the authentication bug in auth.rs                             │
│                                                                     │
│   feature/tui-redesign           1d ago  feat/ claude-opus-4        │
│   Migrate TUI to Claude Code patterns                               │
│                                                                     │
│   add tests for keymap           3d ago  main  claude-sonnet-4     │
│   Add comprehensive keybinding tests                                │
│                                                                     │
│   (2 more matches hidden)                                           │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ Enter: resume  d: delete  f: fork  q: close                        │
└─────────────────────────────────────────────────────────────────────┘

  ↑ selected row is highlighted
  ↑ shows: session title, age, git branch, model
  ↑ search filters by title
  ↑ "▸" cursor for selected item
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/session_picker.rs

pub struct SessionPicker {
    sessions: Vec<SessionInfo>,
    filtered: Vec<usize>,
    selected: usize,
    search: String,
    scroll: usize,
}

struct SessionInfo {
    id: String,
    title: String,
    age: String,         // "2h ago", "1d ago"
    branch: String,      // git branch
    model: String,       // model name
}

impl BottomPaneView for SessionPicker {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;

        // Header with search
        let header = Line::from(vec![
            Span::styled(
                format!("Sessions ({})", self.sessions.len()),
                Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("type to search: {}", self.search),
                Style::default().fg(theme.text_subtle.into()),
            ),
        ]);
        buf.set_line(area.x, y, &header, area.width);
        y += 2;

        // Session list
        for (i, &idx) in self.filtered.iter().enumerate().skip(self.scroll) {
            if y >= area.y + area.height - 2 { break; }
            let session = &self.sessions[idx];
            let is_selected = i == self.selected;

            let cursor = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default().fg(theme.background.into()).bg(theme.accent.into())
            } else {
                Style::default()
            };

            // Title line
            let line = Line::from(vec![
                Span::styled(cursor, style),
                Span::styled(&session.title, style.add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("  {}  {}  {}", session.age, session.branch, session.model),
                    style.fg(theme.text_muted.into()),
                ),
            ]);
            buf.set_line(area.x, y, &line, area.width);
            y += 1;
        }

        // Footer
        let footer = Line::from(vec![
            Span::styled("Enter: resume  d: delete  f: fork  q: close",
                Style::default().fg(theme.text_subtle.into())),
        ]);
        buf.set_line(area.x, area.y + area.height - 1, &footer, area.width);
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match key.key.as_str() {
            "up" | "k" => { self.prev(); true }
            "down" | "j" => { self.next(); true }
            "enter" => { self.resume_selected(); true }
            "d" => { self.delete_selected(); true }
            "f" => { self.fork_selected(); true }
            "escape" | "q" => { self.close(); true }
            _ => {
                // Search filter
                if key.key.len() == 1 && !key.ctrl && !key.alt {
                    self.search.push_str(&key.key);
                    self.apply_filter();
                    true
                } else if key.key == "backspace" {
                    self.search.pop();
                    self.apply_filter();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view_id(&self) -> &str { "session_picker" }
}
```

---

## 21. Footer / Hints Bar

```
Normal (wide terminal):

┌─────────────────────────────────────────────────────────────────────┐
│ Tab:autocomplete  Ctrl+X:leader  Ctrl+O:transcript  /:commands    │
└─────────────────────────────────────────────────────────────────────┘

Narrow terminal (progressive collapse):

┌─────────────────────────────────────────────────────────────┐
│ Ctrl+O:transcript  /:commands                               │
└─────────────────────────────────────────────────────────────┘

Very narrow:

┌─────────────────────────────────────┐
│ /:commands                         │
└─────────────────────────────────────┘

Leader key pressed:

┌─────────────────────────────────────────────────────────────────────┐
│ n:new session  o:transcript  t:todos  r:search  ...               │
└─────────────────────────────────────────────────────────────────────┘

Queue active:

┌─────────────────────────────────────────────────────────────────────┐
│ [1 queued] Tab:send next  Enter:submit                            │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/footer.rs

use std::collections::BTreeMap;

pub struct FooterState {
    pub hints: Vec<HintItem>,
    pub queue_count: usize,
    pub leader_active: bool,
    pub context: KeyContext,
}

struct HintItem {
    key: String,
    action: String,
    priority: u8, // higher = more important, kept first when collapsing
}

pub fn render_footer(state: &FooterState, area: Rect, buf: &mut Buffer, theme: &Theme) {
    let width = area.width as usize;

    let hints = if state.leader_active {
        render_leader_hints(theme)
    } else if state.queue_count > 0 {
        render_queue_hints(state.queue_count, theme)
    } else {
        render_default_hints(state.context, theme)
    };

    // Progressive collapse: drop lowest-priority hints to fit width
    let mut total_len: usize = hints.iter().map(|h| h.len() + 2).sum();
    let mut visible: Vec<&HintItem> = hints.iter().collect();

    while total_len > width && visible.len() > 1 {
        // Remove lowest priority (last in the sorted list)
        let removed = visible.pop().unwrap();
        total_len -= removed.len() + 2;
    }

    // Render
    let mut spans = Vec::new();
    for (i, hint) in visible.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let parts: Vec<&str> = hint.splitn(2, ':').collect();
        if parts.len() == 2 {
            spans.push(Span::styled(
                format!("{}:", parts[0]),
                Style::default().fg(theme.warning.into()),
            ));
            spans.push(Span::styled(
                parts[1].to_string(),
                Style::default().fg(theme.text_subtle.into()),
            ));
        } else {
            spans.push(Span::styled(
                hint.clone(),
                Style::default().fg(theme.text_subtle.into()),
            ));
        }
    }

    let line = Line::from(spans);
    buf.set_line(area.x, area.y, &line, area.width);
}
```

---

## 22. System Messages

```
Notification:

  ℹ Starting new session...

Error:

  ✗ Connection lost. Reconnecting...

Warning:

  ⚠ Context limit approaching (90%)

Tool progress:

  ⚙ Running 5 parallel searches...
```

### Code

```rust
// crates/jcode-tui/src/history_cell/system_message.rs

pub struct SystemMessageCell {
    pub text: String,
    pub level: SystemLevel,
}

pub enum SystemLevel {
    Info,
    Warning,
    Error,
    Progress,
}

impl HistoryCell for SystemMessageCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let (icon, color) = match self.level {
            SystemLevel::Info => ("ℹ", theme.info),
            SystemLevel::Warning => ("⚠", theme.warning),
            SystemLevel::Error => ("✗", theme.error),
            SystemLevel::Progress => ("⚙", theme.text_muted),
        };

        let line = Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color.into())),
            Span::styled(&self.text, Style::default().fg(theme.text_muted.into())
                .add_modifier(Modifier::ITALIC)),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);
    }

    fn desired_height(&self, width: u16) -> u16 {
        (self.text.len() as u16 / width + 1).min(3)
    }
}
```

---

## 23. Mermaid Diagram Pane

```
Side panel showing mermaid diagram:

┌────────────────────────────────────┬──────────────────────┐
│ Chat viewport                     │ Mermaid Diagram      │
│                                   │                      │
│ > Create a diagram of the auth    │ ┌──────────────────┐ │
│                                   │ │   ┌───┐          │ │
│ ┌─ Bash ────────────────────────┐ │ │   │ A │──→┌───┐ │ │
│ │ $ mmdc -i auth.mmd           │ │ │   └───┘   │ B │ │ │
│ │ ✓ exit: 0                     │ │ │           └───┘ │ │
│ └───────────────────────────────┘ │ │                   │ │
│                                   │ └──────────────────┘ │
│ Here's the diagram:              │ [+/-] zoom  ←/→ pan  │
│                                   │                      │
└────────────────────────────────────┴──────────────────────┘

  ↑ side panel shows rendered mermaid diagram
  ↑ zoom/scroll controls at bottom
  ↑ toggled with Ctrl+M or /mermaid command
```

---

## 24. Swarm Gallery (Multi-Agent)

```
┌─────────────────────────────────────────────────────────────────────┐
│ ⋯ swarm · 4 agents · 2 active                                    │
├──────────────────┬──────────────────┬──────────────────────────────┤
│ ★ coordinator    │ ◆ researcher    │ ⚙ worker-1                   │
│ ─────────────── │ ─────────────── │ ─────────────────────────── │
│ Analyzing auth  │ grep "validate" │ $ cargo test                  │
│ module...       │ in src/...      │ ✓ exit: 0                     │
│                 │                 │ 42 passed                      │
│ status: running │ status: done    │ status: done                  │
├──────────────────┼──────────────────┤                              │
│ ⚙ worker-2     │ (+1 more)       │                              │
│ ─────────────── │                  │                              │
│ Reading file... │                  │                              │
│ status: idle    │                  │                              │
└──────────────────┴──────────────────┴──────────────────────────────┘

  ↑ grid layout with agent status tiles
  ↑ each tile shows: role icon, name, current activity, status
  ↑ status accent colors: spawned=gray, running=green, thinking=purple,
    blocked=yellow, failed=red, completed=blue
  ↑ overflow strip "+N more agents"
```

---

## 25. Theme Switching

```
Theme applied:

┌─────────────────────────────────────────────────────────────────────┐
│ [catppuccin-mocha]  sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto    │
├─────────────────────────────────────────────────────────────────────┤
│ (all colors change based on theme)                                 │
└─────────────────────────────────────────────────────────────────────┘

Theme cycling (Ctrl+Shift+T or /theme):

┌─────────────────────────────────────────────────────────────────────┐
│ Theme: [catppuccin-mocha] ▸ catppuccin-latte  ▸ dracula  ▸ nord    │
│        ▸ gruvbox  ▸ tokyonight  ▸ rosepine  ▸ solarized            │
└─────────────────────────────────────────────────────────────────────┘
```

### Available Themes

```
Dark themes:
  catppuccin-mocha    (default)
  dracula
  gruvbox-dark
  nord
  tokyonight-storm
  rosepine-moon
  solarized-dark
  monokai-pro

Light themes:
  catppuccin-latte
  gruvbox-light
  solarized-light

ANSI themes (for limited terminals):
  ansi-dark
  ansi-light
```

---

## 26. Error State

```
Connection error:

┌─────────────────────────────────────────────────────────────────────┐
│ ✗ Connection lost                                                  │
│   Reconnecting in 3s... (attempt 2/5)                             │
│                                                                    │
│ ▌                                                                  │
└─────────────────────────────────────────────────────────────────────┘

API error:

┌─────────────────────────────────────────────────────────────────────┐
│ ✗ API Error: Rate limited                                          │
│   Retry after 30s or switch model with Ctrl+M                     │
│                                                                    │
│ ▌                                                                  │
└─────────────────────────────────────────────────────────────────────┘

Permission denied:

┌─────────────────────────────────────────────────────────────────────┐
│ ✗ Permission denied: Bash tool not allowed in plan mode            │
│   Switch to auto mode with Ctrl+Shift+M                           │
│                                                                    │
│ ▌                                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 27. Splash / Empty State

```
First launch (no messages):

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                                                                     │
│                        jcode                                        │
│                      v0.1.0                                          │
│                                                                     │
│              "What can I help you with?"                             │
│                                                                     │
│                                                                     │
│ ▌                                                                  │
│                                                                    │
├─────────────────────────────────────────────────────────────────────┤
│ Tab:autocomplete  Ctrl+X:leader  Ctrl+O:transcript  /:commands    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 28. Onboarding Flow

```
Step 1 — Welcome:

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                    Welcome to jcode                                  │
│                                                                     │
│              Let's set up your environment.                         │
│                                                                     │
│              [Press Enter to continue]                              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Step 2 — Model Selection:

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│              Select your preferred model:                           │
│                                                                     │
│         ▸ claude-sonnet-4-20250514 (fast, recommended)             │
│           claude-opus-4-20250514 (most capable)                     │
│           claude-haiku-4-5-20251001 (cheapest)                      │
│                                                                     │
│              ↑/↓ to navigate  Enter to select                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Step 3 — API Key:

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│              Enter your Anthropic API key:                          │
│                                                                     │
│              sk-ant-api03-••••••••••••••••••••                      │
│                                                                     │
│              [Enter to confirm]  [Esc to skip]                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Appendix A: Color Palette (Codex Adaptive)

```
Semantic Colors (mapped to theme):

  accent        = #cba6f7  (purple)     tool_bash    = #89b4fa  (blue)
  success       = #a6e3a1  (green)      tool_edit    = #a6e3a1  (green)
  error         = #f38ba8  (red)        tool_read    = #89dceb  (cyan)
  warning       = #fab387  (orange)     tool_glob    = #f9e2af  (yellow)
  info          = #89dceb  (cyan)       tool_grep    = #f9e2af  (yellow)
  text          = #cdd6f4  (white)      tool_agent   = #cba6f7  (purple)
  text_muted    = #a6adc8  (gray)
  text_subtle   = #585b70  (dark gray)

Diff Colors:
  diff_added        = #a6e3a1  (green)
  diff_removed      = #f38ba8  (red)
  diff_added_dim   = #1e292a  (dark green bg)
  diff_removed_dim = #2d1a1e  (dark red bg)

Theme Color Resolution:
  1. Theme defines semantic color as RGB
  2. Runtime queries terminal color level (TrueColor/256/16)
  3. RGB is quantized via CIE76 nearest-match to displayable palette
  4. Falls back gracefully (TrueColor → ANSI256 → ANSI16 → Mono)
```

---

## Appendix B: Keybinding Defaults (OpenCode-style)

```
Leader key: Ctrl+X (timeout: 2000ms)

Global:
  Ctrl+C          interrupt (double-press to confirm)
  Ctrl+D          exit (double-press to confirm)
  Ctrl+L          redraw
  Ctrl+O          toggle transcript overlay
  Ctrl+T          toggle todos
  Ctrl+M          toggle mermaid side panel
  Ctrl+/          toggle help
  Ctrl+Alt+K      which-key panel
  Ctrl+Shift+T    cycle theme

Chat (when composer focused):
  Enter           submit
  Tab             queue message
  Up/Down         history navigation
  Ctrl+R          reverse search
  Ctrl+G          external editor
  Ctrl+S          stash input
  Shift+Up        message actions
  Ctrl+Enter      submit with context

Leader sequences:
  Ctrl+X, N       new session
  Ctrl+X, O       open transcript
  Ctrl+X, T       toggle todos
  Ctrl+X, R       resume session
  Ctrl+X, S       save session
  Ctrl+X, M       switch model
  Ctrl+X, 1-9     quick switch session slot

Approval dialog:
  Y               allow once
  Shift+Y         allow for session
  N               deny
  Esc             abort
  Left/Right      navigate choices
  Ctrl+D          debug info toggle
  Ctrl+E          explanation toggle

Transcript overlay:
  Up/Down (k/j)   scroll
  PageUp/PageDown  page scroll
  G               bottom
  g               top
  /               search
  Ctrl+R          reverse search
  q/Esc           close
```

---

## Appendix C: ASCII Component Reference

```
┌─ Tool ───────────────────┐   Tool call box with colored left border
│                          │
└──────────────────────────┘

╭─ Tool ───────────────────╮   Rounded variant (if terminal supports)
│                          │
╰──────────────────────────╯

─── divider text ──────    Horizontal divider with centered text

▸ selected item            Selection cursor
  unselected item

✓ success                  Success indicator
✗ failure                  Failure indicator
★ create                   Create indicator
→ update                   Update indicator

⠋ spinner                 Animated braille spinner
⚙  tool                    Tool indicator
🔱 agent                   Agent/spawn indicator
📤 delegate                Delegation indicator
📥 receive                 Receive indicator
◆  grep                    Grep indicator
☆  glob                    Glob indicator
💰 cost                    Cost indicator
🔐 permission              Permission indicator
⏳ waiting                 Waiting indicator
⚡ hook                    Hook indicator
🔮 speculation             Speculation indicator
💤 sleep                   Sleep indicator
✨ brief                   Brief generate indicator

▌ cursor                   Input cursor (blinks)
│  left border             Message left border
```
