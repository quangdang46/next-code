pub mod content;
pub mod session;
pub mod image;

pub use content::{ContentBlock, ImageContent, TextContent};

/// Globally-unique identifier for a permission option.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PermissionOptionId(String);

impl PermissionOptionId {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The kind of permission option presented to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PermissionOptionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

/// A single selectable option in a permission prompt.
#[derive(Debug, Clone)]
pub struct PermissionOption {
    id: PermissionOptionId,
    label: String,
    pub kind: PermissionOptionKind,
}

impl PermissionOption {
    pub fn new(id: PermissionOptionId, label: String, kind: PermissionOptionKind) -> Self {
        Self { id, label, kind }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Session identifier type.
pub type SessionId = String;
