//! Display-only CD peel for TUI chrome. Stub peels a simple absolute `cd … &&` prefix.

use std::borrow::Cow;
use std::path::Path;

/// Peel a leading `cd <session_cwd> &&|;` when the target equals session cwd.
pub fn strip_redundant_session_cd<'a>(command: &'a str, session_cwd: &Path) -> Cow<'a, str> {
    let trimmed = command.trim_start();
    let cwd = session_cwd.to_string_lossy();
    for sep in ["&&", ";", "|"] {
        let prefixes = [
            format!("cd {cwd} {sep}"),
            format!("cd /d {cwd} {sep}"),
            format!("cd /D {cwd} {sep}"),
        ];
        for prefix in prefixes {
            if let Some(rest) = trimmed.strip_prefix(&prefix) {
                let rest = rest.trim_start();
                if !rest.is_empty() {
                    return Cow::Borrowed(&command[command.len() - rest.len()..]);
                }
            }
        }
    }
    Cow::Borrowed(command)
}
