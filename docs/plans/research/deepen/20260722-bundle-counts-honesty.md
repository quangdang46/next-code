# Deepen — Bundle Active/Blocked counts honesty (2026-07-22)

**Priority:** P1 · Phase 1  
**Parent:** Master plan Phase 1 §3  
**Paired with:** [`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md)  
**Evidence:** [`../20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md) GAP rows for `hooks/hooks.json` / `.mcp.json`  
**Index:** D3 in [`README.md`](./README.md)

**Status:** Design contract only — **no production code** in this ticket. Implement after master/readiness **go ahead**.

---

## 1. Problem (verified)

Face Plugins UI treats bundle `hooks/hooks.json` and `.mcp.json` as if they are **live runtime contributions**. They are not.

| Claim | Reality today | Citation |
|-------|---------------|----------|
| Bundle has hooks | File existence only: `root.join("hooks").join("hooks.json").is_file()` | `src/cli/face_plugins.rs` → `discover_in_parent` / `looks_like_plugin_dir` |
| Bundle has MCP | File existence only: `root.join(".mcp.json").is_file()` → count `0` or `1` | same |
| Status “Active” | `to_info`: if `has_hooks` and `enabled` → `HookStatus::Active` | `src/cli/face_plugins.rs` → `fn to_info` |
| Status “Blocked” | `to_info`: if `has_hooks` and **not** `enabled` → `HookStatus::Blocked` | same for `McpStatus` |
| Runtime hooks | Only `~/.next-code/hooks.toml` (+ project + env) via `next-code-hooks` | `docs/HOOKS.md`, `crates/next-code-hooks/` |
| Runtime MCP | `~/.next-code/mcp.json` + project locals — **not** plugin-dir `.mcp.json` | `crates/next-code-base/src/mcp/protocol.rs` → `load_project_locals` / `load_for_dir` |
| Docs | Explicitly deferred merge | `docs/HOOKS.md` (“Import of plugin-bundled `hooks/` JSON … deferred”); `docs/plugins.md` lists convention dirs only |

**UI lie summary:** `Active` / `Blocked` and Face component strings imply the host loaded or blocked those handlers/servers. Enable/disable only flips a Face/list enum; it does **not** register into `HookRegistry` or `McpConfig`.

---

## 2. Decision rule (frozen for Phase 1)

| Option | Meaning | When |
|--------|---------|------|
| **A — Wire** | Merge into real registries (see merge deepen). Counts = runtime. | Only when Phase 1.5/2 merge lands in the **same** release window |
| **B — Demote** | Stop showing Active/Blocked as live; label “declared / not loaded” (or hide until wired) | **Phase 1 default** |

**Phase 1 frozen choice: Decision B (Demote).**

Do **not** mix A and B silently on different surfaces (e.g. demote hooks but keep MCP “Active”). Both hooks and MCP bundle surfaces demote together until merge ships.

