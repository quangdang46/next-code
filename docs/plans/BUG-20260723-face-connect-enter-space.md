# BUG — `/connect` Enter inserts space instead of opening picker

## Summary

Bare `/connect` + Enter completes the optional `provider` arg (trailing space / inline dropdown) instead of submitting. Stock `/model` already uses centered ArgPicker; `/connect` did not.

## Root cause (verified)

`SuggestionRow::from_command` appends a trailing space when `takes_args && !command_opens_centered_arg_picker`. That gate only listed `"model" | "m"`. `/connect` has `takes_args=true` + `arg_placeholder="provider"` + empty `suggest_args` → Enter chains into `/connect ` and steals submit.

## Fix (copy `/model` seam; delete dual path)

1. Add `connect` (+ embed `login`) to `command_opens_centered_arg_picker`.
2. Enter on bare `/connect ` → `OpenConnectPicker` (same as `/model ` → `OpenModelPicker`).
3. `dispatch_open_connect_picker` loads items via `build_connect_family_items()` (not empty `suggest_args`).
4. Empty `suggest_args` returns `None` — no inline provider dump on bare command.

## Status

Implementing on `pr-face-connect-open-model` (PR #72).
