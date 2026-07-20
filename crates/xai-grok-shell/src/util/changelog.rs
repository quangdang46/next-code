//! Stub of upstream `xai-grok-shell::util::changelog`. Upstream reads a
//! bundled/downloaded changelog file and renders bullet points for the
//! "what's new" toast; this stub always reports empty (no disk I/O).

#[derive(Debug, Clone, Default)]
pub struct ChangelogEntry {
    pub version: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Changelog {
    pub entries: Vec<ChangelogEntry>,
}

#[derive(Debug, Default)]
pub struct ChangelogManager {
    changelog: Changelog,
}

impl ChangelogManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn changelog(&self) -> &Changelog {
        &self.changelog
    }
}

pub fn bullets_from_entries(entries: &[ChangelogEntry]) -> Vec<String> {
    entries.iter().flat_map(|e| e.bullets.clone()).collect()
}