When merge ships ([`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md)), flip both surfaces to **A** in one PR: `to_info` + Face strings + docs.

---

## 3. Semantics today (exact code)

### 3.1 Discovery (`Discovered`)

```text
has_hooks          := exists(<plugin>/hooks/hooks.json)
mcp_server_count   := 1 if exists(<plugin>/.mcp.json) else 0
hook_count (info)  := 1 if has_hooks else 0   # not # of PreToolUse entries
```

Citations:

- `src/cli/face_plugins.rs` — `looks_like_plugin_dir` (lines recognizing `hooks/hooks.json`, `.mcp.json`)
- `src/cli/face_plugins.rs` — `discover_in_parent` sets `has_hooks`, `mcp_server_count`
- Install-registry branch duplicates the same existence checks

**Honesty gap already in counts:** `hook_count` is boolean-as-1, not parsed matcher count. `mcp_server_count` is boolean-as-1, not parsed `mcpServers` keys. Demote must not pretend those numbers are registry sizes either.

### 3.2 `to_info` (source of Active/Blocked)

Exact mapping today (`src/cli/face_plugins.rs` → `fn to_info`):

| Condition | `hook_status` | `mcp_status` |
|-----------|---------------|--------------|
| `!has_hooks` / `mcp_server_count == 0` | `HookStatus::None` | `McpStatus::None` |
| file present ∧ `enabled` | `HookStatus::Active` | `McpStatus::Active` |
| file present ∧ `!enabled` | `HookStatus::Blocked` | `McpStatus::Blocked` |

`PluginInfo.trusted` is always `true` (deprecated field). Type comments in `crates/xai-hooks-plugins-types/src/lib.rs` say status is “derived from trust + has_hooks …” — **code does not match the comment**. Demote must align comments with behavior or replace the enum usage.

Enable gate: `is_enabled` reads `~/.next-code/plugins-state.json` (`disabled` / `enabled` lists); default enabled. Citation: `face_plugins.rs` → `is_enabled`, `plugins_list_payload`.

### 3.3 Face strings (`build_plugin_fields`)

Citation: `crates/xai-grok-pager/src/views/extensions_modal.rs` → `fn build_plugin_fields`.

| Field | Current string | Implied meaning |
|-------|----------------|-----------------|
| `hook_count > 0` | `"{n} hooks"` | Present as a component (no Active/Blocked word) |
| `McpStatus::Active \| ActiveInline` | `"{n} MCP servers"` | Sounds **live / connected** |
| `McpStatus::Blocked` | `"{n} MCP: blocked"` | Sounds **trust/runtime blocked** |
| `McpStatus::None` | (omit) | — |

Hooks do not currently print `hook_status` text; the lie for hooks is:

1. ACP/JSON still ships `hookStatus: "active"|"blocked"`.
2. Docs / product language / type names say Active = running.
3. Users equate Plugins tab components with `/hooks` and `/mcp` tabs.

MCP string `"N MCP servers"` is the worst Face-visible lie (reads like the MCP Servers tab).

### 3.4 Type surface

`crates/xai-hooks-plugins-types/src/lib.rs`:

```text
HookStatus  = Active | ActiveInline | Blocked | None
McpStatus   = Active | ActiveInline | Blocked | None
PluginInfo  = { hook_status, hook_count, mcp_server_count, mcp_status, ... }
```

`ActiveInline` is unused by next-code `to_info` today (Grok-era leftover). Demote must not invent inline semantics.

ACP: `x.ai/plugins/list` → `PluginsListResponse` via `face_plugins::plugins_list_payload`.

---

## 4. Decision B — exact contract (what to change when implementing)

### 4.1 New status vocabulary (recommended)

Prefer **reusing** existing enums with **honest meanings** until merge, then restore Active = runtime:

| Enum value (wire) | Phase 1 B meaning | After merge (A) |
|-------------------|-------------------|-----------------|
| `none` | No declaration file | No contribution |
| `active` | **Forbidden** for unmerged bundles | Loaded into registry / MCP client and enabled |
| `blocked` | **Forbidden** for “disabled plugin with file” alone | Declared but gated (disabled **or** untrusted **after** merge) |
| `active_inline` | Unused; keep unused | Optional later |

**Problem:** serde wire already uses `active` / `blocked`. Changing enum variants is a Face/ACP break.

**Chosen Phase 1 approach (minimal wire break):**

1. Keep enum variants for serde compatibility.
2. Stop emitting `Active` / `Blocked` from `to_info` for file-only discovery.
3. Emit `None` for hooks/MCP when not actually loaded — **plus** separate declaration counts/flags for Face copy.
4. **Or** (preferred if we can add fields): add optional honesty fields and set status to `None` until merge.

#### Preferred field additions (ABI-compatible)

Add to `PluginInfo` (camelCase wire):

| Field | Type | Meaning |
|-------|------|---------|
| `hooksDeclared` | `bool` | `hooks/hooks.json` exists |
| `mcpDeclared` | `bool` | `.mcp.json` exists |
| `hooksLoaded` | `bool` | At least one handler with provenance `plugin:<id>` in `HookRegistry` (always `false` until merge) |
| `mcpLoaded` | `bool` | At least one MCP server with `config_source` = `plugin:<name>` connected/listed (always `false` until merge) |

Phase 1 B mapping in `to_info`:

```text
hooksDeclared = has_hooks
mcpDeclared   = mcp_server_count > 0   # or bool from file
hooksLoaded   = false                  # until merge
mcpLoaded     = false
hook_status   = HookStatus::None       # never Active/Blocked for file-only
mcp_status    = McpStatus::None
hook_count    = 0                      # OR keep raw file flag only via hooksDeclared
mcp_server_count = 0                   # do not advertise as servers
```

**Alternate (no new fields):** keep `hook_count` / `mcp_server_count` as declaration counts but force statuses to `None` and fix Face strings only. Accept that JSON still has counts without “Active”. Document that counts mean “declared files”, not registry size.

**Frozen recommendation:** Prefer new `*Declared` / `*Loaded` fields **or** Face-only copy fix with statuses forced to `None`. Do **not** continue emitting `Active` for enabled plugins with only files on disk.

### 4.2 Exact `to_info` change (Decision B)

Replace today’s branch:

```text
# TODAY (lie)
if !has_hooks { None } else if enabled { Active } else { Blocked }
```

With:

```text
# PHASE 1 B (honest)
hook_status = None
mcp_status  = None
# declaration surfaced only via hooksDeclared/mcpDeclared OR demoted Face strings
# enabled still drives plugin row Enabled/Disabled filter — unrelated to hook runtime
```

`enabled` continues to control skill-gate work ([`20260722-plugins-state-skill-gate.md`](./20260722-plugins-state-skill-gate.md)) and Plugins tab StatusFilter — **not** fake hook Active.

### 4.3 Exact Face string changes (`build_plugin_fields`)

Citation target: `crates/xai-grok-pager/src/views/extensions_modal.rs` → `build_plugin_fields`.

| Today | Phase 1 B (required) | After merge A |
|-------|----------------------|---------------|
| `"{n} hooks"` when `hook_count > 0` | `"hooks.json (declared, not loaded)"` if declared ∧ !loaded | `"{n} hooks"` only if `hooksLoaded` (n = registry handlers) |
| `"{n} MCP servers"` on Active/ActiveInline | **Never** for file-only | `"{n} MCP servers"` only if loaded into MCP list |
| `"{n} MCP: blocked"` on Blocked | Replace with `"MCP declared (plugin disabled)"` **only if** we still want disable nuance **without** implying runtime gate — or omit MCP line entirely when `!mcpLoaded` | `"MCP: blocked"` only when merge+trust/disable actually strips servers |

**Frozen Phase 1 B Face copy (copy-paste):**

```text
if hooks_declared && !hooks_loaded:
    push "hooks: declared (not loaded into /hooks)"
if mcp_declared && !mcp_loaded:
    push "MCP: declared (not loaded into /mcp)"
# Do not push "N MCP servers" or "N MCP: blocked" for declarations.
```

If new fields are not yet on the wire, Face may temporarily derive:

```text
hooks_declared := hook_count > 0 || hook_status != None   # transitional
# After to_info forces None + zero counts, Face MUST use new fields or stop showing hook/MCP lines.
```

**Tests to update:** any `extensions_modal` / `modals` fixtures that assert `"MCP servers"` / `"MCP: blocked"` for fake `PluginInfo` with Active/Blocked — search `make_plugin`, `plugin_info`, `mcp_status:` in:

- `crates/xai-grok-pager/src/views/extensions_modal.rs` (test module)
- `crates/xai-grok-pager/src/app/agent_view/modals.rs`

### 4.4 Docs copy changes (same PR as B)

| File | Change |
|------|--------|
| `docs/plugins.md` | State clearly: `hooks/hooks.json` and `.mcp.json` are **recognized for discovery** and listed as **declared**; they are **not** merged into `next-code-hooks` / MCP runtime until Phase 1.5/2. |
| `docs/HOOKS.md` | Keep “deferred” note; add pointer: Face must not say Active; cookbook uses `hooks.toml` directly ([`20260722-hooks-cookbook-layout.md`](./20260722-hooks-cookbook-layout.md)). |
| Type comments in `xai-hooks-plugins-types` | Fix “derived from trust” comment to match actual fields / post-merge rules. |

---

## 5. Decision A — what “honest Active” means later

When merge lands, restore:

```text
hook_status =
  None                         if no compiled handlers for plugin
  Active                       if ≥1 handler loaded ∧ plugin enabled ∧ trust OK
  Blocked                      if handlers would load but enable=false OR trust deny
                               (or strip handlers and use None — pick one; prefer strip+None
                                for disabled, Blocked only for “declared but trust-gated”)

mcp_status = analogous against McpManager catalog rows with config_source plugin:…
```

Face strings:

```text
"{n} hooks"           # n = loaded handlers
"{n} MCP servers"     # n = loaded servers
"{n} MCP: blocked"    # only if Blocked semantics chosen
```

Counts must match `/hooks` and `/mcp` tabs for that plugin’s provenance. Cross-check with [`20260722-hook-registry-reload.md`](./20260722-hook-registry-reload.md).

---

## 6. Non-goals

- Parsing full Grok `hooks.json` in the honesty PR (that is merge deepen).
- Changing skill_count / agent_count display (separate GAP for agents).
- Reviving QuickJS / in-process TS hooks.
- Marketplace catalog `has_hooks` flags in Face marketplace stubs (still stub/hidden).

---

## 7. Risks

| Risk | Mitigation |
|------|------------|
| Face clients assume `hookStatus == active` means runnable | Force `None` + docs; grep ACP consumers |
| Users think demote removed features | Copy says “declared, not loaded” + link to cookbook / merge plan |
| Half-demote (MCP only) | Checklist: both statuses + both Face branches in one PR |
| `hook_count: 1` still misread as registry size | Prefer `hooksDeclared` bool; document counts as file flags if kept |

---

## 8. Verification checklist (implement PR)

- [ ] `to_info` never emits `HookStatus::Active` / `McpStatus::Active` solely because files exist.
- [ ] `to_info` never emits `Blocked` solely because plugin disabled while files exist (unless merge+gate).
- [ ] Face Plugins expanded row does **not** show `"N MCP servers"` or `"N MCP: blocked"` for file-only plugins.
- [ ] Face shows demoted declared copy (or omits hook/MCP lines).
- [ ] `docs/plugins.md` + `docs/HOOKS.md` match B.
- [ ] Unit tests: enabled plugin with `hooks/hooks.json` + `.mcp.json` fixtures → statuses `None` (or new Loaded=false).
- [ ] Manual: `/plugins` vs `/hooks` vs `/mcp` — no visual claim that bundle hooks/MCP are live.

---

## 9. Exit criteria

- [ ] No Face string implies runtime load for unmerged bundle hooks/MCP.
- [ ] No ACP `PluginInfo` uses Active/Blocked for file-existence-only discovery.
- [ ] If A later: counts match registry after enable+trust gates (merge deepen).
- [ ] Docs (`plugins.md` / Face copy) match chosen option (B now).

---

## 10. Implementation file list (when approved)

| File | Role |
|------|------|
| `src/cli/face_plugins.rs` | `to_info`, discovery fields |
| `crates/xai-hooks-plugins-types/src/lib.rs` | optional new fields + comment fix |
| `crates/xai-grok-pager/src/views/extensions_modal.rs` | `build_plugin_fields` + tests |
| `docs/plugins.md`, `docs/HOOKS.md` | honesty language |

---

## 11. Open questions (≤2 — non-blocking)

1. Prefer new `hooksDeclared`/`hooksLoaded` fields vs Face-only string demote with zeroed counts?
2. After merge, for disabled plugins: strip contributions (`None`) or keep `Blocked` badge?

Default if unanswered: **new fields if cheap; else Face+status None**; after merge **strip → None** for disabled.

---

## Status

Waiting for master/readiness **go ahead** before Rust. Pair merge deepen for A; ship **B first** if merge slips.
