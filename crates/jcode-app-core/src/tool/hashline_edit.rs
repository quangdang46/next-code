use super::{Tool, ToolContext, ToolOutput};
use crate::bus::{Bus, BusEvent, FileOp, FileTouch};
use crate::tool::hashline_loop_guard::NoopGuard;
use crate::tool::hashline_snapshots;
use anyhow::Result;
use async_trait::async_trait;
use hashline::{anchor, document::FileContent, hash as hashline_hash};
use hashline::types::FileOp as HashlineFileOp;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::sync::LazyLock;

static NOOP_GUARD: LazyLock<NoopGuard> = LazyLock::new(NoopGuard::new);

pub struct HashlineEditTool;

impl Default for HashlineEditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl HashlineEditTool {
    pub fn new() -> Self {
        Self
    }
}

/// Model-facing inputs for the single `edit` tool (hashline backend).
///
/// Order of untagged variants matters: more specific shapes first.
#[derive(Deserialize)]
#[serde(untagged)]
enum HashlineEditInput {
    /// Classic jcode shape: explicit path + patch body.
    Patch {
        file_path: String,
        #[serde(default)]
        intent: Option<String>,
        patch: String,
    },
    /// Oh-my-pi shape: one `input` string with `[path#TAG]` headers.
    /// Optional `file_path` overrides the path in the first header.
    ///
    /// Accepts `patch` as an alias for `input` so callers can send
    /// `{ "patch": "..." }` without `file_path` (multi-file sections still
    /// resolve from headers). `{ "file_path", "patch" }` still matches
    /// [`HashlineEditInput::Patch`] first.
    Input {
        #[serde(alias = "patch")]
        input: String,
        #[serde(default)]
        intent: Option<String>,
        #[serde(default)]
        file_path: Option<String>,
    },
    /// Legacy anchor + old/new string mode.
    Structured {
        file_path: String,
        #[serde(default)]
        intent: Option<String>,
        anchor: OldAnchor,
        #[serde(default)]
        old_string: Option<String>,
        new_string: String,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OldAnchor {
    Body { line: usize, hash: String },
    Str(String),
}

#[async_trait]
impl Tool for HashlineEditTool {
    fn name(&self) -> &str {
        // Model-facing name matches oh-my-pi: single `edit` tool, hashline backend.
        "edit"
    }
    fn description(&self) -> &str {
        "Apply source edits using the hashline patch language (default edit tool). \
         Prefer `{input}` with `[path#TAG]` headers from read/search (oh-my-pi style), \
         or `{file_path, patch}`. Ops: SWAP/DEL/INS (and .BLK), plus REM/MV. \
         Range syntax: `N..M` (also accepts `N..=M`)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property(),
                "input": {
                    "type": "string",
                    "description": "Full hashline patch (preferred). One or more [path#TAG] sections; multi-file OK. Ops: SWAP/DEL/INS/REM/MV."
                },
                "file_path": {
                    "type": "string",
                    "description": "Target file. Required with `patch`; optional override when using `input` with a header."
                },
                "patch": {
                    "type": "string",
                    "description": "Hashline patch body (use with file_path). Same language as `input`."
                },
                "anchor": { "oneOf": [
                    { "type": "object", "required": ["line", "hash"], "properties": { "line": { "type": "integer" }, "hash": { "type": "string" } } },
                    { "type": "string" }
                ]},
                "old_string": { "type": "string" },
                "new_string": { "type": "string" }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: HashlineEditInput = serde_json::from_value(input).map_err(|e| {
            anyhow::anyhow!(
                "invalid edit args ({e}). Use `input` with a [path#TAG] patch, \
                 or `file_path` + `patch` (SWAP/DEL/INS/REM/MV)."
            )
        })?;
        match params {
            HashlineEditInput::Patch {
                file_path,
                intent,
                patch,
            } => dispatch_patch(Some(file_path), intent, patch, ctx).await,
            HashlineEditInput::Input {
                input,
                intent,
                file_path,
            } => dispatch_patch(file_path, intent, input, ctx).await,
            HashlineEditInput::Structured {
                file_path,
                intent,
                anchor,
                old_string,
                new_string,
            } => execute_old(file_path, intent, anchor, old_string, new_string, ctx).await,
        }
    }
}

// ── Patch mode ───────────────────────────────────────────────────

/// Route single- or multi-file hashline patches.
///
/// Multi-file `input` with several `[path#TAG]` sections is applied
/// section-by-section (oh-my-pi style). On failure mid-batch, already-applied
/// paths are reported so the model can re-read and retry only the remainder.
async fn dispatch_patch(
    file_path: Option<String>,
    intent: Option<String>,
    patch: String,
    ctx: ToolContext,
) -> Result<ToolOutput> {
    let sections = split_patch_sections(&patch);
    if sections.len() > 1 {
        if file_path.is_some() {
            crate::logging::warn(
                "edit: multi-section patch ignores top-level file_path; each [path#TAG] is authoritative",
            );
        }
        return execute_multi_sections(sections, intent, ctx).await;
    }

    let path = file_path
        .or_else(|| sections.first().map(|(p, _)| p.clone()))
        .or_else(|| extract_path_from_patch(&patch))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "edit requires `file_path` or a patch starting with `[path#TAG]` \
                 (from your latest read/search)"
            )
        })?;

    // Prefer the isolated section text when a header is present (avoids
    // leftover multi-path residue after split of a single section).
    let body = sections
        .into_iter()
        .next()
        .map(|(_, section)| section)
        .unwrap_or(patch);

    execute_one_section(path, intent, body, ctx).await
}

