//! Pure resolution logic for marketplace install refs (stub).

use crate::types::MarketplaceSource;

/// A parsed marketplace install ref.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceRef {
    /// Plugin name.
    pub name: String,
    /// Optional source qualifier (`owner/repo`, `local/<slug>`, …).
    pub qualifier: Option<String>,
}

/// Recognize `<name>` / `<name>@<qualifier>` install args.
///
/// Stub: always `None` (no marketplace install path).
pub fn parse_marketplace_ref(_arg: &str) -> Option<MarketplaceRef> {
    None
}

/// Lowercase a source name and turn whitespace runs into single hyphens.
pub fn slugify(name: &str) -> String {
    name.to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Qualifier a user would type to pin this source.
pub fn addressable_qualifier(source: &MarketplaceSource) -> String {
    match &source.kind {
        crate::types::SourceKind::Git { url, .. } => crate::canonical_github_owner_repo(url)
            .unwrap_or_else(|| format!("git/{}", slugify(&source.name))),
        crate::types::SourceKind::Local { .. } => format!("local/{}", slugify(&source.name)),
    }
}
