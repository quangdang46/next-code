use super::{Tool, ToolContext, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use ffs_search::directory_grep::grep_directory;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::Path;

const MAX_RESULTS: usize = 100;

pub struct FfsMultiGrepTool;

impl FfsMultiGrepTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct FfsMultiGrepInput {
    patterns: Vec<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default = "default_mode")]
    mode: String,
}

fn default_mode() -> String {
    "any".to_string()
}

#[derive(Clone)]
struct MultiGrepResult {
    file: String,
    line_num: usize,
    line: String,
}

#[async_trait]
impl Tool for FfsMultiGrepTool {
    fn name(&self) -> &str {
        "ffs multi_grep"
    }

    fn description(&self) -> &str {
        "Search for multiple patterns simultaneously. Uses ffs-search directory_grep for each pattern and combines results."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["patterns"],
            "properties": {
                "intent": super::intent_schema_property(),
                "patterns": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Patterns to search for (OR logic)."
                },
                "path": {
                    "type": "string",
                    "description": "Search directory (defaults to current directory)."
                },
                "mode": {
                    "type": "string",
                    "enum": ["any", "all"],
                    "description": "Match mode: 'any' (default) matches lines containing ANY pattern, 'all' matches lines containing ALL patterns."
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: FfsMultiGrepInput = serde_json::from_value(input)?;

        if params.patterns.is_empty() {
            return Err(anyhow::anyhow!("At least one pattern is required"));
        }

        let patterns = params.patterns.clone();
        let patterns_for_display = params.patterns.clone();
        let base_path_str = params.path.clone().unwrap_or_else(|| ".".to_string());
        let base = ctx.resolve_path(Path::new(&base_path_str));
        let mode = params.mode.clone();
        let mode_for_display = params.mode.clone();

        if !base.exists() {
            return Err(anyhow::anyhow!("Directory not found: {}", base_path_str));
        }

        let results =
            tokio::task::spawn_blocking(move || multi_grep_blocking(&base, &patterns, &mode))
                .await??;

        let mut output = String::new();
        output.push_str(&format!(
            "Found {} matches for {} '{}'\n\n",
            results.len(),
            if mode_for_display == "all" {
                "all of"
            } else {
                "any of"
            },
            patterns_for_display.join("', '")
        ));

        let mut current_file = String::new();
        for result in &results {
            if result.file != current_file {
                if !current_file.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!("{}:\n", result.file));
                current_file = result.file.clone();
            }
            output.push_str(&format!("  {:>4}: {}\n", result.line_num, result.line));
        }

        if results.len() >= MAX_RESULTS {
            output.push_str(&format!(
                "\n... results truncated at {} matches",
                MAX_RESULTS
            ));
        }

        Ok(ToolOutput::new(output))
    }
}

/// Multi-pattern grep using ffs-search's grep_directory.
///
/// For "any" mode: runs grep_directory for each pattern and deduplicates results.
/// For "all" mode: runs grep_directory for the first pattern, then filters lines
/// to those containing ALL remaining patterns.
fn multi_grep_blocking(
    base: &Path,
    patterns: &[String],
    mode: &str,
) -> Result<Vec<MultiGrepResult>> {
    if mode == "all" {
        let first = &patterns[0];
        let rest = &patterns[1..];

        let all_matches = grep_directory(base, &regex::escape(first), MAX_RESULTS);

        if rest.is_empty() {
            return Ok(all_matches
                .into_iter()
                .map(|m| MultiGrepResult {
                    file: m.path,
                    line_num: m.line_number as usize,
                    line: m.line,
                })
                .collect());
        }

        let mut results: Vec<MultiGrepResult> = Vec::new();
        for m in all_matches {
            if results.len() >= MAX_RESULTS {
                break;
            }
            if rest.iter().all(|p| m.line.contains(p.as_str())) {
                results.push(MultiGrepResult {
                    file: m.path,
                    line_num: m.line_number as usize,
                    line: m.line,
                });
            }
        }
        return Ok(results);
    }

    // "any" mode
    let mut seen: HashSet<(String, usize)> = HashSet::new();
    let mut results: Vec<MultiGrepResult> = Vec::new();

    for pattern in patterns {
        if results.len() >= MAX_RESULTS {
            break;
        }
        let matches = grep_directory(base, &regex::escape(pattern), MAX_RESULTS);
        for m in matches {
            if results.len() >= MAX_RESULTS {
                break;
            }
            let key = (m.path.clone(), m.line_number as usize);
            if seen.insert(key) {
                results.push(MultiGrepResult {
                    file: m.path,
                    line_num: m.line_number as usize,
                    line: m.line,
                });
            }
        }
    }

    results.sort_by(|a, b| a.file.cmp(&b.file).then(a.line_num.cmp(&b.line_num)));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolExecutionMode;
    use std::io::Write;

    #[test]
    fn test_multi_grep_any_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "fn foo() {{\n}}\n\nfn bar() {{\n}}\n").unwrap();

        let results = multi_grep_blocking(
            temp_dir.path(),
            &["foo".to_string(), "bar".to_string()],
            "any",
        )
        .unwrap();
        assert_eq!(results.len(), 2, "should match both foo and bar lines");
    }

    #[test]
    fn test_multi_grep_all_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(
            file,
            "// combination line\nfn combination() {{\n    let foo = 1;\n    let bar = 2;\n}}\n\nfn both() {{\n    // foo and bar\n    println!(\"foo bar\");\n}}\n"
        )
        .unwrap();

        let results = multi_grep_blocking(
            temp_dir.path(),
            &["foo".to_string(), "bar".to_string()],
            "all",
        )
        .unwrap();
        assert!(
            results
                .iter()
                .any(|r| r.line.contains("foo and bar") || r.line.contains("foo bar")),
            "should find line with both patterns"
        );
    }

    #[test]
    fn test_multi_grep_no_results() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "fn existing() {{}}").unwrap();

        let results = multi_grep_blocking(temp_dir.path(), &["xyzzy".to_string()], "any").unwrap();
        assert!(
            results.is_empty(),
            "should find nothing for non-matching pattern"
        );
    }

    #[test]
    fn test_multi_grep_binary_extension_skipped() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("image.png");
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "this is text inside a png extension").unwrap();

        let results = multi_grep_blocking(temp_dir.path(), &["this".to_string()], "any").unwrap();
        assert!(results.is_empty(), "should skip binary extensions");
    }

    #[test]
    fn test_execute_via_tool_interface() {
        let tool = FfsMultiGrepTool::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("app.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "pub fn main() {{\n    let msg = \"hello\";\n}}\n").unwrap();

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            tool_call_id: "test".to_string(),
            working_dir: Some(temp_dir.path().to_path_buf()),
            stdin_request_tx: None,
            ask_user_question_tx: None,
            best_of_n_pick_tx: None,
            graceful_shutdown_signal: None,
            execution_mode: ToolExecutionMode::Direct,
            best_of_n_run_id: None,
            best_of_n_candidate_id: None,
        };

        let input = json!({
            "patterns": ["main", "hello"],
        });

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(tool.execute(input, ctx));
        assert!(result.is_ok(), "should succeed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.output.contains("Found"), "should have found results");
    }
}