async fn execute_multi_sections(
    sections: Vec<(String, String)>,
    intent: Option<String>,
    ctx: ToolContext,
) -> Result<ToolOutput> {
    let total = sections.len();
    let mut results: Vec<String> = Vec::with_capacity(total);
    let mut applied: Vec<String> = Vec::new();

    for (i, (path, section)) in sections.into_iter().enumerate() {
        match execute_one_section(path.clone(), intent.clone(), section, ctx.clone()).await {
            Ok(out) => {
                applied.push(path);
                results.push(out.output);
            }
            Err(e) => {
                let mut msg = format!(
                    "Multi-file hashline edit failed on section {}/{total} ({path}): {e}\n",
                    i + 1
                );
                if !applied.is_empty() {
                    msg.push_str(&format!(
                        "Already applied: {}. Re-read those files before re-issuing remaining sections.\n",
                        applied.join(", ")
                    ));
                }
                msg.push_str(&format!(
                    "Stopped after {} successful section(s) of {total}; \
                     fix the failure and re-issue only unapplied sections.\n",
                    applied.len()
                ));
                if !results.is_empty() {
                    msg.push_str("\n--- results so far ---\n");
                    msg.push_str(&results.join("\n\n"));
                }
                return Err(anyhow::anyhow!(msg));
            }
        }
    }

    Ok(ToolOutput::new(format!(
        "Applied {} file section(s):\n\n{}",
        results.len(),
        results.join("\n\n")
    ))
    .with_title(format!("{} files", results.len())))
}

