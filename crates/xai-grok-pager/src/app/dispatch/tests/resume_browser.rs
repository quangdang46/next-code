//! Dual-entry resume: bare `--resume` → ResumeBrowser; `/resume` → SessionPicker.

use super::*;
use crate::app::actions::{Action, Effect};
use crate::app::dispatch::router::dispatch;
use crate::views::modal::ActiveModal;

#[test]
fn show_resume_browser_sets_app_state_and_fetches_list() {
    let mut app = test_app();
    let effects = dispatch(Action::ShowResumeBrowser, &mut app);
    assert!(app.resume_browser.is_some());
    assert!(app.resume_browser.as_ref().unwrap().loading);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::FetchSessionList { .. })),
        "expected FetchSessionList, got {effects:?}"
    );
}

#[test]
fn show_session_picker_still_opens_expand_card_modal() {
    let mut app = test_app_with_agent();
    let effects = dispatch(Action::ShowSessionPicker, &mut app);
    assert!(app.resume_browser.is_none());
    assert!(matches!(
        get_active_agent(&app).and_then(|a| a.active_modal.as_ref()),
        Some(ActiveModal::SessionPicker { .. })
    ));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::FetchSessionList { .. }))
    );
}

#[test]
fn session_list_loaded_fills_resume_browser_not_welcome_picker() {
    let mut app = test_app();
    let _ = dispatch(Action::ShowResumeBrowser, &mut app);
    let seq = app.session_picker_list_seq;
    let entry = make_picker_entry("sess-1", "/repo");
    let effects = dispatch(
        Action::TaskComplete(TaskResult::SessionListLoaded {
            sessions: vec![entry],
            partial: None,
            seq,
            query: None,
        }),
        &mut app,
    );
    let rb = app.resume_browser.as_ref().expect("resume browser open");
    assert!(!rb.loading);
    assert_eq!(
        rb.selected_entry().map(|e| e.id.as_str()),
        Some("sess-1")
    );
    assert!(app.session_picker_entries.is_none());
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::LoadResumePreview { session_id, .. } if session_id == "sess-1")),
        "expected preview load, got {effects:?}"
    );
}

#[test]
fn close_resume_browser_clears_state() {
    let mut app = test_app();
    let _ = dispatch(Action::ShowResumeBrowser, &mut app);
    assert!(app.resume_browser.is_some());
    let _ = dispatch(Action::CloseResumeBrowser, &mut app);
    assert!(app.resume_browser.is_none());
}
