//! Agent tree data + rendering — Claude Code style tree in conversation.
//!
//! Stores running agent/subagent state on the App, separate from DisplayMessage.
//! Rendered inline as a conversation section (like streaming text or batch progress).

use ratatui::prelude::*;
use crate::tui::color_support::rgb as rgb_color;

/// Status of an agent in the agent tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Running,
    Completed,
    Failed,
    Stopped,
    Idle,
}

/// A single node in the agent tree (can have children).
#[derive(Debug, Clone)]
pub struct AgentTreeNode {
    pub agent_name: String,
    pub status: AgentStatus,
    pub tool_use_count: u32,
    pub token_count: u64,
    pub is_leaf: bool,
    pub is_leader: bool,
    pub children: Vec<AgentTreeNode>,
    pub session_id: Option<String>,
    pub activity: Option<String>,
}

/// Colors for agent tree rendering.
const AGENT_TREE_COLOR: (u8, u8, u8) = (120, 180, 255);
const AGENT_CHILD_COLORS: &[(u8, u8, u8)] = &[
    (80, 220, 100),
    (255, 180, 80),
    (200, 140, 255),
    (80, 200, 230),
    (255, 150, 150),
];
const DIM_COLOR: (u8, u8, u8) = (100, 100, 110);
const ERROR_COLOR: (u8, u8, u8) = (255, 100, 100);
const SUCCESS_COLOR: (u8, u8, u8) = (100, 180, 100);

/// Render the agent tree into a Vec of styled lines.
pub fn render(trees: &[AgentTreeNode]) -> Vec<Line<'static>> {
    if trees.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();

    for tree in trees {
        render_node(tree, 0, tree.is_leader, &mut lines);
    }

    lines
}

fn render_node(node: &AgentTreeNode, depth: usize, is_leader: bool, out: &mut Vec<Line<'static>>) {
    let status_c = status_color(&node.status);
    let name_color = if is_leader {
        rgb_color(AGENT_TREE_COLOR.0, AGENT_TREE_COLOR.1, AGENT_TREE_COLOR.2)
    } else {
        let idx = depth.min(AGENT_CHILD_COLORS.len() - 1);
        let c = AGENT_CHILD_COLORS[idx];
        rgb_color(c.0, c.1, c.2)
    };

    let prefix = if depth == 0 && is_leader {
        "╒═ "
    } else if depth == 0 {
        "┌─ "
    } else {
        "├─ "
    };

    let mut spans: Vec<Span<'static>> = vec![
        Span::raw("  ".repeat(depth + 1)),
        Span::styled(prefix, Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2))),
        Span::styled(
            if depth == 0 && is_leader {
                node.agent_name.clone()
            } else {
                format!("@{}", node.agent_name)
            },
            Style::default().fg(name_color).add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(ref activity) = node.activity {
        spans.push(Span::styled(
            format!(": {}", activity),
            Style::default().fg(status_c),
        ));
    }
    if node.tool_use_count > 0 {
        spans.push(Span::styled(
            format!(" · {} tool {}", node.tool_use_count, if node.tool_use_count == 1 { "use" } else { "uses" }),
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ));
    }
    if node.token_count > 0 {
        spans.push(Span::styled(
            format!(" · {} tokens", node.token_count),
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ));
    }

    out.push(Line::from(spans));

    // Render children with continuation markers
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == node.children.len() - 1;
        let child_d = depth + 1;
        let child_color = {
            let idx = child_d.min(AGENT_CHILD_COLORS.len() - 1);
            let c = AGENT_CHILD_COLORS[idx];
            rgb_color(c.0, c.1, c.2)
        };
        let cs = status_color(&child.status);
        let tree_char = if is_last { "└─ " } else { "├─ " };

        let mut spans: Vec<Span<'static>> = vec![
            Span::raw("  ".repeat(child_d)),
            Span::styled(tree_char, Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2))),
            Span::styled(
                format!("@{}", child.agent_name),
                Style::default().fg(child_color).add_modifier(Modifier::BOLD),
            ),
        ];

        if let Some(ref activity) = child.activity {
            spans.push(Span::styled(
                format!(": {}", activity),
                Style::default().fg(cs),
            ));
        }
        if child.tool_use_count > 0 {
            spans.push(Span::styled(
                format!(" · {} tool {}", child.tool_use_count, if child.tool_use_count == 1 { "use" } else { "uses" }),
                Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
            ));
        }
        if child.token_count > 0 {
            spans.push(Span::styled(
                format!(" · {} tokens", child.token_count),
                Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
            ));
        }

        out.push(Line::from(spans));
    }
}

fn status_color(status: &AgentStatus) -> Color {
    match status {
        AgentStatus::Running => Color::Rgb(220, 220, 230),
        AgentStatus::Completed => Color::Rgb(SUCCESS_COLOR.0, SUCCESS_COLOR.1, SUCCESS_COLOR.2),
        AgentStatus::Failed => Color::Rgb(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2),
        AgentStatus::Stopped => Color::Rgb(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2),
        AgentStatus::Idle => Color::Rgb(180, 180, 190),
    }
}
