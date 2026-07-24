//! Claude Code–style typed memory taxonomy for Face `/memory`.
//!
//! Claude memdir constrains auto-memory files to four frontmatter types
//! (`user` / `feedback` / `project` / `reference`). Face reuses that closed
//! set for browsing/editing, and surfaces next-code notepad tiers
//! (`priority` / `working` / `manual`) as first-class categories rather than
//! inventing a second store.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use xai_grok_shell::extensions::notification::MemoryFileInfo;

/// Claude memdir closed taxonomy (`src/memdir/memoryTypes.ts`).
pub const MEMORY_TYPES: &[&str] = &["user", "feedback", "project", "reference"];

/// Notepad tiers under `<cwd>/.next-code/notepad/`.
pub const NOTEPAD_TIERS: &[(&str, &str)] = &[
    ("priority", "priority.md"),
    ("working", "working.md"),
    ("manual", "manual.md"),
];

/// Parse a Claude-style `type:` frontmatter value.
///
/// Invalid or missing values return `None` so legacy untyped files still
/// appear under scope headers (Global / Workspace / Sessions).
pub fn parse_memory_type(raw: &str) -> Option<&'static str> {
    let trimmed = raw.trim();
    MEMORY_TYPES.iter().copied().find(|t| *t == trimmed)
}

/// Extract `type:` from YAML frontmatter at the start of a markdown body.
///
/// Only reads the first ~4 KiB. Tolerates missing closing `---` (treats the
/// rest of the prefix as frontmatter until EOF / first blank-line block).
pub fn parse_type_from_markdown(content: &str) -> Option<&'static str> {
    let prefix = content.get(..4096).unwrap_or(content);
    let rest = prefix.strip_prefix("---")?;
    let rest = rest.strip_prefix('\r').unwrap_or(rest);
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let fm = match rest.find("\n---") {
        Some(idx) => &rest[..idx],
        None => rest,
    };
    for line in fm.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("type:") {
            return parse_memory_type(val.trim().trim_matches('"').trim_matches('\''));
        }
    }
    None
}

fn file_meta(path: &Path) -> (u64, Option<u64>) {
    match fs::metadata(path) {
        Ok(meta) => {
            let modified = meta.modified().ok().and_then(|t| {
                t.duration_since(SystemTime::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            });
            (meta.len(), modified)
        }
        Err(_) => (0, None),
    }
}

fn push_md_file(out: &mut Vec<MemoryFileInfo>, path: PathBuf, source: &str, typed: Option<&str>) {
    let (size_bytes, modified_epoch_secs) = file_meta(&path);
    let memory_type = typed
        .map(str::to_string)
        .or_else(|| {
            fs::read_to_string(&path)
                .ok()
                .and_then(|c| parse_type_from_markdown(&c).map(str::to_string))
        });
    out.push(MemoryFileInfo {
        path: path.to_string_lossy().into_owned(),
        source: source.to_string(),
        size_bytes,
        modified_epoch_secs,
        memory_type,
    });
}

fn walk_md_dir(out: &mut Vec<MemoryFileInfo>, dir: &Path, source: &str, depth: usize) {
    if depth > 4 || !dir.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if name == "sessions" {
                walk_md_dir(out, &path, "session", depth + 1);
            } else if !name.starts_with('.') {
                walk_md_dir(out, &path, source, depth + 1);
            }
            continue;
        }
        let is_md = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("md"));
        if !is_md {
            continue;
        }
        // Skip sqlite / lock adjacent names that somehow end in .md — none expected.
        push_md_file(out, path, source, None);
    }
}

/// Collect browsable memory files for Face `/memory`.
///
/// Order of discovery (later de-duped by path):
/// 1. Notepad tiers under `<cwd>/.next-code/notepad/` (always listed; may not exist yet)
/// 2. Workspace markdown under `<cwd>/.next-code/memory/`
/// 3. Global markdown under `{grok_home}/memory/`
pub fn collect_memory_catalog(cwd: &Path) -> Vec<MemoryFileInfo> {
    let mut files = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let notepad_dir = cwd.join(".next-code").join("notepad");
    for (tier, filename) in NOTEPAD_TIERS {
        let path = notepad_dir.join(filename);
        let key = path.to_string_lossy().into_owned();
        if seen.insert(key) {
            push_md_file(&mut files, path, "notepad", Some(tier));
        }
    }

    let mut append_scan = |dir: PathBuf, source: &str| {
        let mut batch = Vec::new();
        walk_md_dir(&mut batch, &dir, source, 0);
        for f in batch {
            if seen.insert(f.path.clone()) {
                files.push(f);
            }
        }
    };

    append_scan(cwd.join(".next-code").join("memory"), "workspace");
    append_scan(xai_grok_config::grok_home().join("memory"), "global");

    files
}

