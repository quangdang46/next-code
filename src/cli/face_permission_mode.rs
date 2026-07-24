//! Face chrome permission modes → DCG daemon wire strings.
//!
//! Policy = DCG (`dcg_bridge::Mode`). Chrome = Grok Face labels.
//! Do not replace DCG with Grok policy — only map Face ids/canonicals.

/// Map a Face ACP session mode id or PersistPermissionMode canonical onto a
/// daemon `Request::SetPermissionMode.mode` string accepted by
/// `client_lifecycle` / `dcg_bridge::apply_session_permission_mode`.
///
/// # Shift+Tab (Face chrome subset) → DCG
///
/// Face cycles **Normal → Plan → Auto → Always-Approve → Normal** (Auto
/// skipped when the auto gate is off). Mapping:
///
/// | Face chrome / canonical     | DCG mode            |
/// |----------------------------|---------------------|
/// | `ask` / `default` / Normal | `default`           |
/// | `plan`                     | `plan`              |
/// | `auto`                     | `auto`              |
/// | `always-approve` (YOLO)    | `bypass-permissions`|
/// | `accept-edits`             | `accept-edits`      |
/// | `dont-ask`                 | `dont-ask`          |
/// | `bypass-permissions`       | `bypass-permissions`|
///
/// AcceptEdits is supported on the wire when Face (or settings) sends that
/// canonical; Face Shift+Tab chrome does not currently include an
/// AcceptEdits arm (no stock Face banner/settings choice), so it is not in
/// the default cycle.
#[must_use]
pub(crate) fn face_mode_to_daemon_permission(mode_id_or_canonical: &str) -> &'static str {
    match mode_id_or_canonical.trim().to_ascii_lowercase().as_str() {
        "plan" => "plan",
        // Face "Ask" / Normal = prompt — DCG Default (NOT DontAsk).
        "ask" | "default" | "" => "default",
        "auto" => "auto",
        // Face AlwaysApprove / YOLO ≡ DCG BypassPermissions.
        "always-approve" | "yolo" | "bypass-permissions" | "bypass" => "bypass-permissions",
        "accept-edits" | "acceptedits" | "accept_edits" => "accept-edits",
        "dont-ask" | "dontask" | "dont_ask" => "dont-ask",
        // Unknown Face SessionMode ids fall through to Default (safe).
        _ => "default",
    }
}

/// Map Face `x.ai/yolo_mode_changed` payload fields to a daemon mode string.
///
/// Prefer explicit `permission_mode` when present; otherwise derive from the
/// bool flags Face always sends (`yolo_mode`, `auto_mode`).
#[must_use]
pub(crate) fn yolo_notification_to_daemon_permission(
    permission_mode: Option<&str>,
    yolo_mode: bool,
    auto_mode: bool,
) -> &'static str {
    if let Some(pm) = permission_mode.map(str::trim).filter(|s| !s.is_empty()) {
        return face_mode_to_daemon_permission(pm);
    }
    if yolo_mode {
        return "bypass-permissions";
    }
    if auto_mode {
        return "auto";
    }
    "default"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_approve_and_yolo_map_to_bypass() {
        assert_eq!(
            face_mode_to_daemon_permission("always-approve"),
            "bypass-permissions"
        );
        assert_eq!(face_mode_to_daemon_permission("yolo"), "bypass-permissions");
        assert_eq!(
            face_mode_to_daemon_permission("bypass-permissions"),
            "bypass-permissions"
        );
    }

    #[test]
    fn ask_maps_to_default_not_dont_ask() {
        assert_eq!(face_mode_to_daemon_permission("ask"), "default");
        assert_eq!(face_mode_to_daemon_permission("default"), "default");
        assert_eq!(face_mode_to_daemon_permission("dont-ask"), "dont-ask");
    }

    #[test]
    fn plan_auto_accept_edits() {
        assert_eq!(face_mode_to_daemon_permission("plan"), "plan");
        assert_eq!(face_mode_to_daemon_permission("auto"), "auto");
        assert_eq!(face_mode_to_daemon_permission("accept-edits"), "accept-edits");
    }

    #[test]
    fn yolo_notification_prefers_permission_mode() {
        assert_eq!(
            yolo_notification_to_daemon_permission(Some("always-approve"), false, false),
            "bypass-permissions"
        );
        assert_eq!(
            yolo_notification_to_daemon_permission(Some("auto"), true, false),
            "auto"
        );
        assert_eq!(
            yolo_notification_to_daemon_permission(None, true, false),
            "bypass-permissions"
        );
        assert_eq!(
            yolo_notification_to_daemon_permission(None, false, true),
            "auto"
        );
        assert_eq!(
            yolo_notification_to_daemon_permission(None, false, false),
            "default"
        );
    }

    #[test]
    fn shift_tab_face_subset_documented_mapping() {
        // Face cycle arms (see xai-grok-pager dispatch_cycle_mode_inner).
        let cycle = ["ask", "plan", "auto", "always-approve"];
        let expected = ["default", "plan", "auto", "bypass-permissions"];
        for (face, dcg) in cycle.iter().zip(expected.iter()) {
            assert_eq!(face_mode_to_daemon_permission(face), *dcg, "face={face}");
        }
    }
}
