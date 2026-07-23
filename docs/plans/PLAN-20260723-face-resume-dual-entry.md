# Plan Report — Face resume dual entry (2-panel `--resume` vs modal `/resume`)

## Summary (read this first)
- **You asked:** Research origin + Face resume UX; keep legacy TUI **2-panel** behavior for bare `--resume`, but Face-styled; keep current Face `SessionPicker` for `/resume`.
- **What is going on:** Today both bare `--resume` and `/resume` open the **same** Face `ActiveModal::SessionPicker` (list + expand-card). Legacy `next-code-tui` bare resume is a **full-screen 40/60 list + transcript preview**. Stock grok-build has **no** session 2-panel; bare `--resume` continues most-recent; `/resume` is expand-card modal only. Closest Face layout prior art is `ActiveModal::MemoryBrowser` (left list / right preview).
- **We recommend:** **Intentional dual entry** — (1) bare `--resume` → **new Face-styled 2-panel resume browser** (copy layout/behavior ideas from legacy TUI; wire inside Face; do **not** default `NEXT_CODE_LEGACY_TUI`); (2) `/resume` → **keep** current `SessionPicker` modal. Optional follow-up: flat-store `LoadCardDetail` for expand-card stats.
- **Risk:** Medium (new Face view + startup seam; preview data path must use flat store correctly)
- **Status:** Implemented — dual entry shipped on `pr-face-resume-dual-entry` (stacked on #60)

## Product delta (approved direction)

| Entry | UX | Chrome |
|-------|-----|--------|
| Bare `next-code --resume` (empty id) | Full-screen **left list / right transcript preview** (legacy TUI shape) | **Face** theme / bordered frames / palette — not ratatui legacy chrome |
| `/resume` (and in-session palette) | Current Face **list + expand-card** `SessionPicker` | Unchanged |
| Stock Grok bare `--resume` | Continue most-recent for cwd | **Not** adopted (next-code keeps picker; now upgraded to 2-panel) |
| `NEXT_CODE_LEGACY_TUI=1` | Escape hatch only | Do **not** make default |

## Evidence (verified)

### Legacy TUI 2-panel (behavior to copy)
| Claim | Citation | Status |
|-------|----------|--------|
| Module is left list + right conversation preview | `crates/next-code-tui/src/tui/session_picker.rs` L1–4 | verified |
| Horizontal split **40% list / 60% preview** | same file `render()` L2177–2187 | verified |
| Pane focus: Sessions vs Preview; `h`/`←` list, `l`/`→` preview, `Tab` toggle | `session_picker/navigation.rs` `handle_focus_navigation_key` L193–213 | verified |
| `j`/`k` / arrows step selection or scroll preview by focus; `J`/`K` / Page page | same L214–237 | verified |
| Preview = last **20** rendered messages (`build_messages_preview`) | `session_picker/loading.rs` L1527–1540 | verified |
| Async preview load + markdown wrap cache | `session_picker.rs` `PendingSessionPreviewLoad`, `PreviewRenderCache` L170–200, `render_preview` L1213+ | verified |
| Crash restore banner; `R`/`B` → `RestoreCrashedGroup` | `render.rs` `render_crash_banner` L718–742; keybinds L1162–1164 | verified |
| Search `/`; Space multi-select; Enter resume; help footer | `render.rs` help L637–649; keys in `session_picker.rs` L1145+, L1169+ | verified |
| Data: flat `~/.next-code/sessions/<id>.json` (+ journal) via `load_sessions` / `parse_next_code_session_info` | `loading.rs` L1587–1610 | verified |

### Face origin SessionPicker (reuse for style; keep for `/resume`)
| Claim | Citation | Status |
|-------|----------|--------|
| Shared helpers for welcome + modal | `crates/xai-grok-pager/src/views/session_picker.rs` L1–6, `SessionEntryData` L85–95 | verified |
| Expand-card fields: ID/CWD/Model/… + Turns/Tools/Prompt from `card_detail` | `build_session_entry_data` L609–640 | verified |
| `/resume` → `Action::ShowSessionPicker` | `slash/commands/resume.rs` L21–22; `modals.rs` `trimmed == "resume"` | verified |
| Modal open sets `ActiveModal::SessionPicker` + fetch list | `dispatch/session/load.rs` `dispatch_show_session_picker` L1192–1210 | verified |
| Face chrome: `Theme`, bordered/fullscreen picker frames | `views/picker.rs` `render_fullscreen_frame` L1463+ | verified |
| Face **2-panel prior art** (files, not sessions): `ActiveModal::MemoryBrowser` | `views/modal.rs` L286; `modals.rs` MemoryBrowser routing | verified |

### Stock grok-build (DeepWiki + local)
| Claim | Citation | Status |
|-------|----------|--------|
| Stock empty `--resume` = most recent for cwd | local `cli.rs` `resume_most_recent()`; DeepWiki confirm | verified |
| Stock session pick UI = modal expand-card / welcome list — **no** session 2-panel | DeepWiki ask_question (MemoryBrowser is for `/memory`, not sessions) | verified |
| Vendored Face matches that model | same paths under `crates/xai-grok-pager` | verified |

### Seam today (must fork for dual entry)
| Claim | Citation | Status |
|-------|----------|--------|
| Empty resume sets `NEXT_CODE_OPEN_SESSION_PICKER_AT_STARTUP=1` | `src/cli/pager_launch.rs` L44–51 | verified |
| Event loop reads env → **`Action::ShowSessionPicker`** (same as `/resume`) | `event_loop.rs` L1653–1670 | verified |
| `NEXT_CODE_LEGACY_TUI` is opt-in escape only | `pager_launch.rs` `legacy_tui_requested` L15–24 | verified |

### Flat store — enough for right-pane transcript?
| Claim | Citation | Status |
|-------|----------|--------|
| Snapshot path `sessions/<id>.json` with `messages: Vec<Value>` | `xai-grok-shell/.../persistence.rs` L1–4, `SessionSnapshot` L28–44, `session_snapshot_path` L269–271 | verified |
| Journal sidecar `sessions/<id>.journal.jsonl` (deleted with session) | same L222–229 | verified |
| List API uses **startup stub** (may omit transcript vectors → thin `numMessages`) | `src/cli/face_auth.rs` L541–552 | verified |
| `LoadCardDetail` still reads nested `session_dir(...)/chat_history.jsonl` (wrong for flat store) | `effects/mod.rs` L990–1020; `session_dir` = `sessions_root().join(id)` L183–185 | verified |
| **Verdict:** Full snapshot (+ journal merge like legacy loader) **is enough** for transcript preview; list stubs alone are **not**. Need a dedicated preview loader, not list payload. | — | verified |

## Copy / delete / wire map

| Kind | What | Notes |
|------|------|-------|
| **Copy (ideas only)** | Legacy 40/60 layout, pane focus, preview-last-N-messages, search, Enter-to-resume, optional crash banner | Do **not** copy ratatui legacy colors/widgets wholesale |
| **Wire (Face)** | New view/modal e.g. `ActiveModal::ResumeBrowser` (or full-screen welcome-class view) styled with Face `Theme` + bordered frames; layout pattern inspired by `MemoryBrowser` | New action e.g. `ShowResumeBrowser` |
| **Wire (seam)** | `pager_launch` empty `--resume` → env (reuse or rename) → event_loop opens **ResumeBrowser**, **not** `ShowSessionPicker` | `/resume` stays `ShowSessionPicker` |
| **Wire (data)** | Preview loader: read flat `sessions/<id>.json` messages (+ journal if needed); last ~20 turns as text lines | Prefer shell helper next to `persistence.rs` |
| **Reuse** | Face list row builders / repo grouping / fetch list effects where possible; `list_nextcode_sessions` for left pane metadata | Expand-card path unchanged for `/resume` |
| **Delete / do not** | Do **not** default `NEXT_CODE_LEGACY_TUI`; do not re-home this UI in `next-code-tui` | Legacy remains escape hatch |
| **Optional later** | Fix `LoadCardDetail` for flat store (Turns/Prompt on expand-card) | Separate from 2-panel MVP if scoped tight |

## Steps (if go ahead)

1. [x] Add Face `ResumeBrowser` state + render (40/60, Face theme; left sessions, right transcript).
2. [x] Add `Action::ShowResumeBrowser` + dispatch; preview effect/loader from flat store.
3. [x] Change startup env path so bare `--resume` → `ShowResumeBrowser`; keep `/resume` → `ShowSessionPicker`.
4. [x] Keybinds MVP: j/k, h/l or Tab focus, Enter resume, `/` search, Esc back/quit; document what is deferred (crash banner, multi-select, Ctrl+Enter new terminal).
5. [x] Tests: seam routing (env → browser vs slash → modal); preview non-empty from fixture flat session+journal.
6. [ ] Manual: `next-code --resume` 2-panel; `/resume` expand-card; pick attaches via ACP.

## Files to touch (if go ahead)

- `src/cli/pager_launch.rs` — env comment / optional rename for 2-panel intent
- `crates/xai-grok-pager/src/app/event_loop.rs` — startup dispatch target
- `crates/xai-grok-pager/src/app/actions.rs` — new action
- `crates/xai-grok-pager/src/app/dispatch/**` — show + pick + preview load
- `crates/xai-grok-pager/src/views/modal.rs` (+ render/input) — `ResumeBrowser` variant
- `crates/xai-grok-pager/src/views/` — new resume-browser view module (Face-styled)
- `crates/xai-grok-shell/src/session/persistence.rs` (or sibling) — flat transcript preview API
- Focused tests under pager dispatch / shell persistence

**Leave unchanged for `/resume`:** `slash/commands/resume.rs`, `dispatch_show_session_picker`, expand-card `session_picker.rs` builders (except optional CardDetail follow-up).

## Risk
- **Medium:** New UI surface in Face; wrong loader → empty right pane; must not break `/resume` modal or welcome list.
- Mitigate: dual-entry tests; reuse MemoryBrowser input/layout patterns; no LEGACY_TUI flip.

## Open questions (≤3)

1. **Crash restore banner + multi-select / new-terminal** in v1 Face 2-panel, or MVP = list + preview + Enter only?
2. Preview source priority: full `Session::load` (daemon-style) vs shell-only snapshot+journal parse?
3. Is optional flat-store `LoadCardDetail` fix in the same PR, or a follow-up after dual-entry ships?

## Out of scope

- Making Face the stock Grok “continue most-recent” bare `--resume`
- Defaulting `NEXT_CODE_LEGACY_TUI`
- Cloud/foreign session demos beyond what existing list already shows
- Rewriting `/resume` expand-card chrome
