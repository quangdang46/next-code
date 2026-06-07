//! Lightweight, dependency-minimal secret redaction.
//!
//! This crate holds the single canonical `redact_secrets` sanitizer so that
//! callers which must not pull in the heavier `jcode-secrets` stack (`age`,
//! `keyring`, ...) — notably `jcode-logging` — can still scrub secrets from
//! output. `jcode-secrets` re-exports [`redact_secrets`] for backward
//! compatibility.
//!
//! Patterns aggregated from:
//! - OpenAI codex `codex-rs/secrets/src/sanitizer.rs`
//! - oh-my-codex `src/auth/redact.ts`
//! - jcode's former `jcode-app-core/src/export.rs` redact_secrets()
//!
//! All patterns use [`LazyLock<Regex>`] for once-per-process compilation.

use regex::Regex;
use std::sync::LazyLock;

// ─── Regex Patterns (lazy, compiled once) ───────────────────────────────────

/// OpenAI / Anthropic style keys: `sk-...`, `sk-ant-...`
static SK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bsk-[A-Za-z0-9_\-\.]{20,}").unwrap()
});

/// AWS access key IDs: `AKIA...`
static AWS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap()
});

/// HTTP Bearer tokens (keep the "Bearer " prefix)
static BEARER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(Bearer\s+)[A-Za-z0-9._\-]{16,}").unwrap()
});

/// Generic secret/value assignments in code/config.
///
/// `[^\s"'\[]{8,}` excludes values starting with `[` so that already-redacted
/// placeholders (`[REDACTED:*]`) are not re-matched.
static SECRET_ASSIGNMENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\b(api[_-]?key|token|secret|password)\b(\s*[:=]\s*)(["']?)[^\s"'\[]{8,}"#).unwrap()
});

/// GitHub tokens: `ghp_...`, `gho_...`, `ghs_...`, `ghr_...`, `ghu_...`
static GITHUB_TOKEN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bgh[opsru]_[A-Za-z0-9]{20,}").unwrap()
});

/// z.ai / ZHIPU style tokens: `{32 hex}.{12+ alphanum}`
static ZAI_TOKEN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[a-f0-9]{32}\.[A-Za-z0-9]{12,}").unwrap()
});

// ─── Known Secret Env Vars ──────────────────────────────────────────────────

/// Known environment variable names that carry API keys or secrets.
/// Matched case-insensitively.
const KNOWN_SECRET_ENV_VARS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "OPENAI_COMPAT_API_KEY",
    "OPENROUTER_API_KEY",
    "ZHIPU_API_KEY",
    "ZAI_API_KEY",
    "COHERE_API_KEY",
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "MINIMAX_API_KEY",
    "XAI_API_KEY",
    "DEEPSEEK_API_KEY",
    "FIREWORKS_API_KEY",
    "GROQ_API_KEY",
    "MISTRAL_API_KEY",
    "OPENCODE_API_KEY",
    "OPENCODE_GO_API_KEY",
    "TOGETHER_API_KEY",
    "PERPLEXITY_API_KEY",
    "CEREBRAS_API_KEY",
    "NVIDIA_API_KEY",
    "AZURE_OPENAI_API_KEY",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "ANTHROPIC_AUTH_TOKEN",
];

/// Matches known secret env-var assignments: `KEY=value`, `KEY = value`
static ENV_VAR_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let names = KNOWN_SECRET_ENV_VARS.join("|");
    Regex::new(&format!(
        r#"(?i)\b({})(\s*=\s*)([^\r\n,'"\s]+)"#,
        names
    ))
    .unwrap()
});

// ─── Public API ──────────────────────────────────────────────────────────────

