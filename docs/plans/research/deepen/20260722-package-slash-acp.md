# Deepen — Package slash → ACP advertise (2026-07-22)

**ID:** D9 · **Priority:** P2 · Phase 2  
**Status:** Design contract (docs only — **not** implement approval)  
**Parent:** [`PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md) Phase 2  
**Inventory:** [`../20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md) §1 + rows “Slash: ACP skills”, “Slash: markdown prompts”  
**Sibling ABI:** [`20260722-plugin-manifest-abi-v1.md`](./20260722-plugin-manifest-abi-v1.md) (`[[slash]]`)  
**Enable gate:** [`20260722-plugins-state-skill-gate.md`](./20260722-plugins-state-skill-gate.md) (disable → remove slash)  
**Pack chrome:** [`20260722-nextcode-pack-extraction.md`](./20260722-nextcode-pack-extraction.md) — Face builtins / brand-hide stay pack-owned  
**ACP prior art:** [Agent Client Protocol — Slash Commands](https://agentclientprotocol.com/protocol/slash-commands)

---

## 1. Goal (one sentence)

Package-declared `[[slash]]` entries and package/user `prompts/*.md` templates become **Face-visible slash commands** by advertising them over ACP (`InitializeResponse.meta.availableCommands` seed + `session/update` `available_commands_update`), **not** by forking Face or hardcoding new builtins into `builtin_commands()`.

---

## 2. Why this exists (gap evidence)

### 2.1 Dynamic slash today = skills only

Agent side (`src/cli/pager_agent.rs`):

- `emit_available_skills` loads `SkillRegistry`, maps each skill to `acp::AvailableCommand::new(name, description)` with `meta.path` + `meta.scope`, then sends:

```text
SessionUpdate::AvailableCommandsUpdate(AvailableCommandsUpdate::new(commands))
```

- Comment on that function: skills become Face `/skillname` via InjectSkill; `$skill` still expands on the prompt seam.

Face side:

- `crates/xai-grok-pager/src/views/prompt_widget/mod.rs` → `sync_acp_commands` → `CommandRegistry::set_acp_state`
- `crates/xai-grok-pager/src/slash/acp_command.rs` → `AcpSlashCommand`: skill meta → `CommandResult::InjectSkill`; **non-skill** ACP commands → `CommandResult::PassThrough("/name …")` back to the shell

Initialize seed (before first update):

- `crates/xai-grok-pager/src/acp/mod.rs` — `parse_available_commands(InitializeResponse.meta)` reads `meta.availableCommands` so autocomplete works before any `AvailableCommandsUpdate`.

**There is no** general “package `/review` prompt command” emitter today. Inventory labels markdown prompts as **PLUG data / Face GAP**.

### 2.2 Prompt templates exist but Face does not ingest them

- Discovery: `crates/next-code-app-core/src/prompt_templates.rs` — project `.next-code/prompts/*.md` (walk up) then `~/.next-code/prompts/*.md`; filenames → kebab command names; `discover_cached` / mtime token for hot reload.
- Docs claim TUI `/<name>` (`docs/CONFIG_REFERENCE.md` § prompts) — **product lie until this deepen lands**.
- Module docs say interactive expansion + autocomplete are “tracked separately”; discovery is CLI `prompts list|show` half only.

### 2.3 Face builtins are a different layer

- `crates/xai-grok-pager/src/slash/commands/mod.rs` → `builtin_commands()` (~60 hardcoded Face commands).
- Brand hide: `product_welcome.rs` `EMBED_BRAND_RESTRICTED_COMMANDS` via `set_brand_hidden_commands`.

Those stay **nextcode pack / host chrome** (see D10). Package slash must **not** require editing `builtin_commands()`.

---

## 3. Protocol contract (ACP)

Follow ACP slash-commands semantics ([official protocol](https://agentclientprotocol.com/protocol/slash-commands)):

| Direction | Wire | Meaning |
|-----------|------|---------|
| Agent → Client | `InitializeResponse.meta.availableCommands` | Bootstrap list (skills + package slash + prompts) |
| Agent → Client | `session/update` / `available_commands_update` | Full replace of advertised list when catalog changes |
| Client → Agent | `session/prompt` with text `/name …` | Command runs as ordinary prompt content; agent interprets |

Face already implements the client half for skills. Package slash reuses the same pipes.

### 3.1 `AvailableCommand` shape we emit

| Field | Required | Notes |
|-------|----------|-------|
| `name` | yes | No leading `/`; kebab-case; collide rules below |
| `description` | yes | Dropdown subtitle |
| `input.hint` | optional | Arg placeholder (ACP `AvailableCommandInput`) |
| `meta.kind` | **required for package slash** | `"prompt"` \| `"command"` \| `"skill"` (skills may keep today’s path/scope-only meta for compat) |
| `meta.source` | recommended | `"package"` \| `"user-prompt"` \| `"project-prompt"` \| `"skill"` |
| `meta.package_id` | when from package | Manifest `id` |
| `meta.path` | prompt/skill | Absolute or workspace-resolved path to markdown / SKILL.md |
| `meta.scope` | skill compat | `"user"` \| `"repo"` \| `"plugin"` as today |
| `meta.argv` | `kind=command` | JSON array of strings (runner argv) — Phase 2.1 |

**Compat rule:** Existing skill ads (path+scope, no `kind`) continue to mean skill / InjectSkill. New emitters **should** set `meta.kind = "skill"` when touching that path, but Face must keep accepting path+scope without `kind`.

### 3.2 Replace semantics

`AvailableCommandsUpdate` is a **full catalog replace** for ACP-sourced entries (see `CommandRegistry` / `set_acp_commands` — ACP entries replaced; builtins untouched).

When any of these change, agent must re-emit:

1. Skill registry reload (enable/disable plugin — D2).
2. Package enable/disable or install/uninstall.
3. Prompt template dir mtime change (`prompt_templates_changed_since`).
4. Working-dir / session cwd change that alters project prompt walk.

---

## 4. Manifest surface (`[[slash]]`)

From D8 sketch — frozen enough for this ticket:

```toml
[[slash]]
name = "review"
kind = "prompt"                    # prompt | command
description = "Structured code review"
# kind=prompt:
template = "prompts/review.md"     # relative to package root
# optional:
arg_hint = "focus area"

[[slash]]
name = "ship-check"
kind = "command"                   # Phase 2.1 — may ship after prompt path
description = "Run package ship checklist"
runner = { argv = ["python3", "bin/ship_check.py"] }
arg_hint = "optional target"
```

Also advertise **undeclared** markdown under package `resources.prompts = ["prompts"]` / convention `prompts/*.md` as `kind=prompt` with `name` = stem (same validation as `prompt_templates.rs`).

User/project `~/.next-code/prompts` and `.next-code/prompts` are **not** packages but use the **same ACP advertise + prompt expand path** (close the CONFIG doc gap).

---

## 5. Kinds — behavior matrix

| `kind` | Face dropdown | On Enter (Face) | Agent responsibility | Phase |
|--------|---------------|-----------------|----------------------|-------|
| `prompt` | Yes | Prefer **PassThrough** `/name args` (or InjectSkill-shaped blocks if we add `meta.path` without skill scope) | Expand template → user message (+ optional system_reminder / structured blocks) | **2.0 required** |
| `command` | Yes | PassThrough `/name args` | Spawn argv (trust + enable), stdin/stdout policy TBD; may map to hook-like JSON later | **2.1** (may defer) |
| `skill` (compat) | Yes | InjectSkill → shell expand | Today’s `expand_skill_invocation` | already shipped |

### 5.1 Prompt expansion (target)

Agent owns expansion (mirrors skills):

1. Resolve template body from `meta.path` or package-relative `template`.
2. Apply placeholder / arg substitution (reuse `prompt_templates` / `prompt_placeholders` helpers where possible — do not invent a third template language).
3. Produce prompt content for the model turn (user text and/or structured blocks).
4. Do **not** require a new Face command module.

**Face `AcpSlashCommand` today:** non-skill → PassThrough. That is **enough** for v1 prompt slash if the shell recognizes `/name` from the advertised set and expands. Optionally later: Face-local inject when `meta.kind=prompt` + `meta.path` for offline polish — **not** required for exit criteria.

### 5.2 Command kind (Phase 2.1)

- Same advertise path; execution = argv under trust gate (D1) + enable (D2) + security deepen (D13).
- No in-process TS `registerCommand` (Pi) / no resurrected QuickJS.
- Prefer documenting “command slash may land after prompt slash” rather than blocking Phase 2 exit.

---

## 6. Collision & precedence

Order when names collide (highest wins for ACP advertise; Face builtins always win dispatch for the same canonical name):

1. **Face `CommandSource::Builtin`** — never overwritten by ACP (`registry` design).
2. **Project prompts** (closest ancestor `.next-code/prompts`).
3. **Enabled package** slash / package prompts (deterministic tie-break: package id lexicographic, then path).
4. **User** `~/.next-code/prompts`.
5. **Skills** (existing skill advertise).

On collision with a builtin name: **do not advertise** the package command (log/warn in agent diagnostics). Authors must pick another name.

On collision between two ACP sources: emit **one** winner; optional qualified name (`scope:name`) only if we extend today’s skill collision helper (`client_collision_qualified_name` in `slash/registry.rs`) — default v1 = drop loser + warn.

Brand-hidden builtins (`EMBED_BRAND_RESTRICTED_COMMANDS`) remain hidden even if a package tries to re-advertise the same name (builtin still owns the name; package entry suppressed).

---

## 7. Enable / disable / trust

| Gate | Effect on package slash |
|------|-------------------------|
| `plugins-state.json` disabled (D2) | Package’s `[[slash]]` + package `prompts/` **absent** from next `AvailableCommandsUpdate` |
| Project untrusted (D1) | Executable `kind=command` must not run; `kind=prompt` markdown may still advertise if treated as data (same policy as skills/prompts — **align with D1**; if D1 classifies package prompts as trust-free data, document that) |
| Pack profile bare (D10 / D0) | nextcode **starter** slash/prompts from product pack off; **user/project/package** slash still works |

Exit must include: disable pack/plugin → command disappears from Face after refresh (generation bump / re-sync).

---

## 8. Agent emit API (implementation sketch — not code)

Replace “skills-only” with a unified catalog builder, e.g. conceptual:

```text
build_available_commands(session) =
    skills_as_commands()           # existing emit_available_skills body
  ⊕ user_project_prompt_commands() # prompt_templates::discover_in
  ⊕ enabled_package_slash()        # manifest [[slash]] + resources.prompts
```

Call sites that today call `emit_available_skills` (`pager_agent` session create / relevant reloads) become `emit_available_commands` (or keep name but widen).

Initialize meta should include the same list (or a safe subset) so `parse_available_commands` seeds Face immediately — same pattern as skills comment in `acp/mod.rs`.

---

## 9. Face constraints (frozen)

1. **No** new hardcoded entries in `builtin_commands()` for third-party slash.
2. **No** in-process TypeScript slash registration.
3. Face builtins + brand-hide remain **nextcode pack** (or thin core + pack overlay) — D10.
4. Reuse `sync_acp_commands` + `AcpSlashCommand`; extend meta parsing only as needed for `kind`.
5. Docs parity: `CONFIG_REFERENCE.md` prompts section and plugins docs must match Face behavior when this ships.

---

## 10. Non-goals

- Pi-style `ctx.ui` custom slash UX / arbitrary Face widgets (see D12).
- Replacing Face builtins with packages in Phase 2 (extraction is D10; may move *some* product commands later).
- Argv tools (`[[tools]]`) — D11.
- Making prompt templates execute shell by default.
- Dual-write into legacy TUI (PR11 retiring).

---

## 11. Test / verification matrix (when implementing)

| Case | Expect |
|------|--------|
| Drop `~/.next-code/prompts/foo.md`, session refresh | Face shows `/foo`; Enter expands template |
| Package `[[slash]] kind=prompt` enabled | `/name` in dropdown + works |
| Disable plugin | `/name` gone after update; builtin count unchanged |
| Name equals `/help` | Not advertised; `/help` still builtin |
| Skill + prompt same name | Precedence table winner only |
| `AvailableCommandsUpdate` twice | Idempotent registry size (see `sync_acp_commands` tests) |
| Initialize without later update | Seeded commands still completable |

---

## 12. Open questions (narrow)

| # | Question | Default stance until answered |
|---|----------|-------------------------------|
| Q1 | Face-local prompt inject vs PassThrough-only for `kind=prompt`? | **PassThrough + agent expand** (v1) |
| Q2 | Ship `kind=command` in same PR as prompts? | **Defer 2.1** if it blocks trust/security review |
| Q3 | Advertise disabled package prompts greyed vs omit? | **Omit** (honest catalog; matches D2 skills intent) |
| Q4 | Should `prompt_templates` discover also walk package dirs, or only manifest compiler? | Prefer **one compiler** in agent that feeds ACP; avoid Face re-walking packages |

---

## 13. Exit criteria

- [ ] Package `kind=prompt` slash appears in Face after package enable (ACP path only).
- [ ] User/project `prompts/*.md` appear in Face (CONFIG doc parity).
- [ ] Disable pack/plugin removes those commands on next advertise (builtins untouched).
- [ ] Collision with Face builtin never shadows builtin.
- [ ] Docs: CONFIG / plugins / package ABI mention ACP advertise; no “TUI magic” without agent emit.
- [ ] No new Face fork / no in-process TS slash registration.

---

## 14. Related citations (code)

| Area | Path / symbol |
|------|----------------|
| Emit skills | `src/cli/pager_agent.rs` — `emit_available_skills` |
| Face sync | `prompt_widget::sync_acp_commands` |
| ACP wrapper | `slash/acp_command.rs` — `AcpSlashCommand` |
| Registry sources | `slash/registry.rs` — `CommandSource::{Builtin,Acp}` |
| Initialize seed | `acp/mod.rs` — `parse_available_commands` |
| Builtins | `slash/commands/mod.rs` — `builtin_commands` |
| Brand hide | `product_welcome.rs` — `EMBED_BRAND_RESTRICTED_COMMANDS` |
| Prompt discovery | `next-code-app-core/src/prompt_templates.rs` |
| Docs claim | `docs/CONFIG_REFERENCE.md` — prompts as `/<name>` |

---

## 15. Dependency order

```text
D1 trust (for command kind) ──┐
D2 enable gate ───────────────┼──► D9 package slash ACP ──► D8 manifest examples
D8 [[slash]] schema ──────────┘         │
                                        ▼
                              D10 pack (builtins stay pack)
```

Phase 2 master exit (“third party ships hooks + slash + skills without core PR”) **requires** this deepen’s prompt path; command kind may lag.
