//! Keyword detection — scan sanitized input for keyword triggers.

use crate::options::{DetectOptions, MatchMode};
use crate::registry::KeywordEntry;
use crate::sanitizer;

/// A keyword detected in user input.
#[derive(Debug, Clone)]
pub struct DetectedKeyword {
    /// The matched keyword entry from the registry.
    pub entry: &'static KeywordEntry,
    /// The actual text that triggered the match.
    pub matched_text: String,
    /// Byte offset range (start, end) in the sanitized input.
    pub position: (usize, usize),
    /// Confidence score: 1.0 for exact keyword, 0.8-0.9 for alias match.
    pub confidence: f32,
}

/// Detect keywords with **Strict** defaults (see [`DetectOptions::default`]).
pub fn detect_keywords(input: &str) -> Vec<DetectedKeyword> {
    detect_keywords_with(input, &DetectOptions::default())
}

/// Detect keywords using the given options.
pub fn detect_keywords_with(input: &str, opts: &DetectOptions) -> Vec<DetectedKeyword> {
    let sanitized = sanitizer::sanitize(input);
    if sanitized.is_empty() {
        return Vec::new();
    }
    let lower = sanitizer::to_lower(&sanitized);
    let registry = crate::registry::build_registry();
    let mut results = Vec::new();

    for entry in registry.iter() {
        // 1) Canonical keyword: always exact word-boundary match.
        let kw_lower = entry.keyword.to_lowercase();
        if let Some(pos) = find_word_boundary(&lower, &kw_lower) {
            results.push(DetectedKeyword {
                entry,
                matched_text: sanitized[pos..pos + entry.keyword.len()].to_string(),
                position: (pos, pos + entry.keyword.len()),
                confidence: 1.0,
            });
            continue;
        }

        // 2) Token aliases (Strict + Loose): word-boundary exact.
        let mut matched = false;
        for alias in entry.aliases {
            let alias_lower = alias.to_lowercase();
            if let Some(pos) = find_word_boundary(&lower, &alias_lower) {
                let end = pos + alias_lower.len();
                results.push(DetectedKeyword {
                    entry,
                    matched_text: sanitized[pos..end].to_string(),
                    position: (pos, end),
                    confidence: 0.95,
                });
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        // 3) Phrase aliases — Loose only.
        if opts.match_mode != MatchMode::Loose {
            continue;
        }
        for alias in entry.phrase_aliases {
            let alias_lower = alias.to_lowercase();
            if alias_lower.len() < 5 {
                if let Some(pos) = lower.find(&alias_lower) {
                    let after = sanitized[pos..]
                        .find(char::is_whitespace)
                        .map(|ws| pos + ws)
                        .unwrap_or(sanitized.len());
                    results.push(DetectedKeyword {
                        entry,
                        matched_text: sanitized[pos..after].to_string(),
                        position: (pos, after),
                        confidence: 0.9,
                    });
                    break;
                }
                continue;
            }
            let fuzzy_budget = if opts.allow_fuzzy { 2 } else { 0 };
            if let Some(pos) = find_fuzzy(&lower, &alias_lower, fuzzy_budget) {
                let after = if alias_lower.contains(char::is_whitespace) {
                    (pos + alias_lower.len()).min(sanitized.len())
                } else {
                    sanitized[pos..]
                        .find(char::is_whitespace)
                        .map(|ws| pos + ws)
                        .unwrap_or(sanitized.len())
                };
                results.push(DetectedKeyword {
                    entry,
                    matched_text: sanitized[pos..after].to_string(),
                    position: (pos, after),
                    confidence: 0.85,
                });
                break;
            }
        }
    }

    let exact_ranges: Vec<(usize, usize)> = results
        .iter()
        .filter(|r| r.confidence >= 1.0)
        .map(|r| r.position)
        .collect();
    results.retain(|r| {
        if r.confidence >= 1.0 {
            return true;
        }
        !exact_ranges
            .iter()
            .any(|&(es, ee)| r.position.0 < ee && r.position.1 > es)
    });

    results.sort_by(|a, b| {
        b.entry
            .priority
            .cmp(&a.entry.priority)
            .then(a.position.0.cmp(&b.position.0))
    });

    deduplicate_by_workflow(results)
}

/// Find exact `needle` with word boundaries. Returns byte offset.
fn find_word_boundary(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let mut search_from = 0;
    while let Some(rel) = haystack[search_from..].find(needle) {
        let pos = search_from + rel;
        let end = pos + needle.len();
        let left_ok = pos == 0
            || !haystack[..pos]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        let right_ok = end >= haystack.len()
            || !haystack[end..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if left_ok && right_ok {
            return Some(pos);
        }
        search_from = pos + needle.len().max(1);
        if search_from >= haystack.len() {
            break;
        }
    }
    None
}

fn find_fuzzy(haystack: &str, needle: &str, max_dist: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if let Some(pos) = haystack.find(needle) {
        return Some(pos);
    }
    if max_dist == 0 {
        return None;
    }

    let needle_len = needle.chars().count();
    let min_len = needle_len.saturating_sub(max_dist);
    let max_len = needle_len + max_dist;
    let haystack_chars: Vec<char> = haystack.chars().collect();

    for window_len in min_len..=max_len {
        if window_len == 0 {
            continue;
        }
        for i in 0..haystack_chars.len().saturating_sub(window_len - 1) {
            let window: String = haystack_chars[i..i + window_len].iter().collect();
            if levenshtein_distance(&window, needle) <= max_dist {
                let byte_offset: usize = haystack_chars[..i].iter().map(|c| c.len_utf8()).sum();
                return Some(byte_offset);
            }
        }
    }
    None
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev = (0..=m).collect::<Vec<_>>();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

fn deduplicate_by_workflow(mut results: Vec<DetectedKeyword>) -> Vec<DetectedKeyword> {
    let mut seen = std::collections::HashSet::new();
    results.retain(|kw| seen.insert(kw.entry.workflow));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::DetectOptions;
    use crate::registry::WorkflowKind;

    fn strict(input: &str) -> Vec<DetectedKeyword> {
        detect_keywords_with(input, &DetectOptions::strict())
    }

    fn loose(input: &str) -> Vec<DetectedKeyword> {
        detect_keywords_with(input, &DetectOptions::loose(true))
    }

    #[test]
    fn detect_exact_keyword() {
        let results = strict("$ultrawork fix the bug");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.keyword, "$ultrawork");
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn detect_token_alias_ulw() {
        let results = strict("please run ulw on this");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.workflow, WorkflowKind::Ultrawork);
    }

    #[test]
    fn detect_cancel() {
        let results = strict("cancelnext");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.workflow, WorkflowKind::Cancel);
    }

    #[test]
    fn strict_ignores_phrase_work_on() {
        assert!(strict("please work on the login bug").is_empty());
    }

    #[test]
    fn strict_ignores_parallel_prose() {
        assert!(strict("use parallel iterators for speed").is_empty());
    }

    #[test]
    fn strict_ignores_think_hard_prose() {
        assert!(strict("think hard about dinner").is_empty());
    }

    #[test]
    fn strict_ignores_must_complete() {
        assert!(strict("I must complete the payment form").is_empty());
    }

    #[test]
    fn loose_matches_think_deeply() {
        let results = loose("think deeply about this problem");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.workflow, WorkflowKind::Ultrathink);
    }

    #[test]
    fn loose_matches_work_on() {
        let results = loose("please work on this");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.workflow, WorkflowKind::Ultrawork);
    }

    #[test]
    fn no_detection_on_plain_text() {
        assert!(strict("hello world").is_empty());
    }

    #[test]
    fn detect_multiple_keywords_by_priority() {
        let results = strict("$ultrawork $tdd fix this");
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.workflow, WorkflowKind::Ultrawork);
    }

    #[test]
    fn word_boundary_no_mid_token() {
        assert!(strict("xxulwyy").is_empty());
    }

    #[test]
    fn default_detect_keywords_is_strict() {
        assert!(detect_keywords("please work on this").is_empty());
        assert!(!detect_keywords("$ultrawork go").is_empty());
    }

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }
}
