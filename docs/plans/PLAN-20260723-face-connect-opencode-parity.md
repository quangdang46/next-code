# Plan Report — Face `/connect` OpenCode parity (full wire)

## Summary

- **Problem:** Face `/connect` listed only the static ~45 TUI catalog rows; OpenCode Connect dumps **~170** models.dev providers + synthetic **Other** (custom id).
- **Fix:** Source the Face Connect ArgPicker from a bundled models.dev snapshot (OpenCode `provider_next.all` twin): Popular 6 + searchable rest + Other custom. Long-tail / custom ids store API keys in `~/.next-code/auth.json` by provider id (OpenCode `auth.set`). Keep special OAuth/device/multi-step for openai / anthropic / google / copilot / bedrock / azure.
- **Risk:** Medium (long-tail credentials are stored; runtime routing still needs provider config like OpenCode).
- **Status:** Implemented on `pr-face-connect-opencode-ux` (PR #73).

## Evidence (verified)

| Source | Citation |
|--------|----------|
| OpenCode TUI connect | `.tmp-research-plugins/opencode/packages/tui/src/component/dialog-provider.tsx` — `PROVIDER_PRIORITY`, `providerOptions`, `CUSTOM_PROVIDER_OPTION_VALUE`, `normalizeCustomProviderID` |
| Popular ids | `opencode`, `opencode-go`, `openai`, `github-copilot`, `anthropic`, `google` |
| models.dev | live `https://models.dev/api.json` → **170** providers (2026-07-23) |
| next-code pricing twin | `crates/next-code-base/src/model_pricing.rs` (prices only; Connect list is separate snapshot) |

## Provider matrix (OpenCode → next-code Face)

| OpenCode id | Face auth id | Auth | Face status |
|-------------|--------------|------|-------------|
| opencode | opencode | API key | Wired |
| opencode-go | opencode-go | API key | Wired |
| openai | openai (+ openai-api) | OAuth / API key | Wired (method step) |
| github-copilot | copilot | device code | Wired |
| anthropic | claude (+ anthropic-api) | OAuth / API key | Wired (method step) |
| google | gemini (+ gemini-api) | OAuth / API key | Wired (method step) |
| amazon-bedrock | bedrock | API key | Wired |
| azure | azure | multi-step paste | Wired (Entra = CLI) |
| (models.dev rest ~164) | models.dev id | API key → auth.json | **Wired** |
| Other custom | free-text id | API key → auth.json | **Wired** |

## copy / delete / wire

| Kind | What |
|------|------|
| **Copy** | OpenCode Popular order + blurbs; models.dev id/name list; Other custom id regex + credential-only store |
| **Wire** | `build_connect_family_items` ← models.dev; `save_api_key_for_provider_id`; Face custom Other two-step paste |
| **Delete** | Connect list dependence on static-only `tui_login_providers()` |

## Files

- `docs/plans/PLAN-20260723-face-connect-opencode-parity.md`
- `crates/next-code-provider-metadata/assets/models_dev_providers.json`
- `crates/next-code-provider-metadata/src/connect_catalog.rs`
- `crates/xai-grok-pager/src/slash/commands/connect.rs`
- `crates/next-code-provider-env/src/unified_auth.rs`
- `src/cli/face_auth.rs`

## Verification note (2026-07-23) — catalog size honesty

| Surface | Count | Source |
|---------|------:|--------|
| OpenCode Connect list | **~170** (+ 1 synthetic **Other**) | `ModelsDev.get()` → `provider_next.all` |
| Face `/connect` picker | **~170** (+ 1 synthetic **Other**) | bundled `models_dev_providers.json` |
| Overlap | Popular 6 + long-tail | mapped specials keep next-code auth ids |

**Verdict:** List-source parity with OpenCode Connect (models.dev dump + Other custom), not the prior static TUI catalog.
