//! `/release-notes` -- view release notes for the current version.

use crate::app::actions::Action;
use crate::product_welcome::ProductWelcomeStatus;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Show release notes for the current pager version.
pub struct ReleaseNotesCommand;

/// Resolve next-code changelog DocViewer body from product welcome chrome.
pub(crate) fn nextcode_changelog_content(status: &ProductWelcomeStatus) -> String {
    if let Some(content) = status
        .changelog_markdown
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return content.to_string();
    }
    if !status.update_bullets.is_empty() {
        return status
            .update_bullets
            .iter()
            .map(|b| format!("- {b}"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    "No next-code changelog entries for this build.".to_string()
}

impl SlashCommand for ReleaseNotesCommand {
    fn name(&self) -> &str {
        "release-notes"
    }

    fn aliases(&self) -> &[&str] {
        &["changelog"]
    }

    fn description(&self) -> &str {
        "View release notes for the current version"
    }

    fn usage(&self) -> &str {
        "/release-notes"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        if crate::product_welcome::is_nextcode_embed() {
            // Prefer full embedded next-code changelog (build-meta), not grok CDN
            // and not only the welcome "unseen" bullets.
            let content = crate::product_welcome::product_welcome_status()
                .map(nextcode_changelog_content)
                .unwrap_or_else(|| "No next-code changelog entries for this build.".to_string());
            return CommandResult::Action(Action::ShowReleaseNotes {
                title: "Changelog".to_string(),
                content,
            });
        }
        let changelog = xai_grok_shell::util::changelog::ChangelogManager::new().fetch();
        match changelog.markdown {
            Some(content) => CommandResult::Action(Action::ShowReleaseNotes {
                title: "Release Notes".to_string(),
                content: content.trim().to_string(),
            }),
            None => CommandResult::Error("No release notes available (offline).".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_notes_metadata() {
        let cmd = ReleaseNotesCommand;
        assert_eq!(cmd.name(), "release-notes");
        assert_eq!(cmd.aliases(), &["changelog"]);
        assert!(!cmd.takes_args());
    }

    #[test]
    fn release_notes_returns_action_or_error() {
        let models = crate::acp::model_state::ModelState::default();
        let mut ctx = super::super::tests::make_ctx(&models);
        let result = ReleaseNotesCommand.run(&mut ctx, "");
        assert!(
            matches!(result, CommandResult::Action(_) | CommandResult::Error(_)),
            "expected Action or Error, got {result:?}"
        );
    }

    #[test]
    fn nextcode_prefers_full_markdown_over_unseen_bullets() {
        let status = ProductWelcomeStatus {
            update_bullets: vec!["only unseen".into()],
            changelog_markdown: Some("## v1.0\n\n- full history\n".into()),
            ..Default::default()
        };
        let content = nextcode_changelog_content(&status);
        assert!(content.contains("full history"));
        assert!(!content.contains("only unseen"));
    }

    #[test]
    fn nextcode_falls_back_to_unseen_bullets() {
        let status = ProductWelcomeStatus {
            update_bullets: vec!["unseen a".into(), "unseen b".into()],
            changelog_markdown: None,
            ..Default::default()
        };
        assert_eq!(
            nextcode_changelog_content(&status),
            "- unseen a\n- unseen b"
        );
    }
}
