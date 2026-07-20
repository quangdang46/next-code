//! Stub: ACP ↔ TodoItem helpers. Types live in `xai-grok-tools`.

use agent_client_protocol as acp;

pub use xai_grok_tools::implementations::grok_build::todo::{
    TodoId, TodoItem, TodoPriority, TodoStatus,
};

pub fn todo_item_from_plan_entry(entry: acp::PlanEntry) -> TodoItem {
    let status = match entry.status {
        acp::PlanEntryStatus::Pending => TodoStatus::Pending,
        acp::PlanEntryStatus::InProgress => TodoStatus::InProgress,
        acp::PlanEntryStatus::Completed => TodoStatus::Completed,
        _ => TodoStatus::Pending,
    };
    let priority = match entry.priority {
        acp::PlanEntryPriority::High => TodoPriority::High,
        acp::PlanEntryPriority::Medium => TodoPriority::Medium,
        acp::PlanEntryPriority::Low => TodoPriority::Low,
        _ => TodoPriority::Medium,
    };
    TodoItem {
        content: entry.content,
        priority,
        status,
        meta: None,
    }
}

pub fn plan_entry_from_todo_item(item: TodoItem) -> acp::PlanEntry {
    acp::PlanEntry::new(
        item.content,
        match item.priority {
            TodoPriority::High => acp::PlanEntryPriority::High,
            TodoPriority::Medium => acp::PlanEntryPriority::Medium,
            TodoPriority::Low => acp::PlanEntryPriority::Low,
        },
        match item.status {
            TodoStatus::Pending => acp::PlanEntryStatus::Pending,
            TodoStatus::InProgress => acp::PlanEntryStatus::InProgress,
            TodoStatus::Completed | TodoStatus::Cancelled => acp::PlanEntryStatus::Completed,
        },
    )
}
