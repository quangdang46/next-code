//! Bash highlight / soft-break helpers. Tree-sitter parsing is stubbed empty.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BashCommandHighlights {
    pub prefix: Vec<String>,
    pub highlighted_words: Vec<String>,
    pub suffix: Vec<String>,
}

pub fn soft_break_offsets_after_operators(_script: &str) -> Vec<usize> {
    Vec::new()
}

pub fn heredoc_payload_byte_ranges(_script: &str) -> Vec<(usize, usize)> {
    Vec::new()
}

pub fn range_fully_inside(start: usize, end: usize, ranges: &[(usize, usize)]) -> bool {
    if end < start {
        return false;
    }
    ranges.iter().any(|&(rs, re)| start >= rs && end <= re)
}

pub fn split_physical_line_at_soft_breaks<'a>(
    line: &'a str,
    line_start: usize,
    breaks: &[usize],
) -> Vec<&'a str> {
    let line_end = line_start + line.len();
    let mut rel: Vec<usize> = breaks
        .iter()
        .copied()
        .filter(|&b| b > line_start && b < line_end)
        .map(|b| b - line_start)
        .filter(|&b| line.is_char_boundary(b))
        .collect();
    rel.sort_unstable();
    rel.dedup();
    if rel.is_empty() {
        return vec![line];
    }
    let mut out = Vec::with_capacity(rel.len() + 1);
    let mut start = 0usize;
    for b in rel {
        out.push(&line[start..b]);
        start = b;
    }
    out.push(&line[start..]);
    out
}
