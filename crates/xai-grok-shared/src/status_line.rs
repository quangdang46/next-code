//! Face prompt statusline segment selection (config parse + render order).
//!
//! Used by the pager prompt chrome (`PromptInfo`) — not the legacy TUI footer.

use serde::{Deserialize, Serialize};

/// Known statusline segment ids (stable config / Settings / `/statusline` vocabulary).
///
/// Codex parity extras: `Reasoning` (effort) and `RunState` (idle/running/…).
/// Ref: `.tmp/codex/codex-rs/tui/src/bottom_pane/status_line_setup.rs` `StatusLineItem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusLineSegment {
    Mode,
    Model,
    /// Reasoning effort alone (`high`, `low`, …). When on, Model drops the
    /// embedded `(effort)` suffix so the chrome stays scannable.
    Reasoning,
    /// Compact turn/run state (`idle` / `running` / `cancelling` / …).
    RunState,
    Context,
    Cwd,
    Git,
}

impl StatusLineSegment {
    pub const ALL: &'static [Self] = &[
        Self::Mode,
        Self::Model,
        Self::Reasoning,
        Self::RunState,
        Self::Context,
        Self::Cwd,
        Self::Git,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mode => "mode",
            Self::Model => "model",
            Self::Reasoning => "reasoning",
            Self::RunState => "run-state",
            Self::Context => "context",
            Self::Cwd => "cwd",
            Self::Git => "git",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "mode" | "permission" | "permissions" => Some(Self::Mode),
            "model" => Some(Self::Model),
            "reasoning" | "effort" | "thinking" => Some(Self::Reasoning),
            "run-state" | "run_state" | "status" | "state" => Some(Self::RunState),
            "context" | "context%" | "ctx" => Some(Self::Context),
            "cwd" | "dir" | "directory" | "pwd" => Some(Self::Cwd),
            "git" | "branch" => Some(Self::Git),
            _ => None,
        }
    }
}

/// Default left-to-right order (Claude-like density: mode · model · context%).
pub const DEFAULT_STATUS_LINE_ORDER: &[StatusLineSegment] = &[
    StatusLineSegment::Mode,
    StatusLineSegment::Model,
    StatusLineSegment::Context,
];

/// Persisted under `[ui.status_line]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusLineConfig {
    /// Master switch. `None` → on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Show permission/plan/auto mode flags. `None` → on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<bool>,
    /// Show model label. `None` → on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<bool>,
    /// Show reasoning effort as its own segment. `None` → on (Codex-like).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    /// Show compact run/turn state. `None` → on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_state: Option<bool>,
    /// Show context usage percent. `None` → on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<bool>,
    /// Show cwd basename. `None` → off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<bool>,
    /// Show git branch. `None` → off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<bool>,
    /// Optional comma-separated reorder (`"model,mode,context"`). Unknown ids ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

impl StatusLineConfig {
    pub fn is_default(&self) -> bool {
        self.enabled.is_none()
            && self.mode.is_none()
            && self.model.is_none()
            && self.reasoning.is_none()
            && self.run_state.is_none()
            && self.context.is_none()
            && self.cwd.is_none()
            && self.git.is_none()
            && self.order.is_none()
    }

    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    pub fn segment_visible(&self, segment: StatusLineSegment) -> bool {
        let default_on = matches!(
            segment,
            StatusLineSegment::Mode
                | StatusLineSegment::Model
                | StatusLineSegment::Reasoning
                | StatusLineSegment::RunState
                | StatusLineSegment::Context
        );
        match segment {
            StatusLineSegment::Mode => self.mode.unwrap_or(default_on),
            StatusLineSegment::Model => self.model.unwrap_or(default_on),
            StatusLineSegment::Reasoning => self.reasoning.unwrap_or(default_on),
            StatusLineSegment::RunState => self.run_state.unwrap_or(default_on),
            StatusLineSegment::Context => self.context.unwrap_or(default_on),
            StatusLineSegment::Cwd => self.cwd.unwrap_or(default_on),
            StatusLineSegment::Git => self.git.unwrap_or(default_on),
        }
    }

    pub fn set_segment_visible(&mut self, segment: StatusLineSegment, on: bool) {
        match segment {
            StatusLineSegment::Mode => self.mode = Some(on),
            StatusLineSegment::Model => self.model = Some(on),
            StatusLineSegment::Reasoning => self.reasoning = Some(on),
            StatusLineSegment::RunState => self.run_state = Some(on),
            StatusLineSegment::Context => self.context = Some(on),
            StatusLineSegment::Cwd => self.cwd = Some(on),
            StatusLineSegment::Git => self.git = Some(on),
        }
    }

