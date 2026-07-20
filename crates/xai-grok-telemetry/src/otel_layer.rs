#[derive(Debug, Clone, Default)]
pub struct OtelExporterConfig {
    pub traces_url: String,
    pub extra_headers: Vec<(String, String)>,
    pub export_interval: Option<std::time::Duration>,
    pub timeout: Option<std::time::Duration>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OtelLayerConfig {
    pub token_header_value: String,
    pub alpha_test_key: Option<String>,
    pub exporter: OtelExporterConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct OtelClientInfo {
    pub client_name: &'static str,
    pub client_version: &'static str,
    pub service_version: &'static str,
    pub app_entrypoint: &'static str,
}

/// No-op layer. Identity avoids an unused subscriber type parameter that
/// would force turbofish at call sites (real crate returns `impl Layer<S>`).
pub fn build_otel_layer(
    _client: OtelClientInfo,
    _config: OtelLayerConfig,
) -> tracing_subscriber::layer::Identity {
    tracing_subscriber::layer::Identity::new()
}

pub fn shutdown_otel() {}

pub fn init() {}
