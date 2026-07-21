pub const ACP_UPDATE_TARGET: &str = "acp_update";
pub const ACP_UPDATE_PAYLOAD_TARGET: &str = "acp_update_payload";
pub const RMCP_SSE_NOISE_TARGET: &str = "rmcp::transport::common::client_side_sse";

pub fn write(_msg: &str) {}

pub fn install_firehose<S>(registry: S, _role: &str)
where
    S: tracing::Subscriber + Send + Sync + 'static,
{
    let _ = tracing::subscriber::set_global_default(registry);
}

pub fn flush() {}