async fn execute_one_section(
    file_path: String,
    intent: Option<String>,
    patch: String,
    ctx: ToolContext,
) -> Result<ToolOutput> {
    let path = ctx.resolve_path(Path::new(&file_path));
    let (edits, pw, file_op, _has_block) = hashline::parser::parse_patch(&patch);

    // REM is a whole-file op with no line edits.
    if matches!(file_op, Some(HashlineFileOp::Remove)) {
        return execute_remove(&file_path, &path, intent, &pw, ctx).await;
    }

    if edits.is_empty() && file_op.is_none() {
        let m = if pw.is_empty() {
            "empty patch — provide SWAP/DEL/INS ops (or REM/MV)".into()
        } else {
            format!("empty patch: {}", pw.join("; "))
        };
        return Err(anyhow::anyhow!("{m}"));
    }

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "File not found: {file_path}. Create new files with `write`; hashline only edits existing files."
        ));
    }

    let raw = tokio::fs::read_to_string(&path).await?;
    let content = hashline::normalize::normalize_to_lf(&raw);

    let new_text = if edits.is_empty() {
        // MV-only: keep content, then rename below.
        content.clone()
    } else {
        apply_patch_with_snapshot(&file_path, &path, &content, &patch, &edits).await?
    };

    let ch = xxhash64(&content);
    let nd = edits.is_empty()
        || new_text == content
        || new_text == format!("{content}\n");
    if !edits.is_empty() {
        if let Err(e) = NOOP_GUARD.record(path.to_path_buf(), !nd, ch) {
            return Err(anyhow::anyhow!("{e}"));
        }
        if nd {
            return Ok(ToolOutput::new(format!(
                "(no change) {file_path}\n\
                 Patch applied cleanly but body rows are byte-identical to the file at the targeted lines. \
                 Re-read before issuing another edit; do not widen the payload."
            )));
        }
    }

    let le = hashline::normalize::detect_line_ending(&raw);
    let ft = if le == hashline::normalize::LineEnding::Crlf {
        hashline::normalize::restore_line_endings(&new_text, le)
    } else {
        new_text
    };

    // Line edits first (if any), then optional rename.
    if !edits.is_empty() {
        atomic_write(&path, &ft).await?;
        NOOP_GUARD.reset(&path);
        hashline_snapshots::record(&path, &ft, None);
    }

    let mut final_path = path.clone();
    let mut final_display = file_path.clone();
    let mut did_rename = false;
    if let Some(HashlineFileOp::Rename(dest)) = file_op {
        let dest_path = ctx.resolve_path(Path::new(&dest));
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        tokio::fs::rename(&path, &dest_path).await.map_err(|e| {
            anyhow::anyhow!(
                "MV failed ({} → {}): {e}",
                path.display(),
                dest_path.display()
            )
        })?;
        hashline_snapshots::invalidate(&path);
        // Re-record snapshot under the destination key.
        if let Ok(moved) = tokio::fs::read_to_string(&dest_path).await {
            hashline_snapshots::record(&dest_path, &moved, None);
        }
        final_path = dest_path;
        final_display = dest;
        did_rename = true;
        publish_file_event(
            &ctx,
            intent.clone(),
            &final_path,
            FileOp::Edit,
            format!("hashline MV {file_path} → {final_display}"),
        );
    } else if !edits.is_empty() {
        publish_edit_event(&ctx, intent.clone(), &final_path, 1, 1, None);
    }

    let ws = if pw.is_empty() {
        String::new()
    } else {
        format!(" (warnings: {})", pw.join("; "))
    };

    let new_tag = hashline_snapshots::head(&final_path)
        .map(|s| s.hash)
        .unwrap_or_else(|| {
            std::fs::read_to_string(&final_path)
                .map(|t| hashline_snapshots::compute_file_tag(&t))
                .unwrap_or_else(|_| "????".into())
        });
    let header = format!("[{final_display}#{new_tag}]");

    let summary = if edits.is_empty() && did_rename {
        format!("Moved {file_path} → {final_display}{ws}\n{header}")
    } else if did_rename {
        format!(
            "Edited + moved {file_path} → {final_display}: {} edits applied{ws}\n{header}",
            edits.len()
        )
    } else {
        format!(
            "Edited {final_display}: {} edits applied{ws}\n{header}",
            edits.len()
        )
    };

    Ok(ToolOutput::new(summary).with_title(final_display))
}

async fn execute_remove(
    file_path: &str,
    path: &Path,
    intent: Option<String>,
    warnings: &[String],
    ctx: ToolContext,
) -> Result<ToolOutput> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {file_path}"));
    }
    tokio::fs::remove_file(path)
        .await
        .map_err(|e| anyhow::anyhow!("REM failed for {file_path}: {e}"))?;
    hashline_snapshots::invalidate(path);
    NOOP_GUARD.reset(&path.to_path_buf());
    let ws = if warnings.is_empty() {
        String::new()
    } else {
        format!(" (warnings: {})", warnings.join("; "))
    };
    publish_file_event(
        &ctx,
        intent,
        path,
        FileOp::Edit,
        format!("hashline REM deleted {file_path}"),
    );
    Ok(ToolOutput::new(format!("Deleted {file_path}{ws}")).with_title(file_path.to_string()))
}

