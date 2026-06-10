//! Goal management backed by beads_rust Epic issues.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::side_panel::{SidePanelSnapshot, focus_page, snapshot_for_session, write_markdown_page};

// Re-export beads-backed types
pub use jcode_beads_bridge::mapping::{Goal, GoalMilestone, GoalStep, ToBeadsEpic, ToJcodeGoal};

/// Goal status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Draft,
    #[default]
    Active,
    Paused,
    Blocked,
    Completed,
    Archived,
    Abandoned,
}

impl GoalStatus {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Some(Self::Draft),
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "blocked" => Some(Self::Blocked),
            "completed" => Some(Self::Completed),
            "archived" => Some(Self::Archived),
            "abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Archived => "archived",
            Self::Abandoned => "abandoned",
        }
    }

    pub fn sort_rank(self) -> u8 {
        match self {
            Self::Active => 0,
            Self::Blocked => 1,
            Self::Draft => 2,
            Self::Paused => 3,
            Self::Completed => 4,
            Self::Archived => 5,
            Self::Abandoned => 6,
        }
    }

    pub fn is_resumable(self) -> bool {
        matches!(self, Self::Active | Self::Blocked | Self::Draft)
    }
}

/// Goal update record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoalUpdate {
    pub at: DateTime<Utc>,
    pub summary: String,
}

/// Goal display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalDisplayMode {
    Auto,
    Focus,
    UpdateOnly,
    None,
}

impl GoalDisplayMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "focus" => Some(Self::Focus),
            "update_only" => Some(Self::UpdateOnly),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

/// Goal creation input.
#[derive(Debug, Clone, Default)]
pub struct GoalCreateInput {
    pub id: Option<String>,
    pub title: String,
    pub scope: GoalScope,
    pub description: Option<String>,
    pub why: Option<String>,
    pub success_criteria: Vec<String>,
    pub milestones: Vec<GoalMilestone>,
    pub next_steps: Vec<String>,
    pub blockers: Vec<String>,
    pub current_milestone_id: Option<String>,
    pub progress_percent: Option<u8>,
}

/// Goal update input.
#[derive(Debug, Clone, Default)]
pub struct GoalUpdateInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub why: Option<String>,
    pub status: Option<GoalStatus>,
    pub success_criteria: Option<Vec<String>>,
    pub milestones: Option<Vec<GoalMilestone>>,
    pub next_steps: Option<Vec<String>>,
    pub blockers: Option<Vec<String>>,
    pub current_milestone_id: Option<Option<String>>,
    pub progress_percent: Option<Option<u8>>,
    pub checkpoint_summary: Option<String>,
}

/// Goal scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalScope {
    Global,
    #[default]
    Project,
}

impl GoalScope {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "global" => Some(Self::Global),
            "project" => Some(Self::Project),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Project => "project",
        }
    }
}

/// Display result for a goal.
#[derive(Debug, Clone)]
pub struct GoalDisplayResult {
    pub goal: Goal,
    pub snapshot: SidePanelSnapshot,
}

// ─── Beads-backed operations ───────────────────────────────────────────────

fn open_beads(working_dir: Option<&Path>) -> Result<jcode_beads_bridge::BeadsProject> {
    let wd = match working_dir {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().context("no cwd")?,
    };
    jcode_beads_bridge::BeadsProject::open(&wd)
        .map_err(|e| anyhow::anyhow!("open beads project: {e}"))
}

