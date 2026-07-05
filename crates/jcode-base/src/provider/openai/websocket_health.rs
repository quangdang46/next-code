//! Stub: WebSocket health-check items formerly in the base crate.
//!
//! The real implementations live in `jcode-provider-openai` /
//! `jcode-provider-openai-runtime`. These local stubs satisfy the `use`
//! imports in `openai.rs`; the items are never referenced in code body.

pub(crate) const WEBSOCKET_COMPLETION_TIMEOUT_SECS: u64 = 300;
pub(crate) const WEBSOCKET_FALLBACK_NOTICE: &str = "websocket-notice-fallback";
pub(crate) const WEBSOCKET_FIRST_EVENT_TIMEOUT_SECS: u64 = 120;
#[cfg(test)]
pub(crate) const WEBSOCKET_MODEL_COOLDOWN_BASE_SECS: u64 = 30;
#[cfg(test)]
pub(crate) const WEBSOCKET_MODEL_COOLDOWN_MAX_SECS: u64 = 600;

#[allow(dead_code)]
pub(crate) fn classify_websocket_fallback_reason(msg: &str) -> String {
    msg.to_string()
}

#[allow(dead_code)]
pub(crate) fn is_stream_activity_event(_event: &jcode_message_types::StreamEvent) -> bool {
    false
}

#[allow(dead_code)]
pub(crate) fn is_websocket_activity_payload(_payload: &str) -> bool {
    false
}

#[allow(dead_code)]
pub(crate) fn is_websocket_fallback_notice(_event: &jcode_message_types::StreamEvent) -> bool {
    false
}

#[allow(dead_code)]
pub(crate) fn is_websocket_first_activity_payload(_payload: &str) -> bool {
    false
}

#[allow(dead_code)]
pub(crate) fn record_websocket_fallback(_reason: &str) {}

#[allow(dead_code)]
pub(crate) fn record_websocket_success() {}

#[allow(dead_code)]
pub(crate) fn summarize_websocket_fallback_reason(_reason: &str) -> String {
    String::new()
}

#[allow(dead_code)]
pub(crate) fn websocket_activity_timeout_kind(_model: &str, _event: &jcode_message_types::StreamEvent) -> &'static str {
    "completion"
}

#[allow(dead_code)]
pub(crate) fn websocket_cooldown_remaining(_model: &str) -> std::time::Duration {
    std::time::Duration::ZERO
}

#[allow(dead_code)]
pub(crate) fn websocket_next_activity_timeout_secs_with_completion(
    _model: &str,
    _completion_timeout_secs: Option<u64>,
) -> u64 {
    120
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) struct WebsocketFallbackReason {
    pub reason: String,
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn clear_websocket_cooldown(_model: &str) {}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn normalize_transport_model(_model: &str) -> String {
    String::new()
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn set_websocket_cooldown(_model: &str, _now: std::time::Instant) {}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn websocket_cooldown_for_streak(_streak: u32) -> std::time::Duration {
    std::time::Duration::ZERO
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn websocket_next_activity_timeout_secs(_model: &str) -> u64 {
    120
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn websocket_remaining_timeout_secs(_model: &str) -> Option<u64> {
    Some(120)
}
