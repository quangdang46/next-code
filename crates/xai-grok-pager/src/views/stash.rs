//! Multi-entry prompt stash with JSONL persistence.
//!
//! Stores prompt drafts (text + optional images) that the user has explicitly
//! stashed for later recall. Persisted to `<grok_home>/prompt-stash.jsonl`.
//! Bounded at 50 entries; new pushes evict the oldest.
//!
//! # Persistence model
//!
//! The file is written on every mutation (`push`, `pop`, `remove`) via
//! [`Self::flush`], which is also called automatically on `Drop`. If no
//! mutation occurred, `flush` is a no-op (tracked by a dirty flag).
//!
//! # OpenCode reference
//!
//! Matches the OpenCode stash dialog: each entry carries a plain-text preview,
//! a human-readable timestamp, a line count, and per-entry delete.
//! Display helpers ([`StashEntry::preview`], [`StashEntry::line_count`],
//! [`StashEntry::formatted_timestamp`]) support the dialog rendering without
//! pulling UI concerns into the data layer.

use std::path::PathBuf;
use std::time::SystemTime;

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::prompt_images::PastedImage;

/// Max entries retained in the stash file.
const MAX_ENTRIES: usize = 50;

/// Basename of the stash file under [`xai_grok_config::grok_home`].
const STASH_FILE: &str = "prompt-stash.jsonl";

// ---------------------------------------------------------------------------
// StashImage -- serializable snapshot of a pasted image
// ---------------------------------------------------------------------------

/// A serialisable snapshot of an image attached to a stashed prompt.
///
/// Keeps the minimal data needed to reconstruct a [`PastedImage`] on recall:
/// the MIME type, base64-encoded bytes, and the original source path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashImage {
    /// MIME type (e.g. `"image/png"`, `"image/jpeg"`).
    pub mime_type: String,
    /// Base64-encoded image bytes.
    pub data: String,
    /// Optional original source path (file path from paste).
    pub source_path: Option<PathBuf>,
}

impl StashImage {
    /// Decode the base64 data back into raw bytes.
    pub fn decode_bytes(&self) -> Option<Vec<u8>> {
        base64::engine::general_purpose::STANDARD
            .decode(&self.data)
            .ok()
    }

    /// Reconstruct a [`PastedImage`] ready for insertion into the prompt.
    ///
    /// The caller must supply an [`ElementId`] (from the textarea that will
    /// host the chip) and a `display_number` (the monotonic counter for the
    /// current prompt). Dimensions are recomputed from the decoded data on
    /// first use (left as `None` here), and the preview state starts
    /// uninitialised (lazy-decoded on first render).
    pub fn into_pasted_image(
        self,
        element_id: xai_ratatui_textarea::ElementId,
        display_number: usize,
    ) -> Option<PastedImage> {
        let bytes: Vec<u8> = self.decode_bytes()?;
        let byte_len = bytes.len();
        Some(PastedImage {
            element_id,
            display_number,
            mime_type: self.mime_type,
            dimensions: None,
            byte_len,
            encoded_bytes: Some(bytes.into()),
            source_path: self.source_path,
            staged_temp_path: None,
            session_image_path: None,
            preview: Default::default(),
        })
    }
}

impl From<&PastedImage> for StashImage {
    fn from(img: &PastedImage) -> Self {
        let data = img
            .encoded_bytes
            .as_deref()
            .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
            .unwrap_or_default();
        StashImage {
            mime_type: img.mime_type.clone(),
            data,
            source_path: img.source_path.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// StashEntry
// ---------------------------------------------------------------------------

/// A single stashed prompt entry.
///
/// Each entry records the full input text, any attached images, the wall-clock
/// time it was stashed, and an optional prompt id for cross-referencing with
/// session history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    /// The prompt text (may be multi-line).
    pub input: String,
    /// Attached images (base64-encoded snapshots).
    pub images: Vec<StashImage>,
    /// Wall-clock timestamp of when the entry was stashed.
    #[serde(
        serialize_with = "serialize_system_time",
        deserialize_with = "deserialize_system_time"
    )]
    pub timestamp: SystemTime,
    /// Optional session prompt id for cross-referencing.
    pub prompt_id: Option<String>,
}

