# Plan — Face permission cards per tool

## Summary
Face already has `permission_view` with bash command chrome and MCP args. The next-code bridge (`face_permission.rs`) always sent `ToolKind::Other` + generic `Allow {tool}?` and option labels without "edit", so edit cards never got path titles and neither bash nor edit got Claude-style context (cwd / risk / diff preview).

## Approach
1. **Bridge (`face_permission.rs`)**: classify tool → `ToolKind`, tool-specific titles + option labels, normalize raw_input, attach simple bash highlight meta when command is known.
2. **Display (`acp_handler/permissions.rs`)**: enrich `build_permission_display` — bash cwd/risk lines; edit/write path + unified-diff-ish preview in description.
3. **Daemon (`bash.rs`)**: include `tool_input` on PermissionRequested (both Prompt sites).
4. Keep allow-once / always / allow-all / reject. No Shift+Tab mode cycle.

## Status
Implementing (parallel agent brief).
