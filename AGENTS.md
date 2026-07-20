# Repository Guidelines

## Origin Sync (fork management)
This repo (`quangdang46/next-code`) is a rebranded fork of `quangdang46/next-code`. Several modules have been extracted into separate repos. When syncing from upstream, use the `origin-sync` skill: `skill(name="origin-sync")`. It provides a structured workflow for classifying conflicts (extracted-code, local-extension, upstream-only, dep, new-feature) and resolving them correctly.

## Grok Face UI migration
When replacing next-code interactive UI with Grok Face (`xai-grok-pager`) — copy, delete old TUI, wire ACP/daemon — use the `grok-migration-workflow` skill: `skill(name="grok-migration-workflow")`. LOOK→PLAN→BUILD, root-cause before fixes, research grok-build before inventing wire behavior.



## Development Workflow

- **Commit as you go** - Make small, focused commits after completing each feature or fix
- If the git state is not clean, or there are other agents working in the codebase in parallel, do your best to still commit your work. 
- **Push when done** - Push all commits to remote when finishing a task or session
- **Use fast iteration by default** - Prefer `cargo check`, targeted tests, and dev builds while iterating
- **Rebuild when done** - When you are done making changes, build the source.
- **Bump version for releases** - Update version in `Cargo.toml` when making releases. When cutting a new release, look at all the changes that happened since the last release and determine what the version bump should be ie patch or minor, etc. 
- **Remote builds available** - Use `scripts/remote_build.sh` to offload heavy cargo work to another machine. If your build is terminated, likely is because there are not enough resources on this machine to build. use remote build in that case. Try checking the resource avaliablity on the machine before you run a build. 

## Logs
- Logs are written to `~/.next-code/logs/` (daily files like `next-code-YYYY-MM-DD.log`).

## Debug Socket
- Use the debug socket for runtime level debugging

## Install Notes
- `~/.local/bin/next-code` is the launcher symlink used from `PATH`.
- one-release compat: `next-code` → `next-code` symlink at `~/.local/bin/next-code`.
- `~/.next-code/builds/current/next-code` is the active local/source-build channel; self-dev builds and `scripts/install_release.sh` point the launcher here.
- `~/.next-code/builds/stable/next-code` is the stable release channel; `scripts/install.sh` installs this and points the launcher here.
- `~/.next-code/builds/versions/<version>/next-code` stores immutable binaries.
- `~/.next-code/builds/canary/next-code` still exists for canary/testing flows, but it is not the primary self-dev install path.
- On Windows, the equivalents are `%LOCALAPPDATA%\\next-code\\bin\\next-code.exe` for the launcher (plus a one-release `next-code.exe` compat entry), `%LOCALAPPDATA%\\next-code\\builds\\stable\\next-code.exe` for stable, and `%LOCALAPPDATA%\\next-code\\builds\\versions\\<version>\\next-code.exe` for immutable installs; `scripts/install.ps1` currently installs the stable channel.
- Ensure `~/.local/bin` is **before** `~/.cargo/bin` in `PATH`.

### After install (agent-tree / TUI work)

`scripts/install_release.sh` updates symlinks but **running `next-code serve` keeps the old binary mapped**. Always restart serve after install:

```bash
# Prefer the helper:
bash scripts/restart_local_serve.sh

# Or manually: kill the serve PID, then:
#   next-code serve   # or: next-code --provider auto serve
```

Confirm the live binary: `lsof -p $(pgrep -f 'builds/.*/next-code' | head -1) | grep txt` should show the same hash as `readlink ~/.next-code/builds/current/next-code`. The TUI shows a short client git hash in teammate-view chrome while viewing an agent.

## Notepad (compaction-resistant notes)

The notepad (`crates/next-code-base/src/notepad.rs`, `crates/next-code-app-core/src/tool/notepad.rs`) is a 3-tier file-based store under `<working_dir>/.next-code/notepad/` that lets the model persist short notes across turns and across compaction.

Tiers:
- **priority** — auto-injected into the system prompt every turn. Survives compaction because the content is re-read from disk each cycle. Rendered as a fenced code block with a trust marker so the model cannot inject instructions through it.
- **working** — persistent scratchpad for in-progress reasoning. Cleared with `notepad_prune`.
- **manual** — user-authored notes that persist across sessions. Not auto-injected.

Tools (namespaced under `notepad_*`):
- `notepad_read_priority`, `notepad_write_priority` (requires `confirm: true` by default)
- `notepad_read_working`, `notepad_write_working`
- `notepad_read_manual`, `notepad_write_manual`
- `notepad_prune` (clears the working tier only)
- `notepad_stats` (per-tier sizes)

Config (under `[notepad]` in `config.toml`):
- `enabled` (default: `true`) — set to `false` to disable entirely.
- `dir` (default: `.next-code/notepad`) — must be a relative path with no `..` components; absolute paths and `..` are rejected.
- `max_bytes_per_tier` (default: 4096) — the field is byte-based (predictable file size, predictable token cost). Truncation always lands on a UTF-8 char boundary.
- `require_priority_confirm` (default: `true`) — when enabled, `notepad_write_priority` must include `confirm: true` in its input.

Trust model: priority content is rendered as data (fenced code block + trust marker), `notepad_write_priority` requires explicit `confirm: true` by default, and every priority write emits a structured log line. The notepad is **not** auto-cleared on session end — clear it explicitly with `notepad_prune` or by writing empty content.

