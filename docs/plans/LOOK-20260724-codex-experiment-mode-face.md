# LOOK — Codex experiment mode vs Face

**Date:** 2026-07-24  
**Scope:** Research / docs only — no product implementation.  
**Clone:** `C:\Users\ADMIN\Documents\Projects\next-code\.tmp-research-plugins\codex` (shallow `openai/codex`, untracked; keep under ignored tmp path)  
**Sources:** Codex `codex-rs/features`, `codex-rs/tui` (`/experimental` popup); next-code `crates/next-code-experiment-flags`, legacy TUI stubs, Face (`xai-grok-pager`)

---

## Verdict (short)

| Flow | Codex | Face (next-code) | Status |
|------|-------|------------------|--------|
| **Toggle experiment mode** (discover → list → Space toggle → persist) | OK — `/experimental` bottom-pane popup | Missing | **not OK** |
| **Register one feature into experiments** (enum + registry → shows in menu) | OK — `Feature` + `FEATURES` + `Stage::Experimental` | Partial model only — crate exists, unwired / unused | **not OK** for Face |

**Overall:** Face does **not** have Codex-parity experiment UX. next-code has a Codex-shaped flags crate and half-built legacy stubs, but neither Face nor a live CLI/TUI path completes the loop.

---

## LOOK matrix

| Dimension | Codex | next-code Face | Gap |
|-----------|-------|----------------|-----|
| Entry slash | `/experimental` → `SlashCommand::Experimental` → `open_experimental_popup()` | No `/experimental` / `/experiment` in `xai-grok-pager` slash registry | **missing** |
| Discovery UI | `ExperimentalFeaturesView` checklist (`[x]` / `[ ]`), Space toggles, Enter/Esc saves | None in Face | **missing** |
| Persist | `AppEvent::UpdateFeatureFlags` → `features.<key>` in `config.toml` | No Face persist path for experiment flags | **missing** |
| Registry SSOT | `codex-rs/features` `FEATURES: &[FeatureSpec]` + `Stage` lifecycle | `crates/next-code-experiment-flags` `EXPERIMENT_FLAGS` + similar `Stage` | Model **partial OK**; product wire **missing** |
| Menu filter | Only `Stage::Experimental` (UnderDevelopment excluded from menu) | Popup *would* show Experimental + UnderDevelopment — but popup is orphaned | **wrong / dead** |
| Register a feature | Add `Feature` variant + `FeatureSpec { stage: Experimental { name, menu_description, announcement }, … }` | Add `ExperimentFlag` + `FeatureSpec` in crate — **but** nothing consumes flags at runtime; Face never lists them | Register path **partial**; end-to-end **missing** |
| Announcements / tips | `experimental_announcement()` → tip stream | No Face tip hook for experiment flags | **missing** |
| CLI / app-server | Features also exposed via app-server experimental-feature APIs | `src/cli/experiment_flags.rs` exists but **not** in `src/cli/mod.rs` (uncompiled stub). `next-code-app-core` depends on crate with **zero** `use` sites | **missing / dead dep** |
| Legacy TUI | N/A | `/experiment` autocomplete + orphan `experiment_popup.rs` (not `mod`’d; crate dep not declared) | **stub / wrong** — looks finished in slash inventory, is not |

---

## Codex — how it works (evidence)

### Toggle UX

1. User runs `/experimental`.
2. `slash_dispatch.rs` calls `open_experimental_popup()`.
3. Popup builds rows from `FEATURES` where `stage.experimental_menu_name()` is `Some`.
4. `ExperimentalFeaturesView`: Space flips `enabled`; accept/cancel fires `UpdateFeatureFlags` and persists to `config.toml`.
5. Empty state copy: “No experimental features available for now”.

Key paths (clone):

