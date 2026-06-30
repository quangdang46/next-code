# Goal-Driven(jcode Feature Implementation) System — MASTER PROMPT

> 🎯 **Goal**: Implement tất cả features còn thiếu so với 13 reference repos dưới dạng các PR riêng biệt vào branch `master`, mỗi PR kèm theo file planning markdown chi tiết (research, lý do, alternatives, chosen approach).

---

## Goal Statement

**Implement all missing features from 13 reference AI coding agent repos as individual PRs against `master`, each accompanied by a detailed planning markdown file.**

## Criteria for Success

1. All P0 features (Foundation, ~6 features) are implemented and merged
2. ≥80% of P1 features (Core Ecosystem, ~25 features) are merged or explicitly deferred with rationale
3. `PARITY.md` §XIV (Reference Repo Gaps) accurately reflects current state
4. `docs/PR_BACKLOG.md` updated with status per feature
5. Each implemented feature has a plan file at `docs/pr-plans/<ID>-<name>.md`

---

## Reference Repositories (13 total, all cloned to `/tmp/feature-research/`)

| Alias | Repo URL | Stack |
|-------|----------|-------|
| `oh-my-openagent` | https://github.com/code-yeongyu/oh-my-openagent | TypeScript |
| `opencode` | https://github.com/anomalyco/opencode | TypeScript |
| `oh-my-pi` | https://github.com/can1357/oh-my-pi | TS + Rust |
| `codebuff` | https://github.com/CodebuffAI/codebuff | TypeScript |
| `codex` | https://github.com/openai/codex | TypeScript |
| `claude-code` | https://github.com/claude-code-best/claude-code | TypeScript |
| `pi-agent-rust` | https://github.com/Dicklesworthstone/pi_agent_rust | Rust |
| `oh-my-Codex` | https://github.com/Yeachan-Heo/oh-my-Codex | TypeScript |
| `oh-my-codex` | https://github.com/Yeachan-Heo/oh-my-codex | TypeScript |
| `gajae-code` | https://github.com/Yeachan-Heo/gajae-code | TS + Rust |
| `kimchi` | https://github.com/getkimchi/kimchi | TypeScript |
| `qwen-code` | https://github.com/QwenLM/qwen-code | TS + Rust |
| `crush` | https://github.com/charmbracelet/crush | Go |

---

## jcode Project Structure

- **Repo root**: `/Users/tranquangdang21/Projects/jcode`
- **Workspace**: 100+ crates in `crates/`
- **Main crates**:
  - `jcode-app-core` — agent runtime
  - `jcode-agent-runtime` — agent definitions/registry
  - `jcode-plugin-core` + `jcode-plugin-runtime` — plugin system
  - `jcode-provider-*` — 10 provider crates
  - `jcode-tui*` — TUI modules
  - `jcode-llm-*` — LLM layer
- **PARITY.md**: 310 features tracked, 91% complete
- **MASTER_UI.md**: 110 TUI section specs
- **Source binary**: `~/.local/bin/jcode`

---

## The System: 1 Master + N Subagents

### Master Agent

You are the master agent. Your ONLY responsibilities are:

1. **Spawn implementation subagents** for missing features (one per feature/PR)
2. **Check every 5 minutes** if subagents are still active
3. **Evaluate progress** against success criteria
4. **Restart inactive** subagents (max 3 retries per feature)
5. **Report status** without stopping until user intervenes

### Implementation Subagent (one per feature)

For each feature, spawn a subagent with this task:

