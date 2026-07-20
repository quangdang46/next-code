//! Stub of upstream `xai-file-utils` `gcs.rs`.
//!
//! Upstream dispatches to direct GCS, cli-chat-proxy, or S3 backends
//! (`gcloud-storage`, `aws-sdk-s3`, refresh-aware auth credentials). None of
//! that is vendored here — next-code has no upload backend at all. This stub
//! keeps the **call shape** the pager expects (`upload_bytes(config, path,
//! bytes, content_type)`) and always fails, so a caller that accidentally
//! reaches it gets a clear error instead of a silent success.
//!
//! Review note (vs grok-build): pager `trace_cmd.rs` calls
//! `xai_file_utils::gcs::upload_bytes(config, object_path, archive, …)` where
//! `config: &TraceExportConfig`. The previous 3-arg stub would not compile
//! against the real pager — fixed here to match upstream's generic signature.

use crate::upload_config::{TraceExportConfig, UploadMethod};

/// Threshold for switching to multipart upload (50 MB) — kept for call-site
/// constants that mention it; unused by this stub.
pub const MULTIPART_UPLOAD_THRESHOLD: u64 = 50 * 1024 * 1024;

/// Storage configuration that provides bucket URL and upload method.
///
/// Upstream also has optional `proxy_credentials` / `proxy_attribution` /
/// `proxy_http_client` default methods that pull in `xai-grok-auth` +
/// `reqwest`. Those are omitted: the stub never reads them, and pager only
/// needs the two required methods for `TraceExportConfig: StorageConfig`.
pub trait StorageConfig {
    fn bucket_url(&self) -> &str;
    fn upload_method(&self) -> &UploadMethod;
}

impl StorageConfig for TraceExportConfig {
    fn bucket_url(&self) -> &str {
        self.bucket_url.as_deref().unwrap_or("gs://placeholder")
    }

    fn upload_method(&self) -> &UploadMethod {
        &self.upload_method
    }
}

/// Always fails: no real GCS/S3/proxy upload backend is vendored in this
/// facade crate. Signature matches upstream
/// `upload_bytes<C: StorageConfig>(config, object_path, content, content_type)`.
pub async fn upload_bytes<C: StorageConfig>(
    _config: &C,
    _object_path: &str,
    _content: &[u8],
    _content_type: &str,
) -> anyhow::Result<String> {
    anyhow::bail!(
        "xai-file-utils::gcs::upload_bytes is a compile stub (no GCS/S3 backend vendored) \
         — uploads are not supported in this build"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> TraceExportConfig {
        TraceExportConfig {
            bucket_url: Some("gs://test".into()),
            service_account_key: None,
            upload_method: UploadMethod::Direct {
                service_account_key: None,
            },
            prefix_dir: None,
            gcs_prefix: None,
            absolute_paths: false,
            archive_name_override: None,
        }
    }

    #[tokio::test]
    async fn upload_bytes_always_errors() {
        let config = sample_config();
        let result = upload_bytes(&config, "path/to/object", b"data", "application/octet-stream")
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn trace_export_config_implements_storage_config() {
        let config = sample_config();
        assert_eq!(config.bucket_url(), "gs://test");
        assert!(matches!(
            config.upload_method(),
            UploadMethod::Direct { .. }
        ));
    }
}
