//! Stub of upstream `xai-file-utils` `trace_context.rs`.
//!
//! Upstream links a tracing span to a distributed trace context propagated
//! via `_meta.traceparent` (OpenTelemetry `TraceContextPropagator`). This
//! facade has no OpenTelemetry dependency, so it returns a plain span with no
//! parent linkage — same call shape, no distributed tracing behavior.

/// Create a tracing span for ACP dispatch. Upstream parents it to
/// `_meta.traceparent` via OpenTelemetry; this stub returns a plain
/// (unparented) span since no OpenTelemetry propagator is vendored here.
pub fn span_from_meta_traceparent(
    _meta: &serde_json::Map<String, serde_json::Value>,
) -> tracing::Span {
    tracing::info_span!("acp_dispatch")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_span_regardless_of_meta() {
        let mut meta = serde_json::Map::new();
        meta.insert(
            "traceparent".into(),
            serde_json::Value::String(
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".into(),
            ),
        );
        let _span = span_from_meta_traceparent(&meta);
        let _span_empty = span_from_meta_traceparent(&serde_json::Map::new());
    }
}
