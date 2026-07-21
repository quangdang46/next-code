# Plan Report — Face floating Context + KV widgets

## Summary (read this first)
- **You asked:** Two floats like legacy (Context + KV), wired into Face; show **only while scrolling**.
- **Still wrong after first copy attempt:** right float was a lone `Context Nk/Nk` chip (+ left KV), not multi-line Overview.
- **Verified root cause:** paste-copied Overview render path was fine; **model/provider never reached Face**. `build_info_float_data` used `session.models.current_model_name()` which was `None` (prompt chrome also showed `unknown`). Legacy `render_model_info` returns **no lines** without `model` — so Overview collapsed to Context-only. Secondary: TokenUsage applied with `total=0` → `ContextInfo::from_notification` sets `total=used` → `19k/19k` red bar.
- **Fix:** wire daemon History `provider_model` (+ available models + context window meta) into ACP `NewSessionResponse` / `LoadSessionResponse` `.models(...)`; keep provider ext; prefer model window for float limit; pass model window into `apply_context_used`.
- **Risk:** Low–Medium (depends on History always carrying `provider_model`)
- **Status:** Fixed on branch `pr-face-info-widget-floats` — rebuild/install required

## What was wrong vs legacy
| | First attempt (still wrong) | Legacy / expectation |
|--|-----------------------------|----------------------|
| Right float | 1-line `Context Nk/Nk` (model missing) | Overview: model · provider · sessions · Context |
| Prompt chrome | `unknown` | Real model name |
| Context ratio | often `used/used` (collapsed total) | `used / model_window` |
| Left KV | OK (bordered summary) | Same |
| Top-right `19K / 19K` | Face **status** `context_bar` (not the float) | Separate from Overview float |

## Copy map (symbols → Face `views/info_floats.rs`) — still valid
| Source | Symbols |
|--------|---------|
| `info_widget_layout.rs` | `MIN_WIDGET_WIDTH` / `MAX_WIDGET_WIDTH` |
| `info_widget.rs` | Overview compact + KV summary + border chrome |
| `info_widget_model.rs` | `render_model_info` (+ session/provider lines) |
| `info_widget_usage.rs` | `render_context_usage_line` / pill |

## Wire map (this fix)
| Seam | What |
|------|------|
| `pager_agent.rs` | History → `SessionModelState` on `new_session` / `load_session`; `ModelChanged` → `x.ai/models/update`; provider ext unchanged |
| `agent_view/session.rs` | `apply_token_usage_sample` passes model window as total; float limit prefers catalog window |
| `agent_view/render.rs` | still `build_info_float_data` → `render_info_floats` (scroll-gated) |

## Smoke
1. Rebuild + install Face binary (see AGENTS.md install notes); **restart** `next-code serve`.
2. Open a session with a known model (prompt border must **not** say `unknown`).
3. Scroll during the session → top-right Overview: model, provider (blue), `N session · …`, `Context …` + bar.
4. Left KV float only when cache fields exist.
5. Idle ~1s → floats hide. Status-bar `Nk / Nk` chip may still show in the header — that is **not** the Overview float.

## Status
Copy render was already in place; **data wire** was the remaining failure.
