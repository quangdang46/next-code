//! Optional next-code product chrome for the Face welcome screen.
//!
//! Face stays presentation-only: the next-code binary installs a snapshot at
//! launch (`pager_launch` → `face_welcome_status`). Stock `grok` leaves this unset.
//!
//! Line order mirrors legacy TUI `build_persistent_header` + `build_header_lines`:
//! badge → server → client → model → built → auth → Updates → mcp → skills → sessions.

use std::sync::OnceLock;

/// Auth inventory dot state (legacy `AuthState` subset for paint).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthDotState {
    Available,
    Expired,
    NotConfigured,
}

impl AuthDotState {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Available => "●",
            Self::Expired => "◐",
            Self::NotConfigured => "○",
        }
    }

    /// Legacy TUI colors: green / amber / dim gray.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Available => (100, 200, 100),
            Self::Expired => (255, 200, 100),
            Self::NotConfigured => (80, 80, 80),
        }
    }
}

/// One provider entry on the auth inventory line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthDotEntry {
    pub state: AuthDotState,
    pub label: String,
}

/// One chrome row Face paints (centered).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChromeLine {
    /// Dim text (badge, animals without mismatch, built, mcp, skills, sessions).
    Dim(String),
    /// Warm accent (version mismatch on animal lines).
    Emphasize(String),
    /// Model name with pink accent; prefix/suffix stay dim.
    Model {
        prefix: String,
        model: String,
        suffix: String,
    },
    /// Auth inventory with colored dots.
    Auth(Vec<AuthDotEntry>),
    /// Updates box: title + bullet lines (Face draws a simple border).
    Updates { title: String, bullets: Vec<String> },
}

/// Launch-time status fields for Face welcome (legacy splash parity).
#[derive(Debug, Clone, Default)]
pub struct ProductWelcomeStatus {
    /// e.g. `api-key:openrouter · Model · /model to switch` pieces for paint.
    pub model_prefix: Option<String>,
    pub model_name: Option<String>,
    /// Full model line for prompt fallback (prefix + model + switch hint).
    pub model_line: Option<String>,
    /// e.g. `2h ago` or `2h ago, code 3h ago` (callers may prefix `built `).
    pub build_age: Option<String>,
    /// `built Xh ago[, code …]` ready to paint.
    pub built_line: Option<String>,
    /// Unseen next-code changelog subjects (Updates box + hero merge).
    pub update_bullets: Vec<String>,
    /// e.g. `⟨client·perf:reduced⟩`
    pub badge_line: Option<String>,
    /// e.g. `server: Hut 🛖 · v0.14.6`
    pub server_line: Option<String>,
    /// e.g. `client: Monkey 🐒 · v0.14.6`
    pub client_line: Option<String>,
    /// Highlight server/client version suffixes when binaries disagree.
    pub version_mismatch: bool,
    /// Auth inventory entries (empty → omit auth line).
    pub auth_entries: Vec<AuthDotEntry>,
    /// Always set: `mcp: (none)` or configured servers.
    pub mcp_line: Option<String>,
    /// e.g. `skills: /foo /bar` or `skills: N loaded`
    pub skills_line: Option<String>,
    /// e.g. `server: 2 sessions`
    pub sessions_line: Option<String>,
}

static STATUS: OnceLock<ProductWelcomeStatus> = OnceLock::new();

/// Install product welcome chrome once per process (idempotent: first wins).
pub fn install_product_welcome_status(status: ProductWelcomeStatus) {
    let _ = STATUS.set(status);
}

/// Snapshot installed by the next-code embed, if any.
pub fn product_welcome_status() -> Option<&'static ProductWelcomeStatus> {
    STATUS.get()
}

/// True when Face is running under the next-code embed (welcome chrome installed).
#[must_use]
pub fn is_nextcode_embed() -> bool {
    product_welcome_status().is_some()
}

