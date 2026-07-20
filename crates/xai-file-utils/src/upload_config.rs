//! Minimal subset of upstream `upload_config.rs` — only the types that
//! `gcs::upload_bytes` / pager `trace_cmd` need for call-shape fidelity.
//!
//! Upstream also ships skip-dir tables, dedup metadata, and archive schema
//! constants here; those are not pager import sites for PR6 and stay out.

/// Method for uploading to object storage (shape matches upstream).
#[derive(Clone, Debug)]
pub enum UploadMethod {
    Direct {
        service_account_key: Option<String>,
    },
    Proxy {
        proxy_base_url: String,
        user_token: String,
        deployment_key: Option<String>,
        alpha_test_key: Option<String>,
    },
    S3 {
        bucket: String,
        region: String,
        credentials_file: Option<String>,
        credentials_content: Option<String>,
        endpoint_url: Option<String>,
    },
}

/// Configuration for object-storage export (shape matches upstream).
#[derive(Clone, Debug)]
pub struct TraceExportConfig {
    pub bucket_url: Option<String>,
    pub service_account_key: Option<String>,
    pub upload_method: UploadMethod,
    pub prefix_dir: Option<String>,
    pub gcs_prefix: Option<String>,
    pub absolute_paths: bool,
    pub archive_name_override: Option<String>,
}
