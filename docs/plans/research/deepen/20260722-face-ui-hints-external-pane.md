# Deepen — Face UI hints vs external pane (2026-07-22)

**ID:** D12 · **Priority:** P2 · Phase 4  
**Status:** Design sketch (docs only — open Q unresolved)  
**Parent:** Master plan Phase 4; [Face limits](../20260722-face-customization-limits.md)  
**Open Q (master #3):** external plugin pane vs Face-hints-only?  
**Siblings:** D8 manifest `[[ui]]`, D11 tools (no custom Components), D13 argv security

---

## Summary (read first)

Phase 4 customization must **not** resurrect Face plugin-host / Pi `ctx.ui.custom` / Doom-in-Face / QuickJS guest UI. Two product options remain: **hints into fixed Face slots** (default recommend) or **external pane** (spawn user’s TUI outside Face).

Owner must pick before Phase 4 code.

---

## Frozen non-goal

| Forbidden | Why |
|-----------|-----|
| Face plugin-host | Face sealed; migration forbids |
| Pi `ctx.ui.custom` Components | In-process UI kit |
| QuickJS / Bun guest UI | #49 class risks |
| Custom tool `renderCall` Components | Face limits §2.4 |

Tool cards stay first-party paint from ACP data (Phase 3 needs no Face fork — D11).

---

## Option A — Hints only (default recommend)

Package `[[ui]]` (or ACP methods) → string/status ids → **fixed Face slots** via ACP (footer / notify / tool display hints).

| Property | Value |
|----------|-------|
| Face changes | Small: map known hint ids → existing widgets |
| Security | Low — strings only |
| Author power | Limited; predictable |
| Fits sealed Face | Yes |

### Sketch

```toml
[[ui]]
slot = "footer_status"
text = "Deploy: idle"

[[ui]]
slot = "notify"
level = "info"
text = "Hooks reloaded"
```

Host validates slot ∈ allowlist; rejects unknown slots; truncates length.

---

## Option B — External pane (optional)

Spawn user TUI binary (herdr-like pane) **outside** Face; Face stays sealed.

| Property | Value |
|----------|-------|
| Face changes | Minimal (launch/lifecycle UX) |
| Security | High — argv spawn; needs D1 + D13 |
| Author power | Full TUI in separate process |
| UX cost | Window management, focus, lifecycle |

### Requirements if chosen

1. Manifest `[[ui]] kind = "external_pane"` with `runner.argv` array.
2. Trust + path confinement before spawn.
3. Lifecycle: open / close / crash → Face status.
4. No stdin bridge that injects into Face render tree.
5. Document Windows/macOS/Linux window behavior honestly.

---

## Decision matrix

| Criterion | A Hints | B External |
|-----------|---------|------------|
| Time to ship | Faster | Slower |
| Pi surface parity | Partial | Closer for “custom UI” |
| RCE surface | Low | High (mitigated by trust) |
| Face integrity | Strongest | Strong (out-of-process) |

**Docs recommendation:** Ship **A** first; add **B** only if product explicitly wants herdr-like panes.

---

## What Phase 4 must not promise

- “Packages can draw anything in Face.”
- Theme token files without first-party Face mapping.
- In-session `registerUi` without reload (prefer manifest + reload).

---

## Acceptance tests (design — after open Q)

### If A

| ID | Pass |
|----|------|
| UI-A1 | One status hint end-to-end via ACP |
| UI-A2 | Unknown slot rejected |
| UI-A3 | Disable package clears hint |

### If B

| ID | Pass |
|----|------|
| UI-B1 | Spawn + visible external process |
| UI-B2 | Untrusted project skips spawn |
| UI-B3 | Crash surfaces error in Face without Face panic |

---

## Exit criteria

- [ ] Open Q decided in writing before Phase 4 code.
- [ ] If A: one status hint end-to-end.
- [ ] If B: spawn + lifecycle + security (argv trust) specified.
- [ ] Non-goal list still true in product docs.

---

## Non-goals (always)

- Doom-in-Face / guest Component host.
- Replacing Face with OpenTUI.
- Custom tool Components for Phase 3 tools.

---

## Status

**Sketch only — master open Q #3 open.** Default recommend **Option A**. Waiting owner decision + **go ahead** before Phase 4 Rust.
