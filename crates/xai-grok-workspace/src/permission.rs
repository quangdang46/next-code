use agent_client_protocol::PermissionOption;

/// The option id used for the global "always-approve" (YOLO) permission row.
pub const ENABLE_ALWAYS_APPROVE_OPTION_ID: &str = "enable-always-approve";

/// Returns `true` when the given option is the special enable-always-approve
/// (YOLO) row.
pub fn is_enable_always_approve_option(option: &PermissionOption) -> bool {
    option.id() == ENABLE_ALWAYS_APPROVE_OPTION_ID
}
