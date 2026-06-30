# Goal-Driven(jcode Feature Implementation) System

## 🎯 Goal

**Implement all missing features from 13 reference AI coding agent repos as individual PRs against `master`, each accompanied by a detailed planning markdown file.**

Each PR must:
1. Have base branch = `master`
2. Include a plan markdown file (`docs/pr-plans/<ID>-<name>.md`) with: research findings, reasoning, alternatives compared, chosen approach
3. Pass `cargo build` and `cargo test`
4. Update PARITY.md to mark the feature as implemented

---

## ✅ Criteria for Success

**The system is complete when:**
1. All P0 features are implemented and merged
2. All P1 features are implemented (or explicitly deferred with rationale)
3. PARITY.md §XIV (Reference Repo Gaps) shows all P0/P1 items marked ✅ or ❌(skipped)
4. The PR backlog (`docs/PR_BACKLOG.md`) is updated with actual status per feature
5. Each implemented feature has a plan file at `docs/pr-plans/<ID>-<name>.md`

---

## 🏗️ System Architecture

### Master Agent (this session)

The master agent is responsible for:
1. **Supervising** the implementation subagents
2. **Checking progress** every 5 minutes
3. **Restarting inactive** subagents
4. **Evaluating** whether success criteria are met
5. **NOT stopping** until user manually stops

### Implementation Subagents

Each implementation subagent handles ONE feature PR:
- Reads the plan file template at `docs/pr-plans/<ID>-<name>.md`
- Clones/checkouts the relevant reference repo at `/tmp/feature-research/<repo>`
- Compares against jcode's actual implementation
- Writes the plan markdown (research, reasoning, alternatives, chosen approach)
- Implements the feature
- Runs tests
- Opens a PR with proper description
- Updates the backlog

---

## 📋 Workflow

### Step 1 — Prioritized Queue

Features are processed in this order (from `docs/PR_BACKLOG.md`):

```
Phase 1 (Foundation - P0):
  A1 → A2 → A3 → A4 → A5 → B1

Phase 2 (Core Ecosystem - P1):
  A6 → A7 → A8 → A9 → A10 → B2 → B3 → C2 → C3 → C14 → D3 → D4 → D6 → E1 → E2 → F1

Phase 3 (Polish - P1-P2):
  A11 → A12 → A16 → A17 → B4 → B7 → C4 → C6 → C15 → C16 → C20 → D5 → G1 → G2 → G3 → G6 → G7 → G8

Phase 4 (Long Tail - P2-P3):
  Remaining P2/P3 items
```

### Step 2 — Implementation Subagent Task

For each feature, spawn an implementation subagent with:

```
## Task for Feature: <feature-name> (<ID>)

### Context
- Feature description: <from PR_BACKLOG.md>
- Source repos: <from PR_BACKLOG.md>
- Priority: <P0/P1/P2/P3>
- Effort: <S/M/L/XL>
- Plan file: docs/pr-plans/<ID>-<name>.md
- Branch name: feat/<ID>-<short-name>

### Research Phase
1. Check /tmp/feature-research/<repo>/ for cloned reference code
2. If not cloned: git clone --depth=1 <repo_url> /tmp/feature-research/<repo>
3. Read the actual reference implementation code
4. Read jcode's current implementation
5. Compare and identify gaps

### Plan Phase
Write docs/pr-plans/<ID>-<name>.md with:
- Research summary (source files, direct links)
- Why this feature is missing in jcode
- Alternatives considered (table format)
- Chosen approach with rationale
- Implementation plan (file-by-file)
- Risk analysis
- Success criteria checklist

### Implementation Phase
1. git checkout -b feat/<ID>-<short-name>
2. Implement the feature following the plan
3. cargo build (must pass)
4. cargo test (must pass)
5. Update PARITY.md status to ✅
6. git add + commit

### PR Phase
1. Create PR with:
   - Base: master
   - Title: feat(<area>): <feature name>
   - Body: Reference the plan file + summary of changes
   - Labels: feature, <area>
2. Push branch
3. Update docs/PR_BACKLOG.md row status to "PR #<number>"

### Cleanup
- Delete /tmp/feature-research/<repo>/ if you cloned it
```

### Step 3 — Master Loop

```
WHILE criteria not met:
    1. Check PR backlog status
    2. Identify next unstarted feature from Phase 1-4
    3. Spawn implementation subagent for that feature
    4. Wait 5 minutes (or until agent completes)
    5. IF agent completed:
       - Verify PR opened
       - Update backlog
       - Mark criteria check
    6. IF agent inactive:
       - Restart new agent with same task
    7. IF all Phase 1+2 features done:
       - Final evaluation
       - Report summary
```

---

## 🔧 Per-Feature Implementation Pattern

