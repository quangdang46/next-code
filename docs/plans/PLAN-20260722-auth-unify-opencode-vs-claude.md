# Plan Report — Unify next-code auth (OpenCode vs Claude Code)

## Summary (read this first)

- **You asked:** Current next-code auth is hard for users and us — unify like OpenCode or Claude Code. Claude feels easiest; which approach fits next-code better?
- **What is going on:** next-code is a **multi-provider** agent (Claude OAuth, Anthropic API key, OpenAI OAuth, OpenAI API key, Copilot, Gemini, OpenRouter, …) but credentials are **split across two roots** (`~/.next-code` vs platform config dir / `%APPDATA%\next-code`). That split is why greps, Face, and humans keep “losing” working keys. OpenCode already solved multi-provider with **one `auth.json`**. Claude Code solved **one-vendor UX** (browser `/login`, OS keychain) — great UX, wrong storage shape for our catalog.
- **We recommend:** **Hybrid — OpenCode storage + Claude-easy Face UX.**
  - **Storage / model:** follow OpenCode — one durable credential map under the product home (`~/.next-code/auth.json`, OpenCode-compatible `type: oauth | api | …` per provider id).
  - **UX:** follow Claude — first-run / Face `/connect` feels like one browser or paste flow; no “check the other folder” mental model.
  - **Do not** copy Claude’s Anthropic-only keychain-first architecture as the primary multi-provider store.
- **Risk:** Medium (migration of existing OAuth + `.env` keys; Windows path confusion).
- **Status:** Waiting for your OK — reply **go ahead** to implement. **No production code in this report.**

### Vietnamese bullets (plain)

- Auth hiện tại **khó** vì OAuth nằm `~/.next-code`, API key nằm `~/.config/next-code` (Windows: `%APPDATA%\next-code`) — hai “nhà”.
- **OpenCode** = một file `auth.json` cho mọi provider → hợp next-code đa provider.
- **Claude Code** = login một lần rất dễ, nhưng thiết kế quanh Anthropic + keychain → không đủ cho Face multi-provider.
- **Nên làm:** store kiểu OpenCode + UX kiểu Claude trong Face (`/connect`); migrate từ từ, không xóa cred cũ.

---

## How OpenCode stores / unifies auth (verified)