async fn apply_patch_with_snapshot(
    file_path: &str,
    path: &Path,
    content: &str,
    patch: &str,
    edits: &[hashline::types::Edit],
) -> Result<String> {
    let snap_tag = extract_tag(patch);
    let snapshot = snap_tag
        .as_ref()
        .and_then(|t| hashline_snapshots::by_hash(path, t));
    match &snapshot {
        Some(snap) => {
            let al = collect_anchor_lines(edits);
            if !hashline_snapshots::lines_were_seen(snap.seen_lines.as_ref(), &al) {
                let missing: Vec<usize> = match snap.seen_lines.as_ref() {
                    Some(seen) => al.into_iter().filter(|l| !seen.contains(l)).collect(),
                    None => Vec::new(),
                };
                let missing_txt = if missing.is_empty() {
                    String::new()
                } else {
                    format!(
                        " Unseen lines: {}.",
                        missing
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                return Err(anyhow::anyhow!(
                    "edit refs lines not shown in your latest read/search for {file_path}.{missing_txt} \
                     Re-read those lines first (only lines displayed as LINE:TEXT / numbered rows are editable)."
                ));
            }
            let ctag = hashline_snapshots::compute_file_tag(content);
            if ctag != snap.hash {
                let store = hashline_snapshots::global();
                let recovered = {
                    let guard = store.read().expect("hashline snapshots lock poisoned");
                    try_recover(&*guard, path, content, &snap.hash, edits)
                };
                if let Some(rec) = recovered {
                    // Apply recovered text through the normal write path so we still
                    // mint a fresh [path#TAG] in the tool result (oh-my-pi style).
                    return Ok(rec.text);
                }
                return Err(anyhow::anyhow!(
                    "File changed since your last read (tag {}→{}) and recovery failed. \
                     Re-read {file_path} and re-issue the edit with the fresh [path#TAG].",
                    snap.hash,
                    ctag
                ));
            }
            apply_edits_to_text(content, edits)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .map(|r| r.text)
        }
        None => {
            // Soft allow when no snapshot (tests / ad-hoc). Prefer TAG from read.
            apply_edits_to_text(content, edits)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .map(|r| r.text)
        }
    }
}

fn apply_edits_to_text(
    text: &str,
    edits: &[hashline::types::Edit],
) -> std::result::Result<hashline::types::ApplyResult, String> {
    let p = std::path::Path::new("");
    let pstr = p.to_string_lossy();
    let edits = hashline::block::resolve_block_edits(edits, text, &pstr, None)
        .map_err(|e| e.to_string())?;
    let fc = FileContent {
        path: std::path::PathBuf::from(""),
        raw: text.to_string(),
        normalized: text.to_string(),
        newline: hashline::document::NewlineStyle::Lf,
        trailing_newline: text.ends_with('\n'),
        hash: "0000".into(),
    };
    let entries = fc.lines_with_hashes();
    let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    if text.ends_with('\n') && lines.last().map(|s| s.as_str()) == Some("") {
        lines.pop();
    }
    hashline::commands::patch::apply_edits(&mut lines, &entries, p, &edits)
        .map_err(|e| e.to_string())?;
    let nt = if text.ends_with('\n') {
        lines.join("\n") + "\n"
    } else {
        lines.join("\n")
    };
    let changed = if nt == text { None } else { Some(1) };
    Ok(hashline::types::ApplyResult {
        text: nt,
        first_changed_line: changed,
        warnings: Vec::new(),
        block_resolutions: Vec::new(),
    })
}

fn extract_tag(patch: &str) -> Option<String> {
    let line = patch.lines().next()?.trim();
    let tag = line.rsplit('#').next()?.trim_end_matches(']').trim();
    if tag.len() == 4 && tag.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(tag.to_uppercase())
    } else {
        None
    }
}

/// First section path from a `[path#TAG]` or `[path]` header line.
fn extract_path_from_patch(patch: &str) -> Option<String> {
    split_patch_sections(patch)
        .into_iter()
        .next()
        .map(|(path, _)| path)
}

/// Split a multi-file hashline patch into `(path, section_text)` pairs.
///
/// Section text includes the `[path#TAG]` header line. Returns an empty vec
/// when the patch has no file headers (caller then requires explicit `file_path`).
fn split_patch_sections(patch: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = patch.lines().collect();
    let mut header_indices: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if parse_header_line(line).is_some() {
            header_indices.push(i);
        }
    }
    if header_indices.is_empty() {
        return Vec::new();
    }

    let mut sections = Vec::with_capacity(header_indices.len());
    for (i, &start) in header_indices.iter().enumerate() {
        let end = header_indices
            .get(i + 1)
            .copied()
            .unwrap_or(lines.len());
        let path = parse_header_line(lines[start])
            .expect("header index always points at a valid header");
        let mut body = lines[start..end].join("\n");
        // Preserve a trailing newline when the original section had one
        // (last section only, and only if the full patch ends with \n).
        if end == lines.len() && patch.ends_with('\n') && !body.ends_with('\n') {
            body.push('\n');
        }
        sections.push((path, body));
    }
    sections
}