### Creating the Plan File

Each `docs/pr-plans/<ID>-<name>.md` follows this template:

```markdown
# PR Plan: <Feature Name>

## Research Summary
- Source repo(s): <list with URLs>
- Key files inspected: <paths in /tmp/feature-research/...>
- Direct code links:
  - https://github.com/<org>/<repo>/blob/main/<path>#L<line>
  - ...

## Why This Feature Is Missing in jcode
- Gap analysis from PARITY.md §XIV
- Code path that should exist but doesn't
- Architectural reason for absence

## Alternatives Considered

| Approach | Source Repo | Pros | Cons | Decision |
|----------|-------------|------|------|----------|
| Alternative A | oh-my-pi | ... | ... | Rejected because... |
| Alternative B | opencode | ... | ... | Selected ✓ |

## Chosen Approach
- What we're building
- Why this approach fits jcode's architecture
- Key architectural decisions

## Implementation Plan

### Phase 1: Scaffold
- [ ] Add new types to `crates/jcode-<module>/src/`
- [ ] Add tests

### Phase 2: Integrate
- [ ] Wire into existing systems
- [ ] Add CLI/TUI integration

### Phase 3: Test
- [ ] Unit tests
- [ ] Integration tests
- [ ] Manual verification

## File Changes

| File | Change |
|------|--------|
| `crates/jcode-xxx/src/yyy.rs` | New: Z struct, impl Trait |
| `crates/jcode-app-core/src/agent.rs` | Modified: added trait impl |
| `PARITY.md` | Updated: feature row → ✅ |

## Risk Analysis
- **Performance**: <impact description>
- **Compatibility**: <breaking changes>
- **Security**: <considerations>

## Success Criteria
- [ ] `cargo build` exits 0
- [ ] `cargo test` exits 0
- [ ] PARITY.md §XIV updated
- [ ] Manual test: <verification command>
- [ ] PR opened against master
```

### Branch Naming

```
feat/A1-auth-trait-combinators
feat/B1-tool-tier-approval-gate
feat/C2-tree-sitter-codemap
feat/D1-agent-arena
etc.
```

### PR Description Template

```markdown
## Summary
Brief description of what this PR implements.

## Plan
See [docs/pr-plans/<ID>-<name>.md](docs/pr-plans/<ID>-<name>.md) for full research, alternatives, and implementation details.

## Changes
- Added: ...
- Modified: ...
- Removed: ...

## Testing
- [ ] `cargo build` passes
- [ ] `cargo test` passes
- [ ] Manual verification: <command>

## References
- Source: <reference repo URL>
- PARITY.md: §<section> row <feature>
```

---

## 🎛️ Control Panel

### Start from Specific Phase
To start from Phase 2 (skip completed Phase 1 features):
```
Skip Phase 1 implementation. Start with Phase 2 feature A6.
```

### Skip Specific Feature
```
Skip feature <ID>. Mark as deferred in backlog with reason: <reason>.
```

### Change Order
```
Move feature <ID-A> before <ID-B> in the queue.
```

### Emergency Stop
```
STOP: Do not spawn any more agents. Report current status.
```

---

## 📊 Progress Tracking

Track in `docs/PR_BACKLOG.md`:

| Status | Meaning |
|--------|---------|
| 🔜 Pending | Not started |
| 🏗️ In Progress | Agent working on it |
| ✅ Done | Merged to master |
| ⏸️ Deferred | Explicitly deferred with reason |
| ❌ Skipped | Not applicable (sandboxed, etc.) |
| 🔀 PR #N | Open PR |
| ⚠️ Partial | Partially implemented |

---

## 🚨 Error Handling

If an implementation subagent fails:
1. Log the error
2. Restart with same task (max 3 retries)
3. If 3 retries fail, mark as `deferred` with error summary
4. Move to next feature

If `cargo build` fails:
1. Capture error output
2. Add fix commits to the branch
3. Retry build
4. If cannot fix, defer with error summary

If `cargo test` fails:
1. Run specific failing test with output
2. Fix test or update test expectations
3. If test is flaky, add retry logic
4. If cannot fix, defer with error summary

---

## 🏁 Success Conditions

The goal is **COMPLETE** when:

1. **P0 Complete**: All 6 Phase 1 features (A1-A5, B1) are merged
2. **P1 Mostly Done**: ≥80% of Phase 2 features are merged or deferred
3. **Backlog Updated**: Every row in `docs/PR_BACKLOG.md` has a status
4. **PARITY.md Current**: §XIV accurately reflects implemented vs missing

The goal is **PARTIAL** if:
- Some features remain unimplemented
- Report which features remain and why

The goal is **STUCK** if:
- Agent repeatedly fails on same feature
- Network/build issues persist
- Requires human intervention
