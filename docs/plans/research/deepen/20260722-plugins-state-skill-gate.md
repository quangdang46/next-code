# Deepen — `plugins-state.json` must gate skill ingest (2026-07-22)

**ID:** D2 · **Priority:** P0 · Phase 1  
**Status:** Implement contract (docs only — **no production Rust in this file**)  
**Parent:** [`PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md) Phase 1 §2  
**Readiness:** [`PLAN-20260722-platform-implement-readiness.md`](../../PLAN-20260722-platform-implement-readiness.md)  
**Evidence:** [`20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md) — enable/disable is Face-list-only; `SkillRegistry::load_global` loads all `plugins/*/skills`  
**Related:** [`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md) (same state later gates hooks/MCP), [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) (D0 — pack disable is coarser than per-plugin), [`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md) (D1 — trust ≠ enable)

---

## 1. Problem

UI shows a plugin **disabled**; runtime still injects that plugin’s skills into:

- `$skill` / Face `/skill` expand (`pager_agent` + `SkillRegistry`)
- ACP `AvailableCommandsUpdate` slash palette
- Face Extensions Skills tab (`face_auth::list_nextcode_skills`)
- Tool `Skill` / shared registry snapshots used by the agent

That is a **lying product**: enable/disable looks authoritative but is cosmetic for ingest.

### Verified gap (2026-07-22)

| Piece | Behavior today |
|-------|----------------|
| `~/.next-code/plugins-state.json` | Persists `disabled` / `enabled` id lists — `src/cli/face_plugins.rs` (`STATE_FILE`, `PluginsState`, `is_enabled`, `set_disabled`) |
| Face list | `plugins_list_payload` → `PluginInfo.enabled` via `is_enabled` — **UI correct** |
| `enabled_plugin_skill_dirs(cwd)` | **Already filters** `discover_plugins` by `is_enabled` and returns only `…/skills` dirs — `face_plugins.rs` ~1037–1048 |
| Callers of `enabled_plugin_skill_dirs` | **None** (dead helper — only defined, never used) |
| `SkillRegistry::load_global` / `reload_global` | Walks **all** `~/.next-code/plugins` and `installed-plugins` via `load_plugin_skills_from_root` — **ignores** enable state — `crates/next-code-base/src/skill.rs` |
| Enable/Disable ACP action | Writes state + `requires_reload: true` — Face chains `PluginsAction::Reload`, which today only refreshes the **plugins list**, not skill registry contents |

---

## 2. Frozen intent

`plugins-state.json` (and equivalent enable flags) is **authoritative for ingest**, not cosmetic.

### Must gate when disabled (Phase 1)

1. **Skills** under that plugin tree (required).
2. Skill names must disappear from:
   - `SkillRegistry` effective list
   - ACP `AvailableCommandsUpdate`
   - Face Skills tab list
   - `$name` / InjectSkill expand

### Later (same mechanism — not Phase 1 exit, but design for)

| Contribution | Ticket |
|--------------|--------|
| Bundle MCP (`.mcp.json`) | D4 merge |
| Bundle / package hooks compile-in | D4 + D7 |
| Package slash commands | D9 |

When those land, they **must** call the same “enabled plugin roots” predicate — do not invent a second enable store.

### Must still work when enabled

Discovery, list, enable toggle, skill → ACP `AvailableCommandsUpdate` as today.

### Trust vs enable

| Gate | Question |
|------|----------|
| D1 project trust | May we **execute** project-layer hooks/argv? |
| D2 enable state | May we **ingest** this plugin’s skills (and later MCP/hooks)? |

A trusted but **disabled** plugin contributes **no** skills. An enabled **user** plugin contributes skills without D1 project trust (user home install). Project plugins: enable still required; executables additionally need D1.

---

## 3. Wire map (current → target)

### 3.1 Discovery and state (keep)

```text
discover_plugins(cwd)
  ├── ~/.next-code/plugins/*          (PluginScope::User)
  ├── ~/.next-code/installed-plugins/registry.json roots
  ├── <cwd>/.next-code/plugins/*      (PluginScope::Project)
  └── ~/.claude/plugins/*             (UserClaude, list/enable tracked)

plugins-state.json
  { "disabled": ["user/<hex8>/<name>", ...], "enabled": [...] }

is_enabled(state, plugin):
  if id|name in disabled → false
  if id|name in enabled  → true
  else → true   # default enabled (document; see §6)
```

Plugin id format (today): `{scope}/{path_hex8}/{name}` — `face_plugins::plugin_id`.

### 3.2 Target ingest pipeline

```text
discover_plugins(cwd)
  → filter is_enabled == true
  → collect skill roots (plugin.root/skills)     # = enabled_plugin_skill_dirs
  → SkillRegistry loads ONLY those plugin skill dirs
       (+ still loads standalone ~/.next-code/skills, ~/.agents/skills,
          Claude installed_plugins manifest paths — see §5)
  → Face list may still show disabled entries (greyed) without loading their skills
```

### 3.3 Call sites that must see the filtered registry

