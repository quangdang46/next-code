# Plan Report — Face failover countdown + Esc cancel

## Summary (read this first)
- **You asked:** TUI-parity for Face after PR #62: status countdown `Provider auto-switch → X in Ns`, then auto-switch + retry, **Esc** cancels.
- **What is going on:** #62 only humanizes failover + emits `ModelAutoSwitched`. Countdown/Esc/resend still live only in local TUI (`pending_provider_failover`). Face Esc mid-turn is swallowed (Ctrl+C cancels); stock Face already paints `RetryState::Retrying` as turn status.
- **We recommend:** Port the TUI countdown state machine into `pager_agent` (bridge). Drive status via existing `RetryState::Retrying`. Small Face Esc + status label wire so Esc cancels while that retry activity is armed. Add thin daemon `RetryTurn` so resend does not duplicate the user message (TUI `pending_turn` semantics).
- **Risk:** Medium — new wire request + Face Esc exception; countdown only on Face embed path.
- **Status:** Implementing (user said implement now)

## Copy / wire / delete map

| Kind | Action |
|------|--------|
| **Copy** | TUI copy + 3s deadline from `model_context.rs` (`handle_provider_failover_prompt` / `maybe_progress…` / `cancel_pending…`) |
| **Wire** | `pager_agent`: on failover `Error` + config `countdown` → arm pending, emit `RetryState::Retrying` each second, Esc→`cancel()` clears; deadline → `SetModel` (routed spec for `to_provider`) + `RetryTurn` |
| **Wire** | Face Esc policy: mid-turn Esc → `CancelTurn` when retry reason is provider auto-switch |
| **Wire** | Face turn status: show failover countdown reason text (not generic `Retrying (attempt N)`) |
| **Wire** | Protocol `Request::RetryTurn` + agent `retry_turn_streaming_mpsc` (no new user message) |
| **Keep** | Manual mode = #62 human notice only (no countdown) |
| **Do not** | Re-home countdown UI into new Face chrome; invent toast APIs |

## Steps
1. [ ] Protocol + daemon `RetryTurn`
2. [ ] `pager_agent` countdown / cancel / switch / retry
3. [ ] Face Esc + status label for failover `Retrying`
4. [ ] Unit tests (notice copy, cancel clears, RetryTurn decode)
5. [ ] `cargo check` targeted crates; push + PR (depends on #62)

## Files to touch
- `crates/next-code-protocol/src/{wire,lib}.rs`
- `crates/next-code-app-core/src/server/client_lifecycle.rs`
- `crates/next-code-app-core/src/agent/turn_execution.rs`
- `src/cli/pager_agent.rs`
- `crates/xai-grok-pager/src/app/agent_view/prompt.rs`
- `crates/xai-grok-pager/src/views/turn_status.rs`

## Smoke
1. Face local, `cross_provider_failover = "countdown"`, force Fable→other failover.
2. Status shows `Provider auto-switch → … in Ns (Esc to cancel)`.
3. Esc → cancel notice, same provider kept.
4. Let countdown finish → switch notice + turn retries without duplicate user bubble.
