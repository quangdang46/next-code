//! Stub of upstream `xai-file-utils` `gcs.rs`.
//!
//! Upstream dispatches to direct GCS, cli-chat-proxy, or S3 backends
//! (`gcloud-storage`, `aws-sdk-s3`, refresh-aware auth credentials). None of
//! that is vendored here — next-code has no upload backend at all. This stub
//! keeps the call shape callers expect and always fails, so a caller that
//! accidentally reaches it gets a clear error instead of a silent success.

/// Always fails: no real GCS/S3/proxy upload backend is vendored in this
/// facade crate.
pub async fn upload_bytes(
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

    #[tokio::test]
    async fn upload_bytes_always_errors() {
        let result = upload_bytes("path/to/object", b"data", "application/octet-stream").await;
        assert!(result.is_err());
    }
}
