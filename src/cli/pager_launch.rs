//! Launch Grok Face (`xai-grok-pager`) as the interactive next-code UI.
//!
//! Server bootstrap stays in `dispatch::run_default_command`. This module only
//! maps next-code CLI args → `PagerArgs` and installs the next-code ACP agent
//! factory so Face talks to the daemon brain (not the Grok `MvpAgent` stub).

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::rc::Rc;

use super::pager_agent::NextCodeFaceAgent;
use crate::setup_hints;

/// Env escape hatch: set to `1`/`true` to keep the legacy `next-code-tui` client.
pub(crate) fn legacy_tui_requested() -> bool {
    match std::env::var("NEXT_CODE_LEGACY_TUI") {
        Ok(v) => {
            let v = v.trim();
            v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

pub(crate) async fn run_face_pager(
    resume_session: Option<String>,
    _startup_hints: Option<setup_hints::StartupHints>,
    remote_working_dir: Option<String>,
) -> Result<()> {
    // Force direct in-process ACP spawn so our factory is used (not grok leader).
    let mut pager_args = xai_grok_pager::app::PagerArgs::try_parse_from(["grok"])
        .map_err(|e| anyhow::anyhow!("failed to build Face pager args: {e}"))?;
    pager_args.no_leader = true;

    if let Some(dir) = remote_working_dir.filter(|s| !s.trim().is_empty()) {
        pager_args.cwd = Some(PathBuf::from(dir));
    }
    match resume_session {
        Some(id) if id.is_empty() => {
            pager_args.continue_last_session = true;
        }
        Some(id) => {
            pager_args.resume_session = Some(id);
        }
        None => {}
    }

    xai_grok_pager::acp::spawn::install_agent_factory(Box::new(|client_tx| {
        let gateway = xai_acp_lib::AcpGatewaySender::new(client_tx);
        Ok(Rc::new(NextCodeFaceAgent::new(gateway)) as Rc<dyn agent_client_protocol::Agent>)
    }));

    let relaunch = xai_grok_pager::app::run(pager_args, None).await?;
    // Opt-in only: Face still writes face-quit-diag.log; do not spam the quit
    // tail unless the operator asked for the path.
    if std::env::var_os("NEXT_CODE_FACE_QUIT_DIAG").is_some() {
        let diag = std::env::var_os("NEXT_CODE_FACE_QUIT_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                #[cfg(windows)]
                {
                    if let Some(la) = std::env::var_os("LOCALAPPDATA") {
                        return PathBuf::from(la).join("next-code").join("face-quit-diag.log");
                    }
                }
                PathBuf::from(".next-code").join("face-quit-diag.log")
            });
        eprintln!("Face quit diag: {}", diag.display());
    }
    if relaunch {
        eprintln!("Update accepted. Relaunch `next-code` to continue.");
    }
    Ok(())
}
