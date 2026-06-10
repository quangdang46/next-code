//! Session-local todo persistence backed by beads_rust.

pub use jcode_beads_bridge::mapping::TodoItem;

use anyhow::{Context, Result};

/// Load todos — returns open/in-progress/blocked tasks as `TodoItem`s.
pub fn load_todos(_session_id: &str) -> Result<Vec<TodoItem>> {
    let working_dir = std::env::current_dir().context("no cwd")?;
    let project = jcode_beads_bridge::BeadsProject::open(&working_dir)
        .map_err(|e| anyhow::anyhow!("failed to open beads project: {e}"))?;
    let manager = jcode_beads_bridge::BeadsTaskManager::new(&project);
    manager.list_todo_items()
}

/// Check if any todos exist.
pub fn todos_exist(session_id: &str) -> Result<bool> {
    let todos = load_todos(session_id)?;
    Ok(!todos.is_empty())
}

/// Save todos — creates or updates tasks in beads_rust storage.
pub fn save_todos(session_id: &str, items: &[TodoItem]) -> Result<()> {
    let working_dir = std::env::current_dir().context("no cwd")?;
    let project = jcode_beads_bridge::BeadsProject::open(&working_dir)
        .map_err(|e| anyhow::anyhow!("failed to open beads project: {e}"))?;
    let manager = jcode_beads_bridge::BeadsTaskManager::new(&project);

    for item in items {
        if manager.get_task(&item.id)?.is_some() {
            use beads_rust::model::Status;
            use std::str::FromStr;
            let status = Status::from_str(&item.status).unwrap_or(Status::Open);
            manager.set_status(&item.id, status, session_id).ok();
        } else {
            manager.create_todo(item).ok();
        }
    }
    Ok(())
}
