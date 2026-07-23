# Plan Report

## Summary (read this first)
- **You asked:** When failover switches model/provider (e.g. Fable → Kimi), Face must show a **visible** notice so the jump is explained. TUI already does; Face feels silent / dump-y.
- **What is going on:** Cross-provider failover is a **client UX** in the legacy TUI (parse `[next-code-provider-failover]`, countdown, “Auto-switched…”). Face’s ACP bridge (`NextCodeFaceAgent`) does not parse that prompt — it prints `Error: {raw message}`. Midstream / daemon `ModelChanged` only refreshes Face’s model catalog via `x.ai/models/update`; Face’s `ModelChanged` handler is **intentionally silent** (no toast, no scrollback). Stock Face already has a visible path (`ModelAutoSwitched` → scrollback `SessionEvent::ModelUnavailable`), but the embed **never emits** it.
- **We recommend:** Smallest wire at `src/cli/pager_agent.rs`: (1) on failover-prompt `Error`, emit a human notice (not raw JSON); (2) on auto `ModelChanged`, also emit `x.ai/session/update` `ModelAutoSwitched` so Face reuses its existing scrollback line. Defer full TUI countdown/Esc cancel to a follow-up unless you want parity in this PR.
- **Risk:** Low (bridge-only; reuse existing Face render path; no Face rewrite).
- **Status:** Waiting for your OK — reply **go ahead** to implement

