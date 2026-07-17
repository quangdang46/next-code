//! Telemetry phone-home was removed for the open-source next-code fork.
//! These no-op stubs keep existing call sites compiling without contacting any server.

#![allow(unused_variables, dead_code, clippy::too_many_arguments)]

use serde_json::Value;

pub use next_code_usage_types::{ErrorCategory, SessionEndReason};

#[derive(Debug, Clone)]
pub struct DiscoveryTelemetry<'a> {
    pub request_id: &'a str,
    pub phase: &'a str,
    pub category: Option<&'a str>,
    pub selected_tool: Option<&'a str>,
    pub outcome: &'a str,
    pub failure_reason: Option<&'a str>,
    pub http_status: Option<u16>,
    pub latency_ms: u64,
    pub response_bytes: Option<u64>,
    pub result_count: Option<u32>,
    pub query_present: bool,
    pub reason_present: bool,
    pub benchmark_run: bool,
    pub endpoint: &'a str,
}

pub fn is_enabled() -> bool {
    false
}
pub fn content_sharing_enabled() -> bool {
    false
}
pub fn set_content_sharing_enabled(_enabled: bool) -> bool {
    false
}
pub fn record_setup_step_once(_step: &'static str) {}
pub fn record_feedback(_text: &str) {}
pub fn record_discovery_event(_data: DiscoveryTelemetry<'_>) {}
pub fn record_command_family(_command: &str) {}
pub fn record_install_if_first_run() {}
pub fn record_upgrade_if_needed() {}
pub fn record_provider_selected(_provider: &str) {}
pub fn record_auth_started(_provider: &str, _method: &str) {}
pub fn record_auth_failed(_provider: &str, _method: &str) {}
pub fn record_auth_failed_reason(_provider: &str, _method: &str, _reason: &str) {}
pub fn record_auth_cancelled(_provider: &str, _method: &str) {}
pub fn record_auth_surface_blocked(_provider: &str, _method: &str) {}
pub fn record_auth_surface_blocked_reason(_provider: &str, _method: &str, _reason: &str) {}
pub fn record_auth_success(_provider: &str, _method: &str) {}
pub fn begin_session(_provider: &str, _model: &str) {}
pub fn begin_session_with_parent(
    _provider: &str,
    _model: &str,
    _parent_session_id: Option<String>,
    _resumed_session: bool,
) {
}
pub fn begin_resumed_session(_provider: &str, _model: &str) {}
pub fn record_turn() {}
pub fn record_assistant_response() {}
pub fn record_memory_injected(_count: usize, _age_ms: u64) {}
pub fn record_tool_call() {}
pub fn record_tool_failure() {}
pub fn record_connection_type(_connection: &str) {}
pub fn record_token_usage(
    _input_tokens: u64,
    _output_tokens: u64,
    _cache_read_input_tokens: Option<u64>,
    _cache_creation_input_tokens: Option<u64>,
) {
}
pub fn record_error(_category: ErrorCategory) {}
pub fn record_provider_switch() {}
pub fn record_model_switch() {}
pub fn record_user_cancelled() {}
pub fn record_tool_execution(_name: &str, _input: &Value, _succeeded: bool, _latency_ms: u64) {}
pub fn end_session(_provider_end: &str, _model_end: &str) {}
pub fn end_session_with_reason(
    _provider_end: &str,
    _model_end: &str,
    _reason: SessionEndReason,
) {
}
pub fn record_crash(_provider_end: &str, _model_end: &str, _reason: SessionEndReason) {}
pub fn current_provider_model() -> Option<(String, String)> {
    None
}
