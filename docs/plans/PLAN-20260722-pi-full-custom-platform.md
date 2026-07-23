# Plan Report — Platform (Pi × herdr) + product pack “nextcode”

## Summary (read this first)

- **You asked:** Full custom like Pi, multilang like herdr — and a clear split between **platform** and the product people run today as **nextcode**.
- **What is going on:** Five deep-research reports (2026-07-22) confirm: Pi defines *which surfaces* to cover; herdr defines *how* (manifest + argv/HTTP, any language); OpenCode’s “plugin = in-process Hooks” is ideas-only; Face cannot host Pi-style guest TUI without a fork; today’s next-code already has the multilang hooks half and a Grok-shaped bundle half, with real wiring gaps.
- **We recommend:** **Option B′** — build a **language-agnostic platform** (Pi surface breadth × herdr ABI). Ship today’s wired defaults as a **default distribution pack named “nextcode”** (brand slash, Extensions chrome, settings catalog, starter skills/MCP, welcome). Others can **disable/bare** the pack and ship their own packages. Face stays a fixed Rust shell; no Bun-only, no Face custom TUI first.
- **Risk:** Medium for hybrid ABI + trust; **High** if chasing Pi in-process TUI or TS-only packages.
- **Status:** Waiting for your OK — reply **go ahead** to implement **Phase 1**. Phase 0 (this plan + research) is done. **No production code in this report.**

### Research index (source of truth for evidence)

| Report | Path |
|--------|------|
| Index | [`docs/plans/research/README.md`](./research/README.md) |
| Pi surfaces | [`research/20260722-pi-extension-surfaces.md`](./research/20260722-pi-extension-surfaces.md) |
| Herdr multilang ABI | [`research/20260722-herdr-multilang-abi.md`](./research/20260722-herdr-multilang-abi.md) |
| OpenCode hooks model | [`research/20260722-opencode-plugin-hooks.md`](./research/20260722-opencode-plugin-hooks.md) |
| next-code inventory | [`research/20260722-nextcode-extension-inventory.md`](./research/20260722-nextcode-extension-inventory.md) |
| Face customization limits | [`research/20260722-face-customization-limits.md`](./research/20260722-face-customization-limits.md) |

---

## Product vision (must stay clear)

### Two layers

```text
┌─────────────────────────────────────────────────────────────┐
│ PLATFORM (host + ABI)                                       │
│  Dynamic like Pi: hooks, slash, tools, skills, packages,    │
│  MCP, trust, install, UI *hints*                            │
│  Multilang like herdr: manifest declares WHAT;              │
│  runners HOW (argv stdio | HTTP | MCP | later socket/WASM)  │
│  Face = fixed Rust presentation (ACP + closed widgets)      │
│  NOT Bun-only · NOT Face custom TUI first · NOT QuickJS     │
└───────────────────────────┬─────────────────────────────────┘
                            │ default distribution
┌───────────────────────────▼─────────────────────────────────┐
│ PRODUCT PACK: “nextcode”                                    │
│  Today’s wired defaults as a disableable bundle:            │
│  · Brand-hidden slash set + visible /plugins /hooks         │
│  · Face Extensions chrome, welcome/status, settings catalog │
│  · Default tool enablement, auth connect UX                 │
│  · Optional starter skills / MCP / hook recipes             │
│  · Grok-compat plugin.json layouts as default schema        │
│  Disable / bare → other products rewrite their own packs    │
└─────────────────────────────────────────────────────────────┘
```

| Term | Meaning |
|------|---------|
| **Platform** | Host + package ABI. Any product can build on it. Surfaces = Pi breadth; implementation = herdr multilang. |
| **nextcode (product pack)** | The opinionated default distribution on that platform — what users get when they install next-code today. Not the same as “the only way to use the host.” |
| **Bare / alternate pack** | Disable nextcode defaults; ship different slash/brand/skills/hooks without forking Face or the agent loop. |

### Rule of thumb ([inventory § Recommended split](./research/20260722-nextcode-extension-inventory.md))

