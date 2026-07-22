# Deep research — Pi surfaces × herdr multilang (2026-07-22)

Supporting evidence for the master plan:

**[`../PLAN-20260722-pi-full-custom-platform.md`](../PLAN-20260722-pi-full-custom-platform.md)**

All five reports are research-only (no production code). Read the master plan first; use these for citations and depth.

| # | File | Question answered |
|---|------|-------------------|
| 1 | [`20260722-pi-extension-surfaces.md`](./20260722-pi-extension-surfaces.md) | What makes Pi “dynamic”? Exhaustive extension surface catalog (hooks, slash, tools, skills, packages, providers, TUI, trust). |
| 2 | [`20260722-herdr-multilang-abi.md`](./20260722-herdr-multilang-abi.md) | How does herdr stay language-agnostic? Manifest + argv + env + CLI/socket; cookbook Bash/Node/Lua/Rust. |
| 3 | [`20260722-opencode-plugin-hooks.md`](./20260722-opencode-plugin-hooks.md) | What to steal/avoid from OpenCode’s “plugin = Hooks” Bun/TS model vs next-code’s out-of-process hooks. |
| 4 | [`20260722-nextcode-extension-inventory.md`](./20260722-nextcode-extension-inventory.md) | What is CORE / PLUG / GAP / PROD in today’s next-code tree? Platform vs **nextcode** default pack split. |
| 5 | [`20260722-face-customization-limits.md`](./20260722-face-customization-limits.md) | Face embed ceiling: brand/ACP/widgets yes; Pi `ctx.ui.custom` / Face plugin-host **no** without fork. |

## How to use

1. Product / phase decisions → master plan Summary + roadmap.
2. “Does Pi have X?” → report 1.
3. “How do we support Python/Bash?” → report 2.
4. “Should plugins be in-process TS?” → report 3 (avoid) + master plan Option B′.
5. “Is this host or nextcode pack?” → report 4.
6. “Can Face draw guest UI?” → report 5 (defer).
