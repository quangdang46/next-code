#[derive(Debug, Clone, Default)]
pub struct ExternalClientInfo {
    pub service_version: String,
    pub client_version: String,
    pub app_entrypoint: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExternalOtelConfig {
    pub client: ExternalClientInfo,
}
