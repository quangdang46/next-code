//! Minimal workspace permission shim for Face render.

pub mod permission {
    use agent_client_protocol as acp;

    pub const ENABLE_ALWAYS_APPROVE_OPTION_ID: &str = "enable-always-approve";

    pub fn is_enable_always_approve_option(opt: &acp::PermissionOption) -> bool {
        opt.option_id.0.as_ref() == ENABLE_ALWAYS_APPROVE_OPTION_ID
    }
}
