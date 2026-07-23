# Deepen — Face UI hints vs external pane (2026-07-22)

**Priority:** P2 · Phase 4  
**Parent:** Master plan Phase 4; [Face limits](../20260722-face-customization-limits.md)  
**Open Q:** external plugin pane vs Face-hints-only?

---

## Frozen non-goal

No Face plugin-host / Pi `ctx.ui.custom` / Doom-in-Face / QuickJS guest UI.

## Option A — Hints only (default recommend)

Package `[[ui]]` → string/status ids → fixed Face slots via ACP (footer/notify/tool display hints).

## Option B — External pane (optional)

Spawn user TUI binary (herdr-like pane) **outside** Face; Face stays sealed. Higher UX cost; only if product wants it.

## Exit criteria

- [ ] Open Q decided in writing before Phase 4 code.
- [ ] If A: one status hint end-to-end.
- [ ] If B: spawn + lifecycle + security (argv trust) specified.
