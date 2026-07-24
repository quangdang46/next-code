use std::io::{self, stdout};
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use xai_grok_pager::theme::Theme;
use xai_grok_pager::views::prompt_widget::StashedPrompt;
use xai_grok_pager::views::question_view::{
    QUESTION_VIEW_HPAD, QuestionViewState, question_view_height, render_question_view,
};
use xai_grok_tools::implementations::grok_build::ask_user_question::{Question, QuestionOption};

fn opt(label: &str, description: &str, preview: Option<&str>) -> QuestionOption {
    QuestionOption {
        label: label.into(),
        description: description.into(),
        preview: preview.map(str::to_string),
        id: None,
    }
}

fn q(
    question: &str,
    header: Option<&str>,
    multi_select: Option<bool>,
    options: Vec<QuestionOption>,
) -> Question {
    Question {
        question: question.into(),
        options,
        multi_select,
        header: header.map(str::to_string),
        id: None,
    }
}

/// Hardcoded example question sets for UI playground scenarios.
fn example_scenarios() -> Vec<(&'static str, Vec<Question>)> {
    vec![
        (
            "Commit confirmation (preview with multi-line message)",
            vec![q(
                "Ready to commit the staged changes with this conventional commit message?",
                Some("Commit"),
                None,
                vec![
                    opt(
                        "Yes, commit now",
                        "Run git commit with the message below and push to origin",
                        Some(
                            "fix(example-skills): resolve post-setup review findings\n\n\
                             - Move path resolution before vendor existence check (HIGH)\n\
                             - Improve awk parser to skip -b/-B args (MEDIUM)\n\n\
                             Addresses review-bot inline comments on PR #1001.",
                        ),
                    ),
                    opt("Edit message first", "Provide a different commit message", None),
                    opt("Cancel", "Do not commit yet", None),
                ],
            )],
        ),
        (
            "Multi-select with previews",
            vec![q(
                "Which database engines should we evaluate?\n\nSelect all that apply for the backend service.",
                Some("Engines"),
                Some(true),
                vec![
                    opt(
                        "PostgreSQL (Recommended)",
                        "Battle-tested relational DB with JSONB support",
                        Some("CREATE TABLE users (id SERIAL PRIMARY KEY);"),
                    ),
                    opt("SQLite", "Embedded, zero-config, single-file", None),
                    opt("Cassandra", "Wide-column store for large datasets", None),
                ],
            )],
        ),
        (
            "Multi-tab chips: radio + checkbox mix (←/→ or click chips)",
            vec![
                q(
                    "Which database engine should we use for the backend?",
                    Some("Database"),
                    None,
                    vec![
                        opt(
                            "PostgreSQL (Recommended)",
                            "Battle-tested relational DB with JSONB",
                            Some(
                                "CREATE TABLE users (\n  id SERIAL PRIMARY KEY,\n  email TEXT UNIQUE NOT NULL\n);",
                            ),
                        ),
                        opt("SQLite", "Embedded, zero-config, single-file", None),
                        opt("Cassandra", "Wide-column store for large datasets", None),
                    ],
                ),
                q(
                    "Which caching strategy do you want?",
                    Some("Cache"),
                    None,
                    vec![
                        opt(
                            "Redis",
                            "In-memory key-value store, distributed",
                            Some("SET session:abc123 '{\"user_id\": 42}' EX 3600"),
                        ),
                        opt(
                            "In-process LRU",
                            "No external dependency, per-instance cache",
                            None,
                        ),
                    ],
                ),
                q(
                    "Which features should be enabled at launch?\n\nSelect all that apply.",
                    Some("Features"),
                    Some(true),
                    vec![
                        opt("Auth", "JWT-based authentication middleware", None),
                        opt("Rate limiting", "Token bucket per API key", None),
                        opt("Audit logging", "Structured logs for compliance", None),
                        opt("Metrics", "Prometheus /metrics endpoint", None),
                    ],
                ),
            ],
        ),
    ]
}

struct App {
    scenarios: Vec<(&'static str, Vec<Question>)>,
    active_scenario: usize,
    state: QuestionViewState,
    theme: Theme,
    status: String,
}

impl App {
    fn new() -> Self {
        let scenarios = example_scenarios();
        let state = QuestionViewState::new(
            "playground".into(),
            scenarios[0].1.clone(),
            StashedPrompt::default(),
        );
        Self {
            scenarios,
            active_scenario: 0,
            state,
            theme: Theme::default(),
            status: "n/N switch scenario · ←/→ question · Enter select · Esc quit".into(),
        }
    }

    fn reload_scenario(&mut self) {
        let qs = self.scenarios[self.active_scenario].1.clone();
        self.state = QuestionViewState::new("playground".into(), qs, StashedPrompt::default());
    }
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .split(area);

            let title = format!(
                " AskUserQuestion playground [{}/{}] — {} ",
                app.active_scenario + 1,
                app.scenarios.len(),
                app.scenarios[app.active_scenario].0
            );
            let header = Paragraph::new(Line::from(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            let content_w = chunks[1].width.saturating_sub(QUESTION_VIEW_HPAD) as usize;
            let qv_h = question_view_height(&mut app.state, chunks[1].height, content_w);
            let q_area = ratatui::layout::Rect {
                x: chunks[1].x,
                y: chunks[1].y,
                width: chunks[1].width,
                height: qv_h.min(chunks[1].height),
            };
            let _ = render_question_view(
                frame.buffer_mut(),
                q_area,
                &app.state,
                None,
                &app.theme,
                true,
            );

            let footer = Paragraph::new(app.status.as_str())
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::TOP));
            frame.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
        {
            match code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Char('n') if modifiers.is_empty() => {
                    app.active_scenario = (app.active_scenario + 1) % app.scenarios.len();
                    app.reload_scenario();
                }
                KeyCode::Char('N') | KeyCode::Char('p') => {
                    if app.active_scenario == 0 {
                        app.active_scenario = app.scenarios.len() - 1;
                    } else {
                        app.active_scenario -= 1;
                    }
                    app.reload_scenario();
                }
                KeyCode::Left => app.state.prev_question(),
                KeyCode::Right => app.state.next_question(),
                KeyCode::Up => {
                    let c = app.state.cursor();
                    app.state.set_cursor(c.saturating_sub(1));
                }
                KeyCode::Down => {
                    let c = app.state.cursor();
                    app.state.set_cursor(c + 1);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let idx = app.state.cursor();
                    let active = app.state.active_tab;
                    let opt_count = app
                        .state
                        .questions
                        .get(active)
                        .map(|q| q.options.len())
                        .unwrap_or(0);
                    if idx < opt_count {
                        app.state.select_option(active, idx);
                    }
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