fn parse_header_line(line: &str) -> Option<String> {
    let line = line.trim();
    if !(line.starts_with('[') && line.ends_with(']')) {
        return None;
    }
    let inner = &line[1..line.len() - 1];
    // Reject bare `[` `]` or apply-patch style paths with spaces only if empty.
    let path = inner.split('#').next()?.trim();
    if path.is_empty() || path.contains('\n') {
        return None;
    }
    // Hashline headers never contain spaces in practice for relative paths;
    // still accept them (quoted paths are rare).
    Some(path.to_string())
}

fn try_recover(
    store: &dyn hashline::snapshot_store::SnapshotStore,
    path: &Path,
    content: &str,
    snap_hash: &str,
    edits: &[hashline::types::Edit],
) -> Option<hashline::recovery::RecoveryResult> {
    let recovery = hashline::recovery::Recovery::new(store);
    let rargs = hashline::recovery::RecoveryArgs {
        path: path.to_string_lossy().to_string(),
        current_text: content.to_string(),
        file_hash: snap_hash.to_string(),
        edits: edits.to_vec(),
    };
    recovery.try_recover(&rargs, apply_edits_to_text)
}

fn collect_anchor_lines(edits: &[hashline::types::Edit]) -> Vec<usize> {
    use hashline::types::{Cursor, Edit};
    let mut lines = Vec::new();
    for e in edits {
        match e {
            Edit::Insert { cursor, .. } => match cursor {
                Cursor::BeforeAnchor(a) | Cursor::AfterAnchor(a) => lines.push(a.line),
                Cursor::Bof | Cursor::Eof => {}
            },
            Edit::Delete { anchor, .. } => lines.push(anchor.line),
            Edit::Block { anchor, .. } => lines.push(anchor.line),
        }
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn xxhash64(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::hash::DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

// ── Old mode ────────────────────────────────────────────────────

async fn execute_old(
    file_path: String,
    intent: Option<String>,
    anchor: OldAnchor,
    old_string: Option<String>,
    new_string: String,
    ctx: ToolContext,
) -> Result<ToolOutput> {
    let path = ctx.resolve_path(Path::new(&file_path));
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {file_path}"));
    }
    let raw = tokio::fs::read_to_string(&path).await?;
    let le = hashline::normalize::detect_line_ending(&raw);
    let content = hashline::normalize::normalize_to_lf(&raw);
    match anchor {
        OldAnchor::Body { line, hash } => {
            let old = old_string.as_deref().unwrap_or("");
            if old.is_empty() {
                return Err(anyhow::anyhow!("old_string required"));
            }
            if old == new_string {
                return Err(anyhow::anyhow!("old and new must differ"));
            }
            let (nc, sl, el, method) =
                apply_with_xxh32_fallback(&content, line, &hash, old, &new_string);
            let ft = if le == hashline::normalize::LineEnding::Crlf {
                hashline::normalize::restore_line_endings(&nc, le)
            } else {
                nc
            };
            atomic_write(&path, &ft).await?;
            let tag = hashline_snapshots::record(&path, &ft, None);
            publish_edit_event(&ctx, intent, &path, sl, el, None);
            Ok(ToolOutput::new(format!(
                "Edited {file_path}: lines {sl}-{el} ({method})\n[{file_path}#{tag}]"
            ))
            .with_title(file_path))
        }
        OldAnchor::Str(anchor_str) => {
            if old_string.is_some() {
                return Err(anyhow::anyhow!(
                    "old_string not supported with string anchor"
                ));
            }
            let line: usize = anchor_str
                .split(':')
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow::anyhow!("invalid anchor: {anchor_str}"))?;
            let nc = replace_lines(
                &content,
                line.saturating_sub(1),
                line.saturating_sub(1),
                &new_string,
            );
            let ft = if le == hashline::normalize::LineEnding::Crlf {
                hashline::normalize::restore_line_endings(&nc, le)
            } else {
                nc
            };
            atomic_write(&path, &ft).await?;
            let tag = hashline_snapshots::record(&path, &ft, None);
            publish_edit_event(&ctx, intent, &path, line, line, None);
            Ok(ToolOutput::new(format!(
                "Edited {file_path}: line {line} replaced\n[{file_path}#{tag}]"
            ))
            .with_title(file_path))
        }
    }
}

// ── Shared helpers ──────────────────────────────────────────────

async fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let pid = std::process::id();
    let temp_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => format!("{n}.jcode-tmp.{pid}"),
        None => format!("jcode-tmp.{pid}"),
    };
    let tp = path.with_file_name(temp_name);
    if let Err(e) = tokio::fs::write(&tp, content).await {
        let _ = tokio::fs::remove_file(&tp).await;
        return Err(anyhow::anyhow!("write temp {}: {}", tp.display(), e));
    }
    if let Err(e) = tokio::fs::rename(&tp, path).await {
        let _ = tokio::fs::remove_file(&tp).await;
        return Err(anyhow::anyhow!("rename: {}", e));
    }
    Ok(())
}

