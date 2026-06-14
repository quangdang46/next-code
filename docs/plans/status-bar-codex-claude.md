# Status Bar — Codex + Claude Code Hybrid
> Combined approach: Codex's `/statusline setup` configurable segments + Claude Code's built-in + custom shell command

## Architecture (3 layers)

```
┌──────────────────────────────────────────────────────────────┐
│ Layer 1: Built-in (default, always on)                       │
│ ⊘ bypass · deepseek-v4-flash · OpenCode Go · 35% · ↑12K ↓8K│
├──────────────────────────────────────────────────────────────┤
│ Layer 2: User-configured segments (/statusline setup)        │
│ same format, but user picks order + which segments to show   │
├──────────────────────────────────────────────────────────────┤
│ Layer 3: Custom shell command (Claude Code style)            │
│ user writes any shell script → jcode pipes JSON via stdin    │
│ → output gets rendered below input bar                       │
└──────────────────────────────────────────────────────────────┘
```

### Layer 1 — Built-in Default (jcode `draw_status` rewrite)

Always visible when idle. Segments:

| Segment | Example | Color |
|---------|---------|-------|
| Permission mode | `⊘ bypass` `🔒 default` `✏ accept-edits` | accent (yellow/blue/green) |
| Model | `deepseek-v4-flash` | provider color (Anthropic=#D97757, OpenAI=#10A37F, etc.) |
| Provider | `OpenCode Go` | dim |
| Context % | `35%` | normal, yellow (>70%), red (>90%) |
| Tokens | `↑12K ↓8K` | dim |

Separator: ` · ` (middle dot, dim). Transient `status_notice` replaces the bar for 3s.

### Layer 2 — `/statusline setup` (Codex pattern)

User runs `/statusline setup` → opens interactive segment picker:

```
┌─── Status Line Setup ─────────────────────────────┐
│  🔍 filter...                                      │
│                                                    │
│  [x] Mode (permission)           ↑ move up         │
│  [x] Model                       ↓ move down       │
│  [x] Provider                    Space toggle      │
│  [x] Context %                   Enter done         │
│  [ ] Git branch                                     │
│  [ ] Directory                                      │
│  [ ] Tokens in/out                                  │
│  [ ] Session limit (5h/7d)                         │
│  [ ] Cost ($)                                       │
│  [ ] Cache hit rate                                 │
│                                                    │
│  Preview: ⊘ bypass · deepseek · 35% · ↑12K ↓8K    │
└────────────────────────────────────────────────────┘
```

Segments stored in config:
```toml
[status_line]
enabled = true
segments = ["mode", "model", "provider", "context", "tokens", "git"]
```

### Layer 3 — Custom Shell Command (Claude Code pattern)

Advanced users set a shell command in config:
```toml
[status_line]
command = "~/.jcode/statusline.sh"
```

jcode runs the command every 5s, passes this JSON via stdin:
```json
{
  "model": "deepseek-v4-flash",
  "provider": "OpenCode Go",
  "mode": "bypass-permissions",
  "context_pct": 35,
  "tokens_in": 12000,
  "tokens_out": 8000,
  "session_remaining_secs": 18000,
  "git_branch": "main",
  "cwd": "/Users/me/project"
}
```

Command stdout (single line) gets rendered as status bar. Falls back to Layer 2 on error.

### Implementation Plan

| Phase | Files | What |
|-------|-------|------|
| **A** | `ui_input.rs`, `TuiState` trait, `tui_state.rs` | Layer 1: rewrite draw_status to show model + provider + mode + context + tokens when idle |
| **B** | `StatusLineConfig` (config-types), `commands.rs` | Layer 2: status line segments config + `/statusline` command |
| **C** | `status_line_ui.rs` (new) | Layer 2: interactive `/statusline setup` popup |
| **D** | `status_line.rs` (new) | Layer 3: shell command runner with JSON stdin, fallback |
| **E** | `default_file.rs` | Default config template for `[status_line]` |

### Reference Files

| Repo | File | What to copy |
|------|------|-------------|
| codex | `tui/src/bottom_pane/status_line_setup.rs` | Segment picker UI pattern |
| codex | `tui/src/bottom_pane/footer.rs` | Footer rendering with context, tokens |
| claude-code | `src/components/BuiltinStatusLine.tsx` | Built-in status data collection |
| claude-code | `src/utils/hooks.ts:executeStatusLineCommand` | Shell command runner pattern |
| claude-code | `src/types/statusLine.ts` | StatusLineCommandInput JSON schema |
