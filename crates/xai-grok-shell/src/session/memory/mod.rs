//! Stub of upstream `xai-grok-shell::session::memory`.

pub mod storage;

pub use storage::{MemoryScope, MemoryStorage};

#[derive(Debug, Clone, Default)]
pub struct MemoryStatus {
    pub enabled: bool,
    pub entries: usize,
}

pub fn status() -> MemoryStatus {
    MemoryStatus::default()
}