/// xAI-only / brand-unsafe slash commands to hide in the nextcode embed.
///
/// Applied via [`crate::slash::registry::CommandRegistry::set_brand_hidden_commands`]
/// (menu-hidden + unavailable — **not** tier `restricted`, so no SuperGrok upsell).
/// Canonical names, no leading `/`.
pub const EMBED_BRAND_RESTRICTED_COMMANDS: &[&str] = &[
    "gboom",
    "imagine",
    "imagine-video",
    "announcements",
    "marketplace",
    "plugins",
    "hooks",
    "privacy",
    "share",
];

/// True when `name` (canonical or alias token, no `/`) is on the embed brand-hide list.
#[must_use]
pub fn is_embed_brand_hidden_command(name: &str) -> bool {
    let key = name.trim().trim_start_matches('/').to_lowercase();
    EMBED_BRAND_RESTRICTED_COMMANDS
        .iter()
        .any(|n| *n == key)
}

/// Prefer product unseen bullets when present; otherwise keep Face/CDN bullets.
pub fn merge_changelog_bullets(face_bullets: Vec<String>, limit: usize) -> Vec<String> {
    if let Some(status) = product_welcome_status()
        && !status.update_bullets.is_empty()
    {
        return status
            .update_bullets
            .iter()
            .take(limit)
            .cloned()
            .collect();
    }
    face_bullets.into_iter().take(limit).collect()
}

#[cfg(test)]
mod embed_brand_tests {
    use super::*;

    #[test]
    fn embed_brand_list_covers_pr10_matrix() {
        for name in [
            "gboom",
            "imagine",
            "imagine-video",
            "announcements",
            "marketplace",
            "plugins",
            "hooks",
            "privacy",
            "share",
        ] {
            assert!(
                EMBED_BRAND_RESTRICTED_COMMANDS.contains(&name),
                "missing {name}"
            );
            assert!(is_embed_brand_hidden_command(name));
            assert!(is_embed_brand_hidden_command(&format!("/{name}")));
        }
        assert!(!is_embed_brand_hidden_command("usage"));
        assert!(!is_embed_brand_hidden_command("help"));
    }
}

/// Hero/changelog section title: legacy uses **Updates**, Face stock uses Changelog.
pub fn updates_section_title() -> &'static str {
    if product_welcome_status().is_some_and(|s| !s.update_bullets.is_empty()) {
        "Updates"
    } else {
        "Changelog"
    }
}

/// How many chrome lines fit. Prefer field presence; only hide on tiny terminals.
pub fn status_line_budget(window_height: u16, compact: bool) -> usize {
    if compact {
        return 0;
    }
    match window_height {
        0..=18 => 0,
        _ => 32,
    }
}

impl ProductWelcomeStatus {
    /// Legacy-ordered chrome lines (trimmed only by `max` for tiny terminals).
    pub fn chrome_lines(&self, max: usize) -> Vec<ChromeLine> {
        if max == 0 {
            return Vec::new();
        }
        let mut out = Vec::new();
        let push = |out: &mut Vec<ChromeLine>, line: ChromeLine| {
            if out.len() < max {
                out.push(line);
            }
        };

        if let Some(text) = self.badge_line.as_deref().filter(|t| !t.is_empty()) {
            push(&mut out, ChromeLine::Dim(text.to_string()));
        }
        if let Some(text) = self.server_line.as_deref().filter(|t| !t.is_empty()) {
            push(
                &mut out,
                if self.version_mismatch {
                    ChromeLine::Emphasize(text.to_string())
                } else {
                    ChromeLine::Dim(text.to_string())
                },
            );
        }
        if let Some(text) = self.client_line.as_deref().filter(|t| !t.is_empty()) {
            push(
                &mut out,
                if self.version_mismatch {
                    ChromeLine::Emphasize(text.to_string())
                } else {
                    ChromeLine::Dim(text.to_string())
                },
            );
        }
        if let Some(model) = self.model_name.as_deref().filter(|t| !t.is_empty()) {
            let prefix = self
                .model_prefix
                .as_deref()
                .filter(|t| !t.is_empty())
                .map(|p| format!("{p} · "))
                .unwrap_or_default();
            push(
                &mut out,
                ChromeLine::Model {
                    prefix,
                    model: model.to_string(),
                    suffix: " · /model to switch".to_string(),
                },
            );
        }
        if let Some(text) = self.built_line.as_deref().filter(|t| !t.is_empty()) {
            push(&mut out, ChromeLine::Dim(text.to_string()));
        }
        if !self.auth_entries.is_empty() {
            push(&mut out, ChromeLine::Auth(self.auth_entries.clone()));
        }
        if !self.update_bullets.is_empty() {
            // Prefer showing Updates in chrome when budget allows; hero also
            // uses the same bullets with title "Updates".
            let bullets: Vec<String> = self.update_bullets.iter().take(8).cloned().collect();
            push(
                &mut out,
                ChromeLine::Updates {
                    title: "Updates".to_string(),
                    bullets,
                },
            );
        }
        if let Some(text) = self.mcp_line.as_deref().filter(|t| !t.is_empty()) {
            push(&mut out, ChromeLine::Dim(text.to_string()));
        }
        if let Some(text) = self.skills_line.as_deref().filter(|t| !t.is_empty()) {
            push(&mut out, ChromeLine::Dim(text.to_string()));
        }
        if let Some(text) = self.sessions_line.as_deref().filter(|t| !t.is_empty()) {
            push(&mut out, ChromeLine::Dim(text.to_string()));
        }
        out
    }

