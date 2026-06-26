# MASTER_UI.md — jcode TUI Redesign Specification
> Full UI/UX spec with ASCII mockups + code for every feature
> Based on research: Claude Code + OpenCode + Codex
> Stack: Rust + ratatui + crossterm

### ⚠️ Layout Correction (verified from source code)

```
WRONG (my initial mistake):          CORRECT (Claude Code actual):
┌──────────────────────┐            ┌──────────────────────┐
│ STATUS BAR (top)     │ ← WRONG   │ Messages             │
│ Messages             │            │ ...                  │
│ Spinner              │            │ ⠋ Thinking...        │ ← Spinner in scrollbox
│ Input                │            │ (spacer pushes it    │
│ Hints                │            │  to bottom)          │
└──────────────────────┘            ├──────────────────────┤
                                    │ ▌ Input              │
                                    ├──────────────────────┤
                                    │ Status line (bottom) │ ← Below input
                                    │ Hints                │
                                    └──────────────────────┘
```

**Key facts from Claude Code source (`FullscreenLayout.tsx`, `REPL.tsx`, `PromptInputFooter.tsx`):**
1. **Spinner is INSIDE ScrollBox** — it scrolls with conversation (REPL.tsx:5950)
2. **StatusLine is BELOW input** — inside PromptInputFooter (PromptInputFooter.tsx:159)
3. **Bottom slot max 50%** — input area can't exceed half terminal (FullscreenLayout.tsx:393)
4. **NewMessagesPill overlays** — absolute positioned at bottom of scroll area

---

## Table of Contents

