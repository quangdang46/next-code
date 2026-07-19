/// Image source types for the agent-client protocol.

/// Represents the source of an image.
#[derive(Debug, Clone)]
pub enum ImageSource {
    /// Image sourced from a URL.
    Url(String),
    /// Image sourced from base64-encoded data.
    Base64(String),
    /// Image sourced from a local file path.
    File(String),
}
