# LOOK — Codex experiment mode vs Face

**Date:** 2026-07-24  
**Status:** Implemented (Face `/experimental` + `Experiments::check` gates)  
**Clone:** `C:\Users\ADMIN\Documents\Projects\next-code\.tmp-research-plugins\codex` (shallow `openai/codex`, untracked; keep under ignored tmp path)  
**Sources:** Codex `codex-rs/features`, `codex-rs/tui` (`/experimental` popup); next-code `crates/next-code-experiment-flags`, Face (`xai-grok-pager`)

---

## Verdict (short)

| Flow | Codex | Face (next-code) | Status |
|------|-------|------------------|--------|
| **Toggle experiment mode** (discover → list → Space toggle → persist) | OK — `/experimental` bottom-pane popup | Face modal checklist + `[experiments]` persist | **OK** |
| **Register one feature into experiments** (enum + registry → shows in menu) | OK — `Feature` + `FEATURES` + `Stage::Experimental` | `ExperimentFlag` + `EXPERIMENT_FLAGS` + Face menu auto-lists Experimental | **OK** |

**Overall:** Face has Codex-parity experiment UX. Runtime gates use `Config::experiment_enabled` → `Experiments::check`. Dead legacy TUI/CLI stubs removed.

---

## Implemented

1. Face `/experimental` (alias `/experiment`) — checklist modal; Space toggle; Enter saves; Esc cancels.
2. Persist writes `[experiments]` keys into `$GROK_HOME`/`$NEXT_CODE_HOME`/`~/.next-code` `config.toml` (same home remapping as Face).
3. `next-code-experiment-flags`: `experimental_menu_items()` / `experimental_menu_overrides()`; only `Stage::Experimental`.
4. `Config::experiment_enabled` wired for swarm + persist-memory injection paths.
5. Deleted orphan `experiment_popup.rs`, `src/cli/experiment_flags.rs`, and false legacy `/experiment` autocomplete.

### Register recipe

1. Add `ExperimentFlag` variant.
2. Append `FeatureSpec` to `EXPERIMENT_FLAGS` with:

```text
stage: Stage::Experimental {
  name: "...",
  menu_description: "...",
  announcement: Some("..."), // optional
}
```

3. Gate runtime with `config.experiment_enabled(ExperimentFlag::…)`.
4. Flag appears in Face `/experimental` with no Face code changes.

---

## Smoke checklist

- [x] Face: `/experimental` opens list of only Experimental-stage flags
- [x] Space toggles; Enter writes config; toast says next conversation
- [x] Registry `Stage::Experimental` → menu without Face edits (unit test)
- [x] UnderDevelopment / Stable excluded from menu (unit tests)
- [x] Dead stubs removed
- [x] `cargo test -p next-code-experiment-flags` + Face modal/slash tests

---

## References

- Audit PR: https://github.com/quangdang46/next-code/pull/97
- Codex clone: `.tmp-research-plugins/codex`
- Upstream: https://github.com/openai/codex
