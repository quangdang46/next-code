# Plan Report — Face `/connect` OpenCode parity

## Summary

- **Problem:** Face `/connect` Popular/Providers catalog diverges from OpenCode; Bedrock (and peers) appear but Face API-key login is unwired → fatal Quit screen.
- **Fix:** Copy OpenCode TUI connect shape (`dialog-provider.tsx`): Popular order + descriptions, **Other** section, only Face-wired methods; after auth keep PR #72 model picker. Delete sparse nextcode-only Popular list and dead Face API-key arms that crash.
- **Risk:** Low–medium (picker + catalog filter; CLI Bedrock/Azure remain).
- **Status:** Implemented — PR https://github.com/quangdang46/next-code/pull/73 (`38d6050f1`); issue #74.

## Evidence (verified)

| Source | Citation |
|--------|----------|
| OpenCode TUI connect | `.tmp-research-plugins/opencode/packages/tui/src/component/dialog-provider.tsx` — `PROVIDER_PRIORITY`, `providerOptions`, `ApiMethod`, `DialogModel` handoff |
| OpenCode Popular ids | `opencode`, `opencode-go`, `openai`, `github-copilot`, `anthropic`, `google` |
| OpenCode Go key copy | `ApiMethod` → body + `https://opencode.ai/go`; Zen → `https://opencode.ai/zen` |
| Face picker today | `crates/xai-grok-pager/src/slash/commands/connect.rs` — `POPULAR_CONNECT_IDS`, `build_connect_family_items` |
| Bedrock crash | `src/cli/face_auth.rs` `run_api_key_face_login` bail for unwired targets |
| Post-connect model | PR #72 `open_model_picker_after_auth` |

## copy / delete / wire

| Kind | What |
|------|------|
| **Copy** | OpenCode Popular order + per-id descriptions; Zen/Go setup URLs; StepFun Step Plan + Mixlayer as OpenAI-compatible API-key rows under Other |
| **Wire** | Face picker lists only Face-wired providers; `face_auth` rejects unwired with clear error (no fatal “not wired” for listed items) |
| **Delete** | Old Popular ids (`openrouter`/`xai` as Popular anchors); Popular dedupe by `OpenRouterLike` key (hid Zen/Go); dead `LoginProviderTarget::Gemini` API-key arm; Bedrock/Azure/AutoImport from Face connect list |

## Files

- `docs/plans/PLAN-20260723-face-connect-opencode-parity.md`
- `crates/xai-grok-pager/src/slash/commands/connect.rs`
- `crates/next-code-provider-metadata/src/{lib,catalog}.rs`
- `src/cli/face_auth.rs`

## Out of scope

- Full models.dev (159) catalog dump
- Wiring Face Bedrock/Azure/AutoImport (CLI keeps them)
- Auth.json unify (#61)
