# Plan Report — Face `/connect` UX like OpenCode

## Summary (read this first)

- **You asked:** Make next-code Face `/connect` feel like OpenCode — provider list, then API-key paste **or** OAuth/browser — without rewriting the auth brain.
- **What is going on:** OpenCode’s `/connect` is a **dialog wizard** (pick provider → optional “Select auth method” → API-key prompt **or** OAuth auto/code screens → model picker). next-code Face already has **daemon-side** multi-provider auth (`face_auth.rs` + `nextcode.<provider>` ACP methods) and reuses Face welcome paste/URL chrome, but bare `/connect` mostly dumps a text list; there is **no method chooser** for Hybrid providers; credentials still write dual-rooted stores (`.env` under config dir vs OAuth under `~/.next-code`).
- **We recommend:** **Copy OpenCode’s connect *flow shape* into Face chrome**, **wire** through existing `Action::NextCodeConnect` → `face_auth` (keep next-code daemon/store as brain). Defer unified `auth.json` writes to [PLAN-20260722-auth-unify](./PLAN-20260722-auth-unify-opencode-vs-claude.md) / PR #61 unless this UX PR dual-writes as a thin follow-on.
- **Risk:** Medium (Face modal/welcome reuse; Hybrid OAuth-vs-key branching; confusion with stock Grok `/login`).
- **Status:** Implemented on `pr-auth-unify-opencode-store` (PR #61) — bare `/connect` opens Face picker; multi-method families chain to “Select auth method”; brain remains `face_auth` / auth store.

### Vietnamese bullets (plain)

- OpenCode: `/connect` → chọn provider → (nếu có) chọn OAuth hay API key → dán key hoặc mở browser → xong.
- next-code Face: đã có auth backend multi-provider, nhưng UX còn “Tab + gõ provider” / dump text, chưa wizard rõ như OpenCode.
- Nên **copy flow UI**, **giữ** `face_auth` / daemon làm brain; thống nhất `auth.json` là ticket riêng (#61).

---

## Evidence (verified citations)

### OpenCode — connect UX models / steps / copy

| Step | What user sees | Evidence |
|------|----------------|----------|
| Entry | TUI `/connect` / CLI `opencode auth login` | Docs: https://opencode.ai/docs/providers/ ; CLI: https://opencode.ai/docs/cli/ |
| Provider list | Dialog title **“Connect a provider”**; Popular vs Providers; hints e.g. `(Recommended)`, `(API key)`, `(ChatGPT Plus/Pro or API key)`; **Other** custom id; connected ✓ | Local: `.tmp-research-plugins/opencode/packages/tui/src/component/dialog-provider.tsx` — `DialogProvider`, `providerOptions`, `createDialogProviderOptions` (~L47–L231) |
| Method fork | If `provider_auth[id].length > 1` → dialog **“Select auth method”** (labels from plugin/server) | Same file ~L148–L171; CLI twin: `packages/opencode/src/cli/cmd/providers.ts` `handlePluginAuth` — Prompt **“Login method”** |
| API key | `DialogPrompt` placeholder **“API key”**; provider-specific blurb (Zen/Go URLs); `sdk.client.auth.set({ type: "api", key })` | `dialog-provider.tsx` `ApiMethod` (~L352+) |
| OAuth auto | Show auth URL + instructions; **“Waiting for authorization…”**; `c` copy; poll `oauth.callback` | `AutoMethod` (~L239–L305) |
| OAuth code | Prompt **“Authorization code”** + URL/instructions; invalid → error | `CodeMethod` (~L314–L349) |
| After success | Replace with model picker `DialogModel` | Both Auto/Code/Api paths call `dialog.replace(() => <DialogModel …/>)` |
| Storage | `~/.local/share/opencode/auth.json` | Docs providers page; local `packages/opencode/src/auth/index.ts` |
| Web/desktop twin | Same mental model: picker → connect | `packages/app/src/components/dialog-connect-provider.tsx` (`DialogConnectProvider`, `ProviderPicker`) |

**OpenCode flow (plain):**

```text
/connect
  → DialogSelect "Connect a provider"  (searchable, Popular/Other)
  → [optional] DialogSelect "Select auth method"  (oauth | api labels)
  → api  → DialogPrompt "API key" → auth.set → DialogModel
  → oauth → authorize → AutoMethod (wait) | CodeMethod (paste code) → DialogModel
```

### next-code Face `/connect` today

| Piece | Behavior | Evidence |
|-------|----------|----------|
| Slash | `/connect [provider]`; bare run = **Message** listing providers + “Tab after /connect” | `crates/xai-grok-pager/src/slash/commands/connect.rs` `connect_run` (~L51–L80) |
| Dropdown | `suggest_args` → `tui_login_providers()` with `auth_kind.label()` | Same file `provider_arg_items`; catalog `crates/next-code-provider-metadata/src/lib.rs` `LoginProviderAuthKind` |
| Dispatch | `Action::NextCodeConnect { provider }` → `login_method_id = nextcode.{provider}` → `dispatch_login` | `crates/xai-grok-pager/src/app/dispatch/router.rs` (~L1074–L1080) |
| Embed `/login` | Alias of `/connect` (not Grok OAuth) | `crates/xai-grok-pager/src/slash/commands/login.rs` |
| Arg picker path | Model list / `provider_connect` rows also fire `NextCodeConnect` | `crates/xai-grok-pager/src/app/modals.rs` (~L728–L733) |
| Brain | ACP authenticate + Face paste box; API key → `save_named_api_key` (`*.env`); OAuth → `face_begin_scriptable` / `face_complete_scriptable`; opens browser | `src/cli/face_auth.rs` `authenticate_method`, `run_api_key_face_login`, `run_oauth_face_login` |
| Chrome | Reuses Face welcome Pending / auth URL / paste (Grok device-code chrome), gated by `external_provider` meta | `face_auth.rs` `connect_auth_method` / `get_auth_url_payload`; `crates/xai-grok-pager/src/acp/mod.rs` AuthStartMode comments |
| Auth kinds | OAuth, ApiKey, DeviceCode, Cli, Hybrid, Local — Face branches ApiKey/Local/Hybrid → key path; OAuth/DeviceCode → OAuth path; Cli bail | `face_auth.rs` ~L113–L126; metadata labels in `lib.rs` |

### Gaps vs OpenCode (UX, not brain)

| Gap | OpenCode | Face today |
|-----|----------|------------|
| Bare `/connect` | Opens provider dialog immediately | Prints help text; user must Tab or re-run with id |
| Categories / hints | Popular + per-id copy | Flat id list + auth_kind label only |
| Method chooser | Explicit when multiple methods | Catalog has Hybrid but Face picks one path by `auth_kind` enum (no “OAuth vs API key” dialog) |
| Post-connect | Jump to model select | Relies on existing session/welcome; no dedicated DialogModel equivalent |
| Credential home | One `auth.json` | Dual root (see auth-unify plan) — UX copy may still say “~/.next-code” while keys land in config `*.env` |

### Related plan (storage)

- `docs/plans/PLAN-20260722-auth-unify-opencode-vs-claude.md` — hybrid: OpenCode store + Claude/OpenCode-easy Face UX. **This UX plan can ship without finishing Phase 1 writes**, but success copy should not lie about where keys land until unify lands.

---

## Recommended approach (LOOK → PLAN only)

**Goal:** Match OpenCode’s *stepping stones* in Face, keep next-code as authority for credentials.

### copy / delete / wire map

| Kind | What |
|------|------|
| **Copy (pattern, not TS)** | OpenCode dialog sequence: provider picker → method select (when >1) → API-key prompt **or** waiting/code OAuth → optional “pick model” handoff. Titles/copy: “Connect a provider”, “Select auth method”, “API key”, “Waiting for authorization…”. Prefer Face native modals (`ArgPicker` / welcome) over inventing a Solid port. |
| **Wire** | Bare `/connect` → open Face arg-picker modal seeded with `tui_login_providers()` (same items as `suggest_args`). Selection → existing `Action::NextCodeConnect`. For Hybrid (and any future multi-method): insert a second picker (OAuth vs API key) **before** setting `login_method_id`, or encode method in method id (e.g. `nextcode.claude` vs `nextcode.anthropic-api`) using catalog ids already present. Keep `face_auth::authenticate_method` / `submit_auth_code` / URL poll as brain. |
| **Delete / stop** | Bare `/connect` **Message dump as primary UX** (keep as fallback only if picker fails). Stop framing success as “not Grok OAuth” in the middle of the happy path (move to docs/footer). Do **not** delete Grok stock `/login` for non-embed. |
| **Do not rewrite** | Login helpers, OAuth scriptable flows, provider catalog truth, daemon ACP — extend only for method disambiguation + picker open. |

### Feasible phases (after OK)

1. **UX-1 — Open picker on bare `/connect`**  
   Return `Action`/modal open instead of `CommandResult::Message`; reuse ArgPicker + `provider_connect` (already in `modals.rs`).

2. **UX-2 — Method step for Hybrid / dual catalog entries**  
   Mirror OpenCode “Select auth method”: e.g. Claude OAuth vs Anthropic API key as two options (prefer **existing provider ids** over new string protocols).

3. **UX-3 — Polish chrome**  
   Popular grouping + hint strings (from catalog or small static map like OpenCode); after success nudge `/model` or open model picker if already wired; toast with **actual** credential path once known.

4. **Store (optional same PR / or #61)**  
   Dual-write API keys into `~/.next-code/auth.json` per auth-unify Phase 1 — only if you want UX + store in one go.

---

## Files to touch (after OK)

| Area | Paths |
|------|--------|
| Face slash | `crates/xai-grok-pager/src/slash/commands/connect.rs`, `login.rs` |
| Face modals / dispatch | `crates/xai-grok-pager/src/app/modals.rs`, `dispatch/router.rs`, possibly `actions.rs` |
| Welcome / copy | `crates/xai-grok-pager/src/views/welcome/mod.rs` (labels only if needed) |
| Brain (minimal) | `src/cli/face_auth.rs` — Hybrid method branch / clearer pending mode |
| Catalog hints | `crates/next-code-provider-metadata/src/{lib,catalog}.rs` — optional Popular + hint fields |
| Tests | `connect.rs` unit tests; Face modal tests if present; `face_auth` / auth_login_flow |
| Docs | This plan; optionally `docs/AUTH_CREDENTIAL_SOURCES.md` one-liner after store change |

**Out of scope for this UX ticket:** Replacing Face with OpenCode TUI; inventing new OAuth providers; full auth.json migration (owned by auth-unify).

---

## Open questions (≤3)

1. **Ship UX before or with #61 auth.json?** Prefer UX-1/2 first (picker + method), dual-write later — confirm.
2. **Hybrid method UI:** second Face picker (“Login with Claude” vs “Anthropic API key”) vs force user to pick distinct catalog ids (`claude` / `anthropic-api`) from one list — which matches your product intent?
3. **After connect:** open model picker (OpenCode `DialogModel` parity) or only toast + stay in session?

---

## Risk

| Risk | Level | Mitigation |
|------|-------|------------|
| Bare `/connect` modal fights command palette / ArgPicker state | Med | Reuse proven `provider_connect` path in `modals.rs` |
| Hybrid users get wrong path (key vs OAuth) | Med | Explicit method step; map to existing provider ids |
| Success copy lies about `~/.next-code` while keys in `%APPDATA%` | Med | Honest path from writer; or pair with auth-unify Phase 1 |
| Stock grok `/login` regresses in non-embed | Low | Gate on `is_nextcode_embed()` (already) |

**Overall:** Medium.

---

## Status

**Waiting for your OK — reply `go ahead` to implement UX-1 (+ decide Q1–Q3).**

Worktree: `C:\Users\ADMIN\Documents\Projects\next-code-worktrees\pr-face-connect-opencode-ux`  
Branch: `pr-face-connect-opencode-ux` (from `origin/dev` @ `fb8a527b6`)

No production code in this report.