/// Create a goal (Epic issue) in beads_rust storage.
pub fn create_goal(input: GoalCreateInput, working_dir: Option<&Path>) -> Result<Goal> {
    let project = open_beads(working_dir)?;
    let id = input
        .id
        .clone()
        .unwrap_or_else(|| format!("goal-{}", short_id()));

    let goal = Goal {
        id,
        title: input.title,
        scope: GoalScope::Project.as_str().to_string(),
        status: GoalStatus::Active.as_str().to_string(),
        description: input.description.unwrap_or_default(),
        why: input.why.unwrap_or_default(),
        milestones: input.milestones,
        next_steps: input.next_steps,
        blockers: input.blockers,
        progress_percent: input.progress_percent,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let issue = goal.to_epic();
    project.storage_mut().create_issue(&issue, "jcode")?;
    project.flush()?;
    Ok(goal)
}

/// Update a goal.
pub fn update_goal(
    id: &str,
    _scope_hint: Option<GoalScope>,
    working_dir: Option<&Path>,
    input: GoalUpdateInput,
) -> Result<Option<Goal>> {
    let project = open_beads(working_dir)?;
    use beads_rust::storage::sqlite::IssueUpdate;
    let mut update = IssueUpdate::default();
    if let Some(title) = &input.title {
        update.title = Some(title.clone());
    }
    if let Some(status) = input.status {
        let s = match status {
            GoalStatus::Active => beads_rust::model::Status::InProgress,
            GoalStatus::Blocked => beads_rust::model::Status::Blocked,
            GoalStatus::Completed => beads_rust::model::Status::Closed,
            GoalStatus::Draft => beads_rust::model::Status::Draft,
            GoalStatus::Paused | GoalStatus::Archived | GoalStatus::Abandoned => {
                beads_rust::model::Status::Deferred
            }
        };
        update.status = Some(s);
    }
    if let Some(desc) = &input.description {
        update.description = Some(Some(desc.clone()));
    }
    project.storage_mut().update_issue(id, &update, "jcode")?;
    project.flush()?;
    load_goal(id, None, working_dir)
}

/// Load a single goal from beads_rust storage.
pub fn load_goal(
    id: &str,
    _scope_hint: Option<GoalScope>,
    working_dir: Option<&Path>,
) -> Result<Option<Goal>> {
    let project = match open_beads(working_dir) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    Ok(project
        .storage()
        .get_issue(id)
        .ok()
        .flatten()
        .map(|i| i.to_goal()))
}

/// List all relevant goals (Epic issues).
pub fn list_relevant_goals(working_dir: Option<&Path>) -> Result<Vec<Goal>> {
    let project = open_beads(working_dir)?;
    let filters = beads_rust::storage::ListFilters {
        types: Some(vec![beads_rust::model::IssueType::Epic]),
        ..Default::default()
    };
    let issues = project.storage().list_issues(&filters)?;
    Ok(issues.into_iter().map(|i| i.to_goal()).collect())
}

/// Resume a goal for a session.
pub fn resume_goal(session_id: &str, working_dir: Option<&Path>) -> Result<Option<Goal>> {
    let goals = list_relevant_goals(working_dir)?;
    Ok(goals
        .into_iter()
        .find(|g| g.id == session_id || g.title.contains(session_id)))
}

/// Attach a goal to a session (adds a label).
pub fn attach_goal_to_session(
    session_id: &str,
    goal: &Goal,
    working_dir: Option<&Path>,
) -> Result<()> {
    let project = open_beads(working_dir)?;
    project
        .storage_mut()
        .add_label(&goal.id, &format!("session:{session_id}"), "jcode")?;
    project.flush()?;
    Ok(())
}

/// Load the goal attached to a session.
pub fn load_attached_goal(session_id: &str, working_dir: Option<&Path>) -> Result<Option<Goal>> {
    let project = open_beads(working_dir)?;
    let issues = project
        .storage()
        .list_issues(&beads_rust::storage::ListFilters {
            types: Some(vec![beads_rust::model::IssueType::Epic]),
            labels: Some(vec![format!("session:{session_id}")]),
            ..Default::default()
        })?;
    Ok(issues.into_iter().next().map(|i| i.to_goal()))
}

// ─── Side-panel / UI helpers (beads-aware) ─────────────────────────────────

/// Open goals overview in side panel.
pub fn open_goals_overview_for_session(
    session_id: &str,
    working_dir: Option<&Path>,
    focus: bool,
) -> Result<SidePanelSnapshot> {
    let goals = list_relevant_goals(working_dir)?;
    let content = format_goals_overview(&goals);
    let page_id = "goals";
    write_markdown_page(session_id, page_id, Some("Goals (beads)"), &content, focus)?;
    if focus {
        focus_page(session_id, page_id)
    } else {
        snapshot_for_session(session_id)
    }
}

/// Refresh goals overview in side panel.
pub fn refresh_goals_overview_for_session(
    session_id: &str,
    working_dir: Option<&Path>,
) -> Result<Option<SidePanelSnapshot>> {
    let snapshot = snapshot_for_session(session_id)?;
    if snapshot.pages.iter().any(|page| page.id == "goals") {
        open_goals_overview_for_session(session_id, working_dir, false).map(Some)
    } else {
        Ok(None)
    }
}

/// Write goal detail page to side panel.
pub fn write_goal_page(
    session_id: &str,
    working_dir: Option<&Path>,
    goal: &Goal,
    display: GoalDisplayMode,
) -> Result<SidePanelSnapshot> {
    let _ = (working_dir, display);
    let content = format_goal_detail(goal);
    let page_id = goal_page_id(&goal.id);
    write_markdown_page(
        session_id,
        &page_id,
        Some(&format!("Epic: {}", goal.title)),
        &content,
        true,
    )?;
    focus_page(session_id, &page_id)
}

/// Open a goal detail in the side panel.
pub fn open_goal_for_session(
    session_id: &str,
    working_dir: Option<&Path>,
    id: &str,
    explicit_focus: bool,
) -> Result<Option<GoalDisplayResult>> {
    let goal = match load_goal(id, None, working_dir)? {
        Some(g) => g,
        None => return Ok(None),
    };
    let snapshot = write_goal_page(
        session_id,
        working_dir,
        &goal,
        if explicit_focus {
            GoalDisplayMode::Focus
        } else {
            GoalDisplayMode::Auto
        },
    )?;
    Ok(Some(GoalDisplayResult { goal, snapshot }))
}

/// Resume a goal and show it in the side panel.
pub fn resume_goal_for_session(
    session_id: &str,
    working_dir: Option<&Path>,
    explicit_focus: bool,
) -> Result<Option<GoalDisplayResult>> {
    let goal = match resume_goal(session_id, working_dir)? {
        Some(g) => g,
        None => return Ok(None),
    };
    let snapshot = write_goal_page(
        session_id,
        working_dir,
        &goal,
        if explicit_focus {
            GoalDisplayMode::Focus
        } else {
            GoalDisplayMode::Auto
        },
    )?;
    Ok(Some(GoalDisplayResult { goal, snapshot }))
}

pub fn goal_page_id(id: &str) -> String {
    format!("goal-{id}")
}

pub fn header_badge(
    working_dir: Option<&Path>,
    snapshot: &crate::side_panel::SidePanelSnapshot,
) -> Option<String> {
    if let Some(page) = snapshot.focused_page()
        && page.id.starts_with("goal-")
    {
        return Some(format!("🎯 beads:{}", page.title));
    }
    let goals = list_relevant_goals(working_dir).ok()?;
    if goals.is_empty() {
        return None;
    }
    let active = goals.iter().filter(|g| g.status == "active").count();
    if active > 0 {
        Some(format!("🎯 {} active", active))
    } else {
        Some(format!("🎯 {} goals", goals.len()))
    }
}

pub fn render_goals_overview(goals: &[Goal]) -> String {
    format_goals_overview(goals)
}

pub fn render_goal_detail(goal: &Goal) -> String {
    format_goal_detail(goal)
}

// ─── Rendering helpers ─────────────────────────────────────────────────────

fn format_goals_overview(goals: &[Goal]) -> String {
    if goals.is_empty() {
        return "# Goals\n\nNo goals found. Use `initiative` tool to create one.".to_string();
    }
    let mut html = "# Goals (beads)\n\n".to_string();
    for goal in goals {
        html.push_str(&format!(
            "- **{}** [{}] — {}\n",
            goal.title, goal.status, goal.description
        ));
    }
    html
}

fn format_goal_detail(goal: &Goal) -> String {
    format!(
        "# {}\n\n**Status:** {} | **Scope:** {}\n\n{}\n\n---\n*beads-backed epic: {}*",
        goal.title, goal.status, goal.scope, goal.description, goal.id
    )
}

// ─── Helper ────────────────────────────────────────────────────────────────

fn short_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos & 0xFFFF_FFFF)
}
