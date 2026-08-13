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


<!-- bv-agent-instructions-v3 -->

---

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`) for issue tracking and [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) (`bv`) for graph-aware triage. Issues are stored in `.beads/` and tracked in git. Current `br` workspaces normally export `.beads/issues.jsonl`; older `bd`/legacy workspaces may use `.beads/beads.jsonl`. `bv` auto-discovers the supported JSONL files, so agents should use `br`/`bv` commands instead of hard-coding a single filename.

### Using bv as an AI sidecar

bv is a graph-aware triage engine for Beads projects. Instead of parsing .beads/issues.jsonl / .beads/beads.jsonl directly or hallucinating graph traversal, use robot flags for deterministic, dependency-aware outputs with precomputed metrics (PageRank, betweenness, critical path, cycles, HITS, eigenvector, k-core).

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). `br` handles creating, modifying, and closing beads.

**CRITICAL: Use ONLY --robot-* flags. Bare bv launches an interactive TUI that blocks your session.**

#### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns everything you need in one call:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command

# Token-optimized output (TOON) for lower LLM context usage:
bv --robot-triage --format toon
```

Before claiming, verify current state with `br show <id> --json` or `br ready --json`. `recommendations` can include graph-important blocked or assigned work; only `quick_ref.top_picks` and non-empty `claim_command` fields represent claimable work.

#### Other bv Commands

| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with unblocks lists |
| `--robot-priority` | Priority misalignment detection with confidence |
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS, eigenvector, critical path, cycles, k-core |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions, cycle breaks |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |

#### Scoping & Filtering

```bash
bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work (no blockers)
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank scores
```

### br Commands for Issue Management

```bash
br ready --json                       # Show issues ready to work (no blockers)
br list --status=open --json          # All open issues
br show <id> --json                   # Full issue details with dependencies
br create --title="..." --type=task --priority=2 --json
br update <id> --status=in_progress --json
br close <id> --reason="Completed" --json
br close <id1> <id2> --reason="Completed" --json
br sync --flush-only                  # Export DB to JSONL after Beads mutations
```

### Workflow Pattern

1. **Triage**: Run `bv --robot-triage` to find the highest-impact actionable work
2. **Claim**: Use `br update <id> --status=in_progress --json`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id> --reason="Completed" --json`
5. **Sync**: Run `br sync --flush-only` after Beads mutations so the JSONL export is current

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready --json` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers 0-4, not words)
- **Types**: task, bug, feature, epic, chore, docs, question
- **Blocking**: `br dep add <issue> <depends-on>` to add dependencies

### Git Policy

`br` never commits or pushes. Follow this repository's own git instructions before staging, committing, or pushing. If the repository says "commit only when asked," that rule overrides any generic workflow advice.

<!-- end-bv-agent-instructions -->
