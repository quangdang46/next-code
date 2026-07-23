# BUG — Face chrome model ≠ daemon turn model (provider key / switch ack)

Branch: `pr-face-provider-persist-fix`  
Related: `BUG-20260722-face-model-select-persist.md`, failover notice/countdown plans  
Status: **Implemented**

## Summary (read this first)

- **Symptom:** Badge shows `deepseek-v4-flash`; turn / error mentions `kimi-k2.7-code` on OpenCode Go (`opencode.ai/zen/go`); Face also prints `Couldn't switch model to kimi-k2.7-code: OPENROUTER_API_KEY not found` then continues chatting with chrome still on deepseek.
- **Bug class:** **Provider / model management** — display-name provider pin + fire-and-forget ACP switch + misleading failure copy. Not “kimi missing on Console Go” as the root story (400 on kimi is a **secondary** upstream symptom when the runtime actually is kimi).
- **Verified root:** Face persists / routes with `default_provider = "opencode go"` (display label). Daemon only accepts catalog id `opencode-go`, so `deepseek-v4-flash` is applied **bare** → OpenRouter auth path → fail → runtime stays / falls back to OpenCode Go default `kimi-k2.7-code`. Face chrome still shows deepseek because SetSessionModel returns OK before the daemon finishes, and the error path does not refresh chrome to the fallback model.
- **User ask “là sao / quản lí provider”:** Correct instinct — chrome and daemon disagree on which provider+model are active.

## Fix shipped

1. **Normalize provider pin on write** — `config_provider_key_from_float` + `derive_session_provider_key` map display names via `resolve_login_provider_loose` (`OpenCode Go` / `opencode go` → `opencode-go`).
2. **Normalize on read/apply** — `resolve_openai_compatible_profile_selection` uses loose resolve; `model_switch_request_for_session_model` emits catalog id prefixes.
3. **ACP SetSessionModel** — waits for matching `ModelChanged` before Ok; Err rolls Face chrome via `prev_model_id`.
4. **Error copy** — `ModelChanged.model` on failure is the **requested** model; `fallback_model` carries daemon stay-on; pager/TUI format `Couldn't switch to X (staying on Y)`.

## Ranked hypotheses

| # | Hypothesis | Verdict |
|---|------------|---------|
| 1 | Face writes display name `"opencode go"` instead of `"opencode-go"` → bare model → OpenRouter key required → fail → kimi fallback | **Verified** (config + logs + code) |
| 2 | ACP `SetSessionModel` returns success before daemon `ModelChanged`; Face commits chrome to requested model even when switch later fails | **Verified** |
| 3 | Error text uses **fallback** model id (`kimi`), so user thinks switch-*to*-kimi failed | **Verified** |
| 4 | Failed switch leaves half-applied state (chrome deepseek, daemon kimi) | **Verified** |
| 5 | Persist stores kimi while chrome shows deepseek | **Partially** — cold-start apply fails so daemon default is kimi; config still has `default_model=deepseek-v4-flash` + bad provider pin |
| 6 | Pure OpenCode Go 400 / model-not-on-Go | **Secondary** — some kimi turns succeed, some 400; mismatch is primary |

## Evidence

### Operator disk + live log (`~/.next-code`, 2026-07-23)

```toml
[provider]
default_model = "deepseek-v4-flash"
default_provider = "opencode go"   # ← space / display name, not catalog id
```

Log (repeated all morning):

```text
Failed to apply config default_model 'deepseek-v4-flash'
  (via 'deepseek-v4-flash', provider=Some("opencode go")):
    OPENROUTER_API_KEY not found ...;
  falling back to provider default kimi-k2.7-code
```

Note `via 'deepseek-v4-flash'` — **no** `opencode-go:` prefix. Comment in `agent.rs` documents exactly this failure mode.

### Why `"opencode go"` breaks routing

1. Face `config_provider_key_from_float("OpenCode Go")` → lowercase `"opencode go"` (no mapping to `opencode-go`).
2. Catalog id is `opencode-go`, display `"OpenCode Go"`.
3. Bare `deepseek-v4-flash` mis-routes to OpenRouter slot → `OPENROUTER_API_KEY not found`.

## Smoke after fix

1. Set `default_provider = "opencode-go"`, pick deepseek in Face → log shows `via 'opencode-go:deepseek-v4-flash'`, no OPENROUTER warning, chrome = deepseek, next turn model = deepseek.
2. Force bad pin `"opencode go"` once → still normalizes and applies.
3. Force switch fail → chrome reverts; message names **requested** model; no “Couldn't switch to kimi” when requesting deepseek.
4. Quit/reopen → same model on badge and turn.

## Citations

| Piece | Path |
|-------|------|
| Bare deepseek → OpenRouter comment | `crates/next-code-app-core/src/agent.rs` |
| Prefix + display-name normalize | `crates/next-code-base/src/provider/selection.rs` |
| Face float → config key | `crates/xai-grok-pager/.../setters.rs` `config_provider_key_from_float` |
| Loose profile resolve | `crates/next-code-base/src/provider_catalog.rs` |
| ACP wait for ModelChanged | `src/cli/pager_agent.rs` `set_session_model` |
| Requested + fallback on Err | `crates/next-code-app-core/src/server/provider_control.rs` |
| Chrome revert on ACP Err | `crates/xai-grok-pager/.../lifecycle.rs` |

## Decision gate

**Implemented** — full fix (1)–(4) on `pr-face-provider-persist-fix`.