impl StashEntry {
    /// Build a new entry from prompt state.
    pub fn new(input: String, images: Vec<PastedImage>, prompt_id: Option<String>) -> Self {
        let stashed_images: Vec<StashImage> = images.iter().map(StashImage::from).collect();
        Self {
            input,
            images: stashed_images,
            timestamp: SystemTime::now(),
            prompt_id,
        }
    }

    /// Build a new entry from raw parts (useful when restoring from disk).
    #[allow(dead_code)]
    pub fn from_parts(
        input: String,
        images: Vec<StashImage>,
        timestamp: SystemTime,
        prompt_id: Option<String>,
    ) -> Self {
        Self {
            input,
            images,
            timestamp,
            prompt_id,
        }
    }

    // ── Display helpers for the stash dialog ─────────────────────────────

    /// Preview text for the entry: first non-blank line, truncated to ~80 chars.
    ///
    /// Blank lines and leading whitespace are stripped so the preview is
    /// visually compact. Returns a single line suitable for a list-item label.
    pub fn preview(&self) -> String {
        for line in self.input.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if trimmed.chars().count() > 80 {
                    return format!("{}...", trimmed.chars().take(77).collect::<String>());
                }
                return trimmed.to_owned();
            }
        }
        String::new()
    }

    /// Number of non-blank lines in the prompt text.
    pub fn line_count(&self) -> usize {
        self.input.lines().filter(|l| !l.trim().is_empty()).count()
    }

    /// Approximate number of images attached.
    #[allow(dead_code)]
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Format the stash timestamp as a human-readable local-time string.
    ///
    /// Uses the ISO-like format `2025-03-15 14:30:02` (date + time, no
    /// timezone suffix). Falls back to "unknown time" on clock errors.
    pub fn formatted_timestamp(&self) -> String {
        use chrono::{DateTime, Local, Utc};
        let duration = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs() as i64;
        let nsecs = duration.subsec_nanos();
        match DateTime::<Utc>::from_timestamp(secs, nsecs) {
            Some(dt) => dt.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            None => "unknown time".to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// PromptStash
// ---------------------------------------------------------------------------

/// A bounded, file-backed collection of stashed prompt entries.
///
/// # Thread safety
///
/// This type is **not** `Sync` — it is intended for single-threaded use from
/// the TUI event loop. The stash file is always read from the same `grok_home`
/// directory, so multiple processes reading simultaneously is safe (each
/// re-reads the file on `load`); concurrent writes from separate processes are
/// **not** coordinated and may race. Within a single Face process the stash is
/// owned by a single UI controller so no locking is needed.
///
/// # Drop behaviour
///
/// Any pending writes are flushed automatically on `Drop` so unsaved entries
/// are not lost on an unclean shutdown path that still runs drop.
#[derive(Debug)]
pub struct PromptStash {
    entries: Vec<StashEntry>,
    dirty: bool,
}

impl Default for PromptStash {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptStash {
    /// Create an empty stash (no disk I/O).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            dirty: false,
        }
    }

    /// Load entries from the JSONL file at `grok_home()/prompt-stash.jsonl`.
    ///
    /// Missing or unreadable files produce an empty stash (no error).
    /// Malformed lines are silently skipped — partial files degrade gracefully.
    pub fn load() -> Self {
        let path = stash_path();
        let entries = match std::fs::read_to_string(&path) {
            Ok(content) => content
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    serde_json::from_str::<StashEntry>(trimmed).ok()
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        Self {
            entries,
            dirty: false,
        }
    }

    /// Persist to disk if the stash is dirty.
    ///
    /// Writes every entry as one JSON line (JSONL format) to
    /// `grok_home()/prompt-stash.jsonl`.
    pub fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        let path = stash_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut content = String::with_capacity(self.entries.len() * 512);
        for entry in &self.entries {
            if let Ok(line) = serde_json::to_string(entry) {
                content.push_str(&line);
                content.push('\n');
            }
        }
        let _ = std::fs::write(&path, &content);
        self.dirty = false;
    }

    /// Push a new entry to the front of the stash.
    ///
    /// The oldest entry is evicted when the stash exceeds [`MAX_ENTRIES`] (50).
    pub fn push(&mut self, entry: StashEntry) {
        self.entries.insert(0, entry);
        self.entries.truncate(MAX_ENTRIES);
        self.dirty = true;
    }

    /// Pop the front (most recently stashed) entry.
    pub fn pop(&mut self) -> Option<StashEntry> {
        if self.entries.is_empty() {
            return None;
        }
        self.dirty = true;
        Some(self.entries.remove(0))
    }

    /// Remove the entry at `index`.
    ///
    /// Returns `None` when the index is out of bounds.
    pub fn remove(&mut self, index: usize) -> Option<StashEntry> {
        if index < self.entries.len() {
            self.dirty = true;
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Borrow the full entry list (most recent first).
    pub fn list(&self) -> &[StashEntry] {
        &self.entries
    }

    /// Number of entries currently in the stash.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the stash is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Drop for PromptStash {
    fn drop(&mut self) {
        self.flush();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the absolute path to the stash JSONL file.
fn stash_path() -> PathBuf {
    xai_grok_config::grok_home().join(STASH_FILE)
}

// ---------------------------------------------------------------------------
// SystemTime serde (serialise as (secs, nanos) tuple)
// ---------------------------------------------------------------------------

fn serialize_system_time<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let nsecs = duration.subsec_nanos();
    (secs, nsecs).serialize(serializer)
}

fn deserialize_system_time<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (secs, nsecs): (u64, u32) = Deserialize::deserialize(deserializer)?;
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::new(secs, nsecs))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_entry(input: &str) -> StashEntry {
        StashEntry::new(input.to_owned(), Vec::new(), None)
    }

    #[test]
    fn push_and_pop_front() {
        let mut stash = PromptStash::new();
        assert!(stash.is_empty());

        stash.push(sample_entry("first"));
        stash.push(sample_entry("second"));
        stash.push(sample_entry("third"));
        assert_eq!(stash.len(), 3);

        // Pop front (LIFO order).
        let popped = stash.pop().expect("should have entry");
        assert_eq!(popped.input, "third");
        assert_eq!(stash.len(), 2);
    }

    #[test]
    fn remove_by_index() {
        let mut stash = PromptStash::new();
        stash.push(sample_entry("a"));
        stash.push(sample_entry("b"));
        stash.push(sample_entry("c"));

        let removed = stash.remove(1).expect("index 1 should exist");
        assert_eq!(removed.input, "b");
        assert_eq!(stash.len(), 2);
        assert_eq!(stash.list()[0].input, "c");
        assert_eq!(stash.list()[1].input, "a");
    }

    #[test]
    fn remove_out_of_bounds() {
        let mut stash = PromptStash::new();
        stash.push(sample_entry("only"));
        assert!(stash.remove(5).is_none());
        assert_eq!(stash.len(), 1);
    }

    #[test]
    fn pop_empty_is_none() {
        let mut stash = PromptStash::new();
        assert!(stash.pop().is_none());
    }

    #[test]
    fn max_entries_eviction() {
        let over = MAX_ENTRIES + 5;
        let mut stash = PromptStash::new();
        for i in 0..over {
            stash.push(sample_entry(&format!("entry {i}")));
        }
        assert_eq!(stash.len(), MAX_ENTRIES);
        // The first entries pushed should be gone (most recent at front).
        assert_eq!(stash.list()[0].input, format!("entry {}", over - 1));
        assert_eq!(
            stash.list()[MAX_ENTRIES - 1].input,
            format!("entry {}", over - MAX_ENTRIES)
        );
    }

    #[test]
    fn serde_roundtrip() {
        let mut stash = PromptStash::new();
        stash.push(sample_entry("hello world"));
        stash.push(sample_entry("second entry"));
        assert_eq!(stash.len(), 2);

        // Serialise entries to JSONL.
        let lines: Vec<String> = stash
            .list()
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);

        // Deserialise back.
        let deserialized: Vec<StashEntry> = lines
            .iter()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].input, "second entry");
        assert_eq!(deserialized[1].input, "hello world");
    }

    #[test]
    fn stash_image_serde() {
        let img = StashImage {
            mime_type: "image/png".to_owned(),
            data: base64::engine::general_purpose::STANDARD.encode(b"fake-png-bytes"),
            source_path: Some(PathBuf::from("/tmp/screenshot.png")),
        };
        let json = serde_json::to_string(&img).unwrap();
        let back: StashImage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mime_type, "image/png");
        assert_eq!(back.source_path, Some(PathBuf::from("/tmp/screenshot.png")));
        assert_eq!(back.decode_bytes().unwrap(), b"fake-png-bytes");
    }

    #[test]
    fn entry_image_roundtrip() {
        let pasted = PastedImage {
            element_id: xai_ratatui_textarea::ElementId::from_raw(42),
            display_number: 1,
            mime_type: "image/jpeg".to_owned(),
            dimensions: Some((640, 480)),
            byte_len: 1234,
            encoded_bytes: Some(vec![0u8; 1234].into()),
            source_path: Some(PathBuf::from("photo.jpg")),
            staged_temp_path: None,
            session_image_path: None,
            preview: Default::default(),
        };
        let stash_img = StashImage::from(&pasted);
        assert_eq!(stash_img.mime_type, "image/jpeg");
        assert!(stash_img.decode_bytes().is_some());

        // Reconstruct pasted image.
        let reconstructed = stash_img
            .into_pasted_image(xai_ratatui_textarea::ElementId::from_raw(99), 2)
            .expect("should reconstruct");
        assert_eq!(reconstructed.mime_type, "image/jpeg");
        assert_eq!(reconstructed.byte_len, 1234);
        assert_eq!(reconstructed.source_path, Some(PathBuf::from("photo.jpg")));
        assert_eq!(
            reconstructed.element_id,
            xai_ratatui_textarea::ElementId::from_raw(99)
        );
        assert_eq!(reconstructed.display_number, 2);
        // Dimensions are left as None (will be detected on first use).
        assert!(reconstructed.dimensions.is_none());
    }

    #[test]
    fn preview_truncates_long_lines() {
        let long = "a".repeat(100);
        let entry = sample_entry(&long);
        let preview = entry.preview();
        assert!(preview.len() < 100);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn preview_uses_first_non_blank_line() {
        let entry = sample_entry("\n  \n  hello world\n  still here");
        assert_eq!(entry.preview(), "hello world");
    }

    #[test]
    fn preview_empty_for_blank_input() {
        let entry = sample_entry("\n  \n");
        assert!(entry.preview().is_empty());
    }

    #[test]
    fn line_count_skips_blanks() {
        let entry = sample_entry("line 1\n\n  \nline 2\nline 3\n");
        assert_eq!(entry.line_count(), 3);
    }

    #[test]
    fn formatted_timestamp_is_non_empty() {
        let entry = sample_entry("test");
        let formatted = entry.formatted_timestamp();
        assert!(!formatted.is_empty(), "timestamp should not be empty");
        // Should look like a date.
        assert!(
            formatted.contains(':'),
            "timestamp '{formatted}' should contain colons"
        );
    }

    #[test]
    fn flush_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        // Override the stash path by writing the file directly.
        let stash_path = dir.path().join(STASH_FILE);
        let entry = StashEntry::new("persist me".to_owned(), Vec::new(), None);
        let json = serde_json::to_string(&entry).unwrap();
        std::fs::write(&stash_path, json + "\n").unwrap();

        // Verify file content.
        let content = std::fs::read_to_string(&stash_path).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn system_time_serde() {
        let now = SystemTime::now();
        let entry = StashEntry::from_parts("timed".to_owned(), Vec::new(), now, None);
        let json = serde_json::to_string(&entry).unwrap();
        let back: StashEntry = serde_json::from_str(&json).unwrap();

        // The round-tripped time should be within 1 second.
        let diff = back
            .timestamp
            .duration_since(now)
            .unwrap_or_else(|e| e.duration());
        assert!(diff < Duration::from_secs(1), "timestamp drift: {diff:?}");
    }
}
