use agent_client_protocol as acp;

pub mod bash_command_splitting;
pub mod resolution;
pub mod rules;
pub mod types;

pub use bash_command_splitting::BashCommandHighlights;

pub const ALLOW_EDITS_SESSION_OPTION_ID: &str = "allow-edits-session";
pub const ENABLE_ALWAYS_APPROVE_OPTION_ID: &str = "enable-always-approve";
pub const MCP_TOOL_NAME_DELIMITER: &str = "__";

pub fn is_enable_always_approve_option(opt: &acp::PermissionOption) -> bool {
    opt.option_id.0.as_ref() == ENABLE_ALWAYS_APPROVE_OPTION_ID
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashCommandPermission {
    pub prompt_prefix: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashCommandSelectedTerms {
    pub command_parts: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpToolPermission {
    pub prompt_prefix: String,
    pub tool_name: String,
    pub server_prefix: Option<String>,
}

impl McpToolPermission {
    pub fn action(&self) -> &str {
        mcp_tool_action(&self.tool_name, self.server_prefix.as_deref())
    }

    pub fn display_name(&self) -> String {
        mcp_tool_display_name(&self.tool_name, self.server_prefix.as_deref())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum McpScopeSelection {
    Tool { tool_name: String },
    Server { server: String },
}

pub fn mcp_tool_action<'a>(tool_name: &'a str, server_prefix: Option<&str>) -> &'a str {
    let Some(prefix) = server_prefix else {
        return tool_name;
    };
    tool_name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix(MCP_TOOL_NAME_DELIMITER))
        .unwrap_or(tool_name)
}

pub fn mcp_titleize_segment(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn mcp_tool_display_name(tool_name: &str, server_prefix: Option<&str>) -> String {
    let action = mcp_tool_action(tool_name, server_prefix);
    match server_prefix {
        Some(server) => format!(
            "({}) {}",
            mcp_titleize_segment(server),
            mcp_titleize_segment(action)
        ),
        None => mcp_titleize_segment(tool_name),
    }
}

fn parse_mcp_qualified_name(name: &str) -> Option<(usize, &str, &str)> {
    let idx = name.find(MCP_TOOL_NAME_DELIMITER)?;
    let server = &name[..idx];
    let action = &name[idx + MCP_TOOL_NAME_DELIMITER.len()..];
    if server.is_empty() || action.is_empty() {
        return None;
    }
    Some((idx, server, action))
}

pub fn mcp_pretty_name_if_qualified(name: &str) -> String {
    match parse_mcp_qualified_name(name) {
        Some((_, server, action)) => format!(
            "({}) {}",
            mcp_titleize_segment(server),
            mcp_titleize_segment(action)
        ),
        None => name.to_owned(),
    }
}

/// Safe default always-allow scope length (stub: first word only).
pub fn default_always_allow_scope(words: &[String]) -> usize {
    if words.is_empty() {
        0
    } else {
        1.min(words.len())
    }
}
