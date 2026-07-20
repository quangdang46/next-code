//! Façade stub of upstream `xai-grok-shell::agent` — grow for PR7 pager compile.

pub mod activity;
pub mod auth_method;
pub mod chat_modes;
pub mod config;
pub mod folder_trust;
pub mod init;
pub mod models;
pub mod mvp_agent;
pub mod roster;
pub mod session_registry_client;

pub use mvp_agent::MvpAgent;
pub use roster::{
    RosterActivity, RosterChanged, RosterEntry, RosterListResponse, RosterOrigin,
    SESSIONS_CHANGED_METHOD, SESSIONS_LIST_METHOD,
};
