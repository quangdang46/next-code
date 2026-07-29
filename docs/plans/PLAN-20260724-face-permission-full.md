# PLAN — Face permission full (cards + plan + DCG mode wire)

**Status:** implementing (combined PR)
**Risk:** medium — permission mode is security-sensitive; policy stays DCG

## Summary

One PR merges Face permission cards (#104) + Plan→execute (#106), then wires Face AlwaysApprove / YOLO and Shift+Tab PersistPermissionMode through to DCG `BypassPermissions` / mode updates (no longer Face-local auto-AllowOnce only).

## Mapping (chrome = Face, policy = DCG)

Shift+Tab Face subset: **Normal → Plan → Auto → Always-Approve → Normal**

| Face | Daemon / DCG |
|------|----------------|
| ask / default / Normal | Default |
| plan | Plan |
| auto | Auto |
| always-approve (YOLO) | BypassPermissions |
| accept-edits (wire only) | AcceptEdits |

AcceptEdits is parsed on the wire when sent; Face has no AcceptEdits Shift+Tab chrome arm yet (not natural to invent).

## Wire paths

1. `session/set_mode` → `face_mode_to_daemon_permission` → `Request::SetPermissionMode` → `CurrentModeUpdate`
2. Face `PersistPermissionMode` → ACP `x.ai/yolo_mode_changed` → agent `ext_notification` → same SetPermissionMode path (AlwaysApprove / Auto / ask)

## Supersedes

- #104 permission cards
- #106 Enter/Exit plan tools (kept over #99)
- #99 superseded by #106 tip