    /// Parse `order` CSV into known segments (deduped, first wins). Empty/invalid → default order.
    pub fn parse_order(raw: Option<&str>) -> Vec<StatusLineSegment> {
        let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
            return DEFAULT_STATUS_LINE_ORDER.to_vec();
        };
        let mut out = Vec::new();
        for part in raw.split(|c: char| c == ',' || c.is_whitespace()) {
            if part.is_empty() {
                continue;
            }
            if let Some(seg) = StatusLineSegment::parse(part) {
                if !out.contains(&seg) {
                    out.push(seg);
                }
            }
        }
        if out.is_empty() {
            DEFAULT_STATUS_LINE_ORDER.to_vec()
        } else {
            // Append any known segments missing from the order string so toggles
            // for cwd/git still have a stable slot when enabled later.
            for seg in StatusLineSegment::ALL {
                if !out.contains(seg) {
                    out.push(*seg);
                }
            }
            out
        }
    }

    /// Canonical CSV for Settings / `/statusline order`.
    pub fn order_csv(&self) -> String {
        Self::parse_order(self.order.as_deref())
            .into_iter()
            .map(StatusLineSegment::as_str)
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Normalize a user-supplied order string (or reset to default when empty).
    pub fn canonicalize_order(raw: &str) -> String {
        Self::parse_order(Some(raw))
            .into_iter()
            .map(StatusLineSegment::as_str)
            .collect::<Vec<_>>()
            .join(",")
    }

    /// True when input is empty or only lists the default trio (mode, model, context).
    /// Used to clear `order` back to `None` instead of persisting redundant CSV.
    pub fn is_implicit_default_order(raw: &str) -> bool {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return true;
        }
        let mut out = Vec::new();
        for part in trimmed.split(|c: char| c == ',' || c.is_whitespace()) {
            if part.is_empty() {
                continue;
            }
            if let Some(seg) = StatusLineSegment::parse(part) {
                if !out.contains(&seg) {
                    out.push(seg);
                }
            }
        }
        out == DEFAULT_STATUS_LINE_ORDER.to_vec()
    }

    /// Visible segments in configured order (empty when master switch is off).
    ///
    /// Segments enabled but absent from the order prefix (e.g. `cwd`/`git` with
    /// the default trio) are appended so Settings toggles take effect without
    /// forcing a custom `order` string.
    pub fn selected_segments(&self) -> Vec<StatusLineSegment> {
        if !self.enabled() {
            return Vec::new();
        }
        let mut out: Vec<StatusLineSegment> = Self::parse_order(self.order.as_deref())
            .into_iter()
            .filter(|s| self.segment_visible(*s))
            .collect();
        for seg in StatusLineSegment::ALL {
            if self.segment_visible(*seg) && !out.contains(seg) {
                out.push(*seg);
            }
        }
        out
    }
}

/// Runtime values for building prompt chrome labels.
#[derive(Debug, Clone, Default)]
pub struct StatusLineSnapshot {
    pub model: String,
    /// Reasoning effort label (`high`, `low`, …) when the model supports it.
    pub reasoning: Option<String>,
    /// Compact run/turn state (`idle`, `running`, …).
    pub run_state: Option<String>,
    pub context_pct: Option<u8>,
    pub cwd_basename: Option<String>,
    pub git_branch: Option<String>,
    /// Pre-formatted mode flag texts (plan / always-approve / auto / …).
    pub mode_labels: Vec<String>,
}

/// One rendered left-side token for the prompt info line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusLinePart {
    /// Primary model slot (maps to `PromptInfo.model_name` when first).
    Model(String),
    /// Secondary flag text (mode / context% / cwd / git).
    Flag(String),
}

/// Select ordered display parts from config + snapshot. Empty strings are skipped.
pub fn select_status_line_parts(
    cfg: &StatusLineConfig,
    snap: &StatusLineSnapshot,
) -> Vec<StatusLinePart> {
    let mut parts = Vec::new();
    for segment in cfg.selected_segments() {
        match segment {
            StatusLineSegment::Mode => {
                for label in &snap.mode_labels {
                    if !label.is_empty() {
                        parts.push(StatusLinePart::Flag(label.clone()));
                    }
                }
            }
            StatusLineSegment::Model => {
                if !snap.model.is_empty() {
                    // When Reasoning is its own segment, keep Model bare;
                    // otherwise embed effort for backward-compatible density.
                    let label = if cfg.segment_visible(StatusLineSegment::Reasoning) {
                        snap.model.clone()
                    } else if let Some(eff) = snap.reasoning.as_ref().filter(|s| !s.is_empty()) {
                        format!("{} ({eff})", snap.model)
                    } else {
                        snap.model.clone()
                    };
                    parts.push(StatusLinePart::Model(label));
                }
            }
            StatusLineSegment::Reasoning => {
                if let Some(eff) = snap.reasoning.as_ref().filter(|s| !s.is_empty()) {
                    parts.push(StatusLinePart::Flag(eff.clone()));
                }
            }
            StatusLineSegment::RunState => {
                if let Some(state) = snap.run_state.as_ref().filter(|s| !s.is_empty()) {
                    parts.push(StatusLinePart::Flag(state.clone()));
                }
            }
            StatusLineSegment::Context => {
                if let Some(pct) = snap.context_pct {
                    // Claude BuiltinStatusLine: "Context N%"
                    parts.push(StatusLinePart::Flag(format!("Context {pct}%")));
                }
            }
            StatusLineSegment::Cwd => {
                if let Some(cwd) = snap.cwd_basename.as_ref().filter(|s| !s.is_empty()) {
                    parts.push(StatusLinePart::Flag(cwd.clone()));
                }
            }
            StatusLineSegment::Git => {
                if let Some(branch) = snap.git_branch.as_ref().filter(|s| !s.is_empty()) {
                    parts.push(StatusLinePart::Flag(branch.clone()));
                }
            }
        }
    }
    parts
}