| Call site | Path | Notes |
|-----------|------|-------|
| Global load | `SkillRegistry::load_global` | Replace blind `load_plugin_skills_from_root(user_plugins)` / `installed-plugins` |
| Global reload | `SkillRegistry::reload_global` | Same filter |
| Working-dir load | `load_for_working_dir` / `effective_for_working_dir` | Inherits global base + project **standalone** overlays (`.next-code/skills` etc.) — project **plugin** skills come through discovery with cwd |
| ACP init meta | `pager_agent::load_initial_available_commands` | Uses `load_global` |
| ACP session | `pager_agent::emit_available_skills` | Uses `load_for_working_dir` |
| Face Skills tab | `face_auth::list_nextcode_skills` | Uses `load_global` + overlay |
| Skill tool | `crates/next-code-app-core/src/tool/skill.rs` | Uses shared / effective registry |
| Startup snapshot | `src/cli/startup.rs` | `SkillRegistry::shared_snapshot` |

**Layering note:** `SkillRegistry` lives in `next-code-base`; `enabled_plugin_skill_dirs` lives in `src/cli/face_plugins.rs` (binary/CLI). BUILD must either:

1. Move “enabled plugin skill dirs” into `next-code-base` (or a small shared crate) so `skill.rs` can call it without depending on Face CLI, **or**
2. Inject skill roots via a callback / trait registered at process start from `face_plugins`.

Prefer (1) for daemon + Face + TUI consistency — state file format stays the same.

---

## 4. `enabled_plugin_skill_dirs` contract (expand existing helper)

Today (`src/cli/face_plugins.rs`):

```rust
pub fn enabled_plugin_skill_dirs(cwd: Option<&Path>) -> Vec<PathBuf> {
    let state = load_state();
    discover_plugins(cwd)
        .into_iter()
        .filter(|p| is_enabled(&state, p))
        .filter_map(|p| {
            let skills = p.root.join("skills");
            skills.is_dir().then_some(skills)
        })
        .collect()
}
```

**Keep this semantics.** Wire it into skill load instead of directory-walking every plugin root.

### Differences vs `load_plugin_skills_from_root`

| `load_plugin_skills_from_root(plugins_root)` | `enabled_plugin_skill_dirs` |
|----------------------------------------------|-----------------------------|
| Claude-shaped: `installed_plugins.json`, cache, repos | next-code bundle + install registry + project |
| No enable filter | Enable filter |
| Used for `~/.claude/plugins` | Used for next-code plugin trees |

**Claude compat:** keep loading Claude plugin skills via existing `claude_plugins_root` + `installed_plugins.json` path **unless** the same plugin id appears in `plugins-state.json` as disabled (Face already tracks Claude entries in state). Phase 1 minimum: gate **next-code** `plugins/` + `installed-plugins` + project plugins; Claude gating is best-effort if already in `discover_plugins`.

---

## 5. What must **not** be gated by plugins-state

| Source | Reason |
|--------|--------|
| `~/.next-code/skills/` | Standalone user skills |
| `~/.agents/skills/` | Cross-tool convention |
| Project `./.next-code/skills`, `./.agents/skills`, `./.claude/skills` | Project-local skill dirs via `load_project_local_dirs` — not bundle plugins |
| nextcode / bare pack identity | D0 — pack off removes product prompts; orthogonal to per-plugin disable |

Disabling the last plugin must **not** wipe standalone skills.

---

## 6. Edge cases