1. [Overall Layout](#1-overall-layout)
2. [Status Line](#2-status-line)
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
29. [Sub-Agent Delegation Flow](#29-sub-agent-delegation-flow)
30. [Shell / Interactive Terminal](#30-shell--interactive-terminal)
31. [Agent Team / Coordination UI](#31-agent-team--coordination-ui)
32. [Background Tasks / Progress Panel](#32-background-tasks--progress-panel)
33. [Usage / Cost Overlay](#33-usage--cost-overlay)
34. [Copy Selection Mode](#34-copy-selection-mode)
35. [Workspace Map (Niri-style)](#35-workspace-map-niri-style)
36. [Toast Notifications](#36-toast-notifications)
37. [Model Picker](#37-model-picker)
38. [Todos / Task Management Panel](#38-todos--task-management-panel)
39. [File Tree Sidebar](#39-file-tree-sidebar)
40. [Configurator / Settings Dialog](#40-configurator--settings-dialog)
41. [Plugin Manager](#41-plugin-manager)
42. [Git Info Widget](#42-git-info-widget)
43. [Changelog Dialog](#43-changelog-dialog)
44. [Account Picker](#44-account-picker)
45. [Notification Center](#45-notification-center)
46. [Memory Tiles](#46-memory-tiles)
47. [Timeline / Session History](#47-timeline--session-history)
48. [Experiment Popup](#48-experiment-popup)
49. [Side Conversations](#49-side-conversations--fork-threads)
50. [Backtrack / Rollback](#50-backtrack--undo-rollback)
51. [Request User Input](#51-request-user-input-overlay)
52. [@-Mentions Popup](#52--mentions-popup)
53. [Plan Mode](#53-plan-mode)
54. [Goal / Task Tracking](#54-goal--task-tracking)
55. [Turn Metrics / Separator](#55-turn-metrics--worked-for-separator)
56. [Keypress Debug Inspector](#56-keypress-debug-inspector)
57. [Service Tier Selection](#57-service-tier-selection)
58. [Raw Output / Accessibility](#58-raw-output-mode--accessibility)
59. [Terminal Pets](#59-terminal-pets)
60. [Collaboration Modes](#60-collaboration-modes)
61. [Reasoning Effort Picker](#61-reasoning-effort-picker)
62. [Interactive Keybinding Editor](#62-interactive-keybinding-editor)
63. [Copy Agent Response](#63-copy-agent-response-copy)
64. [Image Paste](#64-image-paste-ctrlaltv)
65. [Terminal Title Configuration](#65-terminal-title-configuration-title)
66. [Auto-Review Denials](#66-auto-review-denials-approve)
67. [Desktop Notifications](#67-desktop-notifications)
68. [Code Review Setup](#68-code-review-setup-review)
69. [Model Migration Dialog](#69-model-migration-dialog)
70. [Personality Picker](#70-personality-picker)
71. [IDE Context Integration](#71-ide-context-integration-ide)
72. [Plan Mode Nudge](#72-plan-mode-nudge)
73. [Safety Buffering Status](#73-safety-buffering-status)
## Appendix D: [Per-Tool UI Matrix](#appendix-d-per-tool-ui-matrix)
## Appendix E: [Edge Cases & Error Handling](#appendix-e-edge-cases--error-handling)
## Appendix F: [Animation Reference](#appendix-f-animation-reference)
## Appendix G: [Complete Feature Inventory](#appendix-g-complete-feature-inventory)
## Appendix H: [Codex Missing Features](#appendix-h-codex-missing-features-summary)

---

## 1. Overall Layout

The main TUI layout follows Claude Code's exact structure.
**Key insight:** Spinner is INSIDE the scrollable area (scrolls with conversation).
Status line is BELOW the input box (inside PromptInputFooter).

```
┌─────────────────────────────────────────────────────────────────────┐
│ ┌─ User ──────────────────────────────────────────────────────────┐ │
│ │ > Fix the bug in auth.rs                                        │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─ Assistant ─────────────────────────────────────────────────────┐ │
│ │ I'll look at the auth module to find the bug.                  │ │
│ │                                                                │ │
│ │ ┌─ Bash ─────────────────────────────────────────────────────┐ │ │
│ │ │ $ grep -n "validate" src/auth.rs                           │ │ │
│ │ │ ✓ exit: 0                                                  │ │ │
│ │ │   12: fn validate_token(token: &str) -> bool {             │ │ │
│ │ │   45:   if !validate_expiry(expiry) {                      │ │ │
│ │ └────────────────────────────────────────────────────────────┘ │ │
│ │                                                                │ │
│ │ ┌─ Edit ─────────────────────────────────────────────────────┐ │ │
│ │ │ → Update src/auth.rs                                       │ │ │
│ │ │   -   if !validate_expiry(expiry) {                        │ │ │
│ │ │   +   if !validate_expiry(expiry, now) {                   │ │ │
│ │ └────────────────────────────────────────────────────────────┘ │ │
│ │                                                                │ │
│ │ Fixed the bug — `validate_expiry` was missing the current      │ │
│ │ time parameter.                                                │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ⠋ Thinking...                          ← SPINNER (inside scrollbox)│
│                                                     ┌────────────┐│
│                              ┌──────────────────────│ 3 new msgs ││← Pill (overlay)
│──────────────────────────────┴──────────────────────┴────────────┘│
├─────────────────────────────────────────────────────────────────────┤
│ ▌  (input area — grows with content, max 50% of terminal)          │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto   ← STATUS LINE        │
│ Tab:autocomplete  Ctrl+X:leader  Ctrl+O:transcript  /:commands     │ ← Hints
└─────────────────────────────────────────────────────────────────────┘
```

### Vertical Stack Order (exact Claude Code structure)

```
Position  Content                     Scrolls?    Notes
─────────────────────────────────────────────────────────────
[1]       StickyPromptHeader          No          Only when scrolled up (1 row)
[2]       ScrollBox                  YES         Messages + Spinner inside
          ├─ Messages (VirtualMessageList)        Chat transcript
          ├─ Spacer (flexGrow=1)                  Pushes spinner to bottom
          └─ SpinnerWithVerb                      Animated spinner
[3]       NewMessagesPill             No          Absolute overlay at bottom
[4]       SuggestionsOverlay          No          Slash-command autocomplete
[5]       Bottom slot (max 50%)      No          Fixed at terminal bottom
          ├─ QueuedCommands                       If message is queued
          ├─ PermissionDialog                     If permission pending
          ├─ PromptInput                          Input box
          │   ├─ Mode indicator                   Permission mode icon
          │   ├─ TextInput                        The actual text input
          │   ├─ Border (round)                   With model name
          │   └─ PromptInputFooter                Below input
          │       ├─ StatusLine                   Model/ctx/cost/cache
          │       └─ Hints bar                    Keyboard hints
          └─ BackgroundAgentSelector              If bg agent active
[6]       Modal                       No          Slash-command dialogs
```

### Layout Types (Rust) — corrected

```rust
// crates/jcode-tui/src/layout.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout computation — matches Claude Code's FullscreenLayout exactly.
///
/// Key differences from naive layout:
/// 1. Spinner is INSIDE the scrollable area (part of ScrollBox content)
/// 2. Status line is BELOW the input (inside PromptInputFooter)
/// 3. Bottom slot is capped at 50% of terminal height
/// 4. NewMessagesPill overlays the bottom of the scroll area
pub fn compute_main_layout(area: Rect) -> MainLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),              // ScrollBox (messages + spinner inside)
            Constraint::Length(3),            // Input area (composer)
            Constraint::Length(1),            // Status line (inside footer)
        ])
        .split(area);

    MainLayout {
        viewport: chunks[0],     // Scrollable: messages + spinner
        composer: chunks[1],     // Fixed: input box
        footer: chunks[2],       // Fixed: status line + hints
    }
}

pub struct MainLayout {
    /// Scrollable area containing messages AND spinner.
    /// Spinner sits at the bottom of this area (inside scroll content).
    pub viewport: Rect,

    /// Fixed input area (PromptInput).
    /// Capped at 50% of terminal height in Claude Code.
    pub composer: Rect,

    /// Status line + hints, BELOW the input.
    /// NOT at the top of the screen.
    pub footer: Rect,
}

/// Compute the scrollable content area within the viewport.
/// This is where messages + spinner live.
pub fn compute_scroll_content(viewport: Rect) -> Rect {
    // The entire viewport is scrollable
    // Spinner is rendered at the bottom of the scroll content
    viewport
}

/// Compute where the spinner should appear within scroll content.
/// In Claude Code, spinner is pushed to the bottom by a flexGrow=1 spacer.
/// In ratatui, we compute this based on total content height.
pub fn compute_spinner_position(
    viewport: Rect,
    total_content_height: u16,
) -> Option<Rect> {
    // If content doesn't fill the viewport, spinner is at the bottom
    // If content overflows, spinner is below the last message
    if total_content_height < viewport.height {
        // Content doesn't fill viewport — spinner at bottom
        Some(Rect {
            x: viewport.x,
            y: viewport.y + total_content_height,
            width: viewport.width,
            height: 1,
        })
    } else {
        // Content overflows — spinner is below visible area (user scrolls to see it)
        Some(Rect {
            x: viewport.x,
            y: viewport.y + viewport.height, // just below visible
            width: viewport.width,
            height: 1,
        })
    }
}

/// With side panel (optional, toggled):
///
/// ┌────────────────────────────┬──────────────┐
/// │ Main Column                │ Side Panel   │
/// │ (viewport+composer+footer) │ (pinned/     │
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

## 2. Status Line

**Position: BELOW the input box** (inside PromptInputFooter, NOT at the top of the screen).

This is the actual Claude Code layout:

```
┌─────────────────────────────────────────────────────────────────────┐
│ ▌ Fix the bug in auth.rs                                          │ ← Input box
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto                        │ ← Status line
│ Tab:autocomplete  Ctrl+X:leader  Ctrl+O:transcript  /:commands    │ ← Hints
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
// crates/jcode-tui/src/status_line.rs
//
// NOTE: This is the STATUS LINE, positioned BELOW the input box.
// It is part of PromptInputFooter (Claude Code: PromptInputFooter.tsx:159).
// NOT at the top of the screen.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct StatusLineState {
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

/// Render the status line — positioned BELOW the input box.
/// Part of PromptInputFooter.
pub fn render_status_line(state: &StatusLineState, theme: &Theme, area: Rect, buf: &mut Buffer) {
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

### Code

```rust
// crates/jcode-tui/src/bottom_pane/permission_dialog/edit_permission.rs

use crate::bottom_pane::{BottomPaneResult, BottomPaneView};
use jcode_tui_core::keymap::KeyCombo;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct EditPermissionDialog {
    file_path: String,
    diff_lines: Vec<DiffLine>,
    selected: usize,
    choices: Vec<(&'static str, &'static str)>,
    show_debug: bool,
}

struct DiffLine { line_num: Option<u32>, sign: char, content: String }

impl EditPermissionDialog {
    pub fn new(file_path: String, diff: String) -> Self {
        let diff_lines: Vec<DiffLine> = diff.lines().map(|l| {
            let (sign, content) = match l.chars().next() {
                Some('+') => ('+', l[1..].to_string()),
                Some('-') => ('-', l[1..].to_string()),
                _ => (' ', l.to_string()),
            };
            DiffLine { line_num: None, sign, content }
        }).collect();
        Self { file_path, diff_lines, selected: 0, choices: vec![("y","Allow"),("a","Always Allow"),("n","Deny")], show_debug: false }
    }
}

impl BottomPaneView for EditPermissionDialog {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        buf.set_line(area.x, y, &Line::from(vec![
            Span::styled("🔐 ", Style::default().fg(theme.warning.into())),
            Span::styled("Permission required", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD)),
        ]), area.width); y += 2;

        buf.set_line(area.x, y, &Line::from(vec![Span::styled("Edit wants to modify:", Style::default().fg(theme.text_muted.into()))]), area.width); y += 1;
        buf.set_line(area.x + 2, y, &Line::from(vec![
            Span::styled("→ ", Style::default().fg(theme.tool_edit.into())),
            Span::styled(&self.file_path, Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD)),
        ]), area.width - 4); y += 2;

        for dl in self.diff_lines.iter().take(6) {
            if y >= area.y + area.height - 4 { break; }
            let color = match dl.sign { '+' => theme.diff_added, '-' => theme.diff_removed, _ => theme.text_muted };
            let prefix = if dl.sign == ' ' { " " } else { &dl.sign.to_string() };
            buf.set_line(area.x + 2, y, &Line::from(vec![
                Span::styled(format!(" {}{}", prefix, dl.content), Style::default().fg(color.into())),
            ]), area.width - 4); y += 1;
        }
        if self.diff_lines.len() > 6 {
            buf.set_line(area.x + 2, y, &Line::from(vec![
                Span::styled(format!("... {} more diff lines", self.diff_lines.len() - 6), Style::default().fg(theme.text_subtle.into())),
            ]), area.width - 4); y += 1;
        }
        y += 1;

        let mut spans = Vec::new();
        for (i, (key, label)) in self.choices.iter().enumerate() {
            let style = if i == self.selected { Style::default().fg(theme.background.into()).bg(theme.accent.into()).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text_muted.into()) };
            spans.push(Span::styled(format!(" [{}] {} ", key, label), style));
        }
        buf.set_line(area.x, y, &Line::from(spans), area.width);
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match key.key.as_str() { "y" => { self.selected = 0; true } "a" => { self.selected = 1; true } "n" => { self.selected = 2; true } "d" if key.ctrl => { self.show_debug = !self.show_debug; true } "left" | "h" => { self.selected = (self.selected + 2) % 3; true } "right" | "l" => { self.selected = (self.selected + 1) % 3; true } _ => false }
    }
    fn completion(&self) -> Option<BottomPaneResult> { Some(BottomPaneResult::Approved { choice: ["allow","allow_always","deny"][self.selected].into() }) }
    fn handle_ctrl_c(&mut self) -> BottomPaneResult { BottomPaneResult::Cancelled }
    fn view_id(&self) -> &str { "permission_edit" }
}
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

### Code — ReadPermissionDialog

```rust
// crates/jcode-tui/src/bottom_pane/permission_dialog/read_permission.rs

use crate::bottom_pane::{BottomPaneResult, BottomPaneView};
use jcode_tui_core::keymap::KeyCombo;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct ReadPermissionDialog {
    file_path: String,
    selected: usize,
    choices: Vec<(&'static str, &'static str)>,
}

impl ReadPermissionDialog {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            selected: 0,
            choices: vec![("y", "Allow"), ("a", "Always Allow"), ("n", "Deny")],
        }
    }
}

impl BottomPaneView for ReadPermissionDialog {
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

        // Description
        buf.set_line(area.x, y, &Line::from(vec![
            Span::styled("Read wants to access:", Style::default().fg(theme.text_muted.into())),
        ]), area.width);
        y += 1;

        // File path
        let path_line = Line::from(vec![
            Span::styled("→ ", Style::default().fg(theme.tool_read.into())),
            Span::styled(&self.file_path, Style::default().fg(theme.text.into())
                .add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(area.x + 2, y, &path_line, area.width.saturating_sub(4));
        y += 2;

        // Choices
        let mut spans = Vec::new();
        for (i, (key, label)) in self.choices.iter().enumerate() {
            let style = if i == self.selected {
                Style::default().fg(theme.background.into()).bg(theme.accent.into())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_muted.into())
            };
            spans.push(Span::styled(format!(" [{}] {} ", key, label), style));
        }
        buf.set_line(area.x, y, &Line::from(spans), area.width);
    }

    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        match key.key.as_str() {
            "y" => { self.selected = 0; true }
            "a" => { self.selected = 1; true }
            "n" => { self.selected = 2; true }
            "left" | "h" => {
                self.selected = (self.selected + 2) % 3;
                true
            }
            "right" | "l" => {
                self.selected = (self.selected + 1) % 3;
                true
            }
            _ => false,
        }
    }

    fn completion(&self) -> Option<BottomPaneResult> {
        Some(BottomPaneResult::Approved {
            choice: match self.selected {
                0 => "allow".into(),
                1 => "allow_always".into(),
                _ => "deny".into(),
            },
        })
    }

    fn handle_ctrl_c(&mut self) -> BottomPaneResult { BottomPaneResult::Cancelled }
    fn view_id(&self) -> &str { "permission_read" }
}
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

**Position: INSIDE the ScrollBox** (scrolls with conversation).
Pushed to bottom by a spacer when content doesn't fill the viewport.

```
Idle:
(empty)

Thinking (inside scrollbox, pushed to bottom):
┌─────────────────────────────────────────────────────────────────────┐
│ > Fix the bug in auth.rs                                          │
│                                                                    │
│ I'll look at the auth module...                                    │
│                                                                    │
│                                                                    │
│ ⠋ Thinking...                       ← spinner at bottom of scroll  │
└─────────────────────────────────────────────────────────────────────┘

Tool running:
│ ⠋ Running Bash...                                                  │

Agent spawning:
│ 🔱 Spawning sub-agent...                                           │

Waiting for network:
│ ⏳ Waiting for network...                                          │

Permission pending:
│ 🔐 Awaiting permission...                                          │

Streaming:
│ ⠋ Streaming response...                                            │

Compact (during tool output):
│ ·                                                                   │

Hook executing:
│ ⚡ Running hook...                                                  │

Agent delegating:
│ 📤 Delegating...                                                   │

Brief (quick operations):
│ ✨                                                                  │

Speculation:
│ 🔮 Speculating...                                                  │
```

### Code

```rust
// crates/jcode-tui-style/src/spinner.rs
//
// NOTE: The spinner is rendered INSIDE the ScrollBox (scrollable area).
// It sits at the very bottom of the scroll content, pushed there by
// a flexGrow=1 spacer. It SCROLLS with the conversation.
//
// Claude Code source: REPL.tsx:5950 — SpinnerWithVerb is inside ScrollBox.
// This is different from a fixed spinner at the bottom of the screen.

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

**Position: Below the status line** (at the very bottom of the screen).
Part of PromptInputFooter.

```
┌─────────────────────────────────────────────────────────────────────┐
│ ▌ Fix the bug in auth.rs                                          │ ← Input
├─────────────────────────────────────────────────────────────────────┤
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto                        │ ← Status
│ Tab:autocomplete  Ctrl+X:leader  Ctrl+O:transcript  /:commands    │ ← Hints
└─────────────────────────────────────────────────────────────────────┘
                ↑ Status + Hints are BOTH below the input

Narrow terminal (progressive collapse):

┌─────────────────────────────────────────────────────────────┐
│ sonnet-4  42%  $0.12  auto                                  │
│ Ctrl+O:transcript  /:commands                               │
└─────────────────────────────────────────────────────────────┘

Leader key pressed:

┌─────────────────────────────────────────────────────────────────────┐
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto                        │
│ n:new session  o:transcript  t:todos  r:search  ...               │
└─────────────────────────────────────────────────────────────────────┘

Queue active:

┌─────────────────────────────────────────────────────────────────────┐
│ [1 queued] Tab:send next  Enter:submit                            │
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto                        │
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

### Code

```rust
// crates/jcode-tui/src/ui/mermaid_pane.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct MermaidPane {
    pub svg_data: Option<Vec<u8>>,
    pub zoom: f64,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub loading: bool,
    pub width: u16,
    pub height: u16,
}

impl MermaidPane {
    pub fn new() -> Self {
        Self { svg_data: None, zoom: 1.0, scroll_x: 0, scroll_y: 0, loading: false, width: 0, height: 0 }
    }

    pub fn set_svg(&mut self, data: Vec<u8>) {
        self.svg_data = Some(data);
        self.loading = false;
    }

    pub fn zoom_in(&mut self) { self.zoom = (self.zoom * 1.2).min(5.0); }
    pub fn zoom_out(&mut self) { self.zoom = (self.zoom / 1.2).max(0.2); }
    pub fn pan(&mut self, dx: i32, dy: i32) { self.scroll_x += dx; self.scroll_y += dy; }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        // Title
        buf.set_line(area.x + 1, y, &Line::from(vec![
            Span::styled("Mermaid Diagram", Style::default().fg(theme.accent.into()).add_modifier(Modifier::BOLD)),
        ]), area.width.saturating_sub(2));
        y += 2;

        if self.loading {
            buf.set_line(area.x + 1, y, &Line::from(vec![
                Span::styled("⠋ Rendering diagram...", Style::default().fg(theme.spinner.into())),
            ]), area.width.saturating_sub(2));
            return;
        }

        if self.svg_data.is_none() {
            buf.set_line(area.x + 1, y, &Line::from(vec![
                Span::styled("No diagram. Use /mermaid to create one.", Style::default().fg(theme.text_subtle.into())),
            ]), area.width.saturating_sub(2));
            return;
        }

        // Render SVG placeholder
        buf.set_line(area.x + 1, y, &Line::from(vec![
            Span::styled("[Diagram rendered - use Zoom/Pan to navigate]", Style::default().fg(theme.text_muted.into())),
        ]), area.width.saturating_sub(2));

        // Zoom controls
        buf.set_line(area.x, area.y + area.height - 1, &Line::from(vec![
            Span::styled("[+/-] zoom  ←/→ pan  [Esc] close", Style::default().fg(theme.text_subtle.into())),
        ]), area.width);
    }
}
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

### Code

```rust
// crates/jcode-tui-render/src/swarm_gallery.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub enum AgentRole { Coordinator, Researcher, Worker, SearchAgent, Reviewer }

impl AgentRole {
    pub fn glyph(&self) -> &'static str {
        match self { Self::Coordinator => "★", Self::Researcher => "◆", Self::Worker => "⚙", Self::SearchAgent => "☆", Self::Reviewer => "◇" }
    }
}

pub enum AgentLifecycle { Spawning, Running, Thinking, Blocked, Failed(String), Completed, Stopped }

impl AgentLifecycle {
    pub fn color(&self, theme: &Theme) -> ratatui::style::Color {
        match self { Self::Running => theme.success.into(), Self::Thinking => theme.accent.into(),
            Self::Blocked => theme.warning.into(), Self::Failed(_) => theme.error.into(),
            Self::Completed => theme.info.into(), _ => theme.text_subtle.into() }
    }
    pub fn dot(&self) -> &'static str { "●" }
}

pub struct AgentTile {
    pub role: AgentRole, pub name: String, pub status: AgentLifecycle,
    pub task: String, pub stats: String,
}

pub struct SwarmGallery {
    pub agents: Vec<AgentTile>, pub total: usize, pub active: usize,
}

impl SwarmGallery {
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        // Header
        buf.set_line(area.x, y, &Line::from(vec![
            Span::styled("⋯ ", Style::default().fg(theme.accent.into())),
            Span::styled(format!("swarm · {} agents · {} active", self.total, self.active), Style::default().fg(theme.text.into())),
        ]), area.width);
        y += 1;

        // Grid
        let cols = (self.agents.len() as f64).sqrt().ceil() as u16;
        let rows = (self.agents.len() as u16 + cols - 1) / cols;
        let tw = area.width / cols.max(1);
        let th = (area.height - 1) / rows.max(1);

        for (i, a) in self.agents.iter().enumerate() {
            let cx = area.x + (i as u16 % cols) * tw;
            let cy = y + (i as u16 / cols) * th;
            let color = a.status.color(theme);
            buf.set_line(cx + 1, cy, &Line::from(vec![
                Span::styled(format!("{} {} ", a.role.glyph(), a.name), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ]), tw.saturating_sub(2));
            if th > 1 {
                buf.set_line(cx + 1, cy + 1, &Line::from(vec![
                    Span::styled(&a.task, Style::default().fg(theme.text_muted.into())),
                ]), tw.saturating_sub(2));
            }
            if th > 2 {
                buf.set_line(cx + 1, cy + 2, &Line::from(vec![
                    Span::styled(a.status.dot(), Style::default().fg(color)),
                    Span::styled(format!(" {}", a.stats), Style::default().fg(theme.text_subtle.into())),
                ]), tw.saturating_sub(2));
            }
        }
    }
}
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

### Code

```rust
// crates/jcode-tui-style/src/theme_names.rs

pub static THEMES: &[(&str, &str, &str)] = &[
    ("catppuccin-mocha", "Dark — default", "#1e1e2e"),
    ("catppuccin-latte", "Light", "#eff1f5"),
    ("dracula", "Dark", "#282a36"),
    ("gruvbox-dark", "Dark", "#282828"),
    ("gruvbox-light", "Light", "#fbf1c7"),
    ("nord", "Dark", "#2e3440"),
    ("tokyonight-storm", "Dark", "#24283b"),
    ("rosepine-moon", "Dark", "#232136"),
    ("solarized-dark", "Dark", "#002b36"),
    ("solarized-light", "Light", "#fdf6e3"),
    ("monokai-pro", "Dark", "#2d2a2e"),
    ("ansi-dark", "16-color dark", "#000000"),
    ("ansi-light", "16-color light", "#ffffff"),
];

pub fn next_theme(current: &str) -> &'static str {
    let idx = THEMES.iter().position(|(n, _, _)| *n == current).unwrap_or(0);
    THEMES[(idx + 1) % THEMES.len()].0
}

pub fn prev_theme(current: &str) -> &'static str {
    let idx = THEMES.iter().position(|(n, _, _)| *n == current).unwrap_or(0);
    THEMES[(idx + THEMES.len() - 1) % THEMES.len()].0
}

pub fn theme_description(name: &str) -> &str {
    THEMES.iter().find(|(n, _, _)| *n == name).map(|(_, d, _)| *d).unwrap_or("Unknown")
}

pub fn render_theme_picker(current: &str, area: Rect, buf: &mut Buffer, theme: &Theme) {
    let mut y = area.y;
    buf.set_line(area.x, y, &Line::from(vec![
        Span::styled("Theme: ", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD)),
    ]), area.width);
    y += 1;

    for (name, desc, _) in THEMES {
        if y >= area.y + area.height { break; }
        let is_current = *name == current;
        let prefix = if is_current { "▸ " } else { "  " };
        let style = if is_current { Style::default().fg(theme.accent.into()).add_modifier(Modifier::BOLD) }
                    else { Style::default().fg(theme.text_muted.into()) };
        buf.set_line(area.x, y, &Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(name, style),
            Span::styled(format!("  {}", desc), Style::default().fg(theme.text_subtle.into())),
        ]), area.width);
        y += 1;
    }
}
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

### Code

```rust
// crates/jcode-tui/src/history_cell/error_cell.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;
use crate::history_cell::HistoryCell;

pub struct ErrorStateCell {
    pub title: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub retry_available: bool,
}

impl HistoryCell for ErrorStateCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;

        let header = Line::from(vec![
            Span::styled("✗ ", Style::default().fg(theme.error.into())),
            Span::styled(&self.title, Style::default().fg(theme.error.into()).add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(area.x, y, &header, area.width);
        y += 1;

        buf.set_line(area.x, y, &Line::from(vec![
            Span::styled(&self.message, Style::default().fg(theme.text_muted.into())),
        ]), area.width);
        y += 1;

        if let Some(sug) = &self.suggestion {
            buf.set_line(area.x, y, &Line::from(vec![
                Span::styled("💡 ", Style::default().fg(theme.info.into())),
                Span::styled(sug.as_str(), Style::default().fg(theme.info.into())),
            ]), area.width);
        }

        if self.retry_available {
            buf.set_line(area.x, y + 1, &Line::from(vec![
                Span::styled("[r] retry  [Esc] dismiss", Style::default().fg(theme.text_subtle.into())),
            ]), area.width);
        }
    }

    fn desired_height(&self, _: u16) -> u16 {
        2 + if self.suggestion.is_some() { 1 } else { 0 } + if self.retry_available { 1 } else { 0 }
    }
}
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

### Code

```rust
// crates/jcode-tui/src/ui/splash.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub fn render_splash(area: Rect, buf: &mut Buffer, theme: &Theme, version: &str) {
    let cx = area.x + area.width / 2;

    // Logo
    let logo = Line::from(vec![
        Span::styled("jcode", Style::default().fg(theme.accent.into()).add_modifier(Modifier::BOLD)),
    ]);
    buf.set_line(cx.saturating_sub(3), area.y + 4, &logo, 10);

    // Version
    buf.set_line(cx.saturating_sub(4), area.y + 5, &Line::from(vec![
        Span::styled(version, Style::default().fg(theme.text_subtle.into())),
    ]), 10);

    // Tagline
    buf.set_line(cx.saturating_sub(15), area.y + 7, &Line::from(vec![
        Span::styled(""What can I help you with?"", Style::default().fg(theme.text_muted.into()).add_modifier(Modifier::ITALIC)),
    ]), 32);

    // Cursor
    buf.set_string(cx.saturating_sub(1), area.y + 9, "▌", Style::default().fg(theme.accent.into()));
}
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

### Code

```rust
// crates/jcode-tui/src/ui/onboarding.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub enum OnboardingStep {
    Welcome,
    ModelSelection { selected: usize, models: Vec<String> },
    ApiKeyInput { input: String, masked: bool },
}

pub struct OnboardingFlow {
    pub step: OnboardingStep,
    pub completed: bool,
}

impl OnboardingFlow {
    pub fn new() -> Self { Self { step: OnboardingStep::Welcome, completed: false } }

    pub fn advance(&mut self) {
        self.step = match &self.step {
            OnboardingStep::Welcome => OnboardingStep::ModelSelection { selected: 0, models: vec!["claude-sonnet-4-20250514 (fast)".into(), "claude-opus-4-20250514 (capable)".into(), "claude-haiku-4-5-20251001 (cheap)".into()] },
            OnboardingStep::ModelSelection { .. } => OnboardingStep::ApiKeyInput { input: String::new(), masked: true },
            OnboardingStep::ApiKeyInput { .. } => { self.completed = true; return; },
        };
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        match &self.step {
            OnboardingStep::Welcome => {
                buf.set_line(area.x, area.y + 3, &Line::from(vec![
                    Span::styled("Welcome to jcode", Style::default().fg(theme.accent.into()).add_modifier(Modifier::BOLD)),
                ]), area.width);
                buf.set_line(area.x, area.y + 5, &Line::from(vec![
                    Span::styled("Let's set up your environment.", Style::default().fg(theme.text_muted.into())),
                ]), area.width);
                buf.set_line(area.x, area.y + 7, &Line::from(vec![
                    Span::styled("[Press Enter to continue]", Style::default().fg(theme.text_subtle.into())),
                ]), area.width);
            }
            OnboardingStep::ModelSelection { selected, models } => {
                buf.set_line(area.x, area.y + 2, &Line::from(vec![
                    Span::styled("Select your preferred model:", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD)),
                ]), area.width);
                for (i, model) in models.iter().enumerate() {
                    if area.y + 4 + i as u16 >= area.y + area.height { break; }
                    let prefix = if i == *selected { "▸ " } else { "  " };
                    let style = if i == *selected { Style::default().fg(theme.accent.into()).add_modifier(Modifier::BOLD) }
                                else { Style::default().fg(theme.text_muted.into()) };
                    buf.set_line(area.x, area.y + 4 + i as u16, &Line::from(vec![
                        Span::styled(prefix, style), Span::styled(model.as_str(), style),
                    ]), area.width);
                }
            }
            OnboardingStep::ApiKeyInput { input, masked } => {
                buf.set_line(area.x, area.y + 3, &Line::from(vec![
                    Span::styled("Enter your API key:", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD)),
                ]), area.width);
                let display = if *masked { input.chars().map(|c| if c == '-' { c } else { '•' }).collect() } else { input.clone() };
                buf.set_string(area.x + 2, area.y + 5, &display, Style::default().fg(theme.text.into()));
                buf.set_string(area.x + 2 + display.len() as u16, area.y + 5, "▌", Style::default().fg(theme.accent.into()));
            }
        }
    }
}
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

---

## 29. Sub-Agent Delegation Flow

This shows the flow when the coordinator spawns a sub-agent to work independently.

```
Step 1 — Coordinator decides to delegate:

┌─ Agent ──────────────────────────────────────────────────────────────┐
│ 🔱 Delegate to sub-agent-claude-sonnet-4                             │
│                                                                      │
│   prompt: "Research auth patterns in the codebase and               │
│            propose a fix for the expiry bug"                         │
│                                                                      │
│ ⠋ Spawning sub-agent... (will appear in separate tmux pane)         │
└─────────────────────────────────────────────────────────────────────┘

Step 2 — Sub-agent running in tmux window:

┌─ Agent: research-auth ───────────────────────────────────────────────┐
│ 📤 Delegated to research-auth (tmux pane)                           │
│ │                                                                   │
│ ├─ Read src/auth.rs ✓ (7 lines)                                    │
│ ├─ Grep "validate" -> 5 matches in 2 files ✓                        │
│ ├─ Read src/token.rs ✓ (15 lines)                                  │
│ ├─ Bash "cargo test --lib auth" ✓ exit: 0                          │
│ │                                                                   │
│ ⠋ Thinking... 3.2s                                                  │
└─────────────────────────────────────────────────────────────────────┘

Step 3 — Sub-agent completed:

┌─ Agent: research-auth (8.5s) ────────────────────────────────────────┐
│ ✓ Sub-agent complete                                                │
│   tools: 3 read, 2 grep, 1 bash                                    │
│                                                                      │
│   Foundings:                                                         │
│   1. validate_expiry at src/auth.rs:45 is missing current time       │
│   2. validate_expiry called from check_permissions at line 78        │
│   3. Token validation works correctly                                │
│                                                                      │
│   Returned: "The bug is at auth.rs:45 -- validate_expiry needs      │
│             a `now: i64` parameter"                                  │
└─────────────────────────────────────────────────────────────────────┘

Step 4 — Coordinator continues with the result:

┌─ Assistant ──────────────────────────────────────────────────────────┐
│ Based on the sub-agent's research, the fix is straightforward:      │
│ add a `now: i64` parameter to `validate_expiry`.                    │
│                                                                      │
│ ┌─ Edit ──────────────────────────────────────────────────────────┐ │
│ │ -> Update src/auth.rs                                            │ │
│ │   - fn validate_expiry(expiry: i64) -> bool                     │ │
│ │   + fn validate_expiry(expiry: i64, now: i64) -> bool           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### Sub-agent Status Badge

When sub-agent is running in background, a pill appears in the status line:

```
│ sonnet-4  ctx:42%  $0.12  [🔱 1 agent active]  ▌auto               │
                               ^ shows running sub-agent count
                               ^ clickable to expand swarm gallery
```

### Sub-agent Lifecycle States

```
States: Pending -> Spawning -> Running -> Completed | Failed | Timeout

+-------+----------------+------------------------------+
| State | Icon           | Color                        |
+-------+----------------+------------------------------+
| Queue | ...            | text_subtle                  |
| Spawn | 🔱             | accent                       |
| Run   | ⠋ (animated)   | accent (spinning)            |
| Done  | ✓              | success                      |
| Fail  | ✗              | error                        |
| Time  | ⚠              | warning                      |
+-------+----------------+------------------------------+
```

### Code

```rust
// crates/jcode-tui/src/history_cell/sub_agent_cell.rs

use std::time::Instant;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;
use crate::history_cell::HistoryCell;

pub struct SubAgentCell {
    pub id: String, pub task: String, pub model: String,
    pub status: SubAgentStatus, pub tools_used: Vec<String>,
    pub result: Option<String>, pub created_at: Instant,
}

pub enum SubAgentStatus { Spawning, Running { tick: u8 }, Completed, Failed(String) }

impl HistoryCell for SubAgentCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        let inner = Rect { x: area.x + 2, width: area.width.saturating_sub(4), ..area };
        let (icon, color) = match &self.status {
            SubAgentStatus::Spawning => ("🔱", theme.accent.into()),
            SubAgentStatus::Running { .. } => ("⠋", theme.accent.into()),
            SubAgentStatus::Completed => ("✓", theme.success.into()),
            SubAgentStatus::Failed(_) => ("✗", theme.error.into()),
        };
        let dur = self.created_at.elapsed().as_secs_f64();
        buf.set_line(inner.x, y, &Line::from(vec![
            Span::styled(format!("{} Sub-agent: {} ({})", icon, self.id, self.model), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {:.1}s", dur), Style::default().fg(theme.text_subtle.into())),
        ]), inner.width);
        y += 2;
        buf.set_line(inner.x, y, &Line::from(vec![
            Span::styled("  prompt: ", Style::default().fg(theme.text_subtle.into())),
            Span::styled(&self.task, Style::default().fg(theme.text_muted.into())),
        ]), inner.width);
        y += 1;
        if !self.tools_used.is_empty() {
            buf.set_line(inner.x, y, &Line::from(vec![
                Span::styled("  tools: ", Style::default().fg(theme.text_subtle.into())),
                Span::styled(self.tools_used.join(", "), Style::default().fg(theme.text_muted.into())),
            ]), inner.width);
            y += 1;
        }
        if let Some(result) = &self.result {
            y += 1;
            buf.set_line(inner.x, y, &Line::from(vec![Span::styled("  Result:", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD))]), inner.width);
            y += 1;
            for line in result.lines().take(5) {
                buf.set_line(inner.x + 2, y, &Line::from(Span::styled(line, Style::default().fg(theme.text_muted.into()))), inner.width);
                y += 1;
            }
        }
    }
    fn desired_height(&self, _: u16) -> u16 { 4 + if self.result.is_some() { 2 } else { 0 } }
    fn is_active(&self) -> bool { matches!(&self.status, SubAgentStatus::Spawning | SubAgentStatus::Running { .. }) }
}
```
---

## 30. Shell / Interactive Terminal

When the user runs a shell command (`!` prefix in composer), it shows as an interactive terminal block.

```
Running shell command:

┌─ Shell ──────────────────────────────────────────────────────────────┐
│ $ cargo build --release                                              │
│                                                                    │
│    Compiling jcode-tui v0.1.0                                        │
│    Compiling jcode-core v0.1.0                                       │
│    Compiling jcode v0.1.0                                            │
│ ⠋ building... 8.5s                                                  │
└─────────────────────────────────────────────────────────────────────┘

Completed shell (scrolling output):

┌─ Shell ──────────────────────────────────────────────────────────────┐
│ $ cargo build --release                                              │
│ ✓ exit: 0  (12.3s)                                                  │
│                                                                    │
│    Compiling jcode-tui v0.1.0                                        │
│    Compiling jcode-core v0.1.0                                       │
│    Compiling jcode v0.1.0                                            │
│     Finished release [optimized] target(s) in 12.3s                  │
│                                                                    │
│ [Use up/down to scroll output, Ctrl+C to interrupt]                 │
└─────────────────────────────────────────────────────────────────────┘

Denied shell command:

┌─ Shell ──────────────────────────────────────────────────────────────┐
│ $ rm -rf /                                                          │
│ ✗ Command denied by permission policy                               │
│   Reason: Destructive command requires explicit approval             │
└─────────────────────────────────────────────────────────────────────┘

Interactive background process (e.g., server):

┌─ Shell ──────────────────────────────────────────────────────────────┐
│ $ python -m http.server 8000                                       │
│ ⠋ running... 45.2s                                                  │
│                                                                    │
│   Serving HTTP on :: port 8000 (http://[::]:8000/)                  │
│   127.0.0.1 - - [26/Jun/2026 13:42:01] "GET /" 200 -               │
│   127.0.0.1 - - [26/Jun/2026 13:42:05] "GET /api" 200 -             │
│                                                                    │
│ [Background process -- type in composer to interact via stdin]      │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/history_cell/shell_cell.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;
use crate::history_cell::HistoryCell;

pub struct ShellCell {
    pub command: String, pub output_lines: Vec<String>,
    pub status: ShellStatus, pub exit_code: Option<i32>,
    pub duration: Option<std::time::Duration>,
}
pub enum ShellStatus { Running, Success, Error(String), Denied(String), Killed(String) }
impl HistoryCell for ShellCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        buf.set_line(area.x, y, &Line::from(vec![Span::styled("$ ", Style::default().fg(theme.tool_bash.into())), Span::styled(&self.command, Style::default().fg(theme.text.into()))]), area.width);
        y += 1;
        match &self.status {
            ShellStatus::Running => { buf.set_line(area.x, y, &Line::from(vec![Span::styled("⠋ running...", Style::default().fg(theme.accent.into()))]), area.width); }
            ShellStatus::Success => { buf.set_line(area.x, y, &Line::from(vec![Span::styled("✓ exit: 0", Style::default().fg(theme.success.into()))]), area.width); }
            ShellStatus::Error(msg) => { buf.set_line(area.x, y, &Line::from(vec![Span::styled("✗ ", Style::default().fg(theme.error.into())), Span::styled(msg.as_str(), Style::default().fg(theme.error.into()))]), area.width); }
            ShellStatus::Denied(r) => { buf.set_line(area.x, y, &Line::from(vec![Span::styled("✗ denied", Style::default().fg(theme.warning.into())), Span::styled(format!(" - {}", r), Style::default().fg(theme.text_muted.into()))]), area.width); }
            ShellStatus::Killed(r) => { buf.set_line(area.x, y, &Line::from(vec![Span::styled("💀 Killed: ", Style::default().fg(theme.error.into())), Span::styled(r.as_str(), Style::default().fg(theme.text_muted.into()))]), area.width); }
        }
        y += 1;
        for line in self.output_lines.iter().take(50) {
            if y >= area.y + area.height { break; }
            buf.set_line(area.x + 2, y, &Line::from(Span::styled(line.as_str(), Style::default().fg(theme.text.into()))), area.width - 2);
            y += 1;
        }
    }
    fn desired_height(&self, _: u16) -> u16 { 2 + self.output_lines.len().min(50) as u16 }
    fn is_active(&self) -> bool { matches!(&self.status, ShellStatus::Running) }
}
```
---

## 31. Agent Team / Coordination UI

When multiple agents work together as a team (coordinator + workers), the UI shows the team overview.

```
Team overview (expanded swarm gallery):

┌─────────────────────────────────────────────────────────────────────┐
│ ... swarm . 4 agents . 2 active                    [+] expand/collapse│
├──────────────────┬──────────────────┬──────────────────────────────┤
│ ★ COORDINATOR    │ ◆ worker-auth    │ ⚙ worker-test               │
│ ─────────────── │ ─────────────── │ ─────────────────────────── │
│ status: running  │ status: done    │ status: running              │
│ task: orchestrate│ reads: 3        │ bash: 2/5 passed             │
│ plan: fix auth   │ writes: 1       │ coverage: +12%               │
├──────────────────┼──────────────────┼──────────────────────────────┤
│ ☆ worker-search  │                  │                              │
│ ─────────────── │  (+1 more)       │                              │
│ status: idle     │                  │                              │
│ result: 2 files  │                  │                              │
│ matched          │                  │                              │
└──────────────────┴──────────────────┴──────────────────────────────┘
                 ^ role icons:
                   ★ = coordinator
                   ◆ = researcher/agent
                   ⚙ = worker
                   ☆ = search specialist
```

### Team Task DAG

When viewing task dependencies:

```
┌─ Coordinator ─────────────────────────────────────────────────────────┐
│ Task DAG for "Fix auth bug"                                          │
│                                                                      │
│          ┌──────────┐                                                │
│          │ Research  │ ◄── done                                       │
│          │ patterns  │                                                │
│          └────┬─────┘                                                │
│               │ depends on                                           │
│          ┌────▼─────┐    ┌──────────┐                                │
│          │ Propose   │    │ Add tests│ ◄── blocked (waiting)          │
│          │ fix       │    └──────────┘                                │
│          └────┬─────┘                                                │
│               │                                                      │
│          ┌────▼─────┐    ┌──────────┐                                │
│          │ Implement │    │ Refactor  │ ◄── ready (will start next)    │
│          │ fix       │    │ token    │                                │
│          └──────────┘    └──────────┘                                │
│                                                                      │
│ [1/6 ready  3/6 running  1/6 done  1/6 blocked]                      │
└─────────────────────────────────────────────────────────────────────┘

Status colors for DAG nodes:
  ready  = dimmed border
  active = accent (animated border)
  done   = success ✓
  block  = warning ⚠
  cycle  = error ✗
```

### Team Info Widget

Shown in side panel when team is active:

```
┌────────────────────────────────────────────────┬─────────────────────┐
│ Chat area                                      │ Team Info          │
│                                                │ ───────────────── │
│                                                │ ★ coordinator      │
│                                                │   Model: sonnet-4  │
│                                                │   Status: ● running │
│                                                │                    │
│                                                │ ◆ worker-auth      │
│                                                │   Model: sonnet-4  │
│                                                │   Status: ● done   │
│                                                │   Result: found    │
│                                                │                    │
│                                                │ ◆ worker-test      │
│                                                │   Model: sonnet-4  │
│                                                │   Status: ● running│
│                                                │                    │
│                                                │ [f] focus [e] view │
└────────────────────────────────────────────────┴─────────────────────┘
```

### Role Glyphs

```
+──────────+──────────────────+─────────────────────────────────────+
| Glyph    | Role             | Purpose                             |
+──────────+──────────────────+─────────────────────────────────────+
| ★       | Coordinator      | Orchestrates the team, delegates    |
| ◆       | Researcher       | Gathers information, reads code     |
| ⚙       | Worker           | Executes tasks, writes code         |
| ☆       | Search agent     | Specialized in glob/grep/search     |
| ◇       | Reviewer         | Reviews code, proposes changes      |
| ⊞       | Planner          | Creates implementation plans        |
| ◎       | Observer         | Monitors progress, reports status   |
+──────────+──────────────────+─────────────────────────────────────+
```

### Code

```rust
// crates/jcode-tui-render/src/team_info.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct TeamMember {
    pub glyph: &'static str, pub name: String, pub model: String,
    pub status_dot: &'static str, pub status_text: String,
}
pub struct TeamInfoPanel { pub members: Vec<TeamMember> }

impl TeamInfoPanel {
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        buf.set_line(area.x, y, &Line::from(vec![Span::styled("Team Info", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD))]), area.width);
        y += 2;
        for m in &self.members {
            if y >= area.y + area.height { break; }
            buf.set_line(area.x, y, &Line::from(vec![
                Span::styled(format!("{} {} ", m.glyph, m.name), Style::default().fg(theme.text.into())),
                Span::styled(m.status_dot, Style::default().fg(theme.accent.into())),
                Span::styled(format!(" {}", m.status_text), Style::default().fg(theme.text_muted.into())),
            ]), area.width);
            y += 1;
        }
    }
}
```
---

## 32. Background Tasks / Progress Panel

Shows long-running background tasks (agent teams, builds, tests, etc.) with progress.

```
Status line pill:

│ sonnet-4  ctx:42%  $0.12  [🔱 1 active]  [⏳ 2 bg tasks]  ▌auto    │

Expanded side panel (toggle with Ctrl+Shift+T or /tasks):

┌─────────────────────────────────────────────────────────────────────┐
│ Background Tasks             [+] expand/collapse                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ▸ 🔱 Sub-agent: research-auth (40%)                                 │
│   ═══════════════════════════════░░░░░░░░░░░░ 8.5s/20s             │
│                                                                      │
│   ⏳ Running concurrently:                                          │
│     ◆ Grep "validate" -> 5 matches (0.3s)                           │
│     ◆ Glob *.rs -> 42 matches (0.1s)                                │
│     ◆ Read src/auth.rs (0.2s)                                       │
│                                                                      │
│   ⚙ worker-test (30%)                                               │
│   ═══════════════════░░░░░░░░░░░░░░░░░ 5/15 tests                   │
│                                                                      │
│ ─────────────────────────────────────────────────────────────────    │
│                                                                      │
│ [f] focus task  [c] cancel  [e] view  up/down scroll               │
└─────────────────────────────────────────────────────────────────────┘
```

### Task Progress Bar

```
States:

Running:   ═══════════════░░░░░░░░░░░░░░  42%    (animated fill)
Waiting:   ............................           (dimmed dots)
Done:      ════════════════════════════  ✓ 100%  (green check)
Failed:    ════════✗════════════════════         (red X, stopped at position)
```

### Code

```rust
// crates/jcode-tui/src/history_cell/task_progress.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;
use crate::history_cell::HistoryCell;

pub struct TaskProgressCell {
    pub label: String, pub progress: f64,
    pub elapsed: std::time::Duration, pub status: &'static str,
    pub details: Vec<String>,
}

impl HistoryCell for TaskProgressCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        // Header with status
        let status_icon = match self.status { "running" => "⏳", "success" => "✓", "failed" => "✗", _ => "⏳" };
        let status_color = match self.status { "success" => theme.success, "failed" => theme.error, _ => theme.accent };
        buf.set_line(area.x, y, &Line::from(vec![
            Span::styled(format!("{} {} ({:.0}%)", status_icon, self.label, self.progress * 100.0), Style::default().fg(status_color.into()).add_modifier(Modifier::BOLD)),
        ]), area.width);
        y += 1;
        // Progress bar
        let bw = area.width.saturating_sub(14) as usize;
        let filled = (self.progress * bw as f64) as usize;
        let bar = format!("{} {} {:3.0}%", "═".repeat(filled), "░".repeat(bw.saturating_sub(filled)), self.progress * 100.0);
        buf.set_line(area.x, y, &Line::from(vec![Span::styled(bar, Style::default().fg(theme.accent.into())), Span::styled(format!(" {:.1}s", self.elapsed.as_secs_f64()), Style::default().fg(theme.text_subtle.into()))]), area.width);
        y += 1;
        for d in &self.details {
            if y >= area.y + area.height { break; }
            buf.set_line(area.x + 2, y, &Line::from(vec![Span::styled(d.as_str(), Style::default().fg(theme.text_muted.into()))]), area.width - 2);
            y += 1;
        }
    }
    fn desired_height(&self, _: u16) -> u16 { 2 + self.details.len() as u16 }
}
```
---

## 33. Usage / Cost Overlay

Shown on `/usage` command or with cost pill.

```
Full usage view:

┌─────────────────────────────────────────────────────────────────────┐
│ Usage Statistics                      model: claude-sonnet-4       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ This Session:                                                        │
│   Input tokens:  42,000  ($6.30)                                    │
│   Output tokens: 8,500   ($12.75)                                   │
│   Cache read:    12,000  (28.6% hit rate)                           │
│   Cache write:   3,000   ($0.45)                                    │
│   ─────────────────────────────────────────────                     │
│   Total:                 $19.50                                     │
│                                                                      │
│ Rate Limits:                                                         │
│   Input:   ████████████████░░░░░  2,000 / 4,000 RPM                │
│   Output:  ███████████████░░░░░░  1,500 / 3,000 RPM                │
│                                                                      │
│ 52-Week History:                                                     │
│   ░░░███░████░░███░░██░░████░░                                       │
│   ██░░████░██████░░█░░██░░█░░░░                                       │
│                                                                      │
│ p50 latency: 2.3s   p95 latency: 4.1s   p99 latency: 8.7s          │
│                                                                      │
│ [q] close                                                           │
└─────────────────────────────────────────────────────────────────────┘
```

### Cost Pill on Status Line

```
| sonnet-4  ctx:42%  $0.12  💰 $19.50 today  cache:78%  ▌auto         |
                          ^ cost pill -- cyan/white
                          ^ shows session total
                          ^ resets per session
```

### Code

```rust
// crates/jcode-tui-usage-overlay/src/lib.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct UsageOverlay {
    pub input_tokens: u64, pub output_tokens: u64,
    pub cache_read: u64, pub cache_write: u64,
    pub total_cost: f64,
}

impl UsageOverlay {
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        buf.set_line(area.x, y, &Line::from(vec![Span::styled("Usage Statistics", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD))]), area.width);
        y += 2;
        let rows = [
            ("Input tokens:", format!("{:>8}  (${:.2})", self.input_tokens, self.input_tokens as f64 * 0.00015)),
            ("Output tokens:", format!("{:>8}  (${:.2})", self.output_tokens, self.output_tokens as f64 * 0.0006)),
            ("Cache read:", format!("{:>8}", self.cache_read)),
            ("Cache write:", format!("{:>8}", self.cache_write)),
        ];
        for (l, v) in &rows {
            if y >= area.y + area.height - 2 { break; }
            buf.set_line(area.x + 2, y, &Line::from(vec![
                Span::styled(format!("{:<16}", l), Style::default().fg(theme.text_muted.into())),
                Span::styled(v.as_str(), Style::default().fg(theme.text.into())),
            ]), area.width);
            y += 1;
        }
        y += 1;
        buf.set_line(area.x + 2, y, &Line::from(vec![
            Span::styled("Total cost:       ", Style::default().fg(theme.text_muted.into())),
            Span::styled(format!("${:.2}", self.total_cost), Style::default().fg(theme.accent.into()).add_modifier(Modifier::BOLD)),
            Span::styled("   [q] close", Style::default().fg(theme.text_subtle.into())),
        ]), area.width);
    }
}
```
---

## 34. Copy Selection Mode

Modal mode for selecting and copying text from the transcript.

```
Activated with Shift+up or /copy:

┌─────────────────────────────────────────────────────────────────────┐
│ ⚡ COPY MODE -- press up/down to expand, Enter to copy, Esc to cancel │
│                                                                      │
│ ┌─ Assistant ──────────────────────────────────────────────────────┐ │
│ │ ██ I'll analyze the auth module. Here's what I found: █████████ │ │
│ │ ██ The bug is on line 42 -- `validate_expiry` is called        ██ │ │
│ │ ██ without the current timestamp.                              ██ │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│ Selected range: lines 1-3  (168 chars)  [Enter to copy]             │
└─────────────────────────────────────────────────────────────────────┘

  ^ selection is highlighted (highlighted background)
  ^ range info shown at bottom
  ^ goal-column tracking for multi-line selection
  ^ edge auto-scroll when cursor hits terminal boundary
```

### Code

```rust
// crates/jcode-tui-core/src/copy_selection.rs

pub struct CopySelection {
    pub active: bool,
    pub start: CopyPoint,
    pub end: CopyPoint,
}
pub struct CopyPoint {
    pub cell_index: usize, pub line_offset: u16, pub col: u16,
}

impl CopySelection {
    pub fn new() -> Self { Self { active: false, start: CopyPoint { cell_index: 0, line_offset: 0, col: 0 }, end: CopyPoint { cell_index: 0, line_offset: 0, col: 0 } } }
    pub fn begin(&mut self, cell: usize, line: u16, col: u16) {
        self.active = true;
        self.start = CopyPoint { cell_index: cell, line_offset: line, col };
        self.end = CopyPoint { cell_index: cell, line_offset: line, col };
    }
    pub fn extend(&mut self, cell: usize, line: u16, col: u16) { self.end = CopyPoint { cell_index: cell, line_offset: line, col }; }
    pub fn cancel(&mut self) { self.active = false; }
    pub fn selected_text(&self, get_text: &dyn Fn(usize, u16) -> String) -> String {
        let start = std::cmp::min(self.start.cell_index, self.end.cell_index);
        let end = std::cmp::max(self.start.cell_index, self.end.cell_index);
        (start..=end).filter_map(|i| { let t = get_text(i, 0); if t.is_empty() { None } else { Some(t) } }).collect::<Vec<_>>().join("
")
    }
    pub fn status_line(&self) -> String {
        if !self.active { return String::new(); }
        format!("Selected: cell {} → {}  (Enter to copy, Esc to cancel)", self.start.cell_index, self.end.cell_index)
    }
}
```
---

## 35. Workspace Map (Niri-style)

Visualization of the workspace/project map in the side panel.

```
┌────────────────────────────────────────────┬────────────────────────┐
│ Chat                                       │ Workspace Map         │
│                                            │ ──────────────────── │
│                                            │                       │
│                                            │ ┌─── src/ ─────────┐ │
│                                            │ │ ■ auth.rs        │ │
│                                            │ │ ■ token.rs      ⬡ │ │
│                                            │ │ ■ main.rs        │ │
│                                            │ └──────────────────┘ │
│                                            │ ┌── tests/ ────────┐ │
│                                            │ │ ■ auth_test.rs   │ │
│                                            │ └──────────────────┘ │
│                                            │                       │
│                                            │ ≡ 3 files modified   │
│                                            │ ⬡ = currently open   │
│                                            │                       │
│                                            │ [HJKL navigate]      │
└────────────────────────────────────────────┴────────────────────────┘
```

### Code

```rust
// crates/jcode-tui-workspace/src/lib.rs

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct FileNode {
    pub name: String, pub is_dir: bool, pub modified: bool,
    pub children: Vec<FileNode>, pub open: bool,
}

pub struct WorkspaceMap { pub root: FileNode, pub selected: usize }

impl WorkspaceMap {
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        buf.set_line(area.x, y, &Line::from(vec![Span::styled("Workspace", Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD))]), area.width);
        y += 1;
        self.render_node(&self.root, 0, area, &mut y, buf, theme);
    }
    fn render_node(&self, node: &FileNode, depth: u16, area: Rect, y: &mut u16, buf: &mut Buffer, theme: &Theme) {
        if *y >= area.y + area.height { return; }
        let icon = if node.is_dir { "📁" } else { "📄" };
        let name_style = if node.modified { Style::default().fg(theme.warning.into()) } else { Style::default().fg(theme.text_muted.into()) };
        buf.set_line(area.x + depth * 2, *y, &Line::from(vec![
            Span::styled(format!("{} {}", icon, node.name), name_style)]), area.width);
        *y += 1;
        if node.is_dir && node.open {
            for child in &node.children { self.render_node(child, depth + 1, area, y, buf, theme); }
        }
    }
}
```
---

## 36. Toast Notifications

Non-blocking transient notifications that appear at the top of the input area.

```
┌─────────────────────────────────────────────────────────────────────┐
│ Messages                                                           │
│ ═══════════════════════════════════════════════════════════════════ │
│                                                                    │
├─────────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ ✓ Build successful (12.3s)                         [2s ago]     │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ ⚠ Cannot connect to server. Retrying...             [5s ago]    │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ ▌                                                                 │
└─────────────────────────────────────────────────────────────────────┘
```

### Toast Types

```
✓  success  -- green    (auto-dismiss 3s)
⚠  warning  -- yellow   (auto-dismiss 5s)
✗  error    -- red      (manual dismiss)
ℹ  info     -- gray     (auto-dismiss 2s)
```

---

## Appendix D: Per-Tool UI Matrix

| Tool      | Icon | Running State                 | Success State                  | Failure State          | Color        |
|-----------|------|-------------------------------|--------------------------------|------------------------|--------------|
| Bash      | `$`  | ⠋ running...                 | ✓ exit: 0 + output             | ✗ exit: N + stderr    | `tool_bash`  |
| Edit      | `->` | ⠋ applying...                | ✓ Updated file.rs              | ✗ Edit failed + error  | `tool_edit`  |
| Create    | `★`  | ⠋ creating...                | ✓ Created file.rs              | ✗ Create failed        | `success`    |
| Read      | `->` | ⠋ reading...                 | ✓ file.rs (42 lines)           | ✗ File not found       | `tool_read`  |
| Glob      | `☆`  | ⠋ searching...               | ☆ glob *.rs -> 42 matches      | ✗ No matches           | `info`       |
| Grep      | `◆`  | ⠋ searching...               | ◆ grep "fn" -> 7 in 3 files    | ✗ No matches           | `info`       |
| Agent     | `🔱` | 🔱 Spawning / ⠋ running     | ✓ Sub-agent complete (8.5s)    | ✗ Failed: timeout     | `accent`     |
| Shell     | `$`  | ⠋ running... (live stream)  | ✓ exit: 0  (12.3s)             | ✗ exit: 1 + error      | `tool_bash`  |
| WebFetch  | `🌐` | ⠋ fetching url...            | ✓ Fetched (1,234 bytes)        | ✗ Connection error     | `info`       |
| WebSearch | `🔍` | ⠋ searching web...           | 🔍 "query" -> 5 results        | ✗ No results           | `info`       |
| Question  | `?`  | ⠋ asking...                  | ? Answered                     | ✗ No answer            | `warning`    |
| TodoWrite | `☐`  | ⠋ writing todos...           | ☑ 3 todos updated              | ✗ Write failed          | `info`       |
| Task      | `⊞`  | ⠋ executing task...          | ⊞ Task complete (3/3 steps)    | ✗ Task failed           | `accent`     |
| ApplyPatch| `->` | ⠋ applying patch...          | ✓ Patch applied (3 files)      | ✗ Patch rejected        | `tool_edit`  |

---

## Appendix E: Edge Cases & Error Handling

### What happens when...

```
1. Terminal too small (< 60x15):
   -> Show warning: "Terminal too small -- please resize to at least 60x15"
   -> Only show messages, hide status line
   -> Keep input functional

2. Network disconnect:
   -> Spinner: ⏳ Reconnecting... (attempt 2/5)
   -> Status line shows ⚠ disconnected
   -> Messages still visible, cannot submit

3. API rate limited:
   -> Error cell: ✗ Rate limited. Retry after 30s
   -> Status line: 🕐 rate limited -- 28s remaining
   -> Auto-retry countdown shown

4. Token overflow (> 200K context):
   -> Status line goes red: ctx:98% 🔴
   -> Warning in transcript: ⚠ Context limit approaching
   -> Auto-prompt for /compact or fork

5. Permission timeout:
   -> Permission dialog auto-denies after 60s
   -> Shows: ⏰ Timed out -- approval not received in time

6. User shell interrupt (Ctrl+C):
   -> Shell cell shows: 💀 Killed by user (SIGINT)
   -> Still showed in transcript for reference

7. Terminal resize during streaming:
   -> Cells recompute desired_height()
   -> Scroll position adjusted
   -> Spinner continues smoothly

8. Multiple sub-agents return simultaneously:
   -> Each appears as separate SubAgentCell
   -> Ordered by completion time
   -> Coordinator synthesis shown at end

9. Background process orphaned:
   -> Shell cell shows: ⏳ Process still running (PID 12345)
   -> Allows typing more input to interact
   -> Kill button available when focused

10. Session fork in progress:
    -> Indicator in status: 🚧 Forking session...
    -> Original session still accessible
    -> New session opens automatically when ready
```

---

## Appendix F: Animation Reference

### Spinner Animation Frames

```rust
// Braille spinner (10 frames, 12.5 FPS, 80ms per frame)
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// Progress bar animation (running state)
// Animated fill moving from left to right, wrapping around:
// ═══░░░░░░░░░░░░░░   ->   ░═══░░░░░░░░░░░   ->   ░░░═══░░░░░░░░  ...

// Blinking cursor (500ms interval)
// ▌ (shown)  ->  (hidden, 300ms)  ->  ▌ (shown)

// Shimmer effect (cosine-based sweep, 2s period):
// Periodically draws a bright bar sweeping across header text.
```

### Timing & Performance Targets

```
Frame rate:    120 FPS max  (8.33ms per frame)
Spinner tick:  80ms per frame (12.5 FPS for smooth animation)
Cursor blink:  500ms on / 300ms off
Toast dismiss: 2-5 seconds depending on type
Permission timeout: 60 seconds
Leader key timeout: 2000ms
Resize reflow: < 50ms
Cache TTL for rendered content: 2 full frames
```

---

## 37. Model Picker

Interactive model selection dialog.

```
Trigger: /model, Ctrl+M, or Ctrl+X M

┌─────────────────────────────────────────────────────────────────────┐
│ Select Model                                           type to filter│
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ▸ claude-sonnet-4-20250514    (fast, recommended)                   │
│   claude-opus-4-20250514      (most capable, $3/M tokens)           │
│   claude-haiku-4-5-20251001   (cheapest, $0.25/M tokens)           │
│   ─────────────────────────────────────────                          │
│   gemini-2.5-pro               (via OpenProxy)                      │
│   gpt-4o                       (via OpenProxy)                      │
│                                                                      │
│ Active: claude-sonnet-4-20250514  [Ctrl+S: switch fast mode]        │
│                                                                      │
│ ↑/↓ navigate  Enter:select  /:filter  q:close                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/model_picker.rs

pub struct ModelPicker {
    pub models: Vec<ModelEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub search: String,
    pub on_select: Box<dyn Fn(&ModelEntry) -> bool>,
}

pub struct ModelEntry {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub is_active: bool,
    pub cost_hint: Option<String>,
    pub capability: ModelCapability,
}

impl BottomPaneView for ModelPicker {
    fn view_id(&self) -> &str { "model_picker" }
    fn is_complete(&self) -> bool { false }
    fn handle_key_event(&mut self, key: &KeyCombo) -> bool {
        // ↑/↓: navigate, Enter: select, /: search
    }
}
```

---

## 38. Todos / Task Management Panel

Interactive todo list for tracking tasks.

```
Trigger: Ctrl+T or /todos

┌─────────────────────────────────────────────────────────────────────┐
│ Todos                    [+] new todo  [/] filter                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ☐ Fix the bug in auth.rs                          high  [due:today]│
│ ☑ Add tests for keymap                   [done]    med              │
│ ☐ Refactor token validation                        low              │
│ ☐ Write documentation for TUI spec                  high            │
│ ─────────────────────────────────────────────────────────────────    │
│ 3 open · 1 done                                                     │
│                                                                      │
│ [a] add  [d] delete  [x] toggle  [e] edit  [p] priority            │
│ Enter:jump to context  ↑/↓ navigate  q:close                       │
└─────────────────────────────────────────────────────────────────────┘

### Task States

```
☐ pending    — open task (dimmed text)
☑ done       — completed task (strikethrough + green check)
⚠ overdue    — past due date (red)
⏳ in-prog   — currently being worked on (accent)
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/todos_panel.rs

pub struct TodosPanel {
    pub todos: Vec<TodoItem>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub show_filter: bool,
}

pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub done: bool,
    pub priority: Priority,
    pub due_date: Option<chrono::NaiveDate>,
    pub context: Option<String>, // file:line reference
}

impl BottomPaneView for TodosPanel {
    fn view_id(&self) -> &str { "todos" }
}
```
```

---

## 39. File Tree Sidebar

Project file tree in the side panel.

```
Trigger: /files, or side panel toggle

┌─────────────────────────────────────────────────────────────────────┐
│ Files                              [@] search  [+] collapse all    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ 📁 crates/                                                          │
│  📁 jcode-tui/                                     ~42 files        │
│   📁 src/                                                           │
│    📁 tui/                                                          │
│     📄 app.rs                                    ◄ active           │
│     📄 keybind.rs                                                   │
│     📄 ui.rs                                      ≡ modified        │
│     📄 mod.rs                                                       │
│  📁 jcode-tui-core/                               ~8 files          │
│  📁 jcode-tui-style/                              ~6 files          │
│ 📁 docs/                                                             │
│ 📁 tests/                                                            │
│ 📄 Cargo.toml                                                        │
│ 📄 README.md                                                         │
│                                                                      │
│ ≡ = uncommitted  ◄ = open file                                      │
│                                                                      │
│ [Enter] open  [j/k] navigate  [/] search  [Esc] close              │
└─────────────────────────────────────────────────────────────────────┘

### Keybindings

```
Ctrl+P         Open file picker (search mode)
Ctrl+W,L       Toggle file tree sidebar
Enter          Open file in editor
/              Filter files
```

### Code

```rust
// crates/jcode-tui/src/ui/section_39.rs

pub struct FileTreeFile { pub name: String, pub is_dir: bool, pub modified: bool, pub depth: u16 } pub struct FileTreeSidebar { pub files: Vec<FileTreeFile>, pub selected: usize }
```
---

## 40. Configurator / Settings Dialog

Interactive settings editor.

```
Trigger: /config

┌─────────────────────────────────────────────────────────────────────┐
│ Configuration                                     [Ctrl+S] save     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ▸ General                                                           │
│   Theme:         catppuccin-mocha      ▸ (select from 12 themes)    │
│   Permission:    auto                  ▸ auto | plan | bypass       │
│   Fast Mode:     off                   ▸ toggle                     │
│   Language:      en                    ▸ en | vi | ja               │
│                                                                      │
│ ▸ Keys & Shortcuts                                                  │
│   Keybindings:   ~/.jcode/keybinds.json   [e] edit file             │
│   Leader Key:    Ctrl+X                  ▸ change                   │
│   Vim Mode:      off                     ▸ toggle                   │
│                                                                      │
│ ▸ Providers                                                         │
│   Anthropic:     sk-ant-***...enabled    [e] change key             │
│   OpenProxy:     http://127.0.0.1:4623   [e] edit                   │
│                                                                      │
│ ↑/↓ navigate  Enter:edit  Tab:next section  q:close                │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/ui/section_40.rs

pub struct ConfigItem { pub label: String, pub current: String, pub options: Vec<String> } pub struct Configurator { pub items: Vec<ConfigItem>, pub selected: usize }
```
---

## 41. Plugin Manager

Enables/disables plugins from the TUI.

```
Trigger: /plugins

┌─────────────────────────────────────────────────────────────────────┐
│ Plugins                  ⚡ 3 enabled / 8 available                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ✓ jcode-pro           1.2.0    Enhanced provider support            │
│ ✓ mermaid-rs          0.3.1    Mermaid diagram rendering            │
│ ✓ swarm-core          0.1.0    Multi-agent orchestration           │
│ ☐ lsp-support         0.2.0    LSP integration                      │
│ ☐ git-blame           0.1.0    Git blame annotations                │
│ ☐ terminal-image      0.4.0    Kitty terminal image protocol        │
│                                                                      │
│ [Enter] toggle  [i] info  [r] remove  [u] update  q:close          │
└─────────────────────────────────────────────────────────────────────┘

### Plugin Details (pressing i)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Plugin Info                    mermaid-rs 0.3.1                     │
│                                                                      │
│ Renders Mermaid diagrams inline in the TUI using                      │
│ the kitty terminal image protocol.                                   │
│                                                                      │
│ Author: jcode team                                                   │
│ License: MIT                                                         │
│ Source: ~/.jcode/plugins/mermaid-rs/                                 │
│ Dependencies: none                                                    │
│                                                                      │
│ [d] disable  [u] uninstall  [b] back                                │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/ui/section_41.rs

pub struct PluginEntry { pub name: String, pub ver: String, pub enabled: bool, pub desc: String } pub struct PluginManager { pub plugins: Vec<PluginEntry>, pub selected: usize }
```
---

## 42. Git Info Widget

Shows current git status in the side panel or as a status line pill.

```
Side panel widget:

┌─────────────────────────────────────────────────────────────────────┐
│ Git Status                                                          │
├─────────────────────────────────────────────────────────────────────┤
│ feat/tui-redesign          ≡ 3 files modified                       │
│                                                                      │
│  M  MASTER_UI.md                                                    │
│  M  crates/jcode-tui/src/ui.rs                                     │
│  ?? untracked_file.rs                                               │
│                                                                      │
│ Recent commits:                                                      │
│  f2fc63b3  fix layout TUI                                           │
│  a4157c0b  refactor code structure                                  │
│                                                                      │
│ Branch: feat/tui-redesign (ahead: 3 commits)                        │
│                                                                      │
│ [r] refresh  [b] branch picker                                      │
└─────────────────────────────────────────────────────────────────────┘

Status line pill:

```
│ sonnet-4  ctx:42%  $0.12  ≡ feat/tui-redesign  cache:78%  ▌auto    │
                                ↑ git branch pill (optional)
```

### Code

```rust
// crates/jcode-tui/src/ui/section_42.rs

pub struct GitInfo { pub branch: String, pub modified: Vec<String>, pub untracked: Vec<String> }
```
---

## 43. Changelog Dialog

Shows version history and changes.

```
Trigger: /changelog or on version upgrade

┌─────────────────────────────────────────────────────────────────────┐
│ Changelog                               jcode v0.1.0 (Jun 2026)    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ v0.1.0 — Jun 26, 2026                                                │
│                                                                      │
│ Features:                                                            │
│   • TUI redesign with Claude Code patterns                           │
│   • Adaptive color system (CIE76 quantization)                       │
│   • Per-tool UI components (Bash, Edit, Read, Agent)                 │
│   • Context-stacked keybinding system                                │
│   • 13-mode spinner                                                  │
│   • Permission dialog with per-tool UIs                              │
│                                                                      │
│ Bug Fixes:                                                           │
│   • validate_expiry missing current timestamp param                  │
│   • Scroll offset reset on resize                                    │
│                                                                      │
│ [q] close  ↑/↓ scroll  [/] search                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/ui/section_43.rs

pub struct ChangeEntry { pub category: String, pub items: Vec<String> } pub struct Changelog { pub entries: Vec<ChangeEntry>, pub scroll: u16 }
```
---

## 44. Account Picker

Switch between multiple API accounts.

```
Trigger: /account or Ctrl+Shift+A

┌─────────────────────────────────────────────────────────────────────┐
│ Switch Account                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ▸ quangdang46                 Anthropic        [active]              │
│   work@company.com            Anthropic                              │
│   temp-dev-account            Anthropic                              │
│   ──────────────────────────────────                                  │
│   openproxy                   OpenProxy       [via env var]          │
│   + Add Account                                                     │
│                                                                      │
│ [Enter] switch  [d] remove  [r] rename  q:close                    │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/ui/section_44.rs

pub struct AccountEntry { pub name: String, pub provider: String, pub active: bool } pub struct AccountPicker { pub accounts: Vec<AccountEntry>, pub selected: usize }
```
---

## 45. Notification Center

History of system notifications.

```
Trigger: Ctrl+` (backtick) or /notifications

┌─────────────────────────────────────────────────────────────────────┐
│ Notifications                                      38 total         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ✓ Build successful                          12.3s ago    [dismiss]  │
│ ⚠ Rate limit approaching (87%)              5m ago      [dismiss]  │
│ ✓ Sub-agent: research-auth completed         10m ago     [dismiss]  │
│ ℹ Session saved to ~/.jcode/last_session     15m ago     [dismiss]  │
│ ✗ Connection lost (recovered)                1h ago      [dismiss]  │
│                                                                      │
│ ───── older ─────                                                     │
│                                                                      │
│ ✓ Tests passed: 42/42                         2h ago                 │
│                                                                      │
│ [d] dismiss  [a] dismiss all  ↑/↓ scroll  q:close                  │
└─────────────────────────────────────────────────────────────────────┘

### Notification Types

```
Icon  Type     Color    Auto-dismiss  Sound
✓     success  green   3s            optional
⚠     warning  yellow  5s            optional
✗     error    red      manual        yes
ℹ     info     gray    2s            no
```

### Code

```rust
// crates/jcode-tui/src/ui/section_45.rs

pub struct NotifEntry { pub icon: &'static str, pub msg: String, pub kind: &'static str, pub age_secs: u64 }
```
---

## 46. Memory Tiles

Masonry layout showing agent memories and context entries in the side panel.

```
┌──────────────────────────────────────┬──────────────────────────────┐
│ Chat                                 │ Memory                      │
│                                      │ ────────────────────────── │
│                                      │                               │
│                                      │ ┌────────┐ ┌────────┐      │
│                                      │ │Auth    │ │Expiry  │      │
│                                      │ │pattern │ │fix     │      │
│                                      │ │3 items │ │2 items │      │
│                                      │ └────────┘ └────────┘      │
│                                      │ ┌────────┐ ┌────────┐      │
│                                      │ │Token   │ │User    │      │
│                                      │ │valid   │ │prefs   │      │
│                                      │ │1 item  │ │4 items │      │
│                                      │ └────────┘ └────────┘      │
│                                      │                               │
│                                      │ [=] edit  [+] add           │
│                                      │ [CTRL+↑/↓] reorder          │
│                                      └──────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/widgets/memory_tiles.rs

pub struct MemoryTile { pub title: String, pub entries: u32, pub content: Vec<String> }
pub struct MemoryTilesWidget { pub tiles: Vec<MemoryTile>, pub selected: usize }
```
---

## 47. Timeline / Session History

Chronological view of all sessions in the side panel.

```
┌──────────────────────────────────────┬──────────────────────────────┐
│ Chat                                 │ Timeline                    │
│                                      │ ────────────────────────── │
│                                      │                               │
│                                      │ Jun 2026                      │
│                                      │ ───────────────────────       │
│                                      │  26  TUI redesign    ● done   │
│                                      │  25  Fix auth bug    ● done   │
│                                      │  24  Add keymap test ● done   │
│                                      │  23  Refactor stream  ● done  │
│                                      │                               │
│                                      │ May 2026                      │
│                                      │ ───────────────────────       │
│                                      │  30  Project init    ● done   │
│                                      │                               │
│                                      │ ≡ 5 sessions this month      │
│                                      │                               │
│                                      │ [Enter] resume  q:close      │
│                                      └──────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/widgets/timeline.rs

pub struct TimelineEntry { pub title: String, pub branch: String, pub age: String, pub status: &'static str }
pub struct TimelineWidget { pub entries: Vec<TimelineEntry>, pub selected: usize }
```
---

## 48. Experiment Popup

One-time dialog for experimental features.

```
┌─────────────────────────────────────────────────────────────────────┐
│ 🧪 Experimental Feature                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Enable Continue Mode?                                              │
│                                                                      │
│   This lets jcode continue working autonomously after               │
│   a task completes. Use this for overnight runs or                  │
│   long-running code reviews.                                        │
│                                                                      │
│   What it does:                                                      │
│   • After a task completes, jcode will continue until                │
│     you press Ctrl+C                                                 │
│   • Runs in a loop: plan → execute → verify → repeat                │
│   • Tokens are used incrementally                                    │
│                                                                      │
│   (y) Enable         (n) Not now         (Esc) Never ask again      │
│                                                                      │
│   [Ctrl+D] learn more  [Ctrl+E] show example                        │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/experiment_popup.rs

use crate::bottom_pane::{BottomPaneResult, BottomPaneView};
use jcode_tui_core::keymap::KeyCombo;
use ratatui::buffer::Buffer; use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style}; use ratatui::text::{Line, Span};
use jcode_tui_style::Theme;

pub struct ExperimentPopup {
    pub title: String, pub description: String, pub details: Vec<String>,
    pub selected: usize, pub choices: Vec<String>,
}
impl BottomPaneView for ExperimentPopup {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let mut y = area.y;
        buf.set_line(area.x, y, &Line::from(vec![Span::styled("🧪 ", Style::default().fg(theme.warning.into())), Span::styled(&self.title, Style::default().fg(theme.text.into()).add_modifier(Modifier::BOLD))]), area.width);
        y += 1;
        buf.set_line(area.x, y, &Line::from(vec![Span::styled(&self.description, Style::default().fg(theme.text_muted.into()))]), area.width);
        y += 2;
        for d in &self.details {
            if y >= area.y + area.height - 3 { break; }
            buf.set_line(area.x + 2, y, &Line::from(vec![Span::styled(format!("• {}", d), Style::default().fg(theme.text_muted.into()))]), area.width);
            y += 1;
        }
    }
    fn handle_key_event(&mut self, key: &KeyCombo) -> bool { matches!(key.key.as_str(), "y" | "n" | "escape") }
    fn completion(&self) -> Option<BottomPaneResult> { Some(BottomPaneResult::Approved { choice: ["enable","skip","dismiss"][self.selected].into() }) }
    fn handle_ctrl_c(&mut self) -> BottomPaneResult { BottomPaneResult::Cancelled }
    fn view_id(&self) -> &str { "experiment_popup" }
}
```
---

## Appendix G: Complete Feature Inventory

### Coverage Summary

```
MASTER_UI.md now covers: 48 sections + 7 appendices = 55 spec items

┌─────────────────────────────────────────────────────────────────────┐
│ CATEGORY             │ COVERED │ MISSING │ TOTAL                    │
├───────────────────────┼─────────┼─────────┼─────────────────────────┤
│ Chat / Messages       │   6     │   0     │   6                      │
│ Tool Calls            │   8     │   0     │   8                      │
│ Input / Composer      │   1     │   0     │   1                      │
│ Navigation / Scroll   │   3     │   0     │   3                      │
│ Permission System     │   3     │   0     │   3                      │
│ Spinner / Status      │   2     │   0     │   2                      │
│ Overlays / Dialogs    │   6     │   4     │  10                      │
│ Side Panels           │   3     │   3     │   6                      │
│ Multi-Agent           │   3     │   0     │   3                      │
│ Shell / Terminal      │   1     │   0     │   1                      │
│ Settings / Config     │   0     │   3     │   3                      │
│ Info Widgets          │   0     │   3     │   3                      │
│ Notifications         │   0     │   1     │   1                      │
├───────────────────────┼─────────┼─────────┼─────────────────────────┤
│ TOTAL                 │  36     │  12     │  48                      │
└─────────────────────────────────────────────────────────────────────┘

### Priority Implementation Order

```
Phase 0 — Foundation (existing crate features, no new code):
  - StreamBuffer, AnchorStability, CopySelection, GraphTopology
  - Keybind parsing (KeyCombo, KeyContext)
  - Theme colors, spinner animation

Phase 1 — Core Chat UX (Weeks 1-2):
  - ✅ Status Line (position: below input)
  - ✅ Chat Viewport (scrollable, auto-pin)
  - ✅ User Message (border, images, queued badge)
  - ✅ Assistant Message (markdown, code blocks)
  - ✅ Thinking/Reasoning Block (collapsed/expanded/hidden)
  - ✅ Spinner (13 modes, inside scrollbox)

Phase 2 — Tool UI (Weeks 2-3):
  - ✅ Tool Call — Bash ($ command, collapsible)
  - ✅ Tool Call — Edit (inline diff, line numbers)
  - ✅ Tool Call — Read (syntax highlighting)
  - ✅ Tool Call — Glob/Grep (compact/expanded)
  - ✅ Tool Call — Agent (sub-agent delegation)
  - ✅ Shell / Interactive Terminal (live stream)

Phase 3 — Permissions + Input (Weeks 3-4):
  - ✅ Permission Dialog — Bash (warning + 4 choices)
  - ✅ Permission Dialog — Edit (diff preview)
  - ✅ Permission Dialog — Read (simple path)
  - ✅ Chat Composer (input, autocomplete, stash)
  - ✅ Unseen Divider ("N new messages")
  - ✅ Footer Hints (progressive collapse)

Phase 4 — Navigation + Overlays (Weeks 4-5):
  - ✅ Transcript Overlay (full-screen pager)
  - ✅ Which-Key Panel (grouped keybinding list)
  - ✅ Session Picker (search + list + resume)
  - ✅ Copy Selection Mode (highlighted range)
  - Keybinding system upgrade (leader key, 150+ bindings)

Phase 5 — Dialogs + Panels (Weeks 5-6):
  - 🔴 Model Picker (/model)
  - 🔴 Todos Panel (/todos)
  - 🔴 Configurator (/config)
  - 🔴 File Tree Sidebar (side panel)
  - ✅ Theme Switching (12 themes)
  - ✅ Toast Notifications (4 types)

Phase 6 — Multi-Agent + Advanced (Weeks 6-7):
  - ✅ Sub-Agent Delegation (4-step flow)
  - ✅ Agent Team / Coordination UI (DAG, swarm gallery)
  - ✅ Background Tasks / Progress Panel (progress bars)
  - ✅ Swarm Gallery (multi-agent grid)
  - ✅ Workspace Map (Niri-style file tree)
  - 🔴 Account Picker (/account)

Phase 7 — Info + Polish (Weeks 7-8):
  - 🔴 Git Info Widget (git status side panel)
  - 🔴 Plugin Manager (/plugins)
  - 🔴 Changelog Dialog (/changelog)
  - 🔴 Notification Center (notification history)
  - 🔴 Memory Tiles (context entries masonry)
  - 🔴 Timeline View (session history)

Legend: ✅ = spec done  🔴 = spec done, needs implementation
```

### Keybinding Append

Add these to Appendix B:

```
Additional keybindings (from missing features):

Model picker:
  Ctrl+M           Open model picker
  Ctrl+X M         Open model picker (leader)

Todos:
  Ctrl+T           Toggle todos panel
  Ctrl+X T         Toggle todos panel (leader)

Files:
  Ctrl+P           Open file picker
  Ctrl+W L         Toggle file tree sidebar

Config:
  Ctrl+,           Open configurator (/config)
  Ctrl+S           Save config

Notifications:
  Ctrl+`           Toggle notification center

Account:
  Ctrl+Shift+A     Switch account

Side panel navigation (Niri-style):
  Ctrl+W H         Focus left panel
  Ctrl+W J         Focus panel below
  Ctrl+W K         Focus panel above
  Ctrl+W L         Focus right panel
  Ctrl+W Q         Close side panel
```

---

## 49. Side Conversations / Fork Threads

Transient fork threads for quick questions without leaving the main conversation.

```
Trigger: /side

┌─ Side  ──────────────────────────────────────────────────────────────┐
│ > Quick question: what's the syntax for HashMap?                    │
│                                                                      │
│ HashMap::new() — you can use `HashMap::from([(k, v)])` for         │
│ initialization or the common `map.insert(k, v)` pattern.            │
│                                                                      │
│ [Return to main conversation (Esc)]                                 │
└─────────────────────────────────────────────────────────────────────┘

Status line indication:

│ sonnet-4  ctx:42%  $0.12  ≡ feat/tui-redesign  [SIDE]             │
                                                       ↑ side session pill
```

**Keybindings:**
- `/side` or `Ctrl+X S` — start a side conversation
- `Esc` or `Ctrl+X R` — return to main conversation
- Side conversations disappear when dismissed (not saved to history)

### Code

```rust
// crates/jcode-tui/src/history_cell/side_conversation.rs

pub struct SideConvCell {
    pub prompt: String, pub response: String, pub active: bool,
}
```
---

## 50. Backtrack / Undo Rollback

Undo the last turn and go back to a previous state.

```
Trigger: Ctrl+Z or /roll

Before undo:

┌─ Assistant ──────────────────────────────────────────────────────────┐
│ I changed the file and now tests are failing...                      │
│                                                                      │
│ ┌─ Edit ──────────────────────────────────────────────────────────┐ │
│ │ → Update src/auth.rs                                            │ │
│ │   - fn validate_expiry(expiry: i64, now: i64)                   │ │
│ │   + fn validate_expiry(expiry: i64, now: i64, strict: bool)     │ │
│ └─────────────────────────────────────────────────────────────────┘ │

After Ctrl+Z (tooltip overlay):

┌─ Rollback ───────────────────────────────────────────────────────────┐
│ ⏪ Rolled back to before "Fix auth bug" turn                         │
│   Undid: 1 Edit, 2 Bash calls                                       │
│                                                                      │
│   [Ctrl+Y] Redo  [Esc] Close                                         │
└─────────────────────────────────────────────────────────────────────┘
```

**Behavior:** Ctrl+Z in quick succession triggers a rollback dialog. Shows preview of what will be undone. Uses git to revert file changes if possible.

### Code

```rust
// crates/jcode-tui/src/history_cell/backtrack.rs

pub struct RollbackCell {
    pub undone_cells: usize, pub tool_count: usize,
    pub redos: usize,
}
impl RollbackCell {
    pub fn status_line(&self) -> String {
        format!("⏪ Rolled back {} cells ({} tools), {} redos available", self.undone_cells, self.tool_count, self.redos)
    }
}
```
---

## 51. Request User Input Overlay

Structured multi-question form displayed when the agent needs user input.

```
When agent calls question tool:

┌─────────────────────────────────────────────────────────────────────┐
│ ✋ Agent needs your input                                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   To fix the auth bug, I need:                                      │
│                                                                      │
│   1. What timeout value should I use?                                │
│      ┌────────────────────────────────┐                             │
│      │ 3000                          │                             │
│      └────────────────────────────────┘                             │
│      Options: 1000 (fast) | 3000 (balanced) | 5000 (safe)          │
│                                                                      │
│   2. Should I add more logging? (Yes/No)                             │
│      [Selected: Yes]                                                 │
│                                                                      │
│   3. Any other notes?                                                │
│      ┌────────────────────────────────┐                             │
│      │ Make sure to use env var      │                             │
│      └────────────────────────────────┘                             │
│                                                                      │
│  [Tab] next  [Shift+Tab] prev  [Enter] submit  [Esc] cancel         │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/history_cell/user_input_request.rs

pub struct UserInputRequest {
    pub question: String, pub options: Vec<String>,
    pub answer: Option<String>, pub timeout_secs: u32,
}
```
---

## 52. @-Mentions Popup

Autocomplete popup when typing `@` in composer.

```
┌─────────────────────────────────────────────────────────────────────┐
│ Fix the bug using @aut                                              │
│ ┌─ @mention ─────────────────────────────────────────────────────┐ │
│ │ 🔍 auth                                                         │ │
│ │ 📄 src/auth.rs                    file                          │ │
│ │ 📄 src/auth_test.rs               file                          │ │
│ │ 🔧 validate_expiry               symbol                        │ │
│ │ 📁 src/                           directory                    │ │
│ │ ⚡ /fix-auth                      skill                        │ │
│ │ ★ my-agent                        agent                        │ │
│ │ 🧠 auth-patterns                  memory                       │ │
│ └────────────────────────────────────────────────────────────────┘ │
│ ▌                                                                   │
└─────────────────────────────────────────────────────────────────────┘

**Mention types:**

```
@file        — fuzzy file search
@symbol      — code symbol lookup
@dir         — directory navigation
/skill       — invoke a skill
@agent       — mention an agent
@memory      — reference a context memory
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/mentions/mod.rs

pub struct MentionPopup {
    pub query: String, pub results: Vec<MentionResult>, pub selected: usize,
}
pub struct MentionResult {
    pub kind: MentionKind, pub label: String, pub detail: String,
}
pub enum MentionKind { File, Symbol, Skill, Agent, Memory, Directory }
```
---

## 53. Plan Mode

Distinct interaction mode for planning before implementing.

```
┌─────────────────────────────────────────────────────────────────────┐
│ ▌auto                                                               │
│ ▌plan  ← mode pill                                                  │
└─────────────────────────────────────────────────────────────────────┘

In Plan mode, tool calls require approval:

┌─ Assistant (Plan mode) ─────────────────────────────────────────────┐
│ Here's my plan to fix the auth bug:                                 │
│                                                                      │
│ 1. Modify `validate_expiry` to accept `now: i64` (Edit)             │
│ 2. Update all call sites (Edit × 3)                                  │
│ 3. Run tests to verify (Bash)                                        │
│                                                                      │
│ ┌─ Plan Approval ─────────────────────────────────────────────────┐ │
│ │  [y] Approve & Implement    [n] Reject    [e] Edit plan        │ │
│ └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘

Plan summary header on each turn:

```
│ 📋 Plan: Fix auth bug (3 steps, 0/3 complete)                      │
```

### Code

```rust
// crates/jcode-tui/src/modes/plan_mode.rs

pub enum PlanModeState { NotPlanning, Planning, AwaitingApproval, Implementing }

pub struct PlanMode {
    pub state: PlanModeState,
    pub plan_steps: Vec<PlanStep>,
}
pub struct PlanStep {
    pub description: String, pub done: bool, pub blocked: bool,
}
```
---

## 54. Goal / Task Tracking

Track goals across agent sessions with token budgets and status.

```
Status line indicator:

│ sonnet-4  ctx:42%  $0.12  🎯 Fix auth bug (active)                │
                             ↑ goal indicator with current goal

Goal menu (trigger: Ctrl+G or /goal):

┌─ Goal ───────────────────────────────────────────────────────────────┐
│ Current Goal                                                        │
│ ─────────────────────────────────────────────────────────────────── │
│ 🎯 Fix the auth bug                                          active │
│   Token budget: 50K used / 200K total  ████████░░░░ 25%            │
│   Duration: 12m 30s                                                 │
│                                                                      │
│   [e] edit goal text    [p] pause    [c] complete    [x] cancel    │
│ ─────────────────────────────────────────────────────────────────── │
│ Past Goals:                                                          │
│ ✓ Add keymap tests          10m 20s  ✅ completed                    │
│ ✓ Refactor structure         5m 00s  ✅ completed                    │
│ ✗ Migrate to new API         2m 00s  ❌ abandoned                    │
│                                                                      │
│ [q] close                                                           │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/goal_tracker.rs

pub struct Goal {
    pub text: String, pub tokens_used: u64, pub token_budget: u64,
    pub elapsed: std::time::Duration,
    pub status: GoalStatus,
}
pub enum GoalStatus { Active, Paused, Blocked, Complete, Cancelled }
impl Goal {
    pub fn progress_pct(&self) -> f64 { self.tokens_used as f64 / self.token_budget.max(1) as f64 }
    pub fn status_line(&self) -> String {
        let icon = match self.status { GoalStatus::Active => "🎯", GoalStatus::Paused => "⏸", GoalStatus::Complete => "✓", _ => "⚠" };
        format!("{} {} ({}%)", icon, self.text, (self.progress_pct() * 100.0) as u32)
    }
}
```
---

## 55. Turn Metrics / Worked-for Separator

Visual divider between turns showing timing and stats.

```
┌─────────────────────────────────────────────────────────────────────┐
│ ───────────── Worked for 1m 23s — 4 tool calls ──────────────────  │
└─────────────────────────────────────────────────────────────────────┘

When user scrolled up, shows timeline markers:

```
│ ───────── 2m ago ──────────────────────────────                    │
│ ───────── 5m ago ──────────────────────────────                    │
```

### Code

```rust
// crates/jcode-tui/src/history_cell/turn_metrics.rs

pub struct TurnMetricsCell {
    pub elapsed: std::time::Duration,
    pub tool_calls: usize,
}
impl TurnMetricsCell {
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let text = format!("─ Worked for {:.0}m {:.0}s — {} tool calls ─",
            self.elapsed.as_secs_f64() / 60.0, self.elapsed.as_secs_f64() % 60.0, self.tool_calls);
        buf.set_string(area.x, area.y, &text, Style::default().fg(theme.text_subtle.into()));
    }
}
```
---

## 56. Keypress Debug Inspector

Developer tool to inspect raw key events and binding resolution.

```
Trigger: /keydebug (hidden debug command)

┌─ Keypress Inspector ─────────────────────────────────────────────────┐
│ Keys pressed: 4    Matches: active contexts                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ Last Key: Ctrl+K                                                     │
│   Raw:      28                                                        │
│   C0 char:  0x0B (vertical tab)                                     │
│   Decoded:  Ctrl+K                                                    │
│   Contexts: [Global, Chat, Composer]                                 │
│   Resolved: "chat:externalEditor"  (from Chat context)              │
│   Bypassed: "global:redraw" (overridden by Chat)                    │
│                                                                      │
│ [Any key to inspect]  [c] clear  [q] close                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/key_debug.rs

pub struct KeypressDebug {
    pub active: bool,
    pub history: Vec<KeypressEvent>,
}
pub struct KeypressEvent {
    pub raw: String, pub decoded: String,
    pub context: Vec<String>, pub action: String,
}
```
---

## 57. Service Tier Selection

Choose between service tiers (Auto/Max/Balanced).

```
Trigger: Ctrl+Shift+T or /tier

┌─ Service Tier ──────────────────────────────────────────────────────┐
│ Select tier for responses:                                          │
│                                                                      │
│ ▸ Auto        Use the best tier based on complexity                  │
│   Max         Maximum intelligence (slower, more expensive)          │
│   Balanced    Middle ground                                           │
│                                                                      │
│   Currently: Auto                                                    │
│                                                                      │
│   Model: claude-sonnet-4-20250514                                    │
│                                                                      │
│ ↑/↓ navigate  Enter:select  q:close                                 │
└─────────────────────────────────────────────────────────────────────┘

Status line:

```
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto  ▌max                  │
```

### Code

```rust
// crates/jcode-tui/src/modes/service_tier.rs

pub struct ServiceTierPopup {
    pub options: Vec<&'static str>,
    pub selected: usize,
}
impl ServiceTierPopup {
    pub fn with_defaults() -> Self {
        Self { options: vec!["Auto", "Max", "Balanced"], selected: 0 }
    }
}
```
---

## 58. Raw Output Mode / Accessibility

Toggle for plain text rendering (for screen readers and accessibility).

```
Trigger: /raw or /accessibility

Normal mode:

│ ✓ exit: 0                                                           │
│ ═══░░░░░░░░░░  42%                                                  │

Raw mode:

```
| ✓ exit: 0
| [progress 42%]
```

Status line:

```
│ sonnet-4  ctx:42%  $0.12  [RAW]  ▌auto                             │
                            ↑ indicator for raw mode
```

### Code

```rust
// crates/jcode-tui/src/modes/raw_mode.rs

pub struct RawModeState { pub enabled: bool }
impl RawModeState {
    pub fn toggle(&mut self) { self.enabled = !self.enabled; }
    pub fn status_pill(&self) -> &'static str { if self.enabled { "[RAW]" } else { "" } }
}
```
---

## 59. Terminal Pets

Animated ASCII-art pet mascot rendered in the terminal.

```
┌─ Pet ───────────────────────────────────────────────────────────────┐
│                                                                      │
│                   /\_/\                                              │
│                  ( o.o )    "Working on it..."                       │
│                   > ^ <                                              │
│                                                                      │
│ [p] next pet  [q] hide                                               │
└─────────────────────────────────────────────────────────────────────┘

Available pets:

```
/\_/\      (cat)      default pet
( 0 0)    (owl)      night mode
( -_-)    (sloth)    slow thinking
(^_^)     (happy)    success animation
>(')      (fish)     swimming animation
```

Triggers: /pet, idle animation when waiting, success animation on completion.

### Code

```rust
// crates/jcode-tui/src/pets/mod.rs

pub struct TerminalPet {
    pub name: &'static str, pub active: bool,
    pub sprite_rows: Vec<&'static str>,
}
pub static PETS: &[TerminalPet] = &[
    TerminalPet { name: "cat", active: true, sprite_rows: vec!["  /\\_/\\", " ( o.o )", "  > ^ <"] },
    TerminalPet { name: "owl", active: false, sprite_rows: vec!["  ( 0 0 )", "  ( - ) "] },
    TerminalPet { name: "sloth", active: false, sprite_rows: vec!["  ( -_-)", "  (   ) "] },
];
```
---

## 60. Collaboration Modes

Switch between Plan/Ask/Agent collaboration styles.

```
Trigger: Ctrl+Shift+M or /mode

┌─ Collaboration Mode ────────────────────────────────────────────────┐
│ How should we work together?                                        │
│                                                                      │
│ ▸ Plan     I suggest → you approve → I implement (step by step)    │
│                                                                      │
│   Ask      You decide each action individually                      │
│                                                                      │
│   Agent    I work autonomously, you interrupt when needed           │
│                                                                      │
│   Currently: Agent                                                   │
│                                                                      │
│ ↑/↓ navigate  Enter:select  q:close                                 │
└─────────────────────────────────────────────────────────────────────┘

Status line:

```
│ sonnet-4  ctx:42%  $0.12  ▌auto  ▌plan                              │
                                          ↑ collaboration mode pill
```

## Appendix H: Codex Missing Features Summary
## Appendix I: Codex Deep Scan Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│ SECTION  │ FEATURE                │ SOURCE FILE                     │
├──────────┼────────────────────────┼─────────────────────────────────┤
│ #49      │ Side Conversations     │ app/side.rs                     │
│ #50      │ Backtrack / Rollback   │ app_backtrack.rs                │
│ #51      │ Request User Input     │ bottom_pane/request_user_input/ │
│ #52      │ @-Mentions Popup       │ bottom_pane/mentions_v2/        │
│ #53      │ Plan Mode              │ chatwidget/plan_implementation  │
│ #54      │ Goal Tracking          │ chatwidget/goal_menu.rs         │
│ #55      │ Turn Metrics           │ history_cell/separators.rs      │
│ #56      │ Key Debug Inspector    │ keymap_setup/debug.rs           │
│ #57      │ Service Tier           │ chatwidget/service_tiers.rs     │
│ #58      │ Raw/Accessibility Mode │ app_event.rs                    │
│ #59      │ Terminal Pets          │ pets/                           │
│ #60      │ Collaboration Modes    │ collaboration_modes.rs          │
└─────────────────────────────────────────────────────────────────────┘

Features already covered by MASTER_UI.md and NOT added:
- Hooks Browser       → Configurator (#40) covers settings
- App Link View       → Internal, not TUI-facing
- Custom Prompt View  → Chat Composer (#15) covers this
- Skills Toggle       → Plugin Manager (#41) covers
- Memory Settings     → Memory Tiles (#46) covers
- Status Line Setup   → Configurator (#40) covers
- Feedback View       → Toast (#36) covers
- Update Prompt       → Changelog (#43) covers
- CWD Prompt          → Onboarding (#28) covers
- Keybinding Remap    → Appendix B covers
- Vim Textarea        → Appendix B covers
- Personality Picker  → Configurator (#40) covers
- Experimental Views  → Experiment Popup (#48) covers
- Desktop Notify      → Toast (#36) covers
- Patch History       → Edit tool (#8) covers
```

### Code

```rust
// crates/jcode-tui/src/modes/collab_mode.rs

pub enum CollabMode { Plan, Ask, Agent }
impl CollabMode {
    pub fn cycle(&self) -> Self { match self { Self::Plan => Self::Agent, Self::Ask => Self::Plan, Self::Agent => Self::Ask } }
    pub fn pill(&self) -> &'static str { match self { Self::Plan => "▌plan", Self::Ask => "▌ask", Self::Agent => "▌agent" } }
    pub fn description(&self) -> &'static str {
        match self { Self::Plan => "I suggest → you approve → I implement", Self::Ask => "You decide each action", Self::Agent => "I work autonomously" }
    }
}
```
---

## 61. Reasoning Effort Picker

Select the model's reasoning effort level (affects thinking time and quality).

```
Trigger: Alt+, (decrease) / Alt+. (increase) or /effort

┌─ Reasoning Effort ──────────────────────────────────────────────────┐
│ Select effort level for claude-sonnet-4-20250514                   │
│                                                                      │
│ ▸ None       No reasoning (fastest, cheapest)                       │
│   Minimal    Quick reasoning                                         │
│   Low        Light reasoning                                         │
│   Medium     Balanced reasoning                                      │
│   High       Deep reasoning                                          │
│   Extra High Very deep reasoning                                     │
│   Ultra      Maximum reasoning (slowest)                             │
│   Custom     User-defined                                            │
│                                                                      │
│   Currently: Medium                                                  │
│                                                                      │
│ ↑/↓ navigate  Enter:select  Alt+,/Alt+.: adjust  q:close           │
└─────────────────────────────────────────────────────────────────────┘

Status line:

```
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto  ▌medium               │
                                            ↑ effort pill
```

### Keybindings

```
Alt+,    Decrease reasoning effort
Alt+.    Increase reasoning effort
/effort  Open effort picker
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/effort_picker.rs

pub struct ReasoningEffortPopup { pub selected: usize }
impl ReasoningEffortPopup {
    pub fn with_defaults() -> Self { Self { selected: 3 } }
    pub fn options() -> Vec<&'static str> {
        vec!["None", "Minimal", "Low", "Medium", "High", "Extra High", "Ultra"]
    }
    pub fn current_label(&self) -> &'static str { Self::options()[self.selected] }
    pub fn adjust(&mut self, delta: i8) {
        let len = Self::options().len();
        self.selected = (self.selected as i8 + delta).rem_euclid(len as i8) as usize;
    }
}
```
---

## 62. Interactive Keybinding Editor

Browse, customize, and capture keyboard shortcuts interactively.

```
Trigger: /keymap

┌─ Keyboard Shortcuts ────────────────────────────────────────────────┐
│ All     Common     Custom     Vim              [filter: ]          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ Global                                                              │
│   Ctrl+C        Interrupt                              [edit]     │
│   Ctrl+D        Exit                                    [edit]     │
│   Ctrl+L        Redraw                                  [edit]     │
│   Ctrl+O        Toggle transcript                      [edit]     │
│   Ctrl+X        Leader key                              [edit]     │
│                                                                      │
│ Chat                                                                │
│   Enter         Submit message                          [edit]     │
│   Ctrl+G        External editor                        [edit]     │
│   Ctrl+S        Stash input                             [edit]     │
│   Alt+,         Decrease reasoning effort                [edit]     │
│   Alt+.         Increase reasoning effort                [edit]     │
│                                                                      │
│ ↑/↓ navigate  Enter:edit binding  [c] capture key  [d] reset       │
│ [r] reset all to defaults  q:close                                  │
└─────────────────────────────────────────────────────────────────────┘

### Key Capture View (when editing)

```
┌─ Press a key for: "Toggle transcript" ──────────────────────────────┐
│                                                                      │
│   Current binding: Ctrl+O                                            │
│                                                                      │
│   [Press any key combination...]                                    │
│   [Esc] cancel  [Backspace] remove                                  │
│                                                                      │
│   Pressed: Ctrl+Shift+O  →  "toggle_transcript"                     │
│                                                                      │
│   [Enter] confirm  [Esc] cancel                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/keybinding_editor.rs

pub struct KeybindingEditor {
    pub entries: Vec<KeybindingEntry>,
    pub selected: usize, pub filter: String,
}
pub struct KeybindingEntry {
    pub action: String, pub binding: String,
    pub context: &'static str, pub builtin: bool,
}
```
---

## 63. Copy Agent Response (/copy)

Copy the last agent response markdown to clipboard.

```
Trigger: /copy or assignable keybinding

Before copy:

┌─ Assistant ──────────────────────────────────────────────────────────┐
│ I'll fix the auth bug. The issue is that `validate_expiry` was      │
│ called without the current timestamp.                               │
│                                                                      │
│ ┌─ Edit ──────────────────────────────────────────────────────────┐ │
│ │ → Update src/auth.rs                                            │ │
│ │   - fn validate_expiry(expiry: i64) -> bool                     │ │
│ │   + fn validate_expiry(expiry: i64, now: i64) -> bool          │ │
│ └─────────────────────────────────────────────────────────────────┘ │

After /copy:

│ ℹ Copied agent response to clipboard (1,234 chars)                  │
│   [Ctrl+V to paste]                                                 │
```

### Multiple copies

Tracks last N responses, accessible via `/copy -N`:

```
/copy      → copies most recent agent response
/copy -2   → copies 2nd most recent
/copy -3   → copies 3rd most recent
```

### Code

```rust
// crates/jcode-tui/src/actions/copy_response.rs

pub struct CopyResponseState {
    pub last_responses: Vec<String>,
    pub max_saved: usize,
}
impl CopyResponseState {
    pub fn new() -> Self { Self { last_responses: Vec::new(), max_saved: 10 } }
    pub fn push(&mut self, text: String) { self.last_responses.push(text); if self.last_responses.len() > self.max_saved { self.last_responses.remove(0); } }
    pub fn get(&self, idx: usize) -> Option<&str> { self.last_responses.iter().rev().nth(idx).map(|s| s.as_str()) }
    pub fn copy_latest(&self) -> Option<String> { self.last_responses.last().cloned() }
}
```
---

## 64. Image Paste (Ctrl+Alt+V)

Paste images from clipboard directly into the composer.

```
┌─ User ──────────────────────────────────────────────────────────────┐
│ > What's wrong with this error? [image attached]                    │
│                                                                    │
│ ┌──────────────────────────────────┐                               │
│ │  [screenshot.png - 42KB]         │                               │
│ │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                               │
│ │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                               │
│ │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                               │
│ └──────────────────────────────────┘                               │
│                                                                      │
│ ▌                                                                   │
└─────────────────────────────────────────────────────────────────────┘

Error state (model doesn't support images):

│ ⚠ Current model doesn't support image input. Image will be excluded.│
```

### Keybindings

```
Ctrl+Alt+V    Paste image from clipboard (Linux/Windows)
Ctrl+Shift+V  Paste image from clipboard (macOS)
```

### Code

```rust
// crates/jcode-tui/src/actions/image_paste.rs

pub struct ImagePasteState { pub supports_images: bool }
impl ImagePasteState {
    pub fn new() -> Self { Self { supports_images: true } }
    pub fn can_paste(&self) -> bool { self.supports_images }
    pub fn error_message(&self) -> &'static str { "⚠ Current model doesn't support image input." }
}
```
---

## 65. Terminal Title Configuration

Customize what appears in the terminal window/tab title.

```
Trigger: /title

┌─ Terminal Title Configuration ─────────────────────────────────────┐
│ "jcode — project-name — spinner — cwd"   [live preview]           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ▸ [✓] App name              jcode                                  │
│   [✓] Project name          jcode (from Cargo.toml)                │
│   [✓] Spinner animation     ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏                          │
│   [✓] Current directory     ~/Projects/jcode                      │
│   [ ] Git branch            feat/tui-redesign                      │
│   [ ] Status indicator      🎯 Fix auth bug                        │
│   [ ] Model name            sonnet-4                               │
│                                                                      │
│ [Space] toggle  ↑/↓ move  [r] reorder  [p] preview  q:close       │
└─────────────────────────────────────────────────────────────────────┘
```

### Live Preview

Bottom of the dialog shows the actual rendered title:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Terminal title will be:                                             │
│  jcode — jcode — ⠋ — ~/Projects/jcode                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/config/terminal_title.rs

pub struct TerminalTitleConfig {
    pub items: Vec<TitleItem>,
}
pub struct TitleItem {
    pub label: &'static str, pub enabled: bool,
    pub value_fn: &'static str,
}
impl TerminalTitleConfig {
    pub fn with_defaults() -> Self {
        Self { items: vec![
            TitleItem { label: "App name", enabled: true, value_fn: "jcode" },
            TitleItem { label: "Spinner", enabled: true, value_fn: "⠋" },
            TitleItem { label: "Directory", enabled: true, value_fn: "%cwd%" },
        ]}
    }
    pub fn render_title(&self, _area: Rect, buf: &mut Buffer, theme: &Theme) {
        let parts: Vec<&str> = self.items.iter().filter(|i| i.enabled).map(|i| i.value_fn).collect();
        buf.set_string(_area.x, _area.y, &parts.join(" — "), Style::default().fg(theme.text_muted.into()));
    }
}
```
---

## 66. Auto-Review Denials (/approve)

Review and retry actions that were blocked by auto-review (guardian).

```
Trigger: /approve

┌─ Auto-Review Denials ───────────────────────────────────────────────┐
│ Recently denied actions                            [filter: ]       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ▸ ✗ Denied: Bash command "rm -rf /tmp"                             │
│    2m ago  │ Reason: Destructive operation                          │
│    [Approve retry]  [View details]                                  │
│                                                                      │
│   ✗ Denied: File Edit "chmod 777"                                  │
│    5m ago  │ Reason: Security risk                                   │
│    [Approve retry]  [View details]                                  │
│                                                                      │
│   ✓ Approved: File Write "create test.rs"                          │
│    10m ago │ Reason: Safe operation                                  │
│                                                                      │
│ ↑/↓ navigate  Enter:retry  [v] view  q:close                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/auto_review.rs

pub struct AutoReviewEntry {
    pub action: String, pub reason: String,
    pub retry_available: bool, pub time_ago: String,
}
pub struct AutoReviewPopup { pub entries: Vec<AutoReviewEntry>, pub selected: usize }
```
---

## 67. Desktop Notifications

Terminal-based desktop notifications for key events.

```
Terminal title flash:

┌─ jcode — ⠋ — ✗ Command blocked — ~/Projects ─────────────────────┐
                          ↑ notification shown in title

OSC 9 escape sequence (supported by Ghostty, iTerm2, Kitty, Warp, WezTerm):

```
\x1b]9;Command blocked by auto-review\x07
```

### Notification Types

```
Event                         | Type    | Preview text
──────────────────────────────┼─────────┼─────────────────────────
Agent turn completes          | success | "Task complete: Fixed auth bug"
Approval request pending      | action  | "🔐 Permission needed: rm -rf"
Auto-review denial            | warning | "✗ Command blocked by auto-review"
Plan mode prompt              | prompt  | "📋 Plan ready for review"
Turn starts                   | info    | "Working on: Fix auth bug..."
```

### Configuration

```
~/.jcode/config.toml:

[notifications]
enabled = true
agent_completion = true
approval_requests = true
auto_review_denials = true
plan_mode = true
```

### Code

```rust
// crates/jcode-tui/src/notifications/mod.rs

pub struct DesktopNotification {
    pub message: String,
    pub kind: &'static str,
    pub icon: &'static str,
}
impl DesktopNotification {
    pub fn send(&self, _theme: &Theme) {
        // Uses OSC 9 escape sequence for Ghostty/iTerm2/Kitty/Warp/WezTerm
        print!("\x1b]9;{} {}{}\x07", self.icon, self.message, self.kind);
    }
}
pub fn notify_success(msg: String) { DesktopNotification { message: msg.clone(), kind: "complete", icon: "✓" }.send(); }
pub fn notify_warning(msg: String) { DesktopNotification { message: msg.clone(), kind: "warning", icon: "⚠" }.send(); }
pub fn notify_error(msg: String)   { DesktopNotification { message: msg.clone(), kind: "error", icon: "✗" }.send(); }
```
---

## 68. Code Review Setup (/review)

Select review scope before running code review.

```
Trigger: /review

┌─ Code Review ────────────────────────────────────────────────────────┐
│ Select review target:                                               │
│                                                                      │
│ ▸ Review against base branch (PR style)                             │
│   Review uncommitted changes                                        │
│   Review a specific commit                                          │
│   Custom review (enter instructions)                                │
│                                                                      │
│ [Enter] select  q:close                                             │
└─────────────────────────────────────────────────────────────────────┘

### Branch Picker (after selecting PR style)

```
┌─ Select base branch ────────────────────────────────────────────────┐
│ [filter: ma]                                                        │
│                                                                      │
│ ▸ main                                                              │
│   master                                                             │
│   feat/tui-redesign                                                  │
│                                                                      │
│ ↑/↓ navigate  Enter:select  /:filter                                │
└─────────────────────────────────────────────────────────────────────┘

### Custom Review Instructions

```
┌─ Custom Review Input ───────────────────────────────────────────────┐
│ Review instructions:                                                │
│                                                                      │
│ Focus on:                                                            │
│ - Security vulnerabilities                                           │
│ - Thread safety issues                                               │
│ - Error handling patterns                                            │
│ ▌                                                                   │
│                                                                      │
│ [Enter] submit  [Esc] cancel  [Ctrl+O] external editor              │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/review_setup.rs

pub struct ReviewSetup {
    pub targets: Vec<ReviewTarget>,
    pub selected: usize,
}
pub enum ReviewTarget {
    BranchBase(String),
    Uncommitted,
    SpecificCommit(String),
    Custom(String),
}
```
---

## 69. Model Migration Dialog

Prompt when the app server recommends switching to a newer model.

```
Shown on startup or when model availability changes:

┌─────────────────────────────────────────────────────────────────────┐
│ 🚀 New Model Available!                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   claude-sonnet-4-20250514 is now available and recommended.        │
│                                                                      │
│   It offers:                                                         │
│   • 2x faster response times                                         │
│   • Better code generation quality                                   │
│   • Lower cost per token                                             │
│                                                                      │
│   Your current default:                                              │
│   claude-sonnet-4-20250401                                          │
│                                                                      │
│   (y) Switch now    (n) Keep current    (d) Don't ask again         │
│   (v) View release notes                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/model_migration.rs

pub struct ModelMigrationDialog {
    pub new_model: String, pub old_model: String,
    pub selected: usize,
    pub benefits: Vec<String>,
}
```
---

## 70. Personality Picker

Choose the model's communication style.

```
Trigger: /personality

┌─ Communication Style ───────────────────────────────────────────────┐
│ Select how the assistant communicates:                              │
│                                                                      │
│ ▸ Friendly    Warm, conversational, explains reasoning              │
│               "Let me break this down..."                           │
│                                                                      │
│   Pragmatic   Direct, concise, code-first                           │
│               "Fix: add `now` param to validate_expiry"              │
│                                                                      │
│   Currently: Friendly                                                │
│                                                                      │
│ ↑/↓ navigate  Enter:select  q:close                                 │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/bottom_pane/personality_picker.rs

pub struct PersonalityPicker { pub selected: usize }
impl PersonalityPicker {
    pub fn options() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Friendly", "Warm, conversational, explains reasoning"),
            ("Pragmatic", "Direct, concise, code-first"),
        ]
    }
}
```
---

## 71. IDE Context Integration (/ide)

Toggle injection of IDE context (active file, selection, open tabs) into messages.

```
Trigger: /ide

Status line:

```
│ sonnet-4  ctx:42%  $0.12  cache:78%  ▌auto  🖮 IDE                │
                                            ↑ IDE active indicator

When enabled, injects context from VS Code / other IDE:

```
│ ℹ Injecting IDE context: src/auth.rs:42-56                         │
│   (from VS Code — active selection)                                │
```

### Supported IDEs

```
VS Code (via extension IPC)
Cursor
JetBrains (via plugin)
```
### Code

```rust
// crates/jcode-tui/src/integrations/ide_context.rs

pub struct IdeContext {
    pub enabled: bool,
    pub active_file: Option<String>,
    pub selection: Option<(u32, u32)>,
}
impl IdeContext {
    pub fn status_pill(&self) -> &'static str { if self.enabled { "🖮 IDE" } else { "" } }
}
```
---

## 72. Plan Mode Nudge

Footer hint that appears when planning keywords are detected but Plan mode is off.

```
When user types "plan", "implement", "steps" while in Default mode:

┌─────────────────────────────────────────────────────────────────────┐
│ 💡 Looks like you're planning! Press Ctrl+Shift+M to switch to      │
│    Plan mode where you can review before implementing.             │
│    [dismiss]  [never show]                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Code

```rust
// crates/jcode-tui/src/modes/plan_nudge.rs

pub struct PlanModeNudge {
    pub visible: bool,
    pub dismissed: bool,
    pub never_show: bool,
}
impl PlanModeNudge {
    pub fn should_show(&self, text: &str) -> bool {
        !self.dismissed && !self.never_show && (text.contains("plan") || text.contains("steps") || text.contains("implement"))
    }
    pub fn dismiss(&mut self) { self.dismissed = true; }
    pub fn never_show_again(&mut self) { self.never_show = true; }
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if !self.visible { return; }
        buf.set_string(area.x, area.y, "💡 Looks like you're planning! Press Ctrl+Shift+M for Plan mode. [dismiss]",
            Style::default().fg(theme.info.into()));
    }
}
```
---

## 73. Safety Buffering Status

Status indicator when safety checks are running.

```
During safety check:

│ ⏳ Running safety checks...  (parallel reviews aggregating)        │
│   🔍 Content analysis...  ✅ Code diff review...  ⏳ Policy...     │

When complete:

│ ✅ Safety checks passed (N/3 checks completed in 1.2s)              │

If blocked:

│ ✗ Content blocked by safety policy                                 │
│   Reason: This content cannot be shown.                             │
│   [Retry with faster model]  [View details]                        │
```

### Code

```rust
// crates/jcode-tui/src/history_cell/safety_buffering.rs

use ratatui::buffer::Buffer; use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style}; use ratatui::text::{Line, Span};
use jcode_tui_style::Theme; use crate::history_cell::HistoryCell;

pub struct SafetyStatusCell {
    pub in_progress: bool,
    pub checks_passed: u32, pub total_checks: u32,
    pub blocked: bool, pub blocked_reason: Option<String>,
}
impl HistoryCell for SafetyStatusCell {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if self.blocked {
            buf.set_line(area.x, area.y, &Line::from(vec![
                Span::styled("✗ Safety checks failed", Style::default().fg(theme.error.into()).add_modifier(Modifier::BOLD)),
            ]), area.width);
            if let Some(reason) = &self.blocked_reason {
                buf.set_line(area.x, area.y + 1, &Line::from(vec![Span::styled(reason.as_str(), Style::default().fg(theme.text_muted.into()))]), area.width);
            }
        } else if self.in_progress {
            buf.set_line(area.x, area.y, &Line::from(vec![
                Span::styled("⏳ Running safety checks...", Style::default().fg(theme.warning.into())),
            ]), area.width);
        } else {
            buf.set_line(area.x, area.y, &Line::from(vec![
                Span::styled("✓ Safety checks passed", Style::default().fg(theme.success.into())),
                Span::styled(format!(" ({}/{})", self.checks_passed, self.total_checks), Style::default().fg(theme.text_subtle.into())),
            ]), area.width);
        }
    }
    fn desired_height(&self, _: u16) -> u16 { if self.blocked && self.blocked_reason.is_some() { 2 } else { 1 } }
}
```
---

## Appendix I: Codex Deep Scan Summary

### New sections added in this pass

```
┌────────┬──────────────────────────────────────┬──────────────────────────┐
│ #61    │ Reasoning Effort Picker              │ chatwidget/              │
│ #62    │ Interactive Keybinding Editor        │ keymap_setup/            │
│ #63    │ Copy Agent Response (/copy)          │ chatwidget/              │
│ #64    │ Image Paste (Ctrl+Alt+V)             │ chatwidget/              │
│ #65    │ Terminal Title Configuration (/title)│ bottom_pane/             │
│ #66    │ Auto-Review Denials (/approve)       │ chatwidget/              │
│ #67    │ Desktop Notifications                │ notifications.rs         │
│ #68    │ Code Review Setup (/review)          │ chatwidget/              │
│ #69    │ Model Migration Dialog               │ model_migration.rs       │
│ #70    │ Personality Picker                   │ chatwidget/              │
│ #71    │ IDE Context Integration (/ide)       │ ide_context.rs           │
│ #72    │ Plan Mode Nudge                      │ chatwidget/              │
│ #73    │ Safety Buffering Status              │ chatwidget/              │
└────────┴──────────────────────────────────────┴──────────────────────────┘
```

### Coverage summary

```
Codex Features     │ In Spec  │ New Now   │ Total
───────────────────┼──────────┼───────────┼───────────
Core UI components │ 60       │ 13        │ 73
Infrastructure     │ 0        │ 0         │ 4
───────────────────┼──────────┼───────────┼───────────
Total              │ 60       │ 13        │ ~300+
```