fn publish_edit_event(
    ctx: &ToolContext,
    intent: Option<String>,
    path: &Path,
    sl: usize,
    el: usize,
    detail: Option<String>,
) {
    Bus::global().publish(BusEvent::FileTouch(FileTouch {
        session_id: ctx.session_id.clone(),
        path: path.to_path_buf(),
        op: FileOp::Edit,
        intent: intent.filter(|v| !v.trim().is_empty()),
        summary: Some(format!("hashline edit lines {sl}-{el}")),
        detail,
    }));
}

fn publish_file_event(
    ctx: &ToolContext,
    intent: Option<String>,
    path: &Path,
    op: FileOp,
    summary: String,
) {
    Bus::global().publish(BusEvent::FileTouch(FileTouch {
        session_id: ctx.session_id.clone(),
        path: path.to_path_buf(),
        op,
        intent: intent.filter(|v| !v.trim().is_empty()),
        summary: Some(summary),
        detail: None,
    }));
}

fn replace_lines(content: &str, start: usize, end: usize, new_text: &str) -> String {
    let has_trailing_nl = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    if start >= lines.len() {
        let mut r = content.to_string();
        if !r.ends_with('\n') {
            r.push('\n');
        }
        r.push_str(new_text);
        if has_trailing_nl && !r.ends_with('\n') {
            r.push('\n');
        }
        return r;
    }
    let end = end.min(lines.len() - 1);
    let rep: Vec<&str> = new_text.lines().collect();
    let mut r = String::new();
    for (i, l) in lines.iter().enumerate() {
        if i >= start && i <= end {
            if i == start {
                for (j, rl) in rep.iter().enumerate() {
                    if j > 0 {
                        r.push('\n');
                    }
                    r.push_str(rl);
                }
            }
            continue;
        }
        if i > 0 {
            r.push('\n');
        }
        r.push_str(l);
    }
    if has_trailing_nl && !r.ends_with('\n') {
        r.push('\n');
    }
    r
}