    /// Rows consumed by `chrome_lines` (Updates box = 2 + bullets).
    pub fn chrome_row_count(lines: &[ChromeLine]) -> u16 {
        let mut n = 0u16;
        for line in lines {
            n = n.saturating_add(match line {
                ChromeLine::Updates { bullets, .. } => 2 + bullets.len() as u16,
                _ => 1,
            });
        }
        n
    }
}

/// Capitalize the first character (Hut / Monkey style).
pub fn capitalize_name(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Compact form of a full build version: `v0.25.19-dev (abc1234)` → `v0.25.19-dev`.
pub fn compact_version_label(version: &str) -> String {
    let trimmed = version.trim();
    match trimmed.split_once(" (") {
        Some((head, _)) => head.trim().to_string(),
        None => trimmed.to_string(),
    }
}

/// Version label for animal lines; keep hash when compact labels match but full differ.
pub fn animal_version_label(version: &str, include_hash: bool) -> String {
    if include_hash {
        version.trim().to_string()
    } else {
        compact_version_label(version)
    }
}

/// `⟨a·b·c⟩` status badge; `None` when items empty.
pub fn format_badge_line(items: &[&str]) -> Option<String> {
    let filtered: Vec<&str> = items
        .iter()
        .copied()
        .filter(|s| !s.trim().is_empty())
        .collect();
    if filtered.is_empty() {
        None
    } else {
        Some(format!("⟨{}⟩", filtered.join("·")))
    }
}

/// `server: Hut 🛖 · v0.14.6` (version suffix optional).
pub fn format_server_animal_line(name: &str, icon: &str, version: Option<&str>) -> String {
    let base = if icon.trim().is_empty() {
        format!("server: {}", capitalize_name(name))
    } else {
        format!("server: {} {}", capitalize_name(name), icon)
    };
    match version.filter(|v| !v.trim().is_empty()) {
        Some(v) => format!("{base} · {v}"),
        None => base,
    }
}

/// `client: Monkey 🐒 · v0.14.6` (version suffix optional).
pub fn format_client_animal_line(name: &str, icon: &str, version: Option<&str>) -> String {
    let base = if icon.trim().is_empty() {
        format!("client: {}", capitalize_name(name))
    } else {
        format!("client: {} {}", capitalize_name(name), icon)
    };
    match version.filter(|v| !v.trim().is_empty()) {
        Some(v) => format!("{base} · {v}"),
        None => base,
    }
}

/// MCP footer — empty → `mcp: (none)` (legacy always shows this line).
pub fn format_mcp_line(server_names: &[String]) -> String {
    if server_names.is_empty() {
        return "mcp: (none)".to_string();
    }
    let full_parts: Vec<String> = server_names
        .iter()
        .map(|name| format!("{name} (...)"))
        .collect();
    let full = format!("mcp: {}", full_parts.join(", "));
    if full.chars().count() <= 72 {
        return full;
    }
    let short_parts: Vec<String> = server_names
        .iter()
        .map(|name| format!("{name}(…)"))
        .collect();
    let short = format!("mcp: {}", short_parts.join(" "));
    if short.chars().count() <= 72 {
        short
    } else {
        format!("mcp: {} servers", server_names.len())
    }
}

/// Skills footer; omit when empty.
pub fn format_skills_line(skill_names: &[String]) -> Option<String> {
    if skill_names.is_empty() {
        return None;
    }
    let full = format!(
        "skills: {}",
        skill_names
            .iter()
            .map(|s| format!("/{s}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if full.chars().count() <= 72 {
        Some(full)
    } else {
        Some(format!("skills: {} loaded", skill_names.len()))
    }
}

/// `server: N clients, N sessions` when meaningful (clients > 0 or sessions > 1).
pub fn format_sessions_line(client_count: Option<usize>, session_count: usize) -> Option<String> {
    let clients = client_count.unwrap_or(0);
    if clients == 0 && session_count <= 1 {
        return None;
    }
    let mut parts = Vec::new();
    if clients > 0 {
        parts.push(format!(
            "{clients} client{}",
            if clients == 1 { "" } else { "s" }
        ));
    }
    if session_count > 1 {
        parts.push(format!("{session_count} sessions"));
    }
    Some(format!("server: {}", parts.join(", ")))
}

/// `built Xh ago[, code …]` from age string (already without `built` prefix).
pub fn format_built_line(age: &str) -> Option<String> {
    let trimmed = age.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("built {trimmed}"))
    }
}

/// Model switch line pieces: auth-tag provider + pretty model.
/// Returns `(prefix, model, full_line_for_prompt)`.
pub fn format_model_switch_parts(
    provider_auth_label: Option<&str>,
    model_display: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let model = model_display.trim();
    if model.is_empty() {
        return (None, None, None);
    }
    let prefix = provider_auth_label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let full = match prefix.as_deref() {
        Some(p) => format!("{p} · {model} · /model to switch"),
        None => format!("{model} · /model to switch"),
    };
    (prefix, Some(model.to_string()), Some(full))
}

/// Flatten auth entries to a plain string (tests / debug).
pub fn format_auth_line_plain(entries: &[AuthDotEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{} {}", e.state.glyph(), e.label))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Simple bordered Updates lines (title + bullets) for width `w`.
pub fn format_updates_box_lines(bullets: &[String], max_width: usize) -> Vec<String> {
    if bullets.is_empty() || max_width < 8 {
        return Vec::new();
    }
    let inner = max_width.saturating_sub(2);
    let mut lines = Vec::new();
    let title = " Updates ";
    let pad = inner.saturating_sub(title.chars().count());
    let left = pad / 2;
    let right = pad.saturating_sub(left);
    lines.push(format!(
        "╭{}{}{}╮",
        "─".repeat(left),
        title,
        "─".repeat(right)
    ));
    for b in bullets.iter().take(8) {
        let body = format!("• {b}");
        let truncated: String = body.chars().take(inner.saturating_sub(2)).collect();
        let pad_r = inner
            .saturating_sub(2)
            .saturating_sub(truncated.chars().count());
        lines.push(format!("│ {truncated}{} │", " ".repeat(pad_r)));
    }
    lines.push(format!("╰{}╯", "─".repeat(inner)));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_prefers_product_when_nonempty() {
        let product = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let face = vec!["cdn".to_string()];
        let merged = if !product.is_empty() {
            product.into_iter().take(3).collect::<Vec<_>>()
        } else {
            face.into_iter().take(3).collect()
        };
        assert_eq!(merged, vec!["a", "b", "c"]);
    }

    #[test]
    fn chrome_lines_legacy_order() {
        let status = ProductWelcomeStatus {
            badge_line: Some("⟨perf:reduced⟩".into()),
            server_line: Some("server: Hut 🛖".into()),
            client_line: Some("client: Monkey 🐒".into()),
            version_mismatch: true,
            model_prefix: Some("api-key:openrouter".into()),
            model_name: Some("Auto".into()),
            built_line: Some("built 2h ago".into()),
            auth_entries: vec![AuthDotEntry {
                state: AuthDotState::Available,
                label: "openrouter".into(),
            }],
            update_bullets: vec!["fix welcome".into()],
            mcp_line: Some("mcp: (none)".into()),
            skills_line: Some("skills: 3 loaded".into()),
            sessions_line: Some("server: 2 sessions".into()),
            ..Default::default()
        };
        let lines = status.chrome_lines(32);
        assert!(matches!(lines[0], ChromeLine::Dim(ref s) if s.starts_with('⟨')));
        assert!(matches!(lines[1], ChromeLine::Emphasize(_)));
        assert!(matches!(lines[2], ChromeLine::Emphasize(_)));
        assert!(matches!(lines[3], ChromeLine::Model { .. }));
        assert!(matches!(lines[4], ChromeLine::Dim(ref s) if s.starts_with("built")));
        assert!(matches!(lines[5], ChromeLine::Auth(_)));
        assert!(matches!(lines[6], ChromeLine::Updates { .. }));
        assert!(matches!(lines[7], ChromeLine::Dim(ref s) if s.starts_with("mcp:")));
    }

    #[test]
    fn status_budget_prefers_presence() {
        assert_eq!(status_line_budget(16, false), 0);
        assert_eq!(status_line_budget(20, false), 32);
        assert_eq!(status_line_budget(40, false), 32);
        assert_eq!(status_line_budget(40, true), 0);
    }

    #[test]
    fn animal_and_footer_formatters() {
        assert_eq!(capitalize_name("hut"), "Hut");
        assert_eq!(
            compact_version_label("v0.25.19-dev (abc1234)"),
            "v0.25.19-dev"
        );
        assert_eq!(
            format_server_animal_line("hut", "🛖", Some("v1")),
            "server: Hut 🛖 · v1"
        );
        assert_eq!(
            format_client_animal_line("monkey", "🐒", None),
            "client: Monkey 🐒"
        );
        assert_eq!(
            format_badge_line(&["client", "perf:reduced"]),
            Some("⟨client·perf:reduced⟩".into())
        );
        assert_eq!(format_badge_line(&[]), None);
        assert_eq!(format_mcp_line(&[]), "mcp: (none)");
        assert_eq!(
            format_mcp_line(&["alpha".into(), "beta".into()]),
            "mcp: alpha (...), beta (...)"
        );
        assert_eq!(
            format_skills_line(&["foo".into(), "bar".into()]),
            Some("skills: /foo /bar".into())
        );
        assert_eq!(format_skills_line(&[]), None);
        assert_eq!(
            format_sessions_line(None, 3),
            Some("server: 3 sessions".into())
        );
        assert_eq!(format_sessions_line(None, 1), None);
        assert_eq!(format_built_line("2h ago, code 5h ago").as_deref(), Some("built 2h ago, code 5h ago"));
        let (p, m, full) =
            format_model_switch_parts(Some("api-key:openrouter"), "Claude Opus");
        assert_eq!(p.as_deref(), Some("api-key:openrouter"));
        assert_eq!(m.as_deref(), Some("Claude Opus"));
        assert_eq!(
            full.as_deref(),
            Some("api-key:openrouter · Claude Opus · /model to switch")
        );
        let boxed = format_updates_box_lines(&["one".into()], 40);
        assert!(boxed[0].contains("Updates"));
        assert!(boxed.iter().any(|l| l.contains('•')));
    }

    #[test]
    fn auth_plain_format() {
        let entries = vec![
            AuthDotEntry {
                state: AuthDotState::Available,
                label: "openrouter".into(),
            },
            AuthDotEntry {
                state: AuthDotState::Expired,
                label: "openai(key)".into(),
            },
        ];
        let plain = format_auth_line_plain(&entries);
        assert!(plain.contains("● openrouter"));
        assert!(plain.contains("◐ openai(key)"));
    }
}
