//! Embedded changelog parse + unseen-entry filtering for UI Updates chrome.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// A changelog entry: hash, optional version tag, and commit subject.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangelogEntry<'a> {
    pub hash: &'a str,
    pub tag: &'a str,
    pub timestamp: Option<i64>,
    pub subject: &'a str,
}

/// Parse changelog entries from the embedded changelog string.
///
/// Current format per entry:
///   `hash<RS>tag<RS>timestamp<RS>subject`
/// where tag is either a version like `v0.4.2` or empty, timestamp is a
/// Unix epoch seconds string, and entries are separated by ASCII unit
/// separator (0x1F).
///
/// Older binaries used `hash:tag:subject`; that format is still accepted.
pub fn parse_changelog(changelog: &str) -> Vec<ChangelogEntry<'_>> {
    if changelog.is_empty() {
        return Vec::new();
    }
    changelog
        .split('\x1f')
        .filter_map(|entry| {
            if entry.contains('\x1e') {
                let mut parts = entry.splitn(4, '\x1e');
                let hash = parts.next()?;
                let tag = parts.next().unwrap_or("");
                let timestamp = parts.next().and_then(|raw| raw.parse::<i64>().ok());
                let subject = parts.next()?;
                Some(ChangelogEntry {
                    hash,
                    tag,
                    timestamp,
                    subject,
                })
            } else {
                let (hash, rest) = entry.split_once(':')?;
                let (tag, subject) = rest.split_once(':')?;
                Some(ChangelogEntry {
                    hash,
                    tag,
                    timestamp: None,
                    subject,
                })
            }
        })
        .collect()
}

/// Subjects the user has not seen yet, given a last-seen commit hash.
///
/// Empty `last_seen_hash` → first-run preview of up to `first_run_limit` entries.
pub fn unseen_changelog_subjects(
    entries: &[ChangelogEntry<'_>],
    last_seen_hash: &str,
    first_run_limit: usize,
) -> Vec<String> {
    if entries.is_empty() {
        return Vec::new();
    }
    if last_seen_hash.is_empty() {
        entries
            .iter()
            .take(first_run_limit)
            .map(|e| e.subject.to_string())
            .collect()
    } else {
        entries
            .iter()
            .take_while(|e| e.hash != last_seen_hash)
            .map(|e| e.subject.to_string())
            .collect()
    }
}

fn default_state_file() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".next-code").join("last_seen_changelog"))
        .unwrap_or_else(|| PathBuf::from(".next-code/last_seen_changelog"))
}

/// Compute unseen subjects and advance the last-seen hash file to the newest entry.
pub fn take_unseen_changelog_entries_at(
    changelog: &str,
    state_file: &Path,
    first_run_limit: usize,
) -> Vec<String> {
    let all_entries = parse_changelog(changelog);
    if all_entries.is_empty() {
        return Vec::new();
    }

    let last_seen_hash = std::fs::read_to_string(state_file)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let new_entries = unseen_changelog_subjects(&all_entries, &last_seen_hash, first_run_limit);

    if let Some(first) = all_entries.first() {
        if let Some(parent) = state_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(state_file, first.hash);
    }

    new_entries
}

/// Process-wide unseen changelog subjects from the embedded build changelog.
///
/// Reads/writes `~/.next-code/last_seen_changelog` (same path as legacy TUI).
pub fn take_unseen_changelog_entries() -> &'static Vec<String> {
    static ENTRIES: OnceLock<Vec<String>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        take_unseen_changelog_entries_at(crate::CHANGELOG, &default_state_file(), 5)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_rs_format() {
        let raw = "abc\x1ev1.0\x1e1700000000\x1efix thing\x1fdef\x1e\x1e\x1eadd other";
        let entries = parse_changelog(raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc");
        assert_eq!(entries[0].tag, "v1.0");
        assert_eq!(entries[0].timestamp, Some(1_700_000_000));
        assert_eq!(entries[0].subject, "fix thing");
        assert_eq!(entries[1].hash, "def");
        assert_eq!(entries[1].tag, "");
        assert_eq!(entries[1].subject, "add other");
    }

    #[test]
    fn parse_legacy_colon_format() {
        let entries = parse_changelog("aaa:v0.1:hello\x1fbbb::world");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].subject, "hello");
        assert_eq!(entries[1].tag, "");
        assert_eq!(entries[1].subject, "world");
    }

    #[test]
    fn unseen_first_run_caps() {
        let entries = parse_changelog("a\x1e\x1e\x1eone\x1fb\x1e\x1e\x1etwo\x1fc\x1e\x1e\x1ethree");
        let unseen = unseen_changelog_subjects(&entries, "", 2);
        assert_eq!(unseen, vec!["one".to_string(), "two".to_string()]);
    }

    #[test]
    fn unseen_until_last_seen() {
        let entries = parse_changelog("a\x1e\x1e\x1eone\x1fb\x1e\x1e\x1etwo\x1fc\x1e\x1e\x1ethree");
        let unseen = unseen_changelog_subjects(&entries, "b", 5);
        assert_eq!(unseen, vec!["one".to_string()]);
    }

    #[test]
    fn take_unseen_writes_latest_hash() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("last_seen_changelog");
        let raw = "aaa\x1e\x1e\x1efirst\x1fbbb\x1e\x1e\x1esecond";
        let unseen = take_unseen_changelog_entries_at(raw, &state, 5);
        assert_eq!(unseen, vec!["first".to_string(), "second".to_string()]);
        assert_eq!(fs::read_to_string(&state).unwrap().trim(), "aaa");

        let unseen2 = take_unseen_changelog_entries_at(raw, &state, 5);
        assert!(unseen2.is_empty());
    }
}
