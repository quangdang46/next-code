# MASTER_UI.md — jcode TUI Redesign Specification
> Full UI/UX spec with ASCII mockups + UX descriptions for every feature
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