/// Ensure a memory path exists before `$EDITOR` (Claude `/memory` create-if-missing).
pub fn ensure_memory_file(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Seed notepad / typed stubs with a small helpful header.
    let seed = match path.file_name().and_then(|n| n.to_str()) {
        Some("priority.md") => {
            "# Priority notes\n\nCritical context injected every turn.\n"
        }
        Some("working.md") => "# Working notes\n\nScratchpad for in-progress reasoning.\n",
        Some("manual.md") => "# Manual notes\n\nUser-authored notes (not auto-injected).\n",
        _ => "",
    };
    fs::write(path, seed)
}

/// Section header label for a catalog entry in typed browse mode.
pub fn typed_section_label(file: &MemoryFileInfo) -> &'static str {
    if file.source == "notepad" {
        return match file.memory_type.as_deref() {
            Some("priority") => "Notepad · Priority",
            Some("working") => "Notepad · Working",
            Some("manual") => "Notepad · Manual",
            _ => "Notepad",
        };
    }
    match file.memory_type.as_deref() {
        Some("user") => "Type · User",
        Some("feedback") => "Type · Feedback",
        Some("project") => "Type · Project",
        Some("reference") => "Type · Reference",
        _ => match file.source.as_str() {
            "global" => "Global",
            "workspace" => "Workspace",
            "session" => "Sessions",
            other if !other.is_empty() => "Other",
            _ => "Other",
        },
    }
}

/// Sort key for typed section ordering (Claude taxonomy + notepad first).
pub fn typed_section_ord(label: &str) -> u8 {
    match label {
        "Notepad · Priority" => 0,
        "Notepad · Working" => 1,
        "Notepad · Manual" => 2,
        "Notepad" => 3,
        "Type · User" => 10,
        "Type · Feedback" => 11,
        "Type · Project" => 12,
        "Type · Reference" => 13,
        "Global" => 20,
        "Workspace" => 21,
        "Sessions" => 22,
        _ => 30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_memory_type_accepts_closed_set() {
        assert_eq!(parse_memory_type("user"), Some("user"));
        assert_eq!(parse_memory_type(" feedback "), Some("feedback"));
        assert_eq!(parse_memory_type("project"), Some("project"));
        assert_eq!(parse_memory_type("reference"), Some("reference"));
        assert_eq!(parse_memory_type("fact"), None);
        assert_eq!(parse_memory_type(""), None);
    }

    #[test]
    fn parse_type_from_markdown_reads_frontmatter() {
        let md = "---\nname: foo\ntype: feedback\n---\n\nbody\n";
        assert_eq!(parse_type_from_markdown(md), Some("feedback"));
    }

    #[test]
    fn parse_type_from_markdown_ignores_body_type_line() {
        let md = "# Title\n\ntype: user\n";
        assert_eq!(parse_type_from_markdown(md), None);
    }

    #[test]
    fn parse_type_from_markdown_unknown_type_is_none() {
        let md = "---\ntype: preference\n---\n";
        assert_eq!(parse_type_from_markdown(md), None);
    }

    #[test]
    fn collect_catalog_lists_notepad_stubs() {
        let dir = tempfile::tempdir().unwrap();
        let files = collect_memory_catalog(dir.path());
        let notepad: Vec<_> = files
            .iter()
            .filter(|f| f.source == "notepad")
            .map(|f| f.memory_type.as_deref().unwrap_or(""))
            .collect();
        assert_eq!(notepad, vec!["priority", "working", "manual"]);
    }

    #[test]
    fn collect_catalog_reads_typed_workspace_md() {
        let dir = tempfile::tempdir().unwrap();
        let mem = dir.path().join(".next-code").join("memory");
        fs::create_dir_all(&mem).unwrap();
        fs::write(
            mem.join("pref.md"),
            "---\ntype: user\n---\n\nPrefer concise replies.\n",
        )
        .unwrap();
        let files = collect_memory_catalog(dir.path());
        let typed = files
            .iter()
            .find(|f| f.path.ends_with("pref.md"))
            .expect("pref.md");
        assert_eq!(typed.memory_type.as_deref(), Some("user"));
        assert_eq!(typed.source, "workspace");
    }

    #[test]
    fn ensure_memory_file_creates_parent_and_seed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notepad").join("priority.md");
        ensure_memory_file(&path).unwrap();
        assert!(path.exists());
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("Priority notes"));
    }

    #[test]
    fn typed_section_labels() {
        let notepad = MemoryFileInfo {
            path: "x".into(),
            source: "notepad".into(),
            size_bytes: 0,
            modified_epoch_secs: None,
            memory_type: Some("priority".into()),
        };
        assert_eq!(typed_section_label(&notepad), "Notepad · Priority");
        let typed = MemoryFileInfo {
            path: "y".into(),
            source: "workspace".into(),
            size_bytes: 1,
            modified_epoch_secs: None,
            memory_type: Some("feedback".into()),
        };
        assert_eq!(typed_section_label(&typed), "Type · Feedback");
    }
}