/// Split selected parts into (`model_name`, flag texts) for `PromptInfo`.
///
/// The first `Model` part becomes `model_name`; any later `Model` parts become
/// flags. If there is no `Model` part, `model_name` is empty and everything is
/// a flag (still rendered by `render_info_line`).
pub fn split_prompt_info_parts(parts: &[StatusLinePart]) -> (String, Vec<String>) {
    let mut model_name = String::new();
    let mut flags = Vec::new();
    for part in parts {
        match part {
            StatusLinePart::Model(s) if model_name.is_empty() => model_name = s.clone(),
            StatusLinePart::Model(s) => flags.push(s.clone()),
            StatusLinePart::Flag(s) => flags.push(s.clone()),
        }
    }
    (model_name, flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_select_mode_model_reasoning_run_state_context() {
        let cfg = StatusLineConfig::default();
        assert!(cfg.enabled());
        assert_eq!(
            cfg.selected_segments(),
            vec![
                StatusLineSegment::Mode,
                StatusLineSegment::Model,
                StatusLineSegment::Context,
                StatusLineSegment::Reasoning,
                StatusLineSegment::RunState,
            ]
        );
    }

    #[test]
    fn disabled_selects_nothing() {
        let cfg = StatusLineConfig {
            enabled: Some(false),
            ..Default::default()
        };
        assert!(cfg.selected_segments().is_empty());
    }

    #[test]
    fn parse_order_ignores_unknown_and_dedupes() {
        let order = StatusLineConfig::parse_order(Some("context, model, context, bogon, git"));
        assert_eq!(order[0], StatusLineSegment::Context);
        assert_eq!(order[1], StatusLineSegment::Model);
        assert_eq!(order[2], StatusLineSegment::Git);
        assert!(order.contains(&StatusLineSegment::Mode));
        assert!(order.contains(&StatusLineSegment::Cwd));
    }

    #[test]
    fn parse_order_empty_falls_back_to_default() {
        assert_eq!(
            StatusLineConfig::parse_order(Some(",,,")),
            DEFAULT_STATUS_LINE_ORDER.to_vec()
        );
        assert_eq!(
            StatusLineConfig::parse_order(None),
            DEFAULT_STATUS_LINE_ORDER.to_vec()
        );
    }

    #[test]
    fn segment_toggles_filter_selection() {
        let cfg = StatusLineConfig {
            context: Some(false),
            cwd: Some(true),
            reasoning: Some(false),
            run_state: Some(false),
            order: Some("model,cwd,mode,context".into()),
            ..Default::default()
        };
        assert_eq!(
            cfg.selected_segments(),
            vec![
                StatusLineSegment::Model,
                StatusLineSegment::Cwd,
                StatusLineSegment::Mode,
            ]
        );
    }

    #[test]
    fn select_parts_builds_claude_like_density() {
        let cfg = StatusLineConfig::default();
        let snap = StatusLineSnapshot {
            model: "grok-4".into(),
            context_pct: Some(35),
            mode_labels: vec!["always-approve".into()],
            cwd_basename: Some("next-code".into()),
            git_branch: Some("dev".into()),
            ..Default::default()
        };
        let parts = select_status_line_parts(&cfg, &snap);
        assert_eq!(
            parts,
            vec![
                StatusLinePart::Flag("always-approve".into()),
                StatusLinePart::Model("grok-4".into()),
                StatusLinePart::Flag("Context 35%".into()),
            ]
        );
        let (model, flags) = split_prompt_info_parts(&parts);
        assert_eq!(model, "grok-4");
        assert_eq!(flags, vec!["always-approve", "Context 35%"]);
    }

    #[test]
    fn select_parts_omits_missing_optional_values() {
        let cfg = StatusLineConfig {
            cwd: Some(true),
            git: Some(true),
            mode: Some(false),
            ..Default::default()
        };
        let snap = StatusLineSnapshot {
            model: "m".into(),
            context_pct: None,
            cwd_basename: None,
            git_branch: Some("main".into()),
            mode_labels: vec!["plan".into()],
            ..Default::default()
        };
        let parts = select_status_line_parts(&cfg, &snap);
        assert_eq!(
            parts,
            vec![
                StatusLinePart::Model("m".into()),
                StatusLinePart::Flag("main".into()),
            ]
        );
    }

    #[test]
    fn serde_roundtrip_skips_defaults() {
        let cfg = StatusLineConfig {
            context: Some(false),
            order: Some("model,context".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&cfg).expect("serialize");
        assert_eq!(json.get("context"), Some(&serde_json::json!(false)));
        assert!(json.get("enabled").is_none());
        let back: StatusLineConfig = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cfg);
    }
}
