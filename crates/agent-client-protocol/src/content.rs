/// Content types for the agent-client protocol.

/// Content block enum representing different message content types.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    /// Plain text content.
    Text(TextContent),
    /// An image content block.
    Image(ImageContent),
}

/// Text content block.
#[derive(Debug, Clone)]
pub struct TextContent {
    /// The text content.
    pub text: String,
}

impl TextContent {
    /// Create a new `TextContent` with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Content for images.
#[derive(Debug, Clone)]
pub struct ImageContent {
    /// The image data (base64-encoded).
    pub data: String,
    /// MIME type of the image.
    pub mime_type: String,
    /// Optional URI reference.
    pub uri: Option<String>,
    /// Optional metadata.
    pub meta: Option<String>,
}

impl ImageContent {
    /// Create a new `ImageContent` with the given base64-encoded data and MIME type.
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            mime_type: mime_type.into(),
            uri: None,
            meta: None,
        }
    }

    /// Set the URI for this image content.
    pub fn uri(mut self, uri: Option<String>) -> Self {
        self.uri = uri;
        self
    }

    /// Set the metadata for this image content.
    pub fn meta(mut self, meta: Option<String>) -> Self {
        self.meta = meta;
        self
    }
}
