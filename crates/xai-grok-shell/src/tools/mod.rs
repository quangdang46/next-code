//! Stub of upstream `xai-grok-shell::tools` — re-exports todo types for the pager.

pub mod todo;

pub use todo::{TodoId, TodoItem, TodoPriority, TodoStatus};
pub use xai_grok_tools::implementations::BashToolInput;

