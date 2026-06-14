# Status Bar — Implementation Plan
> Research across Codex, Claude Code, oh-my-pi, OpenCode

## Goal

Rewrite jcode's footer area (currently shows only processing status / occasional tips) into a persistent configuration status bar showing model, provider, permission mode, git branch, context usage, tokens — like Codex, Claude Code, and oh-my-pi.

## Current Issue

jcode's footer shows NOTHING when idle (no processing, no rate limit):

```
# blank line
# [input line here]
```

While Codex shows:
```
model: gpt-5.4 · /Users/me/project (main) · ⚡ 35% · ↑12K ↓8K
```

## Cross-Repo Patterns

| Aspect | Codex | Claude Code | oh-my-pi | Best Practice |
|--------|-------|-------------|----------|---------------|
| **Configurable** | ✅ `/statusline` UI | ✅ `/statusline` | ✅ settings.json preset | **Configurable** with presets |
| **Model** | ✅ model | ✅ model | ✅ model (Claude stripped) | **All show model** |
| **Git branch** | ✅ | ❌ | ✅ | **Codex/oh-my-pi** |
| **Context %** | ✅ | ✅ | ✅ | **All show context** |
| **Tokens** | ✅ input/output | ✅ used/window | ✅ total/cache | **Codex** format |
| **Session limit** | ✅ 5h/weekly | ✅ 5h/weekly | ✅ | **Codex/Claude Code** |
| **Cost** | ❌ | ✅ $0.00 | ✅ | **Claude Code** |
| **Mode (permissions)** | ✅ | ✅ | ✅ plan/goal/loop | **Add permission mode** |
| **Directory** | ✅ pwd | ❌ | ✅ pwd | **oh-my-pi** |
| **Interactive** | `/statusline` popup | `/statusline` popup | settings | **Follow Codex** |

## Design

### Layout (single row, like Codex)

```
⊘ bypass · model: deepseek-v4-flash · provider: OpenCode Go · ~/project (main) · 35% · ↑12K ↓8K · ⚠ 73% session
```

Color-coded segments separated by ` · ` (middle dot) in dim color:
- **Permission mode icon**: ⊘ bypass, 🔒 default, ✏ accept-edits, 📋 plan, 🤖 auto
- **Model**: bright/colored based on provider family (Anthropic=purple, OpenAI=green, etc.)
- **Provider**: dim
- **Directory + git branch**: dim
- **Context usage %**: normal -> yellow -> red as it fills
- **Tokens (optional)**: dim
- **Session limit (optional)**: yellow/red when approaching

### Status Notice (transient)

When there's an active `status_notice` (e.g., "Model → deepseek-v4-flash"), it REPLACES the status bar temporarily (3 second timeout), then fades back to the persistent status line.

### Implementation Phases

#### Phase A: Add persistent status line (1-2 hours)
1. Add `status_line` method to `TuiState` trait: returns formatted `Line` with model + provider + mode + context
2. In `draw_status` (ui_input.rs:540), when idle AND no `status_notice`, render the persistent status line instead of empty/tips
3. Format: `⊘ bypass · model-deepseek-v4-flash · provider: opencode · context: 35%`

#### Phase B: Add git/directory info (1 hour)
1. Add `git_branch()` and `cwd_label()` to `TuiState`
2. Add git branch detection (async, read .git/HEAD)
3. Display: `⊘ bypass · model · provider · ~/project (main) · 35%`

#### Phase C: Configurable status line (2-3 hours)
1. Add `status_line` config field (vec of segment names)
2. `/statusline` command to show/edit current line
3. Default: mode, model, provider, dir, context, tokens
4. Segments: mode, model, provider, dir, git, context, tokens_in, tokens_out, cost, session_limit, version

#### Phase D: Interactive status line setup (3-4 hours)
1. Add a popup/overlay for selecting/reordering status line items
2. Match Codex's `/statusline setup` UX with arrow navigation
3. Preview changes in real time

### Files to change

| File | Change |
|------|--------|
| `crates/jcode-tui/src/tui/mod.rs` | Add `status_line()`, `git_branch()`, `cwd_label()` to TuiState trait |
| `crates/jcode-tui/src/tui/app/tui_state.rs` | Implement new trait methods |
| `crates/jcode-tui/src/tui/ui_input.rs` | Rewrite `draw_status` for persistent status line |
| `crates/jcode-config-types/src/lib.rs` | Add `StatusLineConfig` with segments, separtator |
| `crates/jcode-tui/src/tui/app/commands.rs` | Add `/statusline` command |
| NEW: `crates/jcode-tui/src/tui/status_line.rs` | Status line segments definition and rendering |
| NEW: `crates/jcode-tui/src/tui/status_line_setup.rs` | Interactive setup popup |

### Reference Code

| Aspect | Repo | File | Lines |
|--------|------|------|-------|
| Status line setup | codex | `tui/src/bottom_pane/status_line_setup.rs` | 55-142 |
| Status segments | oh-my-pi | `packages/coding-agent/src/modes/components/status-line/segments.ts` | 66-593 |
| Footer rendering | codex | `tui/src/bottom_pane/footer.rs` | 1-120 |
| Builtin status | claude-code | `src/components/BuiltinStatusLine.tsx` | 45-128 |
| Status line config | oh-my-pi | `packages/coding-agent/src/modes/components/status-line/presets.ts` | 3-102 |
