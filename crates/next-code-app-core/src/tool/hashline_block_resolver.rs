//! Hashline [`BlockResolver`] for next-code using syntactic heuristics
//! (brace-matching, indentation analysis, and language-specific patterns).
//!
//! This mirrors the proven logic from the hashline CLI's `resolve_block_span`
//! but is exposed as an injectable [`hashline::types::BlockResolver`] trait
//! implementation, eliminating any need for a `.hashline/` dot-folder or
//! subprocess invocation.
//!
//! Injected into [`hashline::block::resolve_block_edits`] so block operations
//! (`SWAP.BLK`, `DEL.BLK`, `INS.BLK.POST`, `INS.BLK.PRE`, `INS.BLK.START`)
//! work when editing through next-code's own `edit` / `propose_hashline` tools.

use hashline::types::{BlockResolver, BlockResolverRequest, BlockSpan};

/// Injectable [`BlockResolver`] that finds syntactic block boundaries
/// using heuristic scanning — no tree-sitter, no extra dependencies.
///
/// Strategy per language (mirrors the hashline CLI):
/// - **Brace languages** (Rust, JS, TS, Go, Java, C, C++, C#, Kotlin, Swift,
///   Scala, Dart, Zig, Objective-C): brace-pair matching with string/comment
///   awareness.
/// - **Python / Verse**: indentation-based block finding.
/// - **Ruby**: `def`/`class`/`module`/`do` … `end` matching.
/// - **Everything else**: brace matching first, indentation fallback.
pub struct NextCodeBlockResolver;

impl BlockResolver for NextCodeBlockResolver {
    fn resolve(&self, request: &BlockResolverRequest) -> Option<BlockSpan> {
        let lines: Vec<&str> = request.text.lines().collect();
        let anchor_idx = request.line.saturating_sub(1);
        if anchor_idx >= lines.len() {
            return None;
        }

        let ext = std::path::Path::new(&request.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "py" | "verse" => find_python_block(&lines, anchor_idx),
            "rb" => find_ruby_block(&lines, anchor_idx),
            _ => find_brace_block(&lines, anchor_idx, ext)
                .or_else(|| find_indent_block(&lines, anchor_idx)),
        }
    }
}

// ---------------------------------------------------------------------------
// Brace-balanced block finding
// ---------------------------------------------------------------------------

/// Brace languages list (same as hashline CLI).
const BRACE_LANGS: &[&str] = &[
    "rs", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp", "cs",
    "kt", "kts", "swift", "scala", "dart", "zig", "m", "mm",
];

/// Find the innermost brace pair whose span contains `anchor_idx`.
fn find_brace_block(lines: &[&str], anchor_idx: usize, ext: &str) -> Option<BlockSpan> {
    let pairs = find_brace_pairs(lines, ext);
    pairs
        .iter()
        .filter(|(start, end)| *start <= anchor_idx && *end >= anchor_idx)
        .max_by_key(|(start, _)| *start)
        .copied()
        .map(|(start, end)| BlockSpan {
            start: start + 1,
            end: end + 1,
        })
}

/// Return all `(start_line, end_line)` brace pairs (0-indexed).
fn find_brace_pairs(lines: &[&str], _ext: &str) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let line_comment: &[u8] = b"//";
    let mut in_block_comment = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let bytes = line.as_bytes();
        let mut i = 0;
        let mut in_sq = false;
        let mut in_dq = false;
        let mut esc = false;

        while i < bytes.len() {
            if esc {
                esc = false;
                i += 1;
                continue;
            }
            if (in_sq || in_dq) && bytes[i] == b'\\' {
                esc = true;
                i += 1;
                continue;
            }
            if in_block_comment {
                if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    in_block_comment = false;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            if !in_sq && !in_dq && bytes[i..].starts_with(line_comment) {
                break;
            }
            if !in_sq && !in_dq
                && i + 1 < bytes.len()
                && bytes[i] == b'/'
                && bytes[i + 1] == b'*'
            {
                in_block_comment = true;
                i += 2;
                continue;
            }
            if in_sq && bytes[i] == b'\'' {
                in_sq = false;
                i += 1;
                continue;
            }
            if in_dq && bytes[i] == b'"' {
                in_dq = false;
                i += 1;
                continue;
            }
            if !in_sq && !in_dq && bytes[i] == b'\'' {
                in_sq = true;
                i += 1;
                continue;
            }
            if !in_sq && !in_dq && bytes[i] == b'"' {
                in_dq = true;
                i += 1;
                continue;
            }
            if !in_sq && !in_dq && !in_block_comment {
                if bytes[i] == b'{' {
                    stack.push(line_idx);
                } else if bytes[i] == b'}' {
                    if let Some(s) = stack.pop() {
                        pairs.push((s, line_idx));
                    }
                }
            }
            i += 1;
        }
    }
    pairs
}

