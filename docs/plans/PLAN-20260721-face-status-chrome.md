# Plan Report — Face welcome / status chrome parity

## Summary (read this first)
- **You asked:** Face welcome must match legacy TUI splash **field-for-field** (screenshot parity), by **copying** formatters from `ui_header` / related helpers — not inventing Face-native tip strips that drop fields.
- **What is going on:** Subset A + Option B landed a density-trimmed tip strip. Gaps vs screenshot: auth green-dot inventory, model auth-tag + `/model to switch`, **Updates** box title, `mcp: (none)`, client animal on fresh welcome, full line order without aggressive hide.
- **We recommend / priority:** **Copy** formatting into a shared Face-callable snapshot (`product_welcome` + `src/cli/face_welcome_status.rs`); Face only paints. Prefer **legacy field presence** over density hiding unless the terminal is truly tiny. Prefer Grok Face layout shell; **content/order/labels from legacy**.
- **Risk:** Medium (taller welcome on short terminals; client animal is pre-generated for display and may differ from ACP session id until live refresh)
- **Status:** **BUILD in progress** — full screenshot parity pass

## Priority (mandatory)
**Copy legacy code + wire into Face** — not Face-native approximations.

## Screenshot checklist (target)

| # | Field | Legacy source | Face target | Status |
|---|-------|---------------|-------------|--------|
| 1 | `⟨client·perf:reduced⟩` (or similar) | `build_persistent_header` status_items + `perf::profile().tier.badge()` | `badge_line` | prior B |
| 2 | `server: Hut 🛖` + version | registry + `server_icon` / version label | `server_line` | prior B |
| 3 | `client: Monkey 🐒` + version dirty | `session_display_name` / memorable id + `session_icon` | generate via `new_memorable_session_id_avoiding` when no resume | **this pass** |
| 4 | `api-key:… · Model · /model to switch` (pink model) | `header_provider_label` + model spans | `model_line` + Face pink accent | **this pass** |
| 5 | `built Xh ago, code Xh ago` | `binary_age` | `built_line` (keep badge age too) | **this pass** |
| 6 | green ● auth inventory | `build_auth_status_line` | `auth_entries` + Face paint | **this pass** |
| 7 | centered **Updates** box + unseen bullets | `build_header_lines` rounded box title `Updates` | hero title **Updates** when product bullets; bordered chrome lines | **this pass** |
| 8 | `mcp: (none)` / `skills: N loaded` / `server: N clients, N sessions` | mcp/skills/sessions in `build_header_lines` | always emit `mcp: (none)` when empty; sessions when meaningful | **this pass** |

## Workflow map

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | Format strings / structure from `ui_header` (`build_persistent_header`, `build_header_lines`, `build_auth_status_line`), `binary_age`, unseen changelog, `session_icon`/`server_icon`, memorable id allocation | Port whole TUI as Face dependency; invent alternate labels |
| **Wire** | `pager_launch` → `face_welcome_status` → `ProductWelcomeStatus` → Face `views/welcome` paint | Fake Hut/Monkey without the TUI naming source; Face→`next-code-tui` |
| **Delete** | Density-first tip-only approach that drops auth/Updates/mcp:(none) | PR11 TUI retirement; unrelated PR9 brain work |

## Evidence (verified)
- Persistent + secondary header: `crates/next-code-tui/src/tui/ui_header.rs` — `build_auth_status_line` ~274, `build_persistent_header` ~591, `build_header_lines` ~816
- Age: `next-code-build-meta` `binary_age` / TUI `ui_status::binary_age` delegates
- Icons / memorable ids: `next-code-core` `id.rs` (`session_icon`, `server_icon`, `new_memorable_session_id_avoiding`)
- TUI session name at connect: local `session.display_name()`; remote from id — Face ACP creates session later, so cold welcome pre-generates with the same helper
- Auth probe: `AuthStatus::check_fast()` in `next-code-base`

## Steps
1. [x] Record priority + screenshot checklist in this plan
2. [ ] Expand `product_welcome` snapshot + formatters (auth entries, model switch line, `mcp: (none)`, chrome order, Updates title helper)
3. [ ] Add `src/cli/face_welcome_status.rs` — gather auth/registry/skills/mcp/client name; install snapshot
4. [ ] Wire Face welcome: centered chrome block (legacy order), pink model, auth dots, Updates title
5. [ ] Tests for formatters; `cargo check` / targeted tests
6. [ ] Try `scripts/_tmp_rebuild_install.ps1`; if exe locked, note quit Face

## Files to touch
- `docs/plans/PLAN-20260721-face-status-chrome.md` — this priority
- `crates/xai-grok-pager/src/product_welcome.rs` — snapshot + formatters + tests
- `crates/xai-grok-pager/src/views/welcome/mod.rs` (+ hero changelog title)
- `src/cli/face_welcome_status.rs` (new) + `pager_launch.rs` + `cli/mod.rs`
- **Not:** `next-code-tui` product path; PR9 ACP brain

## Deferred (still true after this pass)
- Live MCP tool counts after ACP connect
- Connected client count (registry has sessions only)
- Live refresh of chrome after session create (client animal may change once ACP History returns)
- Exact ratatui rounded-box glyph clone (Face bordered / hero Updates title is enough)

## Screenshot checklist (after this pass)

| # | Field | Result |
|---|-------|--------|
| 1 | `⟨client·perf:…⟩` | ✅ `badge_line` from perf tier + client when server known |
| 2 | `server: Hut 🛖` + version | ✅ registry + icons / version label |
| 3 | `client: Monkey 🐒` + version | ✅ resume name **or** `new_memorable_session_id_avoiding` (same as TUI `Session::create`) |
| 4 | `api-key:… · Model · /model to switch` (pink) | ✅ auth-tag prefix + pink model paint |
| 5 | `built Xh ago, code Xh ago` | ✅ `built_line` from `binary_age` (+ badge age still) |
| 6 | green ● auth inventory | ✅ `build_auth_dot_entries` copy of `build_auth_status_line` |
| 7 | centered **Updates** box | ✅ bordered chrome lines titled Updates; hero title Updates when product bullets (hero suppressed when chrome shows them) |
| 8 | `mcp: (none)` / skills / sessions | ✅ always `mcp: (none)` when empty; skills/sessions when meaningful |

## Status
**Implemented — full screenshot parity pass.** Remaining: manual smoke after install; optional live ACP refresh for real session animal + MCP tool counts + client count.

