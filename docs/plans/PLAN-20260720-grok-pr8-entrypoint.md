# Plan Report

## Summary (read this first)
- **You asked:** After PR7 merge, continue with **PR8** — cut over `next-code` interactive entry to Grok Face pager, bridge to next-code brain, next-code logo, then stop shipping old TUI.
- **What is going on:** PR7 vendored Face (`xai-grok-pager`) + stubs; binary still launches old `next-code-tui` via `run_default_command` → `tui_launch::run_tui_client`. Pager already has `app::run(PagerArgs)` + `spawn_grok_shell` → `MvpAgent` stub over ACP. Welcome logo is still Grok braille (`assets/logo/logo*.txt`). SUMMARY’s full `GrokHost` trait rewrite is large; ACP host-as-mediator is the proven protocol pattern.
- **We recommend:** **Copy + adjust wire** — keep Face’s ACP client + event loop; replace stub agent with a next-code-backed ACP Agent (or thin adapter) that talks to existing `serve`/runtime. Do **not** rewrite pager effects to a brand-new `GrokHost` trait in one PR. Ship logo swap in the same PR. Delete/stop old TUI path only after Face default path works.
- **Risk:** High (entrypoint + session/auth/server spawn coupling + logo animation port)
- **Status:** Implementing — reply **go ahead** was received; cutover landed on `pr-8-grok-entrypoint`

## Feature planning
- **Recommended approach:** Phased cutover on branch `pr-8-grok-entrypoint` (already from merged `dev` @ `9d11ffaf2`):
  1. **Wire binary default** — `run_default_command` keeps server spawn/login, but interactive UI calls `xai-grok-pager::app::run` (map `PagerArgs` from next-code CLI: resume, cwd, etc.) instead of `run_tui_client`. Add root `next-code` → `xai-grok-pager` dependency.
  2. **Brain bridge (ACP)** — keep Face as ACP **client**; replace `MvpAgent` in `spawn_grok_shell` (or parallel spawn path) with an agent that forwards prompts/tools/sessions to next-code server/runtime (openproxy). Prefer adapter over deleting ACP handlers.
  3. **Logo** — replace `views/welcome/logo.rs` braille with next-code idle animation (`next-code-tui` `ui_animations` donut / orbit_rings). Port or depend on anim primitives; do **not** ship Grok `logo*.txt`.
  4. **Retire old TUI** — stop calling `run_tui_client` for default interactive; remove/stop shipping `next-code-tui*` only after smoke (`next-code` shows Face + can chat via brain). Keep crates in workspace briefly if needed for anim extract, then delete in follow-up if diff too large.
- **Prior art (GitHub):** [agentclientprotocol/agent-client-protocol](https://github.com/agentclientprotocol/agent-client-protocol) — Client (IDE/Face) ↔ Agent over ACP; host owns UX. [goddard-ai/acp-client](https://github.com/goddard-ai/acp-client) — host launches/mediates agent; app owns transcript/permissions. Reuse: keep Face ACP surface; mediate next-code behind Agent side. Avoid: SUMMARY’s wholesale `GrokHost` replace of ACP in one PR.
- **Integration points:**
  - `src/cli/dispatch.rs` `run_default_command` (~1119) → pager instead of `tui_launch::run_tui_client`
  - `crates/xai-grok-pager/src/app/mod.rs` `run(PagerArgs, …)` — Face entry
  - `crates/xai-grok-pager/src/acp/spawn.rs` `spawn_grok_shell` → today `MvpAgent`
  - `crates/xai-grok-pager/src/views/welcome/logo.rs` + `assets/logo/`
  - `crates/next-code-tui/src/tui/ui_animations.rs` `draw_idle_animation` — logo source
  - `serve` / `agent` CLI paths — unchanged brain
- **Sub-agents used:** skipped — feature-plan Task swarm hit usage limits; parent verified entry/spawn/logo paths + Exa ACP prior art
- **Option B:** Implement SUMMARY `GrokHost` trait and rewrite pager effects off ACP — larger rewrite, higher risk, deferred unless ACP adapter proves impossible
- **Open questions:**
  1. PR8.1 = Face UI + stub agent + logo only, then PR8.2 = real brain? Or both in one PR?
  2. Escape hatch: keep `NEXT_CODE_LEGACY_TUI=1` (or similar) for one release?

## Evidence
1. **Migration SUMMARY:** `docs/grok-migration-SUMMARY.md` § Phase 4 PR8 + §3 GrokHost — desired end state; trait rewrite optional later
2. **GitHub prior art:** ACP client/agent split — Face stays client; next-code implements/mediates Agent
3. **Installed Face API:** `xai-grok-pager::app::run` — `crates/xai-grok-pager/src/app/mod.rs` ~441
4. **Our code:** `dispatch.rs` default → `tui_launch::run_tui_client`; pager spawn still `MvpAgent`; logo still braille `include_str!(…logo07.txt)`
5. **PR7 merged:** `9d11ffaf2` Merge #41; branch `pr-8-grok-entrypoint` clean at that tip

## Steps (simple checklist)
1. [ ] Depend `next-code` on `xai-grok-pager`; map CLI → `PagerArgs`
2. [ ] Swap `run_default_command` interactive path to `xai-grok-pager::app::run` (keep server bootstrap)
3. [ ] ACP Agent adapter: next-code brain behind spawn (or staged stub + flag)
4. [ ] Welcome logo → next-code animated idle (not Grok braille)
5. [ ] Smoke: local install — `next-code` shows Face + logo; `serve` still works
6. [ ] Stop default old TUI path; delete/stop-ship only when smoke green
7. [ ] `cargo check -p next-code` (+ targeted pager tests); update SUMMARY/issue #35 notes

## Files to touch
- `src/cli/dispatch.rs`, possibly `tui_launch.rs` / new `pager_launch.rs`
- `Cargo.toml` (root deps)
- `crates/xai-grok-pager/src/acp/spawn.rs` (+ new bridge module under pager or `next-code`)
- `crates/xai-grok-pager/src/views/welcome/logo.rs` (+ anim source extract)
- `docs/grok-migration-SUMMARY.md` (PR8 status)

## If you want more detail
- Do **not** delete `next-code-tui*` in the first commit of PR8 if logo still needs anim code from there — extract first, then delete.
- Windows ConHost: old logo hid braille; next-code anim should remain visible on legacy console where possible.
- Escape hatch for rollback is strongly recommended for first merge to `dev`.
