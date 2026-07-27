---
name: gjc-dogfood
description: Dogfood GJC on next-code — implement issue → PR with CI watch
tags: [dogfood, github, next-code]
---

# gjc-dogfood

Dogfood GJC workflow for `quangdang46/next-code`.  
Follows the same loop as Yeachan-Heo's dogfood on gajae-code/oh-my-codex.

## Workflow

1. Read the issue or task description.
2. **Research first** — read AGENTS.md, CLAUDE.md, relevant crate docs before editing.
3. Implement:
   - Create a worktree branch: `fix/<issue-slug>-<short>`
   - Make minimal changes, prefer `cargo check` for fast iteration.
   - Update `cargo.lock` if deps change.
   - Run focused tests: `cargo test -p <affected-crate>`.
4. Commit + push:
   - `git add . && git commit -m "fix(scope): description"`
   - `git push origin HEAD`
5. Create PR via `gh pr create`:
   - Title: concise fix description
   - Body: Summary + Changes + Verification + `Closes #N`
   - Flags: draft unless CI is expected green
6. Wait for CI: `gh run watch` or similar.
7. If CI green and review passes → comment `MERGE_READY` summary.
8. **Do NOT merge** unless user explicitly says so.

## Constraints

- Always use worktree (`gjc --worktree <branch>`), never modify main checkout directly.
- Always reference AGENTS.md for project conventions.
- Prefer focused `cargo check` over full build when iterating.
- PR must be **draft** if this is a user-test — only promote when instructed.