## Bug investigation
- **Verified root cause:** Face embed maps failover/model-switch events without the TUI’s notice layer and without Face’s stock `ModelAutoSwitched` notification.
- **Hypotheses ranked:**
  1. **Confirmed:** `pager_agent` `Error` → `emit_text("Error: {message}")` with no `parse_failover_prompt_message` — user sees raw marker or opaque error; no countdown / Auto-switched copy.
  2. **Confirmed:** `pager_agent` `ModelChanged` → `emit_models_update` / `emit_provider_name` only; Face `ModelChanged` applies state silently (by design for followers).
  3. **Confirmed:** Face already renders auto-switch via `ModelAutoSwitched` → `SessionEvent::ModelUnavailable` (“… Switched to \"{new}\".\"); unused by next-code embed.
- **Ruled out:** Missing Face toast API (exists); missing Face auto-switch UI type (exists); need to invent grok-build APIs.
- **Sub-agents used:** skipped — narrow wire gap with direct citations (LOOK only).
- **Citations checked:** paths below.

## Evidence

### 1) TUI failover UX (countdown + Auto-switched + marker)
| Piece | Path | Proof |
|-------|------|--------|
| Wire prefix + JSON prompt | `crates/next-code-provider-core/src/failover.rs` | `PROVIDER_FAILOVER_PROMPT_PREFIX = "[next-code-provider-failover]"`; `ProviderFailoverPrompt::to_error_message()` |
| Provider stops before auto-switch; returns prompt error | `crates/next-code-base/src/provider/mod.rs` ~620–630 | `build_failover_prompt` → `Err(anyhow!(prompt.to_error_message()))` |
| Parse + countdown / system notices | `crates/next-code-tui/src/tui/app/model_context.rs` | `handle_turn_error` → `handle_provider_failover_prompt`; countdown status `"Provider auto-switch → {} in {}s"`; after deadline `"⚡ Auto-switched provider after countdown: {} → {}."` |
| Tests encode marker | `crates/next-code-tui/src/tui/app/tests/support_failover/part_02.rs` | `"[next-code-provider-failover]{}\nignored"` |
| Remote `ModelChanged` also notices | `crates/next-code-tui/src/tui/app/remote/server_events.rs` ~2031–2058 | `DisplayMessage::system("✓ Switched to model: {}")` + status |

### 2) What Face / ACP receives today
| Daemon event | `pager_agent` mapping | Face result |
|--------------|----------------------|-------------|
| `ServerEvent::Error` | `emit_text(..., format!("Error: {message}"))` (`src/cli/pager_agent.rs` ~1441–1444) | Agent message chunk — raw failover line if present; **no** parse / countdown |
| `ServerEvent::ModelChanged` (ok) | `emit_provider_name` + `emit_models_update` (`x.ai/models/update`) (~1419–1428) | Catalog / status chrome update only |
| Face `XaiSessionUpdate::ModelChanged` | `session_notification.rs` ~847–896 | Silent state apply; tests say **no** scrollback / toast (`tests/models.rs` “silently mirrors…”) |
| Face `ModelAutoSwitched` | `session_notification.rs` ~807–844 → `SessionEvent::ModelUnavailable` | Visible scrollback: `"{reason} Switched to \"{new}\"."` (`scrollback/blocks/session_event.rs` ~292–301) |
| Emit of `ModelAutoSwitched` from embed | **none** in `src/cli/pager_agent.rs` | Gap |

Midstream daemon resync (e.g. retired model name → served model): `crates/next-code-app-core/src/agent/turn_streaming_mpsc.rs` ~916–942 emits `ServerEvent::ModelChanged` only — no reason string, no Face notice.

### 3) Smallest wire (recommended)
**Copy / wire / delete map**

| Kind | Action |
|------|--------|
| **Wire** | `src/cli/pager_agent.rs` — failover `Error`: `parse_failover_prompt_message`; emit human text (and/or toast-capable path). Prefer **not** dumping the JSON prefix line. |
| **Wire** | Same file — when applying auto `ModelChanged`, also `ext_notification("x.ai/session/update", ModelAutoSwitched { previous, new, reason })` so Face’s existing handler paints scrollback. Keep `emit_models_update` for chrome. |
| **Copy** | Reuse Face types already vendored: `xai_grok_shell::extensions::notification::{SessionNotification, SessionUpdate::ModelAutoSwitched}` (or equivalent JSON shape Face already deserializes). |
| **Delete** | Nothing. Do not re-home TUI countdown into Face crates. |
| **Out of scope (follow-up)** | Full countdown + Esc cancel + auto-resend on Face (TUI-local `pending_provider_failover`). Manual-mode clear “switch with /model” message is enough for MVP “at least visible.” |

**Optional thinner MVP:** only humanize failover `Error` text via `emit_text` / system-like chunk — still leaves midstream chrome-only jumps silent; prefer also `ModelAutoSwitched`.

## Steps (simple checklist)
1. [ ] In `pager_agent` prompt loop: if `Error` parses as failover prompt, emit readable notice (from/to/reason); skip raw prefix dump.
2. [ ] Track previous model before turn / on `ModelChanged`; emit `ModelAutoSwitched` with a short reason (`"provider failover"` / `"provider switched model mid-request"`).
3. [ ] Unit/regression: failover error → no `[next-code-provider-failover]` in emitted text; `ModelChanged` → ext payload includes `ModelAutoSwitched`.
4. [ ] Manual: force Fable→other failover under Face — scrollback shows why; status/model chrome still updates.
5. [ ] Rebuild/install both `next-code` / `nextcode` aliases if UI path changes.

## Files to touch
- `src/cli/pager_agent.rs` — primary seam
- Tests colocated under `src/cli/pager_agent.rs` `#[cfg(test)]` and/or a small pager_agent failover test module
- Possibly thin helper next to existing `emit_models_update` (same file) — no Face crate rewrite unless emit shape fails deserialize (then fix only the JSON envelope)

## Open questions (≤3)
1. MVP: human notice only, or also Face countdown + Esc cancel like TUI?
2. On cross-provider prompt: Face should **auto-switch + retry** later, or notice + tell user to `/model` (TUI manual mode copy)?
3. Should midstream same-provider renames (fable→opus) use the same `ModelAutoSwitched` line?

## If you want more detail
- TUI remote already treats every successful `ModelChanged` as user-visible; Face stock treats broadcast `ModelChanged` as silent follower sync and reserves visibility for `ModelAutoSwitched` / local `SwitchModelComplete`. Matching Face stock = emit `ModelAutoSwitched` for **automatic** jumps; keep silent `ModelChanged` for catalog-only sync.
- DeepWiki MCP unavailable this session; local vendored Face + Exa grok-build hits used for ACP session-update shape confirmation.
