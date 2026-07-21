//! Stub of upstream `xai-grok-shell::agent::auth_method`.

use agent_client_protocol as acp;

pub const PREFERRED_API_KEY_UNAVAILABLE: &str = "preferred_api_key_unavailable";
pub const XAI_API_KEY_METHOD_ID: &str = "xai.api_key";
pub const CACHED_TOKEN_AUTH_METHOD_ID: &str = "xai.cached_token";
pub const GROK_COM_METHOD_ID: &str = "xai.grok_com";
pub const OIDC_METHOD_ID: &str = "xai.oidc";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreferredAuthMethod {
    ApiKey,
    Oidc,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMethodKind {
    XaiApiKey,
    CachedToken,
    GrokCom,
    Oidc,
    #[default]
    Unknown,
}

impl AuthMethodKind {
    pub fn from_id(id: &acp::AuthMethodId) -> Self {
        match id.0.as_ref() {
            XAI_API_KEY_METHOD_ID => Self::XaiApiKey,
            CACHED_TOKEN_AUTH_METHOD_ID => Self::CachedToken,
            GROK_COM_METHOD_ID => Self::GrokCom,
            OIDC_METHOD_ID => Self::Oidc,
            _ => Self::Unknown,
        }
    }

    pub fn is_api_key(self) -> bool {
        matches!(self, Self::XaiApiKey)
    }

    pub fn is_session_based(self) -> bool {
        matches!(self, Self::CachedToken | Self::GrokCom | Self::Oidc)
    }

    pub fn needs_interactive_login(self) -> bool {
        matches!(self, Self::GrokCom | Self::Oidc)
    }
}

/// Inputs for [`build_auth_methods`] — shape matches upstream; stub ignores most.
pub struct AuthMethodsBuildInputs<'a> {
    pub has_external_api_key: bool,
    pub has_cached_token: bool,
    pub has_enterprise_oidc: bool,
    pub enterprise_oidc_issuer: Option<&'a str>,
    pub login_label: Option<&'a str>,
    pub has_auth_provider_command: bool,
    pub preferred_method: Option<PreferredAuthMethod>,
}

pub struct BuiltAuthMethods {
    pub methods: Vec<acp::AuthMethod>,
    pub default_auth_method_id: Option<acp::AuthMethodId>,
}

pub fn build_auth_methods(_inputs: AuthMethodsBuildInputs<'_>) -> BuiltAuthMethods {
    BuiltAuthMethods {
        methods: vec![],
        default_auth_method_id: None,
    }
}
