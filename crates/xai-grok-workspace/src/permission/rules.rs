use crate::permission::types::{PatternMode, PermissionRule, RuleAction, ToolFilter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleParseError {
    UnsupportedToolPrefix { prefix: String },
    UnknownToolPrefix { prefix: String },
    MalformedRule { detail: String },
}

impl std::fmt::Display for RuleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedToolPrefix { prefix } => write!(f, "unsupported tool prefix: {prefix}"),
            Self::UnknownToolPrefix { prefix } => write!(f, "unknown tool prefix: {prefix}"),
            Self::MalformedRule { detail } => write!(f, "malformed rule: {detail}"),
        }
    }
}

impl std::error::Error for RuleParseError {}

fn tool_name_to_filter(name: &str) -> Option<ToolFilter> {
    Some(match name {
        "Bash" | "bash" => ToolFilter::Bash,
        "Edit" | "edit" | "Write" | "write" => ToolFilter::Edit,
        "Read" | "read" | "ReadFile" => ToolFilter::Read,
        "Grep" | "grep" => ToolFilter::Grep,
        "Mcp" | "mcp" | "MCP" => ToolFilter::Mcp,
        "WebFetch" | "web_fetch" => ToolFilter::WebFetch,
        "WebSearch" | "web_search" => ToolFilter::WebSearch,
        "*" | "Any" => ToolFilter::Any,
        _ => return None,
    })
}

/// Minimal permission-rule DSL parser (subset of upstream).
pub fn parse_permission_rule(
    rule: &str,
    action: RuleAction,
) -> Result<PermissionRule, RuleParseError> {
    let rule = rule.trim();
    if let Some(open_paren) = rule.find('(') {
        let prefix = rule[..open_paren].trim();
        let rest = &rule[open_paren + 1..];
        let close = rest.rfind(')').ok_or_else(|| RuleParseError::MalformedRule {
            detail: "missing closing parenthesis".to_string(),
        })?;
        let raw_content = rest[..close].trim();
        let pattern = if raw_content.is_empty() || raw_content == "*" {
            None
        } else {
            Some(raw_content.to_owned())
        };
        let tool = match tool_name_to_filter(prefix) {
            Some(f) => f,
            None if prefix == "EnterWorktree" => {
                return Err(RuleParseError::UnsupportedToolPrefix {
                    prefix: prefix.to_string(),
                });
            }
            None => {
                return Err(RuleParseError::UnknownToolPrefix {
                    prefix: prefix.to_string(),
                });
            }
        };
        let pattern_mode = if matches!(tool, ToolFilter::WebFetch | ToolFilter::WebSearch) {
            PatternMode::Domain
        } else {
            PatternMode::Glob
        };
        Ok(PermissionRule {
            action,
            tool,
            pattern,
            pattern_mode,
        })
    } else {
        let tool = tool_name_to_filter(rule).unwrap_or(ToolFilter::Any);
        Ok(PermissionRule {
            action,
            tool,
            pattern: None,
            pattern_mode: PatternMode::Glob,
        })
    }
}
