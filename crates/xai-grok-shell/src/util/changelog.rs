//! Stub of upstream `xai-grok-shell::util::changelog`.

#[derive(Debug, Clone, Default)]
pub struct ChangelogEntry {
    pub version: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Changelog {
    pub entries: Option<Vec<ChangelogEntry>>,
    pub markdown: Option<String>,
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

    pub fn fetch(self) -> Changelog {
        self.changelog
    }
}

pub fn bullets_from_entries(entries: &[ChangelogEntry], _limit: usize) -> Vec<String> {
    entries.iter().flat_map(|e| e.bullets.clone()).collect()
}