| Case | Behavior |
|------|----------|
| Missing state entry | Default **enabled** (today’s `is_enabled`) — document in Face help; optional future “default deny for project plugins” is a separate ticket |
| Partial path / renamed plugin | Match by `plugin.id` from discovery (`scope/hex/name`); also accept `name` for legacy state rows (`is_enabled` already checks both) |
| Global vs project overlay | Disable applies to that install record’s id; two plugins same `name` different roots have different ids |
| Disable then delete | `uninstall` already strips id from state lists — keep that |
| Enable after disable | Skills must return after reload path (§7) |
| `requires_reload: true` on Enable/Disable | Keep; Face auto-sends Reload — Reload must **also** refresh skill ingest + ACP commands (§7) |
| Concurrent sessions | Shared `SkillRegistry` reload clears global skills — project overlays stay per working_dir (issue #457 comments in `skill.rs`) |

---

## 7. ACP re-emit / Face reload path

### Today

1. User disables plugin in Extensions modal.  
2. `plugins_action_payload` → `set_disabled` → `requires_reload: true`.  
3. Face `dispatch_action_result` pushes `Effect::PluginsAction { Reload }`.  
4. Reload returns success with `requires_reload: false` (avoids infinite loop — see comment in `face_plugins.rs` and test `hooks_disable_success_refreshes_lists_without_plugins_reload_loop`).  
5. Plugins list refreshes; **skill registry / AvailableCommandsUpdate may still list old skills**.

### Target

On Enable / Disable / Install / Uninstall / Reload success that changes contribution set:

```text
1. Persist plugins-state.json (already)
2. SkillRegistry::reload_global() (or process shared registry equivalent)
   using enabled_plugin_skill_dirs
3. For each active Face session: emit_available_skills(session_id)
   → AvailableCommandsUpdate with filtered commands
4. If Extensions Skills tab open: refetch x.ai/skills/list
5. requires_reload chain may stay for list rebuild, but skill re-emit must not depend on a second user gesture
```

**Restart policy:** Prefer hot reload without process restart. If shared registry locking makes hot path hard, document “restart required” as a temporary exit — but Phase 1 goal is **no restart** for enable/disable skill visibility.

### Face slash registry

Face merges ACP commands into the slash registry (`sync_acp_commands` / tracker `AvailableCommandsUpdate` handling in `xai-grok-pager`). Re-emit is sufficient if it replaces the full command list (existing replace semantics — see pager ACP tests).

---

## 8. Implement steps (BUILD checklist — not done in this doc)

1. [ ] Relocate or share `PluginsState` / `is_enabled` / `enabled_plugin_skill_dirs` so `SkillRegistry` can use them (`next-code-base` preferred).  
2. [ ] Change `load_global` + `reload_global` to load next-code plugin skills **only** from enabled roots (stop blind `load_plugin_skills_from_root` on entire `plugins/` and `installed-plugins/` trees).  
3. [ ] Ensure project plugin skills use `cwd` when available (daemon sessions: pass working_dir into enabled-dirs helper).  
4. [ ] On plugins Enable/Disable/Reload path in `pager_agent` / Face bridge: reload registry + `emit_available_skills`.  
5. [ ] Keep Face list showing disabled plugins (greyed) with skill_count still reflecting **on-disk** skills for discoverability — or show count of *would-be* skills; pick one and document (recommend: keep on-disk counts so users see what enabling restores).  
6. [ ] Tests (§9).  
7. [ ] Inventory + HOOKS/plugins docs: “disable stops skill ingest.”

---

## 9. Tests (required for exit)

### Unit / integration

| Test | Assert |
|------|--------|
| `enabled_plugin_skill_dirs_skips_disabled` | Write plugin under `NEXT_CODE_HOME/plugins`, disable by id, dirs empty / omit that skills path |
| `load_global_omits_disabled_plugin_skills` | Skill name from disabled plugin absent from `SkillRegistry::list()` |
| `load_global_includes_enabled_plugin_skills` | Same plugin enabled → skill present |
| `standalone_skills_survive_all_plugins_disabled` | `~/.next-code/skills/foo` still loads |
| `enable_disable_persists` | Existing Face test — keep |
| `reload_after_disable_drops_skill` | `reload_global` after disable removes skill |
| ACP (if cheap) | After disable + reload path, `AvailableCommandsUpdate` lacks skill name — pager_agent or Face dispatch test |

### Manual smoke

1. Install/copy a plugin with a distinct skill name.  
2. Confirm `/skillname` or `$skillname` works.  
3. Disable in Extensions → skill gone from slash + Skills tab without full app reinstall.  
4. Re-enable → skill returns.

---

## 10. Acceptance checklist

- [ ] Disable plugin → its skills absent from `SkillRegistry` + Face slash (`AvailableCommandsUpdate`) + Skills tab.  
- [ ] Re-enable → skills return via reload path **without** requiring a documented process restart (or restart documented + accepted as interim).  
- [ ] Unit/integration test on load filter (disabled omitted).  
- [ ] `enabled_plugin_skill_dirs` is actually used by skill load (no longer dead code).  
- [ ] Default-missing-state remains enabled (documented).  
- [ ] D0 bare pack and D1 trust remain independent.  
- [ ] Inventory GAP row for plugins-state updated when landed.

---

## 11. Non-goals (Phase 1)

- Per-skill toggles inside a plugin (Face `x.ai/skills/toggle` is already a no-op for next-code — leave unless separate ticket).  
- Implementing D4 hooks/MCP merge in the same PR (but share the enable predicate).  
- Changing `plugins-state.json` schema (keep `disabled`/`enabled` arrays).  
- Production Rust in this deepen file.

---

## 12. Design sketch (ASCII)

```text
┌─────────────────────┐     ┌──────────────────────┐
│ plugins-state.json  │────▶│ is_enabled(id)       │
└─────────────────────┘     └──────────┬───────────┘
                                       │
┌─────────────────────┐                │
│ discover_plugins()  │────────────────┤
└─────────────────────┘                ▼
                            ┌──────────────────────┐
                            │ enabled skill dirs   │
                            │ (helper exists)      │
                            └──────────┬───────────┘
                                       │  TODAY: unused
                                       ▼  TARGET: wire here
                            ┌──────────────────────┐
                            │ SkillRegistry        │
                            │ load_global/reload   │
                            └──────────┬───────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              ▼                        ▼                        ▼
     ACP AvailableCommands     Face Skills tab          $skill / tool Skill
```

---

## 13. Exit criteria (product)

Phase 1 enable-gate is done when a disabled plugin cannot inject skills into model-visible or user-invocable surfaces, and tests prove the filter. Bundle hooks/MCP honesty can then share the same predicate (D3/D4) without a second enable story.
