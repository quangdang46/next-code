# Spike C: Config Schema — Grok vs Next-Code

## Date: 2026-07-17
## Status: ✅ Complete

---

## 1. Config Architecture Difference

| Aspect | Grok | Next-Code |
|--------|------|-----------|
| **Pager config** | Local files (`~/.grok/pager.toml`, `~/.grok/appearance.toml`) | Config is in-memory, loaded at startup |
| **Appearance** | Separate crate (`xai-grok-appearance`, theme, accent, spacing) | Part of `next-code-config-types` |
| **Agent config** | ACP leader manages it (loaded remotely) | In-process `Config` in `next-code-config-types` |
| **Keybindings** | Pager manages its own via `appearance::keybindings` | In `next-code-config-types::keybindings` |
| **Remote config** | xAI OTA updates (`CampaignOverride`, signed managed config) | Not applicable (no xAI infra) |
| **Config watcher** | `ConfigWatcher` hot-reloads on file change | Not observed in next-code |
| **Theme/system appearance** | Pager watches `os_appearance` (dark/light), custom themes | Next-code has theme system |

---

## 2. Pager's Local Config Categories

These config values are used **directly by the pager** (no ACP needed):

| Category | Crate | Used for |
|----------|-------|----------|
| **Appearance** | `xai-grok-appearance` | Colors, fonts, spacing, bullets, animation, accent colors |
| **Keybindings** | `appearance::keybindings` | Keyboard shortcuts for the pager |
| **Display** | `xai-grok-config-types::DisplayRefreshSettings` | Probe Hz, auto cadence |
| **Screen mode** | Pager CLI args | `--fullscreen`, `--minimal` |
| **Doom loop** | `xai-grok-config-types::DoomLoopRecoverySettings` | Recovery thresholds |
| **Theme** | `crate::theme` | Dark/light, theme kind |

These config values come **from the ACP leader** (next-code's domain):

| Category | Origin | Used for |
|----------|--------|----------|
| **Model** | ACP `GetModelCatalog` | Available models, pricing |
| **Provider config** | ACP | API keys, provider choices |
| **Session config** | ACP | Session types, yolo/auto mode |
| **Tool config** | ACP | Allowed tools, permissions |
| **Plugin config** | ACP + local | MCP servers, plugins |
| **Sandbox** | ACP | Sandbox policy |  
| **Memory** | ACP + local | Memory settings, extraction |

---

## 3. Config Mapping

### Pager-only config (SHA direct, ✅ compatible):
```toml
# pager.toml / appearance.toml  →  still works as local file
# Grok reads these from ~/.grok/*.toml  
# Next-code can use same format with different root path (~/.next-code/)
[appearance]
theme = "dark"
accent_color = "blue"
block_spacing = "normal"
bullet_style = "colored"
animation = true

[display_refresh]
probe_enabled = true
auto_cadence_enabled = false

[keybindings]
"ctrl+p" = "settings:open"
"ctrl+s" = "agent:new"
```

### Next-code config (needs adapter):
```rust
// GrokConfig → NextCodeConfig mapping
// xai_grok_config::GrokConfig  →  next_code_config_types::Config

struct GrokConfig {                    struct NextCodeConfig {
    model: Option<String>,                 models: Vec<ModelConfig>,
    provider: Option<String>,              default_provider: String,
    tools: Vec<ToolConfig>,                tool_registry: Vec<ToolDef>,
    workspace: PathBuf,                    worktree_config: PathBuf,
    memory: MemoryConfig,                  memory: MempalaceConfig,
    auth: Option<AuthConfig>,              openproxy: OpenproxyConfig,
    // ...xAI-specific:                    // ...next-code-specific:
    xai_account_id,                         compaction_mode,
    supergrok,                              diff_display_mode,
    mcp_plugins,                            keybindings (native),
    sandbox_policy,                         session_picker_action,
}                                       }
```

The `xai-shim-config` crate:
- Receives the pager's `xai_grok_config::GrokConfig` requests
- Maps them to `next_code_config_types::Config`
- Returns what the pager expects (maybe with stubbed xAI-specific fields)
- Config loading path: `~/.grok/` → `~/.next-code/`

---

## 4. Config Loading Path

| File | Grok | Next-Code |
|------|------|-----------|
| Config root | `~/.grok/` | `~/.next-code/` |
| Pager config | `~/.grok/pager.toml` | `~/.next-code/pager.toml` (same format) |
| Appearance | `~/.grok/appearance.toml` | `~/.next-code/appearance.toml` (same) |
| Agent config | Via ACP (leader) | In-process `Config::load()` |
| Keybindings | `pager.toml` | `keybindings.json` |
| Themes | `~/.grok/themes/` | `~/.next-code/themes/` |
| Plugins | `~/.grok/plugins/` | `~/.next-code/plugins/` |
| Sessions | Via ACP | `~/.next-code/sessions/` |

### Adapter strategy:
The pager reads config from file (same format, different root path). The ACP-level config (model, providers, tools) gets shimmed → reads from next-code's config types. **The pager's own UI config stays as local files — no shim needed.**

---

## 5. Theme Compatibility

Both use:
- `dark` / `light` / `system` (auto-detect via OS appearance)
- catppuccin-style theme files
- Custom accent colors
- ANSI terminal colors (16 base + 256 extended)

The pager's theme system (`crate::theme`) loads theme files from config dir. Next-code has no theme code visible in the types crate — but this is a **UI-only concern**, the pager brings its own theme system.

**No conflict — the pager's appearance/theme system works as-is.** Just point config root to `~/.next-code/`.

---

## Migration Impact Summary

| Config Area | Impact | Strategy |
|------------|:------:|----------|
| Appearance/theme | ✅ None | Same format, different dir |
| Keybindings | ✅ None | Same format, different dir |  
| Display refresh | ✅ None | Same format, different dir |
| Model/provider | 🔴 High | `xai-shim-config` reads next-code config |
| Tool config | 🔴 Medium | `xai-shim-config` reads next-code tool registry |
| Plugin/MCP | 🟡 Medium | Shim local + next-code plugins |
| Auth | 🟡 Medium | Shim via openproxy or next-code auth |
| Session config | 🟡 Medium | Shim via next-code session types |
| Memory config | 🟢 Low | Map to mempalace config |
| Remote/campaign | 🟢 None | **Remove** — xAI-specific |
| xAI OTA | 🟢 None | **Remove** — xAI-specific |
