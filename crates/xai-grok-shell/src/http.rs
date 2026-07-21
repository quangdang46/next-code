//! Stub of upstream `xai-grok-shell::http`.

#[derive(Debug, Default, Clone)]
pub struct Client;

impl Client {
    pub fn new() -> Self {
        Self
    }
}

pub fn client() -> Client {
    Client::new()
}

pub fn set_process_client_mode_headless() {}

pub fn set_process_client_mode_tui() {}