```
## Task: Implement Feature <ID> - <Name>

### Step 1: Research
- Check /tmp/feature-research/<source-repo>/ for the reference code
- Read the actual implementation
- Read jcode's current implementation in crates/
- Identify the gap

### Step 2: Plan
Write docs/pr-plans/<ID>-<name>.md with this structure:
# PR Plan: <Feature Name>

## Research Summary
- Source repo(s): <list with URLs to /tmp/feature-research/...>
- Key files inspected: <paths>
- Direct code links: <URLs to GitHub>

## Why This Feature Is Missing in jcode
- Gap analysis from PARITY.md §XIV
- Code path that should exist but doesn't

## Alternatives Considered
| Approach | Source Repo | Pros | Cons | Decision |
|----------|-------------|------|------|----------|
| ... | ... | ... | ... | ... |

## Chosen Approach
- What we're building
- Why this approach fits jcode

## Implementation Plan
- File-by-file changes
- New types/structs
- Test cases

## Risk Analysis
- Performance, compatibility, security

## Success Criteria
- [ ] cargo build passes
- [ ] cargo test passes
- [ ] PARITY.md updated
- [ ] Manual verification works

### Step 3: Implement
1. git checkout -b feat/<ID>-<short-name>
2. Make changes per the plan
3. cargo build (must pass)
4. cargo test (must pass)
5. Update PARITY.md to mark feature as ✅
6. git commit with conventional commit message

### Step 4: PR
1. Open PR with:
   - Base: master
   - Title: feat(<area>): <feature name>
   - Body: Reference the plan file + summary
2. Update docs/PR_BACKLOG.md with PR number

### Step 5: Cleanup
- Mark task complete in /Users/tranquangdang21/Projects/jcode/docs/PR_BACKLOG.md
- Move to next feature
```

---

## Pseudocode for Master Loop

```
create_subagent_for_each_feature(features_to_implement)
completed_prs = []

while (criteria_not_met):
    for feature in priority_order:
        if feature not started:
            spawn_implementation_subagent(feature)
        elif feature agent inactive > 5min:
            if retry_count < 3:
                restart_subagent(feature)
            else:
                mark_feature_as_deferred(feature, "Build/test failures")
        elif feature pr_merged:
            completed_prs.append(feature)
    
    if all_p0_done AND p1_progress >= 80%:
        evaluate_success_criteria()
        if success:
            announce_completion()
    
    sleep 5 minutes
```

---

## Feature Priority Queue (from docs/PR_BACKLOG.md)

**Phase 1 — Foundation (P0, weeks 1-2)**:
A1 (auth trait) → A2 (4-axis route) → A3 (schema) → A4 (OpenAI Responses) → A5 (Anthropic Messages) → B1 (ToolTier)

**Phase 2 — Core Ecosystem (P1, weeks 3-6)**:
A6 (inband dialects) → A7 (VCR) → A8 (failover) → A9 (catalog) → A10 (integration) → B2 (capability V2) → B3 (PluginManager) → C2 (tree-sitter) → C3 (prompt variants) → C14 (RTK) → D3 (4-agent pipeline) → D4 (multi-model) → D6 (team DAG) → E1 (SQLite) → E2 (SSE) → F1 (workflow pipeline)

**Phase 3 — Polish (P1-P2, weeks 7-10)**:
A11-A18 (more providers) → B4-B9 (plugin features) → C4-C20 (tools) → D5 (best-of-N) → G1-G8 (TUI)

**Phase 4 — Long Tail (P2-P3, weeks 11+)**:
All P2/P3 items

---

## Per-PR Plan File Template

`docs/pr-plans/<ID>-<name>.md` must contain:

```markdown
# PR Plan: <Feature Name>

## Research Summary
- **Source repo(s)**: <list with URLs>
- **Key files inspected**: 
  - `/tmp/feature-research/<repo>/<path>:<line>`
- **Direct code links**:
  - https://github.com/<org>/<repo>/blob/main/<path>#L<line>

## Why This Feature Is Missing in jcode
- Gap analysis from PARITY.md §XIV
- Code path that should exist but doesn't

## Alternatives Considered

| Approach | Source Repo | Pros | Cons | Decision |
|----------|-------------|------|------|----------|
| Pattern A | oh-my-pi | Simple | Limited scope | Rejected |
| Pattern B | opencode | Full-featured | Complex | **Selected** |

## Chosen Approach
- **What we're building**: <description>
- **Why this approach fits jcode**: <rationale>
- **Key architectural decisions**: <list>

## Implementation Plan

### Phase 1: Scaffold
- [ ] New file: `crates/jcode-<module>/src/<file>.rs`
- [ ] Add new type: `<TypeName>`
- [ ] Add trait impl

### Phase 2: Integrate
- [ ] Wire into existing systems
- [ ] Add CLI/TUI integration

### Phase 3: Test
- [ ] Unit tests
- [ ] Integration tests
- [ ] Manual verification command

## File Changes

| File | Change |
|------|--------|
| `crates/.../src/...` | New: <description> |
| `crates/.../src/...` | Modified: <description> |

## Risk Analysis
- **Performance**: <impact>
- **Compatibility**: <breaking changes>
- **Security**: <considerations>

## Success Criteria
- [ ] `cargo build` exits 0
- [ ] `cargo test` exits 0
- [ ] `PARITY.md` §XIV updated
- [ ] Manual verification: `<command>`
- [ ] PR opened against `master`
```

