# Plan — Face `!` bash mode wire

## Summary
- **You asked:** Claude Code–style `!` bash mode for Face (run shell immediately, inject into transcript; chat stays usable).
- **What is going on:** Face already has bang chrome (`PromptInputMode::Bash` → `SendBashCommand` → ACP `session/prompt` with `PromptBlockMeta.bash_command`). next-code `pager_agent::prompt` ignored that meta and always started a model turn.
- **We recommend / did:** Keep Face UI stock; wire `pager_agent` to honor `bash_command`, run shell locally, emit execute tool updates with `_meta.bash_mode: true`, `StopReason::EndTurn` (no model turn).
- **Out of scope:** sticky prompt, `/memory` typed, permission cards; Claude `respondToBashCommands` auto-reply (optional follow-up).
- **Status:** Implemented in this branch.

## Evidence
1. Face bang mode — `crates/xai-grok-pager/src/app/agent_view/prompt.rs`, `dispatch/prompt.rs` `dispatch_send_bash_command`, `effects` `SendBashCommand` + `PromptBlockMeta`
2. Face paint — `acp/tracker.rs` reads `meta.bash_mode` on Execute tool calls
3. Gap — `src/cli/pager_agent.rs` `prompt()` previously only `prompt_text` → `Request::Message`
4. Claude UX — leading `!` / bash mode runs shell and adds output to session ([interactive mode](https://code.claude.com/docs/en/interactive-mode))

## Call graph

```text
! (empty) → PromptInputMode::Bash
Enter → Action::SendBashCommand(cmd)
  → Effect::SendBashCommand
  → session/prompt + PromptBlockMeta.bash_command
  → pager_agent::prompt → face_bash::run_shell_command
  → ACP ToolCall Execute { meta.bash_mode: true }
  → Face ExecuteToolCallBlock.bash_mode render
```

## Files
- `src/cli/face_bash.rs` — meta parse, shell runner, ACP payload helpers + tests
- `src/cli/pager_agent.rs` — bash branch in `prompt` + `handle_bash_mode_prompt`
- `src/cli/mod.rs` — register module
- `docs/plans/PLAN-20260724-face-bang-bash-mode.md` — this file

## Smoke
1. Build/install from this worktree (unique hash if needed).
2. Open Face; type `!` alone → bash mode chrome (`! ` prefix).
3. `git status` / `echo hello` → execute `(user)` block with output; prompt returns to chat.
4. Esc/Backspace on empty bash prompt exits bash mode.
5. Normal chat without `!` still goes to the model.