- `codex-rs/tui/src/slash_command.rs` — `Experimental => "toggle experimental features"`
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs` — `SlashCommand::Experimental => open_experimental_popup()`
- `codex-rs/tui/src/chatwidget/settings_popups.rs` — `open_experimental_popup`
- `codex-rs/tui/src/bottom_pane/experimental_features_view.rs` — UI + save-on-exit

### Register a feature

1. Add variant on `Feature` enum.
2. Append `FeatureSpec` to `FEATURES` with:

```text
stage: Stage::Experimental {
  name: "...",
  menu_description: "...",
  announcement: "NEW: ... from /experimental.",
}
```

3. Feature appears in `/experimental` automatically; optional tip via `experimental_announcement()`.
4. Gate runtime with `config.features.enabled(Feature::…)`.

Key path: `codex-rs/features/src/lib.rs` (`Stage`, `Feature`, `FEATURES`).

---

## next-code / Face — current state (evidence)

### What exists

| Piece | Path | Reality |
|-------|------|---------|
| Flags crate | `crates/next-code-experiment-flags` | Codex-like `Stage` / `ExperimentFlag` / `EXPERIMENT_FLAGS`. Today almost all flags are **Stable**; only `js_plugins` is `Experimental`. |
| Legacy popup | `crates/next-code-tui/src/tui/experiment_popup.rs` | Space toggle + Enter apply — **not** declared in `tui/mod.rs`; **not** a Cargo dep of `next-code-tui` → orphan file |
| Slash autocomplete | `state_ui_input_helpers.rs` `/experiment` | Registered + suggests “open interactive popup” / `list` — **no** dispatch handler found |
| CLI helpers | `src/cli/experiment_flags.rs` | `list` / `enable` / `disable` — **not** in `src/cli/mod.rs` → not part of the binary |
| Face | `crates/xai-grok-pager` | No experiment slash, no popup, no `next_code_experiment_flags` use. Ad-hoc “experimental” labels (memory CLI, vim setting tags, dashboard env gate) ≠ Codex experiment mode |

### What does **not** exist for Face

- User-visible experiment mode toggle
- Persist `[experiments]` (or equivalent) from Face UI
- Runtime gates reading `Experiments::check(...)` from Face/agent path
- Documented “register one feature” loop that ends in a live Face menu

---

## Gap classification

| Ask | Verdict | Notes |
|-----|---------|-------|
| Toggle experiment mode UI/UX | **missing** | Codex complete; Face absent; legacy stubs dead |
| Register feature into experiments | **partial (model) / missing (product)** | Crate registry mirrors Codex; no live menu, no consumer, Face unaware |
| Discovery list / enable UX | **missing** (Face) / **wrong** (legacy stub) | Autocomplete promises a popup that cannot open |

---

## Top recommended fixes (docs only — do not implement here)

Ordered by leverage for Face parity:

1. **Face `/experimental` (or `/experiment`) slash** — open a bottom-pane / modal checklist like Codex `ExperimentalFeaturesView`; Space toggle, Enter persist.
2. **Wire `next-code-experiment-flags` end-to-end** — config `[experiments]` load/save; Face + daemon/agent read `Experiments::check`; drop or use the dead `app-core` dependency intentionally.
3. **Revive or delete stubs** — either `mod experiment_popup` + Cargo dep + slash dispatch, or delete orphan popup/CLI files and remove false `/experiment` autocomplete so inventory matches reality.
4. **Register recipe** — document: add `ExperimentFlag` + `EXPERIMENT_FLAGS` row with `Stage::Experimental { name, menu_description, announcement }` → appears in Face menu; keep UnderDevelopment out of the user menu (match Codex).
5. **Optional:** tip/announcement surface when a flag graduates to Experimental (Codex tip stream).

Non-goals for this LOOK: Codebuff UX, keyword-highlight, `/btw` chrome.

---

## Smoke checklist (when implementing later)

- [ ] Face: `/experimental` opens list of only Experimental-stage flags
- [ ] Space toggles; Enter writes config; restart/session sees new enablement
- [ ] Add one dummy `Stage::Experimental` flag → appears without Face code changes beyond registry
- [ ] UnderDevelopment flag does **not** appear in Face menu
- [ ] Stable flags stay gated in code but hidden from experiment menu
- [ ] Dead stubs either wired with tests or removed
- [ ] `cargo check -p next-code-experiment-flags` + Face slash test for command registration

---

## References

- Codex clone: `.tmp-research-plugins/codex` (local, gitignored / untracked)
- Upstream: https://github.com/openai/codex
- Related inventory: `docs/plans/PLAN-20260721-slash-commands-grok-vs-nextcode.md` (`/experiment` listed as next-code-only — accurate as *intent*, not Face parity)
