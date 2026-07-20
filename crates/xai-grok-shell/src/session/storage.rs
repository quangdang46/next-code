//! Façade stub of upstream `xai-grok-shell::session::storage` — session
//! event-log replay/search. No on-disk store in this compile-stub layer,
//! so these always return empty.

use agent_client_protocol::SessionUpdate;

pub fn load_updates_for_replay(_session_id: &str) -> Vec<SessionUpdate> {
    Vec::new()
}

pub fn load_updates_for_replay_at(_session_id: &str, _offset: usize) -> Vec<SessionUpdate> {
    Vec::new()
}

pub fn search(_query: &str) -> Vec<String> {
    Vec::new()
}
