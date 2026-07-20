# Plan Report

## Summary (read this first)
- **You asked:** After merging PR1 into `dev`, branch PR2 and continue Grok UI migration.
- **What is going on:** PR1 (ratatui leaf crates) is on `dev`. PR2 was planned as “copy `xai-grok-pager-render`”, but that crate is **not a leaf** — it calls into `xai_grok_shared`, `xai_grok_config`, `xai_tty_utils`, `xai_grok_markdown`, telemetry, and a few shell/tools/workspace symbols.
- **We recommend:** Split PR2 into a **compile-green presentation layer**:
  1. Vendor `pager-render` → `next-code-tui-pager-render` (or `next-code-grok-pager-render`)
  2. Add **minimal shims / copies** only for symbols render actually uses (not full Grok brain)
  3. Reuse PR1 crates (`next-code-ratatui-inline` / `textarea`)
  4. Do **not** delete old TUI or change entry yet (that’s later PRs)
- **Risk:** High if we copy render without shims; Medium with scoped shims
- **Status:** Waiting for your OK — reply **go ahead** to implement

## Feature planning
- **Recommended approach:** Treat PR2 as “Face render substrate”, not full Grok UI. Copy `xai-grok-pager-render` sources; replace `xai_ratatui_*` with PR1 crates; introduce thin shim crates (or a single `next-code-grok-shim` crate with modules) that satisfy the import graph below with stubs or next-code mappings where trivial.
- **Prior art (GitHub):** [xai-org/grok-build](https://github.com/xai-org/grok-build) — pager-render is the extracted presentation layer; pager (~374k) still sits on top. Do not copy pager in PR2.
- **Integration points:**
  - `crates/next-code-ratatui-inline` / `textarea` (already on `dev`)
  - New: `crates/next-code-tui-pager-render` (from `grok-build/crates/codegen/xai-grok-pager-render`)
  - New shims (minimal): shared clipboard/paths/config/tty/telemetry/markdown adapters
- **Sub-agents used:** skipped — follow-on from completed PR1 research on this branch
- **Option B:** Copy only `appearance`/`theme`/`terminal` subsets first (smaller), delay full render — slower path to pager
- **Open questions:**
  1. Crate name: `next-code-tui-pager-render` vs `next-code-grok-pager-render`?
  2. Markdown: shim to existing `next-code-tui-markdown` now, or vendor `xai-grok-markdown` in a later PR?
  3. Telemetry/shell/tools symbols: stub no-op OK for PR2 compile, or must map to next-code immediately?

## Evidence
1. **Local grok-build:** `SOURCE_REV` `ba69d70` — `xai-grok-pager-render` ~33k LOC, 64 `.rs` files
2. **Import counts in render `src/`:** `shared` 49, `config` 14, `tty_utils` 13, `telemetry` 11, `markdown` 11, `shell` 6, `paths` 3, `tools` 2, `workspace` 2, ACP types 28
3. **PR1 on `dev`:** merge `f7bc8127e` (PR #32), tracking issue #35
4. **Branch:** `pr-2-grok-pager-render` from `origin/dev`

### Shim surface (minimum for compile)

| Upstream | Symbols render needs (examples) | PR2 strategy |
|----------|----------------------------------|--------------|
| `xai_grok_shared` | `clipboard::*`, `placeholder_images::*`, `stderr::*`, session info types | Copy clipboard/placeholder modules **or** shim wrapping next-code/arboard |
| `xai_grok_config` | `grok_home`, `load_effective_config_disk_only`, `user_grok_home` | Thin shim → next-code config / dirs |
| `xai_tty_utils` | `detach_std_command`, `dup_tui_stderr`, `is_wsl` | Copy small crate (likely leaf) |
| `xai_grok_markdown` | `render_markdown_ratatui_full`, `MarkdownStyle`, syntect hooks | **Biggest unknown** — prefer adapter to `next-code-tui-markdown` if API fits; else defer vendor |
| `xai_grok_telemetry` | `log_event`, clipboard events, `is_enabled` | No-op stub |
| `xai_grok_shell` | placeholder image load/recover; `set_theme` (docs/comments mostly) | Stub / local helpers |
| `xai_grok_paths` | `normalize_lexically` | 1-fn shim |
| `xai_grok_tools` | `detach_std_command`, image validate | Re-export tty shim + local validate |
| `xai_grok_workspace` | always-approve option ids | Const stub |
| `agent-client-protocol` | `ContentBlock`, `ImageContent`, `SessionId` | Keep as real dep if already in tree; else types stub |

## Steps (simple checklist)
1. [ ] Confirm naming + markdown strategy (open questions)
2. [ ] Add workspace members for render + shim crates
3. [ ] Vendor/adapt `pager-render` (ratatui 0.28 path like PR1; drop `ratatui-core`/`tui-scrollbar` where needed)
4. [ ] Implement minimal shims until `cargo check -p <render>` is green
5. [ ] Smoke tests / typecheck; open PR → base **`dev`**, link #35
6. [ ] Do **not** wire binary or delete old TUI in this PR

## Files to touch
- `Cargo.toml` / `Cargo.lock` — workspace members
- `crates/next-code-tui-pager-render/**` (new)
- `crates/next-code-grok-shim-*/**` or one shim crate (new)
- `docs/grok-migration-SUMMARY.md` — correct PR2 assumptions (render not leaf)

## If you want more detail
PR2 success = **render crate compiles in next-code workspace** and depends on PR1 + shims only. Full Grok UI still needs later PRs for pager + ACP host + entry.
