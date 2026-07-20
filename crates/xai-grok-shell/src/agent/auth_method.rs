//! Stub of upstream `xai-grok-shell::agent::auth_method`.

pub const XAI_API_KEY_METHOD_ID: &str = "xai-api-key";
pub const PREFERRED_API_KEY_UNAVAILABLE: &str = "preferred-api-key-unavailable";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMethodKind {
    #[default]
    ApiKey,
    Oauth,
    Managed,
}
