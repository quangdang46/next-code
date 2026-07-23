# Deepen — 3-language PreToolUse cookbook layout (2026-07-22)

**Priority:** P1 · Phase 1 vertical slice  
**Parent:** Master plan Phase 1 “smallest vertical slice”  
**Protocol:** `docs/HOOKS.md` + `next-code-hooks` stdin JSON → stdout; exit `0` allow / `2` deny

---

## Goal

Prove multilang **without** waiting for `next-code-plugin.toml`: same policy in Bash, Python, Node against current hooks execute path.

## Suggested layout (examples — not production pack yet)

```text
examples/hooks-cookbook-pretool/
  README.md                 # how to wire hooks.toml
  policy.env.example
  bash/pre_tool_use.sh
  python/pre_tool_use.py
  node/pre_tool_use.mjs
```

## Shared policy (example)

Deny `Bash` when command matches a denylist (e.g. `rm -rf /`, curl|sh). Otherwise allow. Identical semantics across three runners.

## hooks.toml snippets

```toml
[[events.PreToolUse]]
type = "command"
enabled = true
command = ["bash", "examples/hooks-cookbook-pretool/bash/pre_tool_use.sh"]
matcher = "Bash"

# swap command for python3 …/pre_tool_use.py or node …/pre_tool_use.mjs
```

## README must show

1. HookInput fields read (tool name, input).
2. HookOutput / exit codes.
3. Windows notes (PATHEXT, `py -3`, `node`).
4. Link to trust gate — project examples need trust.

## Exit criteria

- [ ] Three runners; one documented test matrix (allow + deny).
- [ ] No TypeScript-in-process path required.
- [ ] Cited from master plan Phase 1 exit.
