/// First prose paragraph stub (no pulldown-cmark): first non-empty line block.
pub fn extract_first_paragraph(body: &str) -> Option<String> {
    let mut lines = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if t.starts_with('#') || t.starts_with("```") || t.starts_with('|') || t.starts_with('-') {
            if lines.is_empty() {
                continue;
            }
            break;
        }
        lines.push(t);
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}