// ---------------------------------------------------------------------------
// Indentation-based block finding
// ---------------------------------------------------------------------------

/// Find block using indentation analysis (fallback for non-brace languages).
fn find_indent_block(lines: &[&str], anchor_idx: usize) -> Option<BlockSpan> {
    let anchor_indent = leading_ws(lines.get(anchor_idx)?);
    if anchor_indent == 0 {
        // Anchor IS at column 0 → treat as block header.
        find_block_from_header(lines, anchor_idx)
    } else {
        // Anchor inside a body → find enclosing block.
        find_block_from_body(lines, anchor_idx)
    }
}

/// Python / Verse block finding.
fn find_python_block(lines: &[&str], anchor_idx: usize) -> Option<BlockSpan> {
    let anchor_indent = leading_ws(lines.get(anchor_idx)?);
    if anchor_indent == 0 {
        // Anchor at a header line (def/class/if/for/with/try/…):
        find_block_from_header(lines, anchor_idx)
    } else {
        // Anchor in a body line: span from anchor down to next same-or-less indent.
        let mut end = lines.len() - 1;
        for i in (anchor_idx + 1)..lines.len() {
            let t = lines[i].trim();
            if t.is_empty() {
                continue;
            }
            if leading_ws(lines[i]) <= anchor_indent {
                end = i.saturating_sub(1);
                break;
            }
        }
        if end < anchor_idx {
            return None;
        }
        Some(BlockSpan {
            start: anchor_idx + 1,
            end: end + 1,
        })
    }
}

/// Block header at column 0: find the dedent boundary.
fn find_block_from_header(lines: &[&str], start: usize) -> Option<BlockSpan> {
    let si = leading_ws(lines.get(start)?);
    let mut end = lines.len() - 1;
    for i in (start + 1)..lines.len() {
        if leading_ws(lines[i]) <= si {
            end = i.saturating_sub(1);
            break;
        }
    }
    if end < start {
        return None;
    }
    Some(BlockSpan {
        start: start + 1,
        end: end + 1,
    })
}

/// Block body line (indented): scan backward for header, then forward for end.
fn find_block_from_body(lines: &[&str], anchor_idx: usize) -> Option<BlockSpan> {
    let anchor_indent = leading_ws(lines.get(anchor_idx)?);
    let mut start = None;
    for i in (0..anchor_idx).rev() {
        if lines[i].trim().is_empty() {
            continue;
        }
        if leading_ws(lines[i]) < anchor_indent {
            start = Some(i);
            break;
        }
    }
    let start = start?;
    let si = leading_ws(lines[start]);
    let mut end = lines.len() - 1;
    for i in (start + 1)..lines.len() {
        let t = lines[i].trim();
        if t.is_empty() {
            continue;
        }
        if leading_ws(lines[i]) <= si {
            end = i.saturating_sub(1);
            break;
        }
    }
    if end < start {
        return None;
    }
    Some(BlockSpan {
        start: start + 1,
        end: end + 1,
    })
}

// ---------------------------------------------------------------------------
// Ruby `…end` block finding
// ---------------------------------------------------------------------------

const RUBY_OPENERS: &[&str] = &[
    "def ", "class ", "module ", "do ", "do|", "if ", "unless ", "while ",
    "until ", "for ", "begin ", "case ",
];

fn find_ruby_block(lines: &[&str], anchor_idx: usize) -> Option<BlockSpan> {
    // Scan backwards from anchor to find the block opener.
    let mut depth: isize = 0;
    let mut start = None;
    for i in (0..=anchor_idx).rev() {
        let trimmed = lines[i].trim();
        let ec = if trimmed == "end" { 1 } else { 0 };
        let oc = ruby_opener_count(trimmed);
        depth += ec;
        depth -= oc as isize;
        if oc > 0 && depth <= 0 {
            start = Some(i);
            break;
        }
    }
    let start = start?;

    // Scan forward to find the matching `end`.
    depth = 0;
    for i in start..lines.len() {
        let trimmed = lines[i].trim();
        let oc = ruby_opener_count(trimmed);
        let ec = if trimmed == "end" { 1 } else { 0 };
        depth += oc as isize;
        depth -= ec;
        if i > start && depth <= 0 && trimmed == "end" {
            return Some(BlockSpan {
                start: start + 1,
                end: i + 1,
            });
        }
        if i == start && depth <= 0 {
            return Some(BlockSpan {
                start: start + 1,
                end: i + 1,
            });
        }
    }
    None
}