/// Redact known secret patterns from `input`, returning a sanitized string.
///
/// The original string is not modified — a new `String` is returned.
///
/// # Pattern priority
///
/// 1. `sk-*` API keys (OpenAI, Anthropic, etc.)
/// 2. GitHub tokens (`gh[opsru]_*`)
/// 3. AWS access key IDs (`AKIA*`)
/// 4. z.ai style tokens (`{hex}.{alphanum}`)
/// 5. Bearer tokens in Authorization headers
/// 6. Generic `api_key`, `token`, `secret`, `password` assignments
/// 7. Known provider environment variable assignments
///
/// # Example
///
/// ```
/// use jcode_redact::redact_secrets;
///
/// let safe = redact_secrets("My token is sk-abc123DEF456ghi789jkl012");
/// assert!(safe.contains("[REDACTED:sk]"));
/// ```
pub fn redact_secrets(input: &str) -> String {
    let mut output = input.to_string();

    // 1. OpenAI / Anthropic api keys
    output = SK_PATTERN
        .replace_all(&output, "[REDACTED:sk]")
        .into_owned();

    // 2. GitHub tokens
    output = GITHUB_TOKEN_PATTERN
        .replace_all(&output, "[REDACTED:github]")
        .into_owned();

    // 3. AWS access keys
    output = AWS_PATTERN
        .replace_all(&output, "[REDACTED:aws]")
        .into_owned();

    // 4. z.ai style tokens
    output = ZAI_TOKEN_PATTERN
        .replace_all(&output, "[REDACTED:token]")
        .into_owned();

    // 5. Bearer tokens (preserve the "Bearer " prefix)
    output = BEARER_PATTERN
        .replace_all(&output, "${1}[REDACTED]")
        .into_owned();

    // 6. Generic secret assignments
    output = SECRET_ASSIGNMENT_PATTERN
        .replace_all(&output, "${1}${2}[REDACTED]")
        .into_owned();

    // 7. Known env-var assignments
    output = ENV_VAR_PATTERN
        .replace_all(&output, "${1}${2}[REDACTED:env]")
        .into_owned();

    output
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_openai_key() {
        let result = redact_secrets("sk-abc123DEF456ghi789jkl012");
        assert!(result.contains("[REDACTED:sk]"));
        assert!(!result.contains("sk-abc123DEF456ghi789jkl012"));
    }

    #[test]
    fn redacts_anthropic_key() {
        let result = redact_secrets("sk-ant-api03-abcdefghijklmnopqrstuvwxyz123456");
        assert!(result.contains("[REDACTED:sk]"));
    }

    #[test]
    fn redacts_github_token() {
        let result = redact_secrets("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456");
        assert!(result.contains("[REDACTED:github]"));
    }

    #[test]
    fn redacts_bearer_token() {
        let result =
            redact_secrets("Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
        assert!(result.contains("Bearer [REDACTED]"));
        assert!(!result.contains("eyJhbGci"));
    }

    #[test]
    fn redacts_env_var_assignment() {
        let result = redact_secrets("OPENAI_API_KEY=sk-abc123def456");
        assert!(result.contains("[REDACTED:env]"));
        assert!(result.contains("OPENAI_API_KEY="));
    }

    #[test]
    fn redacts_aws_key() {
        let result = redact_secrets("AKIAIOSFODNN7EXAMPLE");
        assert!(result.contains("[REDACTED:aws]"));
    }

    #[test]
    fn preserves_safe_text() {
        let input = "Hello, this is a normal message with no secrets.";
        assert_eq!(redact_secrets(input), input);
    }

    #[test]
    fn redacts_multiple_secrets() {
        let input = concat!(
            "OPENAI_API_KEY=sk-abc123DEF456ghi789jkl012, ",
            "GITHUB_TOKEN=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456, ",
            "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
        );
        let result = redact_secrets(input);
        assert!(!result.contains("sk-abc123DEF456ghi789jkl012"));
        assert!(!result.contains("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456"));
        assert!(!result.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    }

    #[test]
    fn redacts_zai_token() {
        let result =
            redact_secrets("token=b8c37e33defde51cf91e1e03e51657da.AqkKii0K0VqLpQRnP");
        assert!(result.contains("[REDACTED:token]"));
    }

    #[test]
    fn redacts_gho_token() {
        let result = redact_secrets("gho_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456");
        assert!(result.contains("[REDACTED:github]"));
    }

    #[test]
    fn redacts_ghs_token() {
        let result = redact_secrets("ghs_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456");
        assert!(result.contains("[REDACTED:github]"));
    }
}