- **Host/core** — Face render, ACP, agent loop, permissions, hook dispatcher, MCP client, skill ingest engine.
- **Platform ABI** — anything authors add without a core PR (manifest + runners).
- **nextcode pack** — opinions a fork would change (brand, chrome labels, default slash palette, starter content).

### Explicit non-goals (v1)

- Embedding Pi’s TypeScript/jiti runtime or OpenCode Bun plugins into Face ([OpenCode research](./research/20260722-opencode-plugin-hooks.md), [#49 QuickJS removal](./research/20260722-face-customization-limits.md)).
- Face plugin-host / Pi `ctx.ui.custom` / Doom-in-Face ([Face limits](./research/20260722-face-customization-limits.md)).
- Replacing Face with a second full TUI for extension reasons.
- Making TypeScript the privileged author language.

---

## Plain answer (parent / non-dev)

**“Full custom như Pi + đa ngôn ngữ như herdr”** nghĩa là:

1. Tác giả customize được *cùng loại bề mặt* Pi (lifecycle, slash, tools, skills, packages, một phần UI) — **không** nhét Pi TS vào Face.
2. Implement bằng **bất kỳ ngôn ngữ nào** như herdr: **manifest khai báo entrypoint**; host **spawn argv** hoặc HTTP; callback qua JSON stdio / CLI / ACP.
3. Sản phẩm người dùng chạy hôm nay (**nextcode**) chỉ là **gói mặc định** trên nền tảng đó — brand slash, Extensions, settings, skill/MCP starter — có thể tắt / thay bằng pack khác.

---

## Architecture (single picture)

### Principle

```text
manifest declares WHAT  (slash, hook, tool, skill, ui hint, mcp…)
runner implements HOW   (argv stdio | HTTP | MCP | later WASM | later daemon)
Face/ACP renders WHERE  (fixed widgets; no in-process guest UI kit in v1)
nextcode pack supplies DEFAULTS (brand, chrome, starter content)
```

TS/JS = **optional** runner (`["node", "dist/hook.js"]`), equal to Python/Bash/Rust binaries ([herdr cookbook](./research/20260722-herdr-multilang-abi.md)).

### Runner ladder

| Tier | Mechanism | Status | Languages |
|------|-----------|--------|-----------|
| **0** | Declarative (skills, prompts, theme tokens, MCP config) | Partial | Data files |
| **1** | Shell/stdio argv — HookInput/Output JSON | **Exists** (`next-code-hooks`) | Any executable |
| **2** | HTTP hooks | **Exists** | Any HTTP server |
| **3** | Long-lived external process (stdio/socket) | Future | Any |
| **4** | WASM (wasi) sandbox | Future optional | Wasm-capable |
| **5** | In-process Face widget kit / guest VM | **Avoid** | — |

### Package layout (sketch — freeze name in Phase 1 tickets)

```text
my-ext/
  next-code-plugin.toml   # or extend plugin.json — open Q
  skills/…  prompts/…  mcp.json
  bin/ or scripts/        # any language
```

```toml
id = "example.policy"
version = "0.1.0"
min_next_code_version = "…"

[[hooks]]
on = "PreToolUse"
matcher = "Bash"
runner = { argv = ["python3", "hooks/pre_tool.py"] }

[[slash]]
name = "review"
kind = "prompt"   # or kind = "command" + runner argv

[[tools]]
name = "deploy_status"
input_schema = "schemas/deploy_status.json"
runner = { argv = ["./bin/deploy-status"] }

[[ui]]
status_id = "example.policy"   # string → fixed Face slot — not Components

[[resources]]
skills = ["skills"]
prompts = ["prompts"]
```

Package `[[hooks]]` **compile into** the existing hooks registry — do not invent a third runtime ([OpenCode steal/avoid](./research/20260722-opencode-plugin-hooks.md)).

### Pi surface → platform mapping

| Pi surface ([catalog](./research/20260722-pi-extension-surfaces.md)) | Platform | nextcode pack |
|----------------------------------------------------------------------|----------|---------------|
| Lifecycle `pi.on` | `[[hooks]]` → next-code-hooks | Optional policy recipes |
| Slash / prompts / skills | Manifest slash + skills + ACP | Builtin slash palette + brand-hide |
| Tools | MCP + later `[[tools]]` argv | Default Rust toolset enablement |
| Packages install | git/local + trust + enable | Demo/empty + import-on-first-run DX |
| Providers | config / gateway (no guest register) | Default provider UX |
| Custom TUI | **Hints / optional external pane only** | Welcome/status chrome |
| MCP | Keep (Pi has none) | Optional curated MCP set |
| Trust | Project trust before exe resources | nextcode permission defaults |

### Architecture options

| Option | Summary | Verdict |
|--------|---------|---------|
| **(A)** Face + deepen bundles only | Polish Grok bundles + hooks.toml | Too narrow — no unified ABI |
| **(B′)** Pi surfaces × herdr ABI + nextcode pack | Recommended | Fits Rust host + multilang + Face sealed |
| **(C)** True Pi in-process UI + guest runtime | Face plugin-host / TS VM | Conflicts with multilang + migration; reject for v1 |

---

## Baseline today (verified gaps)

From [inventory](./research/20260722-nextcode-extension-inventory.md) + [Face limits](./research/20260722-face-customization-limits.md):

| Already platform-shaped | Gap / lie in UI | nextcode-product today |
|-------------------------|-----------------|------------------------|
| hooks.toml argv/HTTP (~28 events) | Bundle `hooks/hooks.json` counted but **not** merged into registry | Brand-hidden slash; `/plugins` `/hooks` visible |
| MCP user/project json (stdio + HTTP) | Bundle `.mcp.json` count-only | Settings catalog, welcome chrome |
| Skills from plugin trees | `plugins-state.json` enable **ignored** by skill ingest | Face Extensions labels |
| Face ACP for plugins/hooks/skills/mcp | No package `[[slash]]` / argv tools ABI | Marketplace stub (hidden) |
| | Prompt templates → Face slash GAP | |

**Do not promise Face guest Components.** Ceiling without Face fork = workflow power + multilang + data → fixed widgets ([Face research §Verdict](./research/20260722-face-customization-limits.md)).

---

## Phased roadmap

### Phase 0 — Product contract *(docs)* — **DONE**

- [x] Five research reports under `docs/plans/research/`
- [x] This master plan as single source of truth
- [x] Explicit **platform vs nextcode pack**
- [x] Capability stance: TS optional; primary ABI = manifest + argv/HTTP; Face sealed
- **Exit:** Authors/PMs know what is possible in which language; no production code yet.

### Phase 1 — Trust + make current multilang real *(first build — waiting OK)* **← next**

**Goal:** Stop advertising dead wiring; prove multilang; gate executable project resources.

1. **Project trust** for executable project hooks/plugins (Pi `trust.json` lesson + herdr install preview) before marketing “any language extensions.”
2. **Enable-state gating** — `plugins-state.json` disable must stop skill ingest (and later MCP/hooks from that package).
3. **Wire or demote UI lies** — either merge bundle `hooks/hooks.json` + `.mcp.json` into real registries, **or** stop showing Active/Blocked counts as if live.
4. **Document** writing the same `PreToolUse` guard in **Bash / Python / Node** against existing hooks JSON protocol.
5. **Product profile sketch** — table of which slash/settings/welcome pieces are nextcode-pack (disableable later).
6. Finish any remaining PR12 P0 trust/sessions items that unblock the above.

**Recommended Phase 1 first build (smallest vertical slice):**

> Trust gate for project executable hooks + enable-state gating for plugin skills + one cookbook package that runs the **same** `PreToolUse` policy in three languages (Bash, Python, Node) against current `hooks.toml` / hooks execute path — **before** inventing `next-code-plugin.toml`.

**Exit:** One example policy in three languages; disabled plugins don’t inject skills; Face counts match runtime (or are honest).

### Phase 2 — Package ABI v1 (declarative + argv) *(3–6 weeks)*

- Manifest schema: hooks, slash (prompt + command), skills, mcp, resources.
- Install/link (herdr-shaped); compile package hooks into hooks registry.
- Slash prompts from package `prompts/`; ACP advertise.
- Official examples: `examples/ext-bash`, `examples/ext-python`, `examples/ext-node`.
- **Exit:** Third party ships a package without a core PR (hooks + slash + skills).

### Phase 3 — Tools via ABI *(4–8 weeks)*

- `[[tools]]` → allowlisted argv runners and/or MCP bridge.
- Providers remain core/gateway.
- **Exit:** New LLM-callable tool without core PR (policy + schema review).

### Phase 4 — UI without Face plugin-host *(optional)*

- ACP status/footer/notify + tool display **hints** into fixed Face widgets.
- Optional: herdr-like **external pane** (spawn user’s TUI binary) — not in-Face Components.
- Theme token files only if Face can map them first-party.
- **Exit:** Visible customization; still no Doom-in-Face.

### Phase 5 — Optional WASM / long-lived process

- Only if sandbox or latency needs exceed argv-per-event.
- Revisit Option C only with explicit product bet.

### Parallel: nextcode pack extraction

As ABI lands, gradually move brand-hide, welcome defaults, settings catalog groups, and starter content behind a **product profile / default pack** so bare/alternate products can opt out without forking the host. Not a blocker for Phase 1 first build.

---

## Steal vs avoid (compressed)

| Steal | From | Avoid |
|-------|------|-------|
| Surface breadth + package as unit | Pi | jiti / in-process Component host |
| Manifest + argv + env + host CLI callback | herdr | Fire-and-forget-only events (keep gate hooks) |
| Named seams, provenance, global vs project, server≠UI | OpenCode | Plugin ≡ Bun `Hooks` object; throw-to-deny; OpenTUI-in-Face |
| Fixed Face + ACP data | Face migration | Face rewrite / GrokHost / QuickJS |

---

## Comparison matrix

| Goal | Pi | Herdr | next-code now | After B′ Phases 1–4 |
|------|----|-------|---------------|---------------------|
| Custom lifecycle | In-process TS | argv plugin | hooks.toml any exe | Package `[[hooks]]` any lang |
| Multilang first-class | No | Yes | Hooks yes; packages weak | Yes |
| Custom slash | TS / md | actions | builtins + ACP skills | Manifest + nextcode slash pack |
| New tool | `registerTool` TS | N/A | MCP / core | `[[tools]]` argv/MCP |
| Custom UI | `ctx.ui.custom` | pane = argv | Face fixed | Hints + optional external pane |
| Default product | pi packages | — | Wired into host | **nextcode pack** (disableable) |
| Share package | npm pi-package | `herdr-plugin.toml` | git/local bundles | Unified manifest + git |

---

## Risks

| Risk | Level | Mitigation |
|------|-------|------------|
| Accidental TS-only platform | High | ABI = argv/HTTP; Phase 1 = 3-lang cookbook |
| Promising Pi TUI inside Face | High | Phase 4 hints only; Face research forbids plugin-host |
| Executable project plugins = RCE | High | Phase 1 trust gate |
| Manifest sprawl vs hooks.toml | Medium | Package ABI compiles *into* hooks registry |
| nextcode pack never extracted → forever fork | Medium | Product profile table from Phase 1; extract in Phase 2+ |
| WASM / QuickJS resurrection | Medium | Phase 5 only; #49 reject |

---

## Open questions (≤3)

1. Manifest filename: new `next-code-plugin.toml` (herdr-clear) vs extend Grok `plugin.json`?
2. Should `[[tools]]` argv runners ship in Phase 3, or MCP-only until then?
3. Phase 4: external “plugin pane” (spawn TUI binary) desirable, or Face-hints-only?

---

## Recommendation (one line)

**Build the platform as Pi surfaces × herdr multilang ABI; treat “nextcode” as the default disableable product pack on that platform; Face stays sealed — reply go ahead to start Phase 1.**

---

## Status

**Phase 0 complete (research + this plan).**  
**Waiting for your OK** — reply **go ahead** to implement **Phase 1** (trust + enable gating + honest bundle wiring + 3-lang cookbook). No production Rust until then.
