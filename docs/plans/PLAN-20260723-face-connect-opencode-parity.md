# Plan Report — Face `/connect` OpenCode parity (full wire)

## Summary

- **Problem:** Face `/connect` catalog/auth diverged from OpenCode; unwired targets (Bedrock/Azure/AutoImport) crashed with Quit, then a partial PR **hid** them — user rejected that as incomplete.
- **Fix:** Copy OpenCode connect shape end-to-end: Popular order + descriptions + Other section + method step + post-auth model picker; **wire Face login for every TUI catalog provider** (Bedrock API key, Azure multi-step paste, Auto Import). Delete hide-filter / dual “not wired” path.
- **Risk:** Medium (Azure multi-step UX is Face paste chain, not full OpenCode prompt widgets; Entra stays CLI).
- **Status:** Implementing on `pr-face-connect-opencode-ux` (extends `38d6050f1` / issue #74 / PR #73).

## Evidence (verified)

| Source | Citation |
|--------|----------|
| OpenCode TUI connect | `.tmp-research-plugins/opencode/packages/tui/src/component/dialog-provider.tsx` — `PROVIDER_PRIORITY`, `providerOptions`, `ApiMethod`, `DialogModel` |
| Popular ids | `opencode`, `opencode-go`, `openai`, `github-copilot`, `anthropic`, `google` |
| Zen/Go copy + URLs | `ApiMethod` → `opencode.ai/zen`, `opencode.ai/go` |
| Legacy TUI Bedrock/Azure | `crates/next-code-tui/src/tui/app/auth.rs` `start_bedrock_login` / `start_azure_login` |
| CLI twins | `src/cli/login.rs` `login_bedrock_flow` / `login_azure_flow` / AutoImport |

## Provider matrix (OpenCode → next-code Face)

| OpenCode id | next-code id | Auth | Face status |
|-------------|--------------|------|-------------|
| opencode | opencode | API key | Wired (OpenAI-compat) |
| opencode-go | opencode-go | API key | Wired |
| openai | openai (+ openai-api) | OAuth / API key | Wired (method step) |
| github-copilot | copilot (+ alias) | device code | Wired |
| anthropic | claude (+ anthropic-api) | OAuth / API key | Wired (method step) |
| google | gemini (+ gemini-api) | OAuth / API key | Wired |
| (models.dev rest) | TUI catalog Other | mostly API key | Wired via OpenAiCompatible |
| amazon-bedrock | bedrock | API key | **Wired** (default region us-east-2) |
| azure | azure | prompts + key | **Wired** (endpoint→model→key; Entra = CLI follow-up) |
| Other custom | auto-import / custom ids | import / key | Auto Import wired; arbitrary custom id = OpenCode-only follow-up |

## copy / delete / wire

| Kind | What |
|------|------|
| **Copy** | OpenCode Popular order + blurbs; Zen/Go URLs; StepFun + Mixlayer under Other |
| **Wire** | Bedrock Face API-key + region; Azure Face multi-step paste; Auto Import; list **all** `tui_login_providers()` |
| **Delete** | `face_connect_wired` hide filter; Face bail “not available / not wired” for listed providers |

## Follow-up (explicit, not blocking)

- Azure Entra ID in Face (CLI today).
- OpenCode “Other → custom provider id” free-text credential (models.dev dump beyond TUI catalog).
- Richer Azure step labels in welcome chrome (today reuses portal URL + paste box).

## Files

- `docs/plans/PLAN-20260723-face-connect-opencode-parity.md`
- `crates/xai-grok-pager/src/slash/commands/connect.rs`
- `crates/next-code-provider-metadata/src/catalog.rs`
- `src/cli/face_auth.rs`
