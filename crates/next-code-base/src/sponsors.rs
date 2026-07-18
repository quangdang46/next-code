//! Sponsored tool discovery was removed for the open-source next-code fork.
//! Kept as empty stubs so leftover UI/MCP call sites compile.

pub const DISCOVERY_PARTNERS_URL: &str = "";
pub const DISCOVERY_DISCLOSURE_TAG: &str = "(partner discovery disclosure)";
pub const DISCOVERY_DISCLOSURE_NOTICE: &str = "";
pub const DISCOVERY_CATEGORIES: &[&str] = &[];

pub mod provenance {
    #[derive(Debug, Clone)]
    pub struct DiscoveredSetup {
        pub sponsor: String,
        pub command: String,
        pub args: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct ProvenanceReport {
        pub sponsor: String,
        pub connects: u32,
        pub calls: u32,
        pub errors: u32,
    }

    pub fn on_tool_call(_server: &str, _is_error: bool) {}
    pub fn on_server_connected(_name: &str, _command: &str, _args: &[String]) -> Option<String> {
        None
    }
    pub fn flush_now() {}
    pub fn reset_for_tests() {}
    pub fn record_discovered_setups(_setups: Vec<DiscoveredSetup>) {}
    pub fn is_tagged(_name: &str) -> bool {
        false
    }
    pub fn drain_pending_for_tests() -> Vec<ProvenanceReport> {
        Vec::new()
    }
}