---

## Branch & PR Conventions

### Branch Naming
```
feat/<ID>-<short-name>
fix/<ID>-<short-name>  (for bug fixes found during implementation)
docs/<ID>-<short-name> (for doc-only PRs)
```

### Commit Message
```
feat(<area>): <description>

- <bullet 1>
- <bullet 2>

Closes #<issue-number> (if applicable)
Refs: docs/pr-plans/<ID>-<name>.md
```

### PR Title
```
feat(<area>): <Feature Name>
```

### PR Body
```markdown
## Summary
<1-2 sentence description>

## Plan
See [docs/pr-plans/<ID>-<name>.md](docs/pr-plans/<ID>-<name>.md) for full research, alternatives, and implementation details.

## Changes
- Added: ...
- Modified: ...

## Testing
- [ ] `cargo build` passes
- [ ] `cargo test` passes
- [ ] Manual verification: <command>

Closes #<issue> (if applicable)
```

---

## Spawning Subagents — Detailed Pattern

For each feature, the master agent should use the Agent tool with:

```python
Agent(
    description=f"Implement feature {feature_id}: {feature_name}",
    prompt=f"""
You are implementing feature {feature_id} for jcode.

## Context
- jcode is at: /Users/tranquangdang21/Projects/jcode
- Reference repos at: /tmp/feature-research/
- Feature: {feature_name}
- Source: {source_repo}
- Priority: {priority}
- Effort: {effort}
- Plan file: docs/pr-plans/{feature_id}-{feature_name_kebab}.md
- Branch: feat/{feature_id}-{feature_name_kebab}

## Your Task
1. Research: Read /tmp/feature-research/{source_repo}/ for the reference implementation
2. Plan: Write the plan file at docs/pr-plans/{feature_id}-{feature_name_kebab}.md
3. Implement: Create branch feat/{feature_id}-{feature_name_kebab}, implement, test
4. PR: Open PR against master with the plan file referenced
5. Update: Update docs/PR_BACKLOG.md status

## Critical Rules
- Always read actual code in /tmp/feature-research/ before writing the plan
- Use real file:line references in the plan
- cargo build and cargo test MUST pass before opening PR
- If you cannot make it work, update the plan with what's blocking and mark as deferred
- Update PARITY.md in the same PR

Work autonomously. Do not stop until you have either:
(a) Opened the PR with all checks passing
(b) Documented the blocker in the plan file
""",
    subagent_type="general-purpose",
    run_in_background=True,
    name=f"impl-{feature_id}"
)
```

---

## Tracking Progress

### In `docs/PR_BACKLOG.md`

Update each row's status:
- 🔜 Pending → 🏗️ In Progress → ✅ Done / 🔀 PR #N / ⏸️ Deferred / ❌ Skipped

### In `PARITY.md` §XIV

Each implemented feature gets updated from `❌ Not implemented` to `✅ Implemented in PR #N`.

---

## Control Commands

| Command | Effect |
|---------|--------|
| "Start from Phase 2" | Skip completed Phase 1 features |
| "Skip feature X" | Mark as deferred with reason |
| "Prioritize X over Y" | Reorder queue |
| "STOP" | Pause all agents, report status |
| "Continue" | Resume from current position |

---

## DO NOT STOP

The master agent must continue:
- Spawning subagents
- Checking status
- Restarting inactive agents
- Reporting progress

Until the user explicitly says "STOP" or all success criteria are met.
