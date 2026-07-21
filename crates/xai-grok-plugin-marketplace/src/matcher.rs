//! Keyword matching for marketplace CTA suggestions (stub).

/// A candidate plugin for keyword matching.
pub struct KeywordCandidate<'a> {
    /// Plugin name.
    pub name: &'a str,
    /// Domain strings.
    pub domains: &'a [String],
    /// Explicit keywords.
    pub keywords: &'a [String],
}

/// Return the index of the single candidate whose keyword matches `draft`.
///
/// Stub: always `None`.
pub fn match_plugin_keyword(_draft: &str, _candidates: &[KeywordCandidate<'_>]) -> Option<usize> {
    None
}