fn ruby_opener_count(trimmed: &str) -> usize {
    for opener in RUBY_OPENERS {
        if trimmed.starts_with(opener) {
            return 1;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leading_ws(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: resolve block in a snippet given a file extension.
    fn resolve(text: &str, line: usize, ext: &str) -> Option<BlockSpan> {
        let resolver = NextCodeBlockResolver;
        resolver.resolve(&BlockResolverRequest {
            path: format!("test.{ext}"),
            text: text.to_string(),
            line,
        })
    }

    // ── Rust brace blocks ──────────────────────────────────────────────

    #[test]
    fn rust_function_basic() {
        let text = "fn hello() {\n    let x = 1;\n}\n";
        let span = resolve(text, 1, "rs").expect("should resolve");
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 3);
    }

    #[test]
    fn rust_nested_brace_innermost() {
        // Anchor on the inner `if` line → get the if-block, not the outer fn.
        let text = "\
fn outer() {
    let x = 1;
    if true {
        let y = 2;
    }
}
";
        let span = resolve(text, 3, "rs").expect("should resolve");
        assert_eq!(span.start, 3);
        assert_eq!(span.end, 5);
    }

    #[test]
    fn rust_anchor_on_fn_header_gets_outer_block() {
        let text = "\
fn outer() {
    if true {
        let y = 2;
    }
}
";
        let span = resolve(text, 1, "rs").expect("should resolve");
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 5);
    }

    // ── Python blocks ──────────────────────────────────────────────────

    #[test]
    fn python_function_header() {
        let text = "def hello():\n    x = 1\n    return x\n";
        let span = resolve(text, 1, "py").expect("should resolve");
        assert!(span.start <= 1);
        assert_eq!(span.end, 3);
    }

    #[test]
    fn python_body_line_finds_enclosing() {
        let text = "def hello():\n    x = 1\n    return x\n";
        // Anchor on body line 2 → start at the function header (1), end at body end (3).
        let span = resolve(text, 2, "py").expect("should resolve");
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 3);
    }

    #[test]
    fn python_deeply_nested_body() {
        let text = "\
def outer():
    def inner():
        pass
    return
";
        // Anchor on inner body line 3 → should get lines 2-3 (the inner def).
        let span = resolve(text, 3, "py").expect("should resolve");
        assert_eq!(span.start, 2);
        assert_eq!(span.end, 3);
    }

    // ── Ruby blocks ────────────────────────────────────────────────────

    #[test]
    fn ruby_def_end_block() {
        let text = "def hello\n  x = 1\nend\n";
        let span = resolve(text, 1, "rb").expect("should resolve");
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 3);
    }

    #[test]
    fn ruby_if_end_block() {
        let text = "if true\n  puts 'yep'\nend\n";
        let span = resolve(text, 1, "rb").expect("should resolve");
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 3);
    }

    // ── Indent fallback (other languages) ──────────────────────────────

    #[test]
    fn indent_fallback_for_unknown_language() {
        let text = "header\n    body line 1\n    body line 2\nfooter\n";
        let span = resolve(text, 1, "txt").expect("should resolve via indent fallback");
        // Anchor at column 0 → header to dedent.
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 3);
    }

    #[test]
    fn indent_body_anchor_finds_header() {
        let text = "header\n    body line 1\n    body line 2\nfooter\n";
        let span = resolve(text, 2, "txt").expect("should resolve");
        // Body anchor → scan back to header (line 1).
        assert_eq!(span.start, 1);
        assert_eq!(span.end, 3);
    }

    // ── Edge cases ─────────────────────────────────────────────────────

    #[test]
    fn out_of_range_returns_none() {
        let text = "line1\nline2\n";
        assert!(resolve(text, 999, "rs").is_none());
    }

    #[test]
    fn empty_file_returns_none() {
        assert!(resolve("", 1, "rs").is_none());
    }

    #[test]
    fn single_line_no_block() {
        // Brace pair on a single line → matches, returns single-line span.
        let text = "fn foo() {}\n";
        let span = resolve(text, 1, "rs");
        // The `{}` brace pair is on line 1 (single line).
        // The resolver should still return it; hashline's resolve_block_edits
        // will reject single-line spans and degrade the op.
        assert!(span.is_some());
    }
}
