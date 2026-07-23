# Auth Credential Sources (single source of truth)

This document exists because the same confusion keeps recurring: an agent (or a
human) greps for `ANTHROPIC_API_KEY` / `sk-ant-api`, finds nothing, reads a stale
`auth-validation.json` entry that says "expired", and wrongly concludes "there is
no working credential". In reality the credential is present and working; it just
lives somewhere the naive search did not look.

If you are debugging "does provider X have a credential?", read this first.

## Unified home (preferred)

All next-code credentials belong under **`~/.next-code`** (or `$NEXT_CODE_HOME`):

| Path | Contents |
|------|----------|
| `~/.next-code/auth.json` | **Unified store** — Claude OAuth in `anthropic_accounts[]`; API keys and single-account OpenCode-shaped entries as flat provider map (`anthropic` / `openai` / `openrouter` / … → `{ "type": "api", "key": "…" }` or `{ "type": "oauth", … }`) |
| `~/.next-code/openai-auth.json` | OpenAI / Codex OAuth (still separate file during transition; dual-read) |
| `~/.next-code/config.toml` | Non-secret settings |

**Resolution order for API keys:** process env → unified `auth.json` → legacy `app_config_dir()/*.env` → trusted external imports / secrets fallbacks.

Face `/connect` and `next-code login` write API keys into **`auth.json`** (not `%APPDATA%\next-code\*.env`). Legacy `*.env` files are still **read**. Copy them in with:

```sh
next-code auth migrate
```

(`--purge` is reserved and currently refused — migrate never deletes.)

## The two "dual-auth" providers: Anthropic and OpenAI

Anthropic/Claude and OpenAI each support **two completely independent credential
paths**, surfaced as **two separate login providers**:

| Concept            | Login provider id | Auth kind | Where the credential lives |
|--------------------|-------------------|-----------|----------------------------|
| Claude, OAuth/sub  | `claude`          | OAuth     | `~/.next-code/auth.json` → `anthropic_accounts[].access` (`sk-ant-oat...`) |
| Claude, API key    | `anthropic-api`   | API key   | `ANTHROPIC_API_KEY` env **or** `auth.json` → `anthropic: {type:api,key}` **or** legacy `~/.config/next-code/anthropic.env` |
| OpenAI, OAuth      | `openai`          | OAuth     | `~/.next-code/openai-auth.json` (Codex/ChatGPT login) |
| OpenAI, API key    | `openai-api`      | API key   | `OPENAI_API_KEY` env **or** `auth.json` → `openai: {type:api,key}` **or** legacy `openai.env` |

### Provider id mapping (frozen)

| Login id / env | Unified `auth.json` map key | Shape |
|----------------|-----------------------------|-------|
| `claude` | `anthropic_accounts[]` (+ `active_anthropic_account`) | next-code multi-account OAuth |
| `anthropic-api` / `ANTHROPIC_API_KEY` | `anthropic` | OpenCode `{type:api,key}` |
| `openai-api` / `OPENAI_API_KEY` | `openai` | OpenCode `{type:api,key}` |
| `openrouter` / … | OpenCode provider slug (see `provider_ids_for_env_key`) | `{type:api,key}` |

Multi-account choice: **keep account arrays** for Claude OAuth; use **OpenCode flat entries** for single-account API keys (and future single-account OAuth folded into the same map). Do **not** replace `anthropic_accounts` with namespaced keys.

Key facts that trip people up:

- The **OAuth token is not an API key.** Anthropic OAuth tokens are `sk-ant-oat01-...`
  (and refresh tokens `sk-ant-ort01-...`). A direct API key is `sk-ant-api03-...`.
- **`auth.json` holds both** OAuth accounts and API-key map entries. Grepping only
  for `anthropic_accounts` misses API keys under `anthropic.type == "api"`.
- Legacy **`~/.config/next-code/*.env`** (Windows: `%APPDATA%\next-code\*.env`) is
  still dual-read but is no longer the primary write path for catalog API keys.
- `claude` and `anthropic-api` are **different providers**. Having a Claude
  subscription login (OAuth) does **not** make `anthropic-api` usable, and vice versa.

### How to actually check (don't guess)

```sh
# The honest, normalized answer for every provider:
next-code auth status --json
```

The JSON report always includes absolute paths:

- `next_code_home`
- `unified_auth_json`
- `app_config_dir` (legacy `*.env` root)

Each provider entry reports `status`, `auth_kind` ("OAuth" vs "API key"),
`credential_source` (env var / next-code-managed file / app config file / …), and the
exact `method`. This is the canonical surface; prefer it over grepping files.

