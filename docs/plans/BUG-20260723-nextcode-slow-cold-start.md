# BUG — `nextcode` cold start waits a long time for server

Branch: _(none yet — plan only)_  
Related: `BUG-20260723-face-startup-session-load.md` (MCP spinner — secondary UX), `BUG-20260723-face-model-list-empty-on-start.md` / #71 (catalog after attach — different bug)  
Status: **Waiting for OK** — reply **go ahead** to implement

## Summary (plain English)

Cold `nextcode` (no live daemon) blocks on **“Starting server…”** for ~3s before Face can attach. Almost all of that is **`serve` provider bootstrap**, not Face paint, MCP, or the launcher symlink.

Measured root: `AuthStatus::check_fast()` → OpenRouter/OpenAI-compat credential probe → `get_api_key()` autodiscovers among **36** compatible profiles by repeatedly calling `load_api_key_from_env_or_config`. On this machine that path alone is ~2.6–3.2s. Setting a dummy `OPENROUTER_API_KEY` (short-circuits autodetection) drops the same probe to ~0ms and cold serve bootstrap to ~300ms.

Config already pins `default_provider = "opencode-go"`, but that preference is applied **after** `check_fast`, so the expensive scan still runs on every cold serve.

## Verified root cause

**Blocking seam (Windows cold path):**

```
nextcode.exe (launcher)
  → run_default_command (dispatch.rs)
  → spawn_server → child: next-code.exe serve
  → Serve blocks until provider_init finishes
  → ProviderChoice::Auto → detect_auto_provider_flags
  → AuthStatus::check_fast → probe_openrouter_status
  → openrouter::has_credentials → get_api_key
  → configured_api_key_name/env_file → autodetected_openai_compatible_profile
  → scans all 36 OpenAiCompatibleProfile entries (no early exit; collect-then-uniq)
  → only then Server::new + listen socket
  → client connect succeeds → run_face_pager
```

Socket listen is gated on provider init (`dispatch.rs` Serve arm logs `[TIMING] serve bootstrap` **before** `server.run()`). Windows client polls until the socket accepts (`spawn_server` non-unix loop).

### Measurement (this machine, 2026-07-23)

| Run | `auth_check_fast` | of which `openrouter=` | `serve bootstrap` |
|-----|-------------------|------------------------|-------------------|
| Cold serve, only `opencode-go.env` present | 3453ms / 2799ms | **3188ms / 2626ms** | 3661ms / 2941ms |
| Same + `OPENROUTER_API_KEY` env set (A/B) | **179ms** | **0ms** (absent from nonzero) | **321ms** |
| Earlier same day (morning) | detect ~70–150ms | _(AUTH_TIMING off)_ | ~100–700ms |

Log lines (excerpt):

```text
[TIMING] auth_check_fast: total=3453ms, nonzero=[anthropic=35ms, openrouter=3188ms, azure=31ms, ...]
[TIMING] auto_provider_bootstrap: detect=3453ms, ..., final_has_any=true
[TIMING] serve bootstrap: provider_init=3661ms, server_new=1ms, before_run=3662ms
```

A/B short-circuit:

```text
# with OPENROUTER_API_KEY
auth_check_fast: total=179ms, nonzero=[anthropic=29ms, azure=26ms, ...]  # no openrouter=
serve bootstrap: provider_init=321ms

# without (opencode-go.env only)
auth_check_fast: total=2799ms, nonzero=[..., openrouter=2626ms, ...]
serve bootstrap: provider_init=2941ms
```

Enable breakdown with `NEXT_CODE_AUTH_TIMING=1` (`auth_timing_logging_enabled` in `crates/next-code-base/src/auth/mod.rs`).

### Why the OpenRouter probe is slow (code)

| Mechanism | Evidence |
|-----------|----------|
| 36-profile full scan | `OPENAI_COMPAT_PROFILES: [OpenAiCompatibleProfile; 36]` in `crates/next-code-provider-metadata/src/catalog.rs` |
| No early-return on first hit | `autodetected_openai_compatible_profile` collects **all** configured matches, returns only if `len() == 1` (`openrouter.rs` ~110–128) |
| Double scan per `get_api_key` | `configured_api_key_name()` **and** `configured_env_file_name()` each call autodetection |
| Per-miss expensive fallbacks | `load_api_key_from_env_or_config` → secrets resolver + external auth fallbacks (`startup.rs` registers both) |
| Secrets miss → `git rev-parse` | `secrets_api_key_resolver` → `environment_id_from_cwd` → `git rev-parse --show-toplevel` **even when** `secrets/local.age` is absent |
| Microbench | 70× `git rev-parse` in this repo ≈ **1881ms** (matches majority of the openrouter bucket) |
| Windows ACL harden on miss | `harden_secret_file_permissions` rewrites parent dir ACL even when the `*.env` file is missing |

