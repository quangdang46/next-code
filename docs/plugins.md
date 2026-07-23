# Plugins (Grok-style bundles)

next-code uses **Grok Face Extensions** bundle plugins -- not the old QuickJS/TS
`next-code plugin *` runtime (removed).

## Face UI

In the interactive Face UI:

1. Run `/plugins` (or open Extensions and pick the Plugins tab).
2. `/hooks` opens the same modal on the Hooks tab (lists `hooks.toml`; enable/disable/reload wired).
3. Marketplace remains brand-hidden in the nextcode embed.

ACP methods (daemon):

| Method | Role |
|--------|------|
| `x.ai/plugins/list` | Installed / discovered plugins |
| `x.ai/plugins/action` | install / uninstall / enable / disable / reload / update |
| `x.ai/hooks/list` | Lifecycle hooks from `~/.next-code/hooks.toml` layers |
| `x.ai/hooks/action` | reload / enable / disable (add/remove/trust unsupported) |
| `x.ai/skills/list` | Skills tab (includes skills from enabled plugin bundles) |
| `x.ai/mcp/list` | MCP Servers tab (`~/.next-code/mcp.json` + overlays). **stdio** and **HTTP/SSE** (streamable HTTP POST) connect; optional `headers` in mcp.json for static auth |


## Disk layout

| Path | Role |
|------|------|
| `~/.next-code/plugins/<name>/` | User plugin bundles |
| `<project>/.next-code/plugins/<name>/` | Project plugin bundles |
| `~/.next-code/installed-plugins/` | Git / registry installs |
| `~/.next-code/plugins-state.json` | Enable/disable list |
| `~/.claude/plugins/` | Claude-compat (list + skill ingest; uninstall blocked) |

A plugin directory is recognized when it has `plugin.json` (or
`.grok-plugin` / `.claude-plugin` manifest), and/or convention dirs:
`skills/`, `agents/`, `hooks/hooks.json`, `.mcp.json`.

## Skills

Enabled plugin `skills/*/SKILL.md` trees are loaded into the skill registry
alongside `~/.next-code/skills/`. Use `/skills` or `$skillname` as before.

## Install sources (Face action)

- Local path (absolute, relative, or `~/...`)
- Git URL / `git@...` / `user/repo` GitHub shorthand

## Removed

- CLI: `next-code plugin load|list|...`
- Crates: `next-code-plugin-core`, `next-code-plugin-runtime` (QuickJS sandbox)
- Docs that described TS plugin authoring -- see git history if needed