| Piece | Evidence |
|-------|----------|
| Single credential file | `~/.local/share/opencode/auth.json` (docs: [opencode.ai/docs/providers](https://opencode.ai/docs/providers/); troubleshooting: data dir `~/.local/share/opencode/`) |
| Path construction | Local clone: `.tmp-research-plugins/opencode/packages/opencode/src/auth/index.ts` — `path.join(Global.Path.data, "auth.json")` |
| XDG layout | `.tmp-research-plugins/opencode/packages/core/src/global.ts` — `Path.data` = `xdgData/opencode`, `Path.config` = `xdgConfig/opencode` (config ≠ secrets) |
| Schema | Flat map keyed by **provider id**: `{ type: "oauth", refresh, access, expires, … }` \| `{ type: "api", key }` \| `{ type: "wellknown", key, token }` |
| UX entry | TUI `/connect` + CLI `opencode auth login` / `auth list` — pick provider, then OAuth or paste API key |
| Env still works | Startup also loads provider env / project `.env`; Bedrock etc. stay env-chain oriented |
| Known pitfall | Stored `auth.json` can override explicit `opencode.json` provider options ([issue #10950](https://github.com/anomalyco/opencode/issues/10950)) — next-code must keep **clear precedence** when unifying |

**Why this fits next-code:** same problem (many providers, OAuth + API key), already partially mirrored — next-code **imports** OpenCode’s file in place (`crates/next-code-base/src/auth/external.rs` → `~/.local/share/opencode/auth.json`; documented in `docs/AUTH_CREDENTIAL_SOURCES.md`).

---

## How Claude Code makes login easy (verified)

| Piece | Evidence |
|-------|----------|
| Primary UX | First launch / `/login` opens browser; paste code if callback can’t reach localhost ([code.claude.com/docs/en/authentication](https://code.claude.com/docs/en/authentication)) |
| Storage (official) | macOS Keychain; Linux/Windows `~/.claude/.credentials.json` (mode `0600`); override via `CLAUDE_CONFIG_DIR` |
| Shape | Anthropic-centric OAuth blob (`claudeAiOauth`: access/refresh/expires/scopes/subscription) — not a multi-provider map |
| Precedence | Env helpers (`ANTHROPIC_API_KEY`, `apiKeyHelper`, `CLAUDE_CODE_OAUTH_TOKEN`, …) + file/keychain — DeepWiki: [claude-code-best auth system](https://deepwiki.com/claude-code-best/claude-code/8.1-authentication-system) |
| CCB multi-provider fork | Uploaded notes / README: REPL `/login` wizard for Anthropic-compatible / OpenAI / Gemini base URL + key + model ids — still a **wizard UX**, not an OpenCode-style unified store |
| Friction still exists | Keychain ACL / env vs file precedence bugs (e.g. anthropics/claude-code#9403, #68241) — “easy” UX ≠ zero ops pain |

**What to copy:** one obvious command, browser/PKCE when needed, Face chrome that never says “run login in another terminal” (next-code Face already aims here: `src/cli/face_auth.rs`).

**What not to copy as the core store:** single-vendor credentials file + OS keychain as the only durable multi-provider source.

---

## How next-code works today (split home vs config)

Canonical doc: `docs/AUTH_CREDENTIAL_SOURCES.md`. Path helpers: `crates/next-code-storage/src/lib.rs`.

| Kind | Login provider ids | Where it lives |
|------|--------------------|----------------|
| Claude OAuth | `claude` | `~/.next-code/auth.json` (`anthropic_accounts[]`) via `next_code_dir()` |
| Anthropic API key | `anthropic-api` | `ANTHROPIC_API_KEY` **or** `app_config_dir()/anthropic.env` |
| OpenAI OAuth | `openai` | `~/.next-code/openai-auth.json` |
| OpenAI API key | `openai-api` | `OPENAI_API_KEY` **or** `openai.env` under `app_config_dir()` |
| Other API keys | openrouter, zai, … | Same pattern: env **or** `app_config_dir()/<name>.env` (`next-code-provider-env`) |
| External reuse | OpenCode / pi / … | Consent-gated read-in-place (`auth/external.rs`) |

**Directory split (the pain):**

```text
next_code_dir()     → $NEXT_CODE_HOME | ~/.next-code
                    → OAuth JSON, config.toml, logs, sessions…

app_config_dir()    → $NEXT_CODE_HOME/config/next-code   (when HOME override)
                    → else dirs::config_dir()/next-code
                      Linux/macOS: ~/.config/next-code
                      Windows:     %APPDATA%\next-code
                    → *.env API keys
```

Windows docs (`docs/WINDOWS_SETUP.md`) already document this dual layout — which is exactly what confuses operators.

**Face today:** `src/cli/face_auth.rs` writes through the same login helpers as CLI (`/connect` method ids `nextcode.<provider>`). UX intent is Claude-like; **storage remains dual-rooted**, so Face cannot magically fix “where is my key?”.

**Status surface:** `next-code auth status --json` + `AuthStatus::assessment_for_provider` (`auth/mod.rs`) — keep as the single diagnostic; unify storage so assessments stop spanning two trees.

---

## Recommendation (for multi-provider next-code Face)

### Choose: **OpenCode-shaped store + Claude-shaped UX** (hybrid)

| Layer | Follow | Why |
|-------|--------|-----|
| Credential durability | **OpenCode** | One map, provider-keyed, oauth/api in the same file; matches catalog size; next-code already speaks this format for imports |
| Login / Face chrome | **Claude** | Browser or paste once; `/connect` / first-run; no second terminal |
| Config vs secrets | OpenCode XDG idea | Keep `config.toml` / settings under home (or config dir), but **secrets only in auth store** — stop writing secrets to parallel `*.env` trees as the primary path |
| Keychain | Optional later | Nice for macOS Anthropic OAuth only; not a substitute for the unified map |

### Explicit non-goals

- Replacing the provider catalog with Anthropic-only login.
- Silent OAuth override of explicit custom `baseURL` / gateway keys (learn from OpenCode #10950 — document precedence: env > explicit config > store, or warn).
- Breaking external import of OpenCode/pi/Claude Code files (keep consent-gated importers).

### Target end-state (user mental model)

1. “All my next-code credentials live under **`~/.next-code`** (or `$NEXT_CODE_HOME`).”
2. “In Face I run **`/connect`**, pick provider, paste key or finish browser — done.”
3. “`next-code auth status` tells the truth; I don’t grep `APPDATA`.”

Related platform plans (`PLAN-20260722-pi-full-custom-platform.md`) stay orthogonal: plugins/ABI ≠ auth store. Auth unify unblocks Face onboarding before plugin breadth.

---

## Phased migration (do not break existing creds)

### Phase 0 — Spec + diagnostics (no behavior change)

1. [ ] Freeze target schema (OpenCode-compatible entries + next-code multi-account extensions documented).
2. [ ] Extend `auth status --json` to always print **both** roots + which path won (home OAuth vs config `.env`).
3. [ ] Face: show credential path in connect success toast (one line).

### Phase 1 — Unified **write** path (reads stay dual)

1. [ ] New logins (CLI + Face) write API keys into **`~/.next-code/auth.json`** (or `credentials.json`) as `{ type: "api", key }` under provider id — **in addition to** or instead of `app_config_dir()/*.env` (prefer dual-write briefly).
2. [ ] OAuth continues in existing JSON files **or** folds into the same map under stable provider keys (`anthropic` / `openai` / …) while preserving multi-account labels.
3. [ ] Keep reading legacy `*.env` and old OAuth files.

### Phase 2 — Unified **read** path

1. [ ] Resolution order (proposal): process env → unified auth map → legacy `app_config_dir()/*.env` → external consented sources.
2. [ ] `auth-test` / doctor: if only legacy file has the key, offer one-shot migrate (copy into unified map; leave legacy intact).
3. [ ] Windows: update `WINDOWS_SETUP.md` / `AUTH_CREDENTIAL_SOURCES.md` to “one home” story.

### Phase 3 — Soft deprecation

1. [ ] Stop writing new secrets to `%APPDATA%\next-code\*.env` / `~/.config/next-code/*.env`.
2. [ ] Optional `next-code auth migrate` copies legacy → unified; never deletes without `--purge`.
3. [ ] After one release cycle, warn on legacy-only reads; keep forever-read for safety if cheap.

### Rollback

- Dual-read means rolling back writers restores old behavior; leave files untouched.

---

## Evidence (citations)

1. **OpenCode docs:** https://opencode.ai/docs/providers/ — `/connect` → `~/.local/share/opencode/auth.json`
2. **OpenCode local clone:** `.tmp-research-plugins/opencode/packages/opencode/src/auth/index.ts`, `packages/core/src/global.ts`
3. **Claude official auth:** https://code.claude.com/docs/en/authentication — browser `/login`, keychain / `.credentials.json`
4. **Claude-code-best DeepWiki:** https://deepwiki.com/claude-code-best/claude-code/8.1-authentication-system — multi-source resolution
5. **next-code:** `docs/AUTH_CREDENTIAL_SOURCES.md`, `docs/WINDOWS_SETUP.md`, `crates/next-code-storage/src/lib.rs` (`next_code_dir` / `app_config_dir`), `crates/next-code-base/src/auth/external.rs`, `src/cli/face_auth.rs`
6. **Uploaded notes:** `uploads/opencode-0.md`, `uploads/claude-code-1.md` (repo READMEs; CCB `/login` multi-protocol wizard)

---

## Files likely to touch (after OK)

- `crates/next-code-storage/src/lib.rs` — optional single secrets root helper
- `crates/next-code-provider-env/src/lib.rs` — read/write order
- `crates/next-code-base/src/auth/{mod,claude,codex,external,login_flows}.rs`
- `src/cli/face_auth.rs` + Face connect chrome
- `docs/AUTH_CREDENTIAL_SOURCES.md`, `docs/WINDOWS_SETUP.md`

---

## Risk

| Risk | Level | Mitigation |
|------|-------|------------|
| Users lose track of keys during migrate | Medium | Dual-read + dual-write; never delete legacy by default |
| Multi-account Claude/OpenAI shape vs flat OpenCode map | Medium | Keep account arrays **or** namespaced keys (`anthropic:label`); document |
| Windows `%APPDATA%` vs `%USERPROFILE%\.next-code` confusion | Medium | One documented home; status prints absolute paths |
| OAuth store overriding custom gateway | Low–Med | Explicit precedence + warn (OpenCode #10950 lesson) |
| Keychain-only expectation from Claude users | Low | Optional later; Face UX first |

**Overall:** Medium.

---

## Status

**Waiting for your OK — reply `go ahead` to implement Phase 0–1.**

No code changes in this report.
