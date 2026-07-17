# Agent tree — one-shot manual test (after install)

**Binary:** `~/.next-code/builds/current/jcode` (expect hash matching latest `feat/agent-tree`)  
**Serve:** must map that same binary (`bash scripts/restart_local_serve.sh` if unsure)

## Prep

```bash
cd /path/to/jcode
bash scripts/install_release.sh --fast   # if not just installed
bash scripts/restart_local_serve.sh
# Open a *new* jcode client (quit old TUI fully)
```

In view chrome you should see a short **git hash** (e.g. `a5c727c25`).

## Script (one pass)

| # | Action | Expect |
|---|--------|--------|
| 1 | Prompt lead to spawn **1** agent that runs ~30–60s and prints progress | Tree under input: `team-lead` + `@agent` |
| 2 | Scroll chat to **bottom**, input empty, press **`↓`** | Selection mode (`>` pointer) |
| 3 | **`↓`** until agent selected | Hint: enter = view · shift+enter full session |
| 4 | **`Enter`** | Soft view: header `Viewing @…` or `Status preview`, transcript from buffer/tail; **not** stuck; esc path visible |
| 5 | Wait for stream / tools | Soft content updates; tree may show preview line under agent |
| 6 | **`Esc`** | Back to lead transcript instantly |
| 7 | Select agent again → **`Shift+Enter`** | Hard attach: full child session history |
| 8 | Header shows Viewing + hash; tree still has **team-lead** | Free switch chrome |
| 9 | **`Esc`** or select team-lead + **Enter** | Back on lead |
| 10 | Soft view, type short message + Enter | DM/notify to agent (or clear error) |
| 11 | Select agent, **`k`** while running | Stop notice; agent leaves tree eventually |
| 12 | Two agents (optional) | A→B soft switch without losing path home |

## Fail if

- No way back after Enter (no esc chrome / Esc no-op)  
- Soft shows only spawn meta-prompt spam  
- After hard attach, tree gone and Esc does nothing  
- Binary hash old / serve not reloaded  

## Keys cheat sheet

| Key | Meaning |
|-----|---------|
| `↓` at bottom / `Shift+↑↓` | Select agents |
| `Enter` / `f` | Soft view (live buffer when available) |
| `Shift+Enter` | Full child session (hard) |
| `Esc` | Exit soft / return to lead from hard |
| `k` | Stop selected agent |
| Enter on **team-lead** | Exit view / return lead |
