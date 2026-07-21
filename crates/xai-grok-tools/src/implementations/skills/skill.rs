pub fn extract_skill_display_text(text: &str) -> Option<String> {
    let name_open = "<command-name>";
    if !text.contains(name_open) {
        return None;
    }
    let cmd_open = "<command-message>";
    let cmd_close = "</command-message>";
    if let Some(start) = text.find(cmd_open) {
        let start = start + cmd_open.len();
        if let Some(end) = text[start..].find(cmd_close) {
            let msg = text[start..start + end].trim();
            if !msg.is_empty() {
                return Some(msg.to_owned());
            }
        }
    }
    let name_close = "</command-name>";
    let start = text.find(name_open)? + name_open.len();
    let end = text[start..].find(name_close)?;
    Some(text[start..start + end].trim().to_owned())
}
