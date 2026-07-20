//! Façade stub of upstream `xai-grok-shell::sampling::error` — rate-limit /
//! usage-error copy and ACP error-data helpers. Constants are copied
//! verbatim (user-facing strings, cheap to keep faithful); helper bodies
//! are simplified but behavior-preserving for the free-usage/HTTP-status
//! extraction paths since those are pure functions with no I/O.

use agent_client_protocol as acp;

use crate::extensions::notification::PromptUsage;

pub const RATE_LIMITED_ERROR_CODE: i32 = -32003;

pub const RATE_LIMITED_USER_MESSAGE_OAUTH: &str =
    "You\u{2019}ve hit the rate limit for your plan. Upgrade your account or try again later.";

pub const RATE_LIMITED_USER_MESSAGE_API_KEY: &str = "You\u{2019}ve hit your team\u{2019}s API rate limit. Ask a team admin to purchase more credits for higher limits, or try again later. See https://docs.x.ai/developers/rate-limits#rate-limit-tiers";

/// Well-known free-usage exhaustion code CCP returns on HTTP 429.
pub const FREE_USAGE_EXHAUSTED_ERROR_CODE: &str = "subscription:free-usage-exhausted";

pub const FREE_USAGE_USER_MESSAGE: &str =
    "You\u{2019}ve used up your free usage. Upgrade your plan to keep going.";

pub fn is_free_usage_exhausted_error(detail: &str) -> bool {
    detail.contains(FREE_USAGE_EXHAUSTED_ERROR_CODE)
}

/// User-facing text for an ACP -32003 rate-limit error.
pub fn format_rate_limited_user_message(server_detail: Option<&str>, is_api_key_auth: bool) -> String {
    if server_detail.is_some_and(is_free_usage_exhausted_error) {
        return FREE_USAGE_USER_MESSAGE.to_string();
    }
    if is_api_key_auth {
        RATE_LIMITED_USER_MESSAGE_API_KEY.to_string()
    } else {
        RATE_LIMITED_USER_MESSAGE_OAUTH.to_string()
    }
}

pub fn http_status_from_error(err: &acp::Error) -> Option<u16> {
    err.data
        .as_ref()?
        .get("http_status")?
        .as_u64()
        .map(|s| s as u16)
}

pub fn error_detail_from_data(data: &serde_json::Value) -> Option<String> {
    data.get("detail")
        .or_else(|| data.get("message"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

const PROMPT_USAGE_DATA_KEY: &str = "promptUsage";

pub fn prompt_usage_from_error(err: &acp::Error) -> Option<PromptUsage> {
    let data = err.data.as_ref()?;
    let raw = data.get(PROMPT_USAGE_DATA_KEY)?;
    serde_json::from_value(raw.clone()).ok()
}
