# Deepen — Plugin manifest ABI v1 (2026-07-22)

**Priority:** P2 · Phase 2  
**Parent:** Master plan Phase 2 + package layout sketch  
**Open Q (master):** `next-code-plugin.toml` vs extend Grok `plugin.json`?

---

## Recommendation (docs stance — not frozen filename yet)

Prefer **new** `next-code-plugin.toml` for executable workflow clarity (herdr-shaped). Keep `plugin.json` for content-bundle discovery during transition; dual-read OK in Phase 2.

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

`[[tools]]` / `[[ui]]` — Phase 3–4 (see sibling deepens).

## HOW

Runners: argv stdio | HTTP (map to existing hook execute). TS/JS = optional argv (`node dist/hook.js`), equal to Python/Bash.

## Validation

- Reject unknown `on` events.
- Enforce `min_next_code_version`.
- Trust + enable before argv.

## Exit criteria

- [ ] Schema doc + example packages `examples/ext-{bash,python,node}`.
- [ ] Third party ships hooks+slash+skills without core PR.