Programmatically, the single source of truth is
`AuthStatus::assessment_for_provider(descriptor)` in
`crates/next-code-base/src/auth/mod.rs`, which returns a `ProviderAuthAssessment`.

## Selecting a default via config

`~/.next-code/config.toml`:

```toml
[provider]
default_provider = "claude"        # Claude subscription (OAuth)
# default_provider = "anthropic-api" # Claude via direct Anthropic API key
default_model = "claude-opus-4-8"
anthropic_reasoning_effort = "xhigh"
```

- `default_provider = "claude"` uses the OAuth/subscription credential.
- `default_provider = "anthropic-api"` uses the direct API key. In this mode the
  runtime **does not** fall back to OAuth: if no API key is configured the request
  fails. Prefer `auth.json` (`anthropic` api entry) or `ANTHROPIC_API_KEY`.

The full alias/vocabulary mapping (runtime env, route stable-id, CLI `--provider`,
model prefix) is centralized in
`crates/next-code-provider-core/src/auth_mode.rs` (`AuthRoute`). Do not re-parse these
strings by hand; go through `AuthRoute`.

## Why "expired" was misleading: validation cache is not live state

`~/.next-code/auth-validation.json` caches the result of the **last** runtime
auth-test per provider. It is a historical record, not the current credential
state. An OAuth token that has since auto-refreshed can still show a days-old
"validation failed / expired" entry here.

To avoid presenting stale records as current fact, `format_record_label`
(`crates/next-code-base/src/auth/validation.rs`) flags any record older than
`doctor::VALIDATION_STALE_AFTER_MS` (7 days) as `stale, ... re-validate`. Treat a
stale record as "unknown, re-check", never as ground truth. Re-validate with:

```sh
next-code auth-test --provider <id>
```

## Quick decision tree for "is provider X authenticated?"

1. Run `next-code auth status --json` and read the entry for the **specific** login
   provider id (`claude` vs `anthropic-api` are different!).
2. If you must inspect files: OAuth Claude → `auth.json` `anthropic_accounts`;
   API keys → `auth.json` flat map, then env, then legacy `*.env`.
3. Ignore `auth-validation.json` verdicts older than 7 days (shown as `stale`);
   re-run `next-code auth-test` instead.

## Importing credentials from other agent tools

On a fresh install next-code can **reuse logins left behind by other coding
agents**, both OAuth tokens and API keys. Detection is consent-gated: next-code
lists the sources it found and only reads them after you approve each one
(`crates/next-code-base/src/auth/external.rs`, `unconsented_sources` /
`trust_external_auth_source`). Nothing is copied into next-code's own stores; the
external file is read in place.

Shared `auth.json`-style sources (`ExternalAuthSource`):

| Tool      | Auth file path                     | On-disk shape                                                                 |
|-----------|------------------------------------|------------------------------------------------------------------------------|
| OpenCode  | `~/.local/share/opencode/auth.json`| flat `{ provider: { type: "oauth", access, refresh, expires } \| { type: "api", key } }` |
| pi        | `~/.pi/agent/auth.json`            | flat `{ provider: { type: "oauth", ... } \| { type: "api_key", key } }` (key may be `$ENV` ref) |
| OpenClaw  | `~/.openclaw/agent/auth.json`, `~/.openclaw/agents/<id>/agent/auth-profiles.json`, `~/.openclaw/agents/<id>/agent/auth.json`, `~/.openclaw/credentials/oauth.json` | legacy flat pi shape, or the current `{ "profiles": { "<provider>:<name>": ... } }` store (first existing path wins; `main` agent and `:default` profiles preferred) |
| Hermes    | `~/.hermes/auth.json`              | nested `{ credential_pool: { provider: [ { auth_type, access_token, refresh_token, expires_at_ms } ] }, providers: {...} }` |

Notes:

- pi/OpenClaw API-key values that are `$ENV_VAR` references are resolved against
  the environment; values that begin with `!` (shell commands) are **never
  executed** and are skipped.
- Hermes stores literal API keys in the `access_token` field of `api_key`
  credential-pool entries; many of its providers store only env-var *names*, so
  those import nothing unless the env var is set.
- Other tool-specific importers exist for Claude Code, Codex, Gemini CLI,
  GitHub Copilot, and Cursor (see `auth/claude.rs`, `auth/codex.rs`,
  `auth/gemini.rs`, `auth/copilot.rs`, `auth/cursor.rs`).