User config already has `[provider] default_provider = "opencode-go"` (`~/.next-code/config.toml`). Serve logs apply that **after** detect:

```text
Using preferred provider 'opencode-go' from config via OpenAI-compatible profile OpenCode Go
```

So the pin does not protect the cold `check_fast` path.

## Ranked hypotheses (status)

| # | Hypothesis | Verdict |
|---|------------|---------|
| 1 | Cold serve blocked on OpenRouter/compat credential autodetection (36-profile scan + git/secrets/ACL per miss) | **Verified** — AUTH_TIMING + A/B |
| 2 | Config `default_provider` unused during `check_fast` | **Verified** — applied after detect |
| 3 | Launcher / rustc / install scripts | **Ruled out** — wait is inside spawned `serve` before listen |
| 4 | Face MCP “Starting session…” 30s seed | **Secondary UX only** — after socket; see startup-session-load bug |
| 5 | #71 model catalog pump | **Ruled out as primary** — Face inventory after attach; does not delay socket |
| 6 | `model_routes` ~1s on subscribe | **Secondary** — after UI attach; can delay “session ready” feel |
| 7 | Session search index warmup (~1.5s for 1000+ external sessions) | **Background after listen** — not on client wait-for-socket critical path |
| 8 | Auth.json unify commit alone | **Contributing surface** (`bc0b14158` dual-read) but A/B shows **scan short-circuit** is the decisive gate; unify is not required to explain the openrouter bucket |

## What still needs measurement (optional)

- Exact per-call split inside openrouter bucket (git vs ACL vs file IO) — not required to fix; git alone already explains ~2/3 of cost.
- Whether deferring `provider_init` until after listen is safe for first subscribe (larger design).

## Recommended fix (smallest)

Prefer **one** of these (smallest first):

1. **Config-first credential resolve (preferred)**  
   In `openrouter::get_api_key` / `has_credentials` / `autodetected_openai_compatible_profile`, if `config.provider.default_provider` maps to an openai-compatible profile (e.g. `opencode-go`), probe **that** env/file first and skip the 36-way scan when it hits.

2. **Memoize autodetection** once per process so `configured_api_key_name` + `configured_env_file_name` do not double-pay.

3. **Cheap secrets miss** — if `secrets/local.age` (or secrets dir) is absent, skip `environment_id_from_cwd` / git entirely in `secrets_api_key_resolver`.

4. **Do not ACL-harden missing files** during presence probes (harden only when a secret file actually exists / is written).

5. **(Larger)** Listen on the socket before finishing full provider probe so Face can attach while auth completes — only if (1)–(4) are insufficient for “jcode pride.”

### Prove

- `NEXT_CODE_AUTH_TIMING=1 next-code serve` cold: `openrouter=` ≪ 200ms when `default_provider=opencode-go` and key only in that profile’s env file.
- Cold `nextcode` with no daemon: client `server_ready` / time-to-socket ≪ 1s on this machine.
- Unit: autodetection with pinned default provider does not call `openai_compatible_profile_is_configured` for unrelated profiles.
- Regression: multi-compat keys still resolve unambiguously (exactly-one / preferred-pin rules documented).

### Out of scope for this bug

- MCP init spinner clear (secondary Face UX).
- #71 `/model` catalog bridge (already implemented separately).
- Killing background registry / session-index warmups (nice-to-have, not the cold “Starting server…” wait).

## Files to touch (when building)

| Path | Change |
|------|--------|
| `crates/next-code-base/src/provider/openrouter.rs` | Config-first / memoized autodetection |
| `crates/next-code-provider-env/src/lib.rs` | Optional: skip harden on missing path |
| `crates/next-code-secrets/src/resolver.rs` | Skip git when secrets store absent |
| Tests under `provider` / `auth` | Pin + miss-path timing/regression |

## Risk

Low–medium. Wrong preferred-provider short-circuit could hide a second configured compat provider or change multi-match `None` behavior. Mitigate by: pin wins only when config default is set and that profile’s key resolves; keep full scan as fallback when pin misses; add unit coverage for multi-key ambiguity.

## Decision gate

**Waiting for OK** — reply **go ahead** to implement. Do not ship MCP-only or Face-only changes as the fix for cold “Starting server…” lag.
