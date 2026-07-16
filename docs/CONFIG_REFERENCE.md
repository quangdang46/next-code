# Configuration files reference

This document covers the **on-disk configuration files** next-code reads, with a
focus on **offline** / **air-gapped** / **self-hosted** setups (issue
[#48](https://github.com/quangdang46/next-code/issues/48)). All paths are
relative to your next-code home, which is `~/.next-code/` by default (or
`$NEXT_CODE_HOME` if set; legacy `$JCODE_HOME` is dual-read; or `$XDG_DATA_HOME/next-code` when
[`NEXT_CODE_USE_XDG=1`](#xdg-mode) is enabled). Existing `~/.jcode` data is migrated into `~/.next-code` on first run. Project-local files prefer `.next-code/` and fall back to `.jcode/`.

## tl;dr

For a fully offline setup against a local OpenAI-compatible endpoint
(vLLM, llama.cpp, LM Studio, Ollama, etc.):

```toml
# ~/.next-code/config.toml
[provider]
default_provider = "local-vllm"
default_model    = "Qwen/Qwen3-Coder-30B-A3B-Instruct"

[providers.local-vllm]
type        = "openai-compatible"
base_url    = "http://localhost:8000/v1"
default_model = "Qwen/Qwen3-Coder-30B-A3B-Instruct"

[[providers.local-vllm.models]]
id            = "Qwen/Qwen3-Coder-30B-A3B-Instruct"
context_window = 128000
```

Plus run with `--offline` (or `NEXT_CODE_OFFLINE=1`) to disable the update
check + telemetry:

```bash
next-code --offline --provider-profile local-vllm
```

## Locations

| Path | Purpose | Format |
|---|---|---|
| `~/.next-code/config.toml` | Main config (providers, defaults, features) | TOML |
| `~/.next-code/auth.json` | Anthropic / Claude OAuth credentials | JSON |
| `~/.next-code/openai-auth.json` | OpenAI / Codex OAuth credentials | JSON |
| `~/.next-code/gemini_oauth.json` | Gemini OAuth credentials | JSON |
| `~/.next-code/mcp.json` | Global MCP server registry | JSON |
| `.next-code/mcp.json` (project; dual-read `.jcode/mcp.json`) | Project-local MCP servers | JSON |
| `~/.next-code/prompts/*.md` | User-level prompt templates | Markdown |
| `.next-code/prompts/*.md` (project) | Project-level prompt templates | Markdown |
| `~/.next-code/SYSTEM.md` | Global system-prompt override | Markdown |
| `~/.next-code/APPEND_SYSTEM.md` | Global system-prompt append | Markdown |
| `.next-code/SYSTEM.md` (project) | Project system-prompt override | Markdown |
| `.next-code/APPEND_SYSTEM.md` (project) | Project system-prompt append | Markdown |
| `~/.next-code/sessions/` | Persisted session state (autosaved) | JSON per file |
| `~/.next-code/logs/next-code-YYYY-MM-DD.log` | Daily log output | text |
| `~/.config/next-code/<provider>.env` | Per-provider env-file overrides | dotenv |

> Project-level files always win over user-level files of the same name.

## `~/.next-code/config.toml` — main config

The main config has the following top-level tables (all optional):

```toml
[provider]
default_provider = "anthropic"     # provider key (oauth or compat profile)
default_model    = "claude-sonnet-4-5"

[providers.<name>]                 # OpenAI-compatible profile
type          = "openai-compatible"
base_url      = "https://...:port/v1"
default_model = "..."
api_key_env   = "MY_API_KEY"       # optional; reads env var
env_file      = "my-provider.env"  # optional; reads ~/.config/next-code/<file>
no_api_key    = false              # set true for local servers without auth

[[providers.<name>.models]]
id            = "model-id"
context_window = 128000

[features]
memory   = true   # enable embedding-based memory (requires local model)
swarm    = true   # multi-agent collaboration in same repo

[ambient]
enabled = false   # OpenClaw-style ambient mode
```

### Setting up an OpenAI-compatible provider via CLI

You usually don't need to edit this file by hand. The
[`next-code provider add`](../README.md#config-file-setup-for-self-hosted-endpoints-and-mcp)
command writes the profile for you, including secret-safe API key storage:

```bash
# Hosted OpenAI-compatible API (with API key from stdin):
printf '%s' "$MY_API_KEY" | next-code provider add my-api \
  --base-url https://llm.example.com/v1 \
  --model my-model-id \
  --api-key-stdin \
  --set-default

# Local server with no auth:
next-code provider add local-vllm \
  --base-url http://localhost:8000/v1 \
  --model Qwen/Qwen3-Coder-30B-A3B-Instruct \
  --no-api-key \
  --set-default
```

After adding, smoke-test it:

```bash
next-code --provider-profile local-vllm auth-test \
  --prompt 'Reply exactly NEXT_CODE_PROVIDER_SETUP_OK'
```

## `~/.next-code/mcp.json` — MCP server registry

```json
{
  "servers": {
    "filesystem": {
      "command": "/path/to/mcp-server",
      "args": ["--root", "/workspace"],
      "env": {},
      "shared": true
    }
  }
}
```

- Project-local equivalents live at `.next-code/mcp.json` (relative to cwd) and
  override entries with the same name.
- Compatibility fallback: if `~/.next-code/mcp.json` doesn't exist on first run,
  next-code imports from `~/.claude/mcp.json` and `~/.codex/config.toml`.
- Run [`next-code mcp trust <path>`](../docs/SAFE_EVALUATION.md) to mark a config
  as trusted when `--require-mcp-trust` is in effect.

## `~/.next-code/SYSTEM.md` and `APPEND_SYSTEM.md` — system-prompt overrides

- `SYSTEM.md` **replaces** next-code's built-in system prompt entirely.
- `APPEND_SYSTEM.md` **appends** to the built-in system prompt without
  removing it. Use this for "always remember to ..."-style additions.

Project-level `.next-code/SYSTEM.md` and `.next-code/APPEND_SYSTEM.md` (relative to
cwd) override the user-level versions.

## `~/.next-code/prompts/<name>.md` — slash-command templates

Discoverable via `next-code prompts list` and invokable as `/<name>` inside the
TUI. Scaffold one with:

```bash
next-code prompts new <name>           # → ./.next-code/prompts/<name>.md
next-code prompts new <name> --user    # → ~/.next-code/prompts/<name>.md
```

See [PR #207 + #217](https://github.com/quangdang46/next-code/pulls?q=prompts) for
the full template format.

## XDG mode

If you set `NEXT_CODE_USE_XDG=1`, the home moves to:

```
$XDG_DATA_HOME/next-code               (when XDG_DATA_HOME is set)
~/.local/share/next-code               (default fallback)
```

All file names above are unchanged — only the parent directory moves.
See [PR #225](https://github.com/quangdang46/next-code/pull/225) for the
toggle.

## Offline / air-gapped checklist

For machines that cannot reach the public internet:

1. **Disable the update check + telemetry**: `--offline` or
   `NEXT_CODE_OFFLINE=1`.
2. **Configure a local provider**: see the tl;dr above for a vLLM-style
   `config.toml`. Public OAuth flows (Claude, OpenAI, Gemini) won't work
   without internet.
3. **MCP servers**: install + register them in `~/.next-code/mcp.json` ahead of
   time. Project-local `.next-code/mcp.json` is fine for repo-pinned
   integrations.
4. **No memory embeddings**: the default memory backend uses local
   `tract-onnx` weights downloaded once. If your machine never had
   internet, copy `~/.next-code/embeddings/` from another machine, or set
   `[features] memory = false`.
5. **`next-code doctor`**: runs without network access; use it to verify the
   above before depending on next-code in production.

## See also

- [Z.AI Coding Plan quickstart](ZAI_CODING_PLAN.md)
- [Safe evaluation mode](SAFE_EVALUATION.md)
- [`next-code --help`](../README.md#further-reading) for the full CLI surface.
