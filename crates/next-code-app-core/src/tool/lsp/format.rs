//! Format LSP JSON results into tool-readable text (Claude LSPTool-aligned).

use super::uri::format_uri_for_display;
use serde_json::Value;
use std::path::Path;

pub struct Formatted {
    pub text: String,
    pub result_count: usize,
    pub file_count: usize,
}

pub fn format_operation(operation: &str, result: &Value, cwd: Option<&Path>) -> Formatted {
    match operation {
        "goToDefinition" | "goToImplementation" | "findReferences" => {
            format_locations(result, cwd)
        }
        "hover" => format_hover(result),
        "documentSymbol" => format_document_symbols(result),
        "workspaceSymbol" => format_workspace_symbols(result, cwd),
        "prepareCallHierarchy" => format_call_items(result, cwd),
        "incomingCalls" => format_incoming_calls(result, cwd),
        "outgoingCalls" => format_outgoing_calls(result, cwd),
        "diagnostics" => format_diagnostics(result, cwd),
        _ => Formatted {
            text: result.to_string(),
            result_count: 0,
            file_count: 0,
        },
    }
}

fn locations_from(value: &Value) -> Vec<&Value> {
    match value {
        Value::Null => vec![],
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    }
}

fn as_location<'a>(item: &'a Value) -> Option<(&'a str, u64, u64)> {
    if let Some(uri) = item.get("uri").and_then(|u| u.as_str()) {
        let line = item
            .pointer("/range/start/line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let character = item
            .pointer("/range/start/character")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        return Some((uri, line, character));
    }
    // LocationLink
    if let Some(uri) = item.get("targetUri").and_then(|u| u.as_str()) {
        let range = item
            .get("targetSelectionRange")
            .or_else(|| item.get("targetRange"));
        let line = range
            .and_then(|r| r.pointer("/start/line"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let character = range
            .and_then(|r| r.pointer("/start/character"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        return Some((uri, line, character));
    }
    None
}

fn format_locations(result: &Value, cwd: Option<&Path>) -> Formatted {
    let locs = locations_from(result);
    if locs.is_empty() {
        return Formatted {
            text: "No locations found.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let mut files = std::collections::BTreeSet::new();
    let mut lines = Vec::new();
    for item in &locs {
        if let Some((uri, line, character)) = as_location(item) {
            let path = format_uri_for_display(uri, cwd);
            files.insert(path.clone());
            lines.push(format!("{path}:{}:{}", line + 1, character + 1));
        }
    }
    Formatted {
        text: lines.join("\n"),
        result_count: lines.len(),
        file_count: files.len(),
    }
}

fn format_hover(result: &Value) -> Formatted {
    if result.is_null() {
        return Formatted {
            text: "No hover information.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let contents = result.get("contents").unwrap_or(result);
    let text = markup_to_string(contents);
    if text.trim().is_empty() {
        Formatted {
            text: "No hover information.".into(),
            result_count: 0,
            file_count: 0,
        }
    } else {
        Formatted {
            text,
            result_count: 1,
            file_count: 1,
        }
    }
}

fn markup_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Object(obj) => {
            if let Some(Value::String(s)) = obj.get("value") {
                s.clone()
            } else if let Some(Value::String(s)) = obj.get("language") {
                let body = obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("```{s}\n{body}\n```")
            } else {
                value.to_string()
            }
        }
        Value::Array(items) => items
            .iter()
            .map(markup_to_string)
            .collect::<Vec<_>>()
            .join("\n\n"),
        other => other.to_string(),
    }
}

fn format_document_symbols(result: &Value) -> Formatted {
    let Value::Array(items) = result else {
        return Formatted {
            text: "No symbols found.".into(),
            result_count: 0,
            file_count: 0,
        };
    };
    if items.is_empty() {
        return Formatted {
            text: "No symbols found.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let mut lines = Vec::new();
    let mut count = 0usize;
    fn walk(items: &[Value], indent: usize, lines: &mut Vec<String>, count: &mut usize) {
        for item in items {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unnamed>");
            let kind = item.get("kind").and_then(|v| v.as_u64()).unwrap_or(0);
            let line = item
                .pointer("/range/start/line")
                .or_else(|| item.pointer("/location/range/start/line"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            lines.push(format!(
                "{}{name} ({}) @ line {}",
                "  ".repeat(indent),
                symbol_kind_name(kind),
                line + 1
            ));
            *count += 1;
            if let Some(Value::Array(children)) = item.get("children") {
                walk(children, indent + 1, lines, count);
            }
        }
    }
    walk(items, 0, &mut lines, &mut count);
    Formatted {
        text: lines.join("\n"),
        result_count: count,
        file_count: 1,
    }
}

fn format_workspace_symbols(result: &Value, cwd: Option<&Path>) -> Formatted {
    let Value::Array(items) = result else {
        return Formatted {
            text: "No symbols found.".into(),
            result_count: 0,
            file_count: 0,
        };
    };
    if items.is_empty() {
        return Formatted {
            text: "No symbols found.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let mut files = std::collections::BTreeSet::new();
    let mut lines = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");
        let kind = item.get("kind").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(loc) = item.get("location")
            && let Some((uri, line, character)) = as_location(loc)
        {
            let path = format_uri_for_display(uri, cwd);
            files.insert(path.clone());
            lines.push(format!(
                "{name} ({}) — {path}:{}:{}",
                symbol_kind_name(kind),
                line + 1,
                character + 1
            ));
        } else {
            lines.push(format!("{name} ({})", symbol_kind_name(kind)));
        }
    }
    Formatted {
        text: lines.join("\n"),
        result_count: lines.len(),
        file_count: files.len(),
    }
}

fn format_call_items(result: &Value, cwd: Option<&Path>) -> Formatted {
    let Value::Array(items) = result else {
        return Formatted {
            text: "No call hierarchy item found at this position.".into(),
            result_count: 0,
            file_count: 0,
        };
    };
    if items.is_empty() {
        return Formatted {
            text: "No call hierarchy item found at this position.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let mut lines = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");
        let uri = item.get("uri").and_then(|u| u.as_str()).unwrap_or("");
        let line = item
            .pointer("/range/start/line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let path = format_uri_for_display(uri, cwd);
        lines.push(format!("{name} — {path}:{}", line + 1));
    }
    Formatted {
        text: lines.join("\n"),
        result_count: lines.len(),
        file_count: 1,
    }
}

fn format_incoming_calls(result: &Value, cwd: Option<&Path>) -> Formatted {
    format_call_edges(result, cwd, "from")
}

fn format_outgoing_calls(result: &Value, cwd: Option<&Path>) -> Formatted {
    format_call_edges(result, cwd, "to")
}

fn format_call_edges(result: &Value, cwd: Option<&Path>, field: &str) -> Formatted {
    let Value::Array(items) = result else {
        return Formatted {
            text: "No calls found.".into(),
            result_count: 0,
            file_count: 0,
        };
    };
    if items.is_empty() {
        return Formatted {
            text: "No calls found.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let mut files = std::collections::BTreeSet::new();
    let mut lines = Vec::new();
    for item in items {
        if let Some(edge) = item.get(field) {
            let name = edge
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unnamed>");
            let uri = edge.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            let line = edge
                .pointer("/range/start/line")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let path = format_uri_for_display(uri, cwd);
            files.insert(path.clone());
            lines.push(format!("{name} — {path}:{}", line + 1));
        }
    }
    Formatted {
        text: lines.join("\n"),
        result_count: lines.len(),
        file_count: files.len(),
    }
}

fn format_diagnostics(result: &Value, cwd: Option<&Path>) -> Formatted {
    let diags = result
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    if diags.is_empty() {
        return Formatted {
            text: "No diagnostics.".into(),
            result_count: 0,
            file_count: 0,
        };
    }
    let uri = result.get("uri").and_then(|u| u.as_str()).unwrap_or("");
    let path = format_uri_for_display(uri, cwd);
    let mut lines = Vec::new();
    for d in &diags {
        let severity = match d.get("severity").and_then(|s| s.as_u64()).unwrap_or(1) {
            1 => "error",
            2 => "warning",
            3 => "info",
            _ => "hint",
        };
        let message = d
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let line = d
            .pointer("/range/start/line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let character = d
            .pointer("/range/start/character")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        lines.push(format!(
            "{severity}: {path}:{}:{} — {message}",
            line + 1,
            character + 1
        ));
    }
    Formatted {
        text: lines.join("\n"),
        result_count: lines.len(),
        file_count: 1,
    }
}

fn symbol_kind_name(kind: u64) -> &'static str {
    match kind {
        1 => "File",
        2 => "Module",
        3 => "Namespace",
        4 => "Package",
        5 => "Class",
        6 => "Method",
        7 => "Property",
        8 => "Field",
        9 => "Constructor",
        10 => "Enum",
        11 => "Interface",
        12 => "Function",
        13 => "Variable",
        14 => "Constant",
        15 => "String",
        16 => "Number",
        17 => "Boolean",
        18 => "Array",
        19 => "Object",
        20 => "Key",
        21 => "Null",
        22 => "EnumMember",
        23 => "Struct",
        24 => "Event",
        25 => "Operator",
        26 => "TypeParameter",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_definition_location() {
        let result = json!([{
            "uri": "file:///tmp/a.rs",
            "range": { "start": { "line": 9, "character": 4 }, "end": { "line": 9, "character": 8 } }
        }]);
        let formatted = format_locations(&result, None);
        assert!(formatted.text.contains(":10:5"), "{}", formatted.text);
        assert_eq!(formatted.result_count, 1);
    }

    #[test]
    fn formats_hover_markdown() {
        let result = json!({
            "contents": { "kind": "markdown", "value": "**fn** foo()" }
        });
        let formatted = format_hover(&result);
        assert!(formatted.text.contains("foo()"));
    }

    #[test]
    fn formats_diagnostics() {
        let result = json!({
            "uri": "file:///tmp/a.rs",
            "diagnostics": [{
                "severity": 1,
                "message": "unused",
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } }
            }]
        });
        let formatted = format_diagnostics(&result, None);
        assert!(formatted.text.contains("error:"));
        assert!(formatted.text.contains("unused"));
    }
}
