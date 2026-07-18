//! Sponsored discovery disclosure UI — disabled in open-source builds.

use super::App;
use crate::message::ToolCall;

impl App {
    pub(in crate::tui::app) fn inline_sponsor_disclosure_title(
        &mut self,
        _tool: &ToolCall,
    ) -> Option<String> {
        let _ = self.sponsor_disclosure_shown_this_session;
        None
    }
}

fn should_disclose(_shown: bool, _tool: &ToolCall) -> bool {
    false
}