fn apply_with_xxh32_fallback(
    content: &str,
    anchor_line: usize,
    anchor_hash: &str,
    old: &str,
    new_str: &str,
) -> (String, usize, usize, &'static str) {
    if verify_xxh32_anchor(content, anchor_line, anchor_hash).is_ok() {
        let li = anchor_line.saturating_sub(1);
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        if li < lines.len() && lines[li].contains(old) {
            lines[li] = lines[li].replacen(old, new_str, 1);
            return (
                lines.join("\n"),
                anchor_line,
                anchor_line + new_str.lines().count().saturating_sub(1),
                "hashline",
            );
        }
    }
    let nc = content.replacen(old, new_str, 1);
    let start = find_line_number(content, old);
    (
        nc,
        start,
        start + new_str.lines().count().saturating_sub(1),
        "str_replace-fallback",
    )
}

fn find_line_number(content: &str, substring: &str) -> usize {
    content
        .find(substring)
        .map(|pos| content[..pos].lines().count() + 1)
        .unwrap_or(1)
}

fn verify_xxh32_anchor(content: &str, anchor_line: usize, expected_hash: &str) -> Result<()> {
    use std::path::PathBuf;
    let fc = FileContent {
        path: PathBuf::from(""),
        raw: content.to_string(),
        normalized: content.to_string(),
        newline: hashline::document::NewlineStyle::Lf,
        trailing_newline: content.ends_with('\n'),
        hash: "0000".into(),
    };
    let entries = fc.lines_with_hashes();
    let li = anchor_line.saturating_sub(1);
    if li >= entries.len() {
        return Err(anyhow::anyhow!("line {anchor_line} out of range"));
    }
    let actual = hashline_hash::format_short_hash(entries[li].short_hash);
    if actual != expected_hash {
        return Err(anyhow::anyhow!(
            "anchor line {anchor_line}: expected {expected_hash}, actual {actual}"
        ));
    }
    let a = anchor::parse_anchor(&format!("{anchor_line}:{actual}"))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    anchor::resolve(&a, &fc).map_err(|e| anyhow::anyhow!("anchor resolve: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_tag_from_header() {
        assert_eq!(
            extract_tag("[src/main.rs#a3b2]\nSWAP 1:\n+x\n"),
            Some("A3B2".into())
        );
        assert_eq!(extract_tag("SWAP 1:\n+x\n"), None);
    }

    #[test]
    fn extract_path_from_header() {
        assert_eq!(
            extract_path_from_patch("[src/main.rs#A3B2]\nSWAP 1:\n+x\n"),
            Some("src/main.rs".into())
        );
        assert_eq!(
            extract_path_from_patch("[foo.rs]\nREM\n"),
            Some("foo.rs".into())
        );
        assert_eq!(extract_path_from_patch("SWAP 1:\n+x\n"), None);
    }

    #[test]
    fn parse_rem_file_op() {
        let (edits, _w, file_op, _) = hashline::parser::parse_patch("[x.rs#ABCD]\nREM\n");
        assert!(edits.is_empty());
        assert_eq!(file_op, Some(HashlineFileOp::Remove));
    }

    #[test]
    fn parse_mv_file_op() {
        let (edits, _w, file_op, _) =
            hashline::parser::parse_patch("[x.rs#ABCD]\nMV y.rs\n");
        assert!(edits.is_empty());
        assert_eq!(file_op, Some(HashlineFileOp::Rename("y.rs".into())));
    }

    #[test]
    fn parse_swap_range_dotdot() {
        let (edits, warnings, file_op, _) =
            hashline::parser::parse_patch("[x.rs#ABCD]\nSWAP 2..2:\n+hello\n");
        assert!(file_op.is_none(), "warnings={warnings:?}");
        assert!(!edits.is_empty(), "warnings={warnings:?}");
    }

    #[test]
    fn split_multi_file_sections() {
        let patch = "\
[a.rs#AAAA]
SWAP 1:
+one
[b.rs#BBBB]
DEL 2
";
        let sections = split_patch_sections(patch);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].0, "a.rs");
        assert!(sections[0].1.starts_with("[a.rs#AAAA]"));
        assert!(sections[0].1.contains("SWAP 1:"));
        assert_eq!(sections[1].0, "b.rs");
        assert!(sections[1].1.contains("DEL 2"));
    }

    #[test]
    fn split_no_headers_is_empty() {
        assert!(split_patch_sections("SWAP 1:\n+x\n").is_empty());
    }
}
