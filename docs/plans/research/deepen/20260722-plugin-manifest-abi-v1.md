# Deepen — Plugin manifest ABI v1 (2026-07-22)

**ID:** D8 · **Priority:** P2 · Phase 2  
**Status:** Design sketch (docs only — filename **not** frozen)  
**Parent:** Master plan Phase 2 + package layout sketch  
**Open Q (master #1):** `next-code-plugin.toml` vs extend Grok `plugin.json`?  
**Siblings:** D9 slash, D11 tools, D4 merge, D13 security, D6 BIN_PATH

---

## Summary (read first)

Phase 2 introduces a **declarative package unit**: one manifest declares hooks, slash, skills, MCP resources; runners are **argv/HTTP** (herdr-shaped), not in-process TS. This file freezes the **shape** of declarations; the **filename** remains an open Q.

---

## Recommendation (docs stance — not frozen filename yet)

| Option | Pros | Cons |
|--------|------|------|
| **New `next-code-plugin.toml`** | Clear executable workflow; herdr-like | Dual-read with `plugin.json` during transition |
| Extend Grok `plugin.json` | One file discovery | JSON awkward for argv tables; conflates content bundle with exe ABI |

**Prefer new `next-code-plugin.toml`** for executable clarity. Keep `plugin.json` for content-bundle discovery during transition; **dual-read OK** in Phase 2.

---

## Declares WHAT (v1)

```toml
id = "example.policy"
version = "0.1.0"
min_next_code_version = "…"

[[hooks]]
on = "PreToolUse"
matcher = "Bash"
runner = { argv = ["python3", "hooks/pre_tool.py"] }

[[slash]]
name = "review"
kind = "prompt"   # or command + argv

[[resources]]
skills = ["skills"]
prompts = ["prompts"]
mcp = [".mcp.json"]
```

`[[tools]]` / `[[ui]]` — Phase 3–4 (see D11 / D12).

### Field notes

| Field | Rule |
|-------|------|
| `id` | Stable; used by plugins-state + provenance |
| `min_next_code_version` | Reject package if host older (herdr pattern) |
| `runner.argv` | **Array only** — no shell-string (D13) |
| Paths | Relative to plugin root; reject `..` escape |

---

## HOW (runners)

| Mode | Maps to |
|------|---------|
| argv stdio | Existing hook execute / future tool spawn |
| HTTP | Existing `type = http` hook path |
| TS/JS | Optional `node dist/hook.js` — **equal** to Python/Bash, not privileged |

Compile `[[hooks]]` into `next-code-hooks` (same registry as D4/D7). Do not invent a parallel bus.

---

## Validation (link / enable time)

- Reject unknown `on` events.
- Enforce `min_next_code_version`.
- Trust (D1) + enable (D2) before argv spawn.
- Schema validate required fields per entry kind.
- Collision: duplicate `id` install → reject or replace with explicit UX.

---

## Package layout sketch

```text
example.policy/
  next-code-plugin.toml   # or plugin.json during transition
  hooks/
  skills/
  prompts/
  .mcp.json
  bin/                    # optional argv targets
```

Official examples when implementing: `examples/ext-bash`, `examples/ext-python`, `examples/ext-node`.

---

## Transition from Grok bundles

| Legacy | Phase 2 |
|--------|---------|
| `plugin.json` + dirs | Still discoverable |
| `hooks/hooks.json` | Parser → same compile as `[[hooks]]` |
| `.mcp.json` | Via `[[resources]]` or legacy merge (D4) |

One compile pipeline; two front-end parsers.

---

## Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| MF-01 | Valid manifest enable | Hooks+skills load |
| MF-02 | Unknown event | Reject or skip with log (prefer reject on link) |
| MF-03 | Path escape in argv | Reject |
| MF-04 | min version too new | Reject |
| MF-05 | Third-party shaped package | No core PR for hooks+slash+skills |

---

## Exit criteria

- [ ] Schema doc + example packages `examples/ext-{bash,python,node}`.
- [ ] Third party ships hooks+slash+skills without core PR.
- [ ] Filename decided in writing (master open Q #1) before calling ABI “frozen.”
- [ ] Dual-read plan documented if both names supported.

---

## Explicit non-goals

- In-process `registerTool` / Face Components.
- Marketplace publishing protocol (later).
- Freezing filename in this deepen before owner picks open Q #1.

---

## Open questions

1. **Filename** — master open Q #1 (blocking for “frozen ABI”).
2. Whether `[[build]]` steps exist in v1 (herdr has build; next-code may defer).
3. Exact HTTP runner fields parity with hooks.toml.

---

## Status

**Shape ready; filename open.** Waiting **go ahead** + open Q resolution before production Rust.
