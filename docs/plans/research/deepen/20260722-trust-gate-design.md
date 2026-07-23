# Deepen — Project trust gate for executable resources (2026-07-22)

**ID:** D1 · **Priority:** P0 · Phase 1  
**Status:** Implement contract (docs only — **no production Rust in this file**)  
**Parent:** [`PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md) Phase 1 §1  
**Readiness:** [`PLAN-20260722-platform-implement-readiness.md`](../../PLAN-20260722-platform-implement-readiness.md)  
**Steal from:** Pi `trust.json` + `ProjectTrustStore` ([pi surfaces §2.8](../20260722-pi-extension-surfaces.md); clone `.tmp-research-plugins/pi`)  
**Related:** [`20260722-argv-plugin-security.md`](./20260722-argv-plugin-security.md) (trust ≠ sandbox), [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) (D0 — pack/prompt identity orthogonal), [`20260722-plugins-state-skill-gate.md`](./20260722-plugins-state-skill-gate.md) (D2 — enable vs trust), [`20260722-hooks-cookbook-layout.md`](./20260722-hooks-cookbook-layout.md) (project examples need trust)

---

## 1. Summary (read first)

Executable project hooks/plugins are **user-approved RCE**. Phase 1 freezes a Pi-shaped gate: resolve trust **before** spawning project-scoped argv/command handlers.

Trust ≠ sandbox. After trust, D13 argv controls still apply. MCP content trust (`mcp_trust.json`) and Face folder-trust (`trusted_folders.toml`) stay **orthogonal** — do not silently merge.

---

## 2. Problem

| Today | Risk / evidence |
|-------|-----------------|
| Project `.next-code/hooks.toml` can declare `type = "command"` | `crates/next-code-hooks/src/config.rs` — layer 2 loads `<cwd>/.next-code/hooks.toml` with **no** trust check |
| Face hooks list | `src/cli/face_plugins.rs` `hooks_list_payload` hardcodes `project_trusted: true` |
| Upstream Grok docs | `crates/xai-grok-pager/docs/user-guide/10-hooks.md` describe hooks gated by folder-trust — next-code-hooks does **not** implement that |
| Project `.next-code/plugins/**` | Discovered by `face_plugins::discover_plugins`; executable merge lands in D4/ABI — same gate class |
| Marketing multilang without trust UX | False sense of safety |

Pi requires trust for project config that can change agent behavior / execute. Herdr shows install preview before linking. next-code Phase 1 matches that bar for **executable** resources only.

---

## 3. Frozen intent

Before loading or spawning **executable project-scoped** resources, the host must have an explicit trust decision for that project tree (or an ancestor), analogous to Pi’s `TRUST_REQUIRING_PROJECT_CONFIG_RESOURCES` + `ProjectTrustStore`.

**One sentence:** untrusted project trees never run project-layer command/HTTP/agent hooks or executable package runners; trusted trees run them as today (still not sandboxed).

### 3.1 Trust-requiring (v1)

| Resource | Why | Load path today |
|----------|-----|-----------------|
| Project `.next-code/hooks.toml` executable handlers (`command` / http / agent) | Spawn / network | `next-code-hooks` config layer 2 |
| Project `.next-code/plugins/**` with executable runners / future `[[hooks]]` argv | Spawn | `face_plugins::discover_plugins` project scope |
| Future package `[[build]]` / argv tools | Spawn | Phase 2+ ABI (`20260722-plugin-manifest-abi-v1.md`) |
| Future `[[tools]] kind=argv` from project packages | Spawn | Phase 3 / D11 — same gate |

### 3.2 Not trust-requiring alone (v1)

| Resource | Rationale |
|----------|-----------|
| Declarative skills markdown, prompt templates, theme tokens | Data-only |
| User-global `~/.next-code/hooks.toml` | User chose home install |
| User-global `~/.next-code/plugins` | Home install; still honor enable-state (D2) |
| Standalone `~/.next-code/skills`, `~/.agents/skills` | User-global |
| `AGENTS.md` / context overlays | Match Pi: context loads regardless (`pi` `docs/security.md`) |
| MCP stdio / project `mcp.json` | **Orthogonal** — §7 |

**Pi divergence:** Pi also gates project `.pi/skills` and project `.agents/skills`. next-code v1 stays **executable-first**. Expanding the trust-requiring set later needs an explicit docs bump — do not silently widen.

---

## 4. Pi reference (verified in clone)

Path prefix: `.tmp-research-plugins/pi/packages/coding-agent/`.

| Symbol / file | Steal |
|---------------|-------|
| `src/core/trust-manager.ts` — `TRUST_REQUIRING_PROJECT_CONFIG_RESOURCES` | Resource name list under project config dir |
| `hasTrustRequiringProjectResources(cwd)` | Bare config dir alone does **not** require trust |
| `ProjectTrustStore` | File at `join(agentDir, "trust.json")` → Pi: `~/.pi/agent/trust.json` |
| Trust file shape | `Record<canonicalAbsPath, true \| false \| null>`; delete/`null` clears |
| `findNearestTrustEntry` | Ancestor walk; first boolean wins |
| `getProjectTrustOptions` | Trust cwd / trust parent / do not trust (+ session-only) |
| `src/core/project-trust.ts` — `resolveProjectTrusted` | Override → no resources → extension event → store → `defaultProjectTrust` → prompt / deny |
| CLI | `--approve`/`-a`, `--no-approve`/`-na` in `src/cli/args.ts` — one-run, not persisted |
| Settings | `defaultProjectTrust`: `ask` \| `always` \| `never` (`docs/settings.md`) |
| Docs | `docs/security.md` — trust is input-loading guard, not sandbox |
| Lesson | Saved decision often needs **reload/restart** for full effect |

---

## 5. Proposed store: `~/.next-code/trust.json`

### 5.1 Path (frozen preference)

```text
$NEXT_CODE_HOME/trust.json
  default: ~/.next-code/trust.json
```

Same home resolution as `crates/next-code-base/src/mcp/trust.rs` (`next_code_dir()?.join(...)`).

**Do not** put the authoritative store inside the project tree (attacker-writable via clone).

### 5.2 Schema options (pick one in BUILD; both are Pi-compatible enough)

**Option A — Pi map (simplest):**

```json
{
  "C:\\Users\\you\\src\\good-repo": true,
  "C:\\Users\\you\\src\\bad-repo": false
}
```

**Option B — versioned wrapper (easier migration):**

```json
{
  "version": 1,
  "projects": {
    "/abs/path/to/repo": {
      "decision": "allow",
      "decided_at": "2026-07-22T00:00:00Z",
      "scope": "project"
    }
  }
}
```

| Rule | Detail |
|------|--------|
| Keys | Canonical absolute paths (`dunce::canonicalize` / equivalent) |
| Deny vs undecided | Explicit `false`/`deny` ≠ missing entry |
| Nearest ancestor | **Recommend Pi nearest-entry** (trust parent covers nested cwd). Alternate “exact root only” is safer but worse UX — if chosen, document and skip parent option in prompt |
| Unsafe roots | Refuse filesystem root / home dir (mirror `xai-grok-workspace::trust::is_unsafe_trust_root`) |
| Locking | Prefer lock sibling like Pi (`trust.json.lock`) or folder-trust exclusive lock |
| Pretty print | Sorted keys for reviewable diffs |

### 5.3 Three stores — do not conflate

| Store | Path | Question |
|-------|------|----------|
| MCP content trust | `~/.next-code/mcp_trust.json` | Is this `mcp.json` **byte content** approved? |
| Face folder trust | `trusted_folders.toml` under home | May Face open a session / workspace features here? |
| **Project exec trust (this deepen)** | `~/.next-code/trust.json` | May we **execute** project-layer hooks / package argv? |

**BUILD choice:** independent stores in Phase 1 (clearer). Optional later: `/hooks-trust` dual-writes folder-trust + exec trust after UX review.

---

## 6. Resolution order (implement exactly)

```text
1. CLI / env one-run override
     --approve-project-trust  /  NEXT_CODE_APPROVE_PROJECT_TRUST=1  → trusted
     --deny-project-trust     /  NEXT_CODE_DENY_PROJECT_TRUST=1    → untrusted
     (mutually exclusive; deny wins if both set)
     Prefer distinct names from Pi’s --approve to avoid colliding with unrelated flows.

2. If !has_trust_requiring_project_resources(cwd) → trusted (nothing to gate)

3. Optional Phase 2+: host/extension project_trust callback (Pi parity) — skip Phase 1

4. Nearest saved entry in ~/.next-code/trust.json

5. default_project_trust (ask | always | never), default = ask
     always → trusted; never → untrusted; ask → continue

6. Interactive Face / CLI prompt (persist if “remember”)
     Non-interactive / no TTY / no Face UI → untrusted (fail closed)
```

### 6.1 Predicate `has_trust_requiring_project_resources` (v1)

Return true if any of:

1. `<cwd>/.next-code/hooks.toml` exists with ≥1 enabled executable handler.
2. `<cwd>/.next-code/plugins/` has a discoverable plugin with executable surface — **or** (conservative) any project plugin dir present.
3. (Future) project package config declaring argv runners.

Bare `.next-code/` with only skills/markdown → **false** (no prompt).

### 6.2 Suggested module placement

| Piece | Home |
|-------|------|
| Store + nearest resolve | `crates/next-code-base/src/project_trust.rs` (sibling of `mcp/trust.rs`) |
| Predicate + resolve API | Called from hooks load/execute + Face list |
| CLI flags | Beside `--safe-eval` / MCP trust bootstrap |
| Face UX | Hooks tab Trust; `/hooks-trust` and/or `/trust` write **this** store |

---

## 7. MCP trust is orthogonal

| Concern | Mechanism | Env / CLI |
|---------|-----------|-----------|
| Project `mcp.json` bytes | `McpTrustStore` SHA-256 | `NEXT_CODE_REQUIRE_MCP_TRUST`; `next-code mcp trust\|revoke\|list` |
| Project executable hooks | This deepen’s `trust.json` | `--approve-project-trust` / `--deny-project-trust` |
| Extension policy | `src/extension_policy.rs` | `NEXT_CODE_EXTENSION_POLICY` — do not overload without a design note |

`--safe-eval` sets `NEXT_CODE_REQUIRE_MCP_TRUST=1` (`docs/SAFE_EVALUATION.md`). Phase 1 should also **deny project-exec** under safe-eval (or document explicit approve). Update SAFE_EVALUATION + inventory when landing.

Inventory today: “Trust gate optional (`--require-mcp-trust`)” is MCP-only — fix wording when project-exec trust ships.

---

## 8. Face / CLI / non-interactive UX

### 8.1 Face

| Surface | Contract |
|---------|----------|
| First session, requiring + undecided | Prompt: trust / trust parent (if ancestor policy) / do not trust (+ optional session-only) |
| Copy | State what trust allows: project hooks.toml command handlers, future package argv |
| `x.ai/hooks/list` | `project_trusted` = real resolve — **stop hardcoding `true`** |
| `/hooks-trust` / modal Trust | Persist `trust.json`; toast + structured log |
| After grant | Reload hooks registry; if restart needed, say so (Pi lesson) |

### 8.2 Interactive CLI

Stderr/TUI prompt when `ask` + TTY; same options where feasible.

### 8.3 Non-interactive / CI

| Mode | Behavior |
|------|----------|
| `run` / JSON / RPC / CI | No prompt |
| Undecided + `ask` | Deny project executables |
| `--approve-project-trust` | Allow this process |
| `--deny-project-trust` | Deny this process |
| Saved `true` | Allow without flag |

```bash
next-code --approve-project-trust run "…"
# or seed ~/.next-code/trust.json in the CI image for known paths
```

### 8.4 Logging

```text
level=info target=project_trust msg="skipped project hook — untrusted" cwd=… config=…/.next-code/hooks.toml
```

Never silent-allow.

### 8.5 Interaction with D2 enable

```text
discover project executables
  → enabled? (D2) — if no, skip contribution
  → trusted? (D1) — if no, skip spawn / do not compile argv handlers
  → D13 path/argv rules when spawning
```

Disabled plugins never spawn even if trusted. Trust without enable is inert for that plugin’s executables.

---

## 9. Runtime integration map

| Seam | Today | Required |
|------|-------|----------|
| `next_code_hooks::load_hooks_config` / execute | User + project layers unconditional | Skip **project** layer when untrusted; keep user layer |
| `face_plugins::hooks_list_payload` | `project_trusted: true` | Wire resolve |
| Face folder-trust session gate | Separate (`event_loop` / `folder_trust`) | Do not replace |
| Package argv (Phase 2+) | N/A | Same `resolve_project_trusted` before spawn |
| Skills ingest | Ungated by D1 | Out of scope v1 (§3.2) |

---

## 10. Acceptance tests (design IDs)

| ID | Scenario | Pass |
|----|----------|------|
| TG-01 | Untrusted + project command PreToolUse | Handler **does not** spawn |
| TG-02 | Trusted + same hook | Runs as today |
| TG-03 | Trust revoked / after reload | Next event skips |
| TG-04 | User-global hooks without project trust | Still run |
| TG-05 | Non-interactive deny / undecided ask | No hang; executables skipped |
| TG-06 | Skip log includes path + reason | Greppable |
| TG-07 | Ancestor trust (if adopted) | Nested cwd covered |
| TG-08 | `hooks_list_payload.project_trusted` | Matches resolve |
| TG-09 | MCP trust alone | Does **not** enable project hooks |
| TG-10 | Safe-eval default | Project executables denied unless approve |

Unit: store round-trip, nearest walk, unsafe-root refusal.

---

## 11. Acceptance checklist

### Functional

- [ ] Untrusted: project `hooks.toml` executables **do not run**; user-global hooks still run.
- [ ] Trusted (store or `--approve-project-trust`): project hooks run as today.
- [ ] Explicit deny / `--deny-project-trust`: no project executables even if Face folder-trust granted.
- [ ] Bare project without requiring resources: no prompt; no false skips.
- [ ] `hooks_list_payload.project_trusted` truthful.

### Non-interactive

- [ ] CI documented with approve/deny flags (or env).
- [ ] Undecided + ask + no TTY: fail closed.

### Orthogonality

- [ ] `mcp_trust.json` unchanged semantically.
- [ ] D2 disable independent of trust.
- [ ] D0 bare/pack prompt identity unchanged.

### Docs

- [ ] HOOKS.md + SAFE_EVALUATION + inventory updated in landing PR.
- [ ] Cookbook links here for project examples.

---

## 12. Out of scope

- Sandboxing after trust; WASM; seccomp; replacing DCG permissions.
- Collapsing MCP trust into project trust.
- Treating trust as “safe untrusted code” (D13).
- Forcing project skills/prompts behind this gate in Phase 1.
- Production Rust in this deepen file.

---

## 13. Open decisions (resolve in BUILD PR)

| Q | Docs stance |
|---|-------------|
| Home vs project `trust.json`? | **Home store** (Pi-like) |
| Schema A map vs B versioned? | Either; document choice |
| Ancestor inheritance vs exact root? | Prefer Pi nearest-entry; exact-only acceptable if UX documents parent trust |
| Dual-write folder-trust? | Independent Phase 1 |
| Project plugin presence alone triggers predicate? | Open — conservative yes OK |
| Flag names vs Pi `--approve`? | Prefer `--approve-project-trust` (distinct) |

---

## 14. Exit criteria (product)

Phase 1 trust gate is done when §11 is green and CI docs exist — then multilang cookbook / package argv may be marketed without a silent RCE hole.

**Status:** Design contract ready for Phase 1. Waiting master / readiness **go ahead** before production Rust.
