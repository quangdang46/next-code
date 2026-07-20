//! Stub of upstream `xai-grok-shell::session::repo_changes` — upstream
//! re-exports these from `xai-file-utils` (not vendored in this PR); this
//! defines minimal self-contained stand-ins for the two symbols the
//! future pager imports.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UploadMethod {
    #[default]
    Inline,
    Reference,
}

#[derive(Debug, Clone, Default)]
pub struct TraceExportConfig {
    pub upload_method: UploadMethod,
    pub max_inline_bytes: usize,
}
