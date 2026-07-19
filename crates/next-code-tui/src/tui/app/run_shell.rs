#![allow(clippy::drop_non_drop)]
use super::*;
use crate::tui::TuiState;
use crossterm::cursor::{RestorePosition, SavePosition};
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use ratatui::backend::{Backend, ClearType};
use ratatui::{buffer::Buffer, layout::Rect, style::Style};

const STATUS_SPINNER_FPS: f32 = 12.5;
pub(super) const STATUS_SPINNER_ONLY_INTERVAL: Duration = Duration::from_millis(80);

pub(super) fn redraw_timer(period: Duration) -> tokio::time::Interval {
    let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
    // Redraw ticks represent visual liveness, not elapsed simulation steps. An
    // immediate first tick or Burst catch-up after a slow frame only schedules
    // redundant full renders and can lock the UI into a slow-frame loop.
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

pub(super) fn status_spinner_interval() -> tokio::time::Interval {
    status_spinner_interval_after(STATUS_SPINNER_ONLY_INTERVAL)
}

pub(super) fn reset_status_spinner_interval(interval: &mut tokio::time::Interval, app: &App) {
    *interval = status_spinner_interval_after(status_spinner_delay_until_next_frame(
        status_spinner_elapsed(app),
    ));
}

fn status_spinner_interval_after(delay: Duration) -> tokio::time::Interval {
    let mut interval = tokio::time::interval_at(
        tokio::time::Instant::now() + delay,
        STATUS_SPINNER_ONLY_INTERVAL,
    );
    // The spinner is visual liveness, not simulated time. If terminal/input work delays a tick,
    // skip the missed frames instead of bursting them later.
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

fn status_spinner_elapsed(app: &App) -> f32 {
    status_spinner_elapsed_for_sources(app.elapsed().map(|duration| duration.as_secs_f32()))
}

fn status_spinner_elapsed_for_sources(turn_elapsed: Option<f32>) -> f32 {
    turn_elapsed.unwrap_or(0.0).max(0.0)
}

fn status_spinner_delay_until_next_frame(elapsed: f32) -> Duration {
    if !elapsed.is_finite() {
        return STATUS_SPINNER_ONLY_INTERVAL;
    }

    let frame_secs = STATUS_SPINNER_ONLY_INTERVAL.as_secs_f64();
    let elapsed_secs = f64::from(elapsed.max(0.0));
    let into_frame = elapsed_secs % frame_secs;
    let remaining = if into_frame <= f64::EPSILON {
        frame_secs
    } else {
        frame_secs - into_frame
    };

    Duration::from_secs_f64(remaining.max(0.001))
}

pub(super) fn status_spinner_only_symbol(app: &App) -> Option<&'static str> {
    // Claude Code-style layout: processing activity lives in the conversation
    // chrome above the input (`conversation_activity_line`), not on the status
    // bar. Patching a spinner cell into the status area re-creates the dual-
    // spinner look the user reported, so this fast path is intentionally off.
    let _ = app;
    None
}

/// Formerly true for statuses whose status line led with the green spinner.
/// Always false now — activity spinner moved out of the status bar.
pub(crate) fn status_uses_primary_spinner(status: &ProcessingStatus) -> bool {
    let _ = status;
    false
}

/// How the next full frame should invalidate ratatui's diff state, if at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FullFrameInvalidation {
    /// Physical clear + full re-emit. Needed when the real screen diverged
    /// from ratatui's model (native terminal scroll, external commands).
    /// Uses fork-safe clear without DSR cursor queries.
    HardClear,
    /// Sentinel-invalidate the previous buffer: full re-emit with no
    /// intermediate clear escape (issue #404 / ratatui #2357).
    SoftRepaint,
    /// Normal incremental diff.
    None,
}

/// Pure routing for `draw_full`: a hard clear supersedes a soft repaint.
pub(crate) fn full_frame_invalidation(
    force_full_redraw: bool,
    force_full_repaint: bool,
) -> FullFrameInvalidation {
    if force_full_redraw {
        FullFrameInvalidation::HardClear
    } else if force_full_repaint {
        FullFrameInvalidation::SoftRepaint
    } else {
        FullFrameInvalidation::None
    }
}

fn full_repaint_sentinel_cell() -> ratatui::buffer::Cell {
    let mut cell = ratatui::buffer::Cell::EMPTY;
    cell.set_symbol("\u{FDD0}");
    cell.fg = next_code_tui_style::Theme::current().bg_base;
    cell.bg = next_code_tui_style::Theme::current().bg_base;
    cell
}

/// Fill ratatui's "previous" buffer with sentinel cells so the next
/// `Terminal::draw` diff re-emits every cell without ED2 clear flicker.
pub(crate) fn invalidate_previous_terminal_buffer<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
) {
    terminal.swap_buffers();
    let sentinel = full_repaint_sentinel_cell();
    for cell in terminal.current_buffer_mut().content.iter_mut() {
        *cell = sentinel.clone();
    }
    terminal.swap_buffers();
}

#[derive(Default)]
pub(super) struct StatusSpinnerRenderer {
    last_frame: Option<Buffer>,
}

impl StatusSpinnerRenderer {
    pub(super) fn spinner_only_available(&self, app: &App) -> bool {
        status_spinner_only_symbol(app).is_some()
    }

    pub(super) fn invalidate(&mut self) {
        self.last_frame = None;
    }

    pub(super) fn draw_full(
        &mut self,
        app: &mut App,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        let invalidation = full_frame_invalidation(app.force_full_redraw, app.force_full_repaint);
        let force_full_redraw = invalidation != FullFrameInvalidation::None;
        // Wrap the whole frame (optional clear + diff flush) in a synchronized update so the
        // terminal applies every cell change atomically. Without this, ratatui's crossterm
        // backend streams cells one-by-one and eagerly-repainting terminals (and slow/remote or
        // multiplexed sessions) show visible flicker. See issue #282.
        let sync = crossterm::execute!(terminal.backend_mut(), BeginSynchronizedUpdate).is_ok();
        let mut cleared_for_full_redraw = false;
        let mut soft_repaint_armed = false;
        match invalidation {
            FullFrameInvalidation::HardClear => {
                // Never call Terminal::clear() here: it queries cursor position via DSR (CSI 6n)
                // and a 2s timeout was fatal for the client mid-session.
                if let Err(e) = clear_terminal_for_full_redraw(terminal) {
                    crate::logging::warn(&format!(
                        "Force full redraw clear failed ({e}); continuing with buffer reset only"
                    ));
                    force_full_buffer_redraw(terminal);
                }
                cleared_for_full_redraw = true;
                self.invalidate();
            }
            FullFrameInvalidation::SoftRepaint => {
                invalidate_previous_terminal_buffer(terminal);
                soft_repaint_armed = true;
                self.invalidate();
            }
            FullFrameInvalidation::None => {}
        }
        app.force_full_redraw = false;
        app.force_full_repaint = false;

        let previous_frame = self.last_frame.as_ref();
        let draw_start = Instant::now();
        let mut render_elapsed = Duration::ZERO;
        let completed = match terminal.draw(|frame| {
            let render_start = Instant::now();
            crate::tui::ui::draw(frame, app);
            render_elapsed = render_start.elapsed();
        }) {
            Ok(completed) => completed,
            Err(e) if is_cursor_position_timeout(&e) => {
                // Defensive: any residual DSR path should not kill the session.
                crate::logging::warn(&format!(
                    "Skipping frame after cursor-position timeout during draw ({e})"
                ));
                if sync {
                    let _ = crossterm::execute!(terminal.backend_mut(), EndSynchronizedUpdate);
                }
                // If we already cleared / reset buffers, re-arm so the next successful frame
                // still paints a full screen instead of leaving a half-applied redraw.
                if cleared_for_full_redraw {
                    app.force_full_redraw = true;
                }
                if soft_repaint_armed {
                    app.force_full_repaint = true;
                }
                return Ok(());
            }
            Err(e) => {
                if sync {
                    let _ = crossterm::execute!(terminal.backend_mut(), EndSynchronizedUpdate);
                }
                if cleared_for_full_redraw {
                    app.force_full_redraw = true;
                }
                if soft_repaint_armed {
                    app.force_full_repaint = true;
                }
                return Err(e.into());
            }
        };
        let total_elapsed = draw_start.elapsed();
        let changed_cells = previous_frame
            .filter(|previous| previous.area == completed.buffer.area)
            .map(|previous| {
                previous
                    .content
                    .iter()
                    .zip(completed.buffer.content.iter())
                    .filter(|(left, right)| left != right)
                    .count()
            });
        let total_cells = Some(completed.buffer.content.len());
        let completed_buffer = completed.buffer.clone();
        // `completed` borrows the terminal; it is unused past this point, so the
        // borrow ends here (NLL) before we touch the backend again below.
        if sync {
            let _ = crossterm::execute!(terminal.backend_mut(), EndSynchronizedUpdate);
        }
        crate::tui::ui::record_draw_call_attribution(crate::tui::ui::DrawCallAttribution {
            timestamp_ms: crate::tui::ui::wall_clock_ms(),
            total_ms: total_elapsed.as_secs_f64() * 1000.0,
            render_ms: render_elapsed.as_secs_f64() * 1000.0,
            backend_flush_ms: total_elapsed.saturating_sub(render_elapsed).as_secs_f64() * 1000.0,
            changed_cells,
            total_cells,
            force_full_redraw,
            input: crate::tui::ui::frame_input_attribution_snapshot(),
        });
        self.last_frame = Some(completed_buffer);
        Ok(())
    }

    pub(super) fn draw_status_spinner_only(
        &mut self,
        app: &App,
        terminal: &mut DefaultTerminal,
    ) -> Result<bool> {
        let status_symbol = status_spinner_only_symbol(app);
        if status_symbol.is_none() {
            return Ok(false);
        }
        let Some(previous_frame) = self.last_frame.as_ref() else {
            return Ok(false);
        };
        let status_area = crate::tui::ui::last_status_area();
        let status_patchable = status_symbol
            .zip(status_area)
            .is_some_and(|(symbol, area)| {
                render_status_spinner_into_buffer(previous_frame, area, symbol)
            });
        if !status_patchable {
            return Ok(false);
        }

        let next_frame = {
            let current_buffer = terminal.current_buffer_mut();
            current_buffer.clone_from(previous_frame);
            if let Some((symbol, area)) = status_symbol.zip(status_area)
                && status_patchable
            {
                render_status_spinner_into_buffer_mut(current_buffer, area, symbol);
            }
            current_buffer.clone()
        };

        // Keep ratatui's virtual buffers authoritative while preserving the user's cursor position.
        // The only terminal mutation outside ratatui here is cursor save/restore; cell contents still
        // go through Terminal::flush so the next full-frame diff remains synchronized. Wrap the
        // single-cell update in a synchronized update so it applies atomically (see issue #282).
        crossterm::queue!(
            terminal.backend_mut(),
            BeginSynchronizedUpdate,
            SavePosition
        )?;
        terminal.flush()?;
        crossterm::queue!(
            terminal.backend_mut(),
            RestorePosition,
            EndSynchronizedUpdate
        )?;
        terminal.swap_buffers();
        // Disambiguate: Backend and Write both expose flush on CrosstermBackend.
        Backend::flush(terminal.backend_mut())?;
        self.last_frame = Some(next_frame);
        Ok(true)
    }
}

/// Clear the physical terminal and force a full cell rewrite without DSR cursor queries.
///
/// `Terminal::clear()` calls `get_cursor_position()` (CSI `6n` / CPR) so it can restore the
/// cursor after clearing. On Unix that round-trip races with the event reader and times out
/// after ~2s with "The cursor position could not be read within a normal duration", which used
/// to exit the TUI client while the agent/server kept running. Fullscreen viewports do not
/// need cursor restore — clear the whole screen and reset ratatui's diff buffers instead.
fn clear_terminal_for_full_redraw(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // Physical clear without a cursor-position report.
    terminal.backend_mut().clear_region(ClearType::All)?;
    force_full_buffer_redraw(terminal);
    Ok(())
}

/// Reset ratatui's double buffers so the next flush paints every cell.
fn force_full_buffer_redraw(terminal: &mut DefaultTerminal) {
    terminal.current_buffer_mut().reset();
    // swap_buffers resets the new current buffer; previous becomes the blank we just reset.
    terminal.swap_buffers();
}

fn is_cursor_position_timeout(err: &std::io::Error) -> bool {
    is_cursor_position_timeout_msg(&err.to_string())
}

fn is_cursor_position_timeout_msg(msg: &str) -> bool {
    msg.to_ascii_lowercase()
        .contains("cursor position could not be read")
}

/// Draw one UI frame without treating cursor DSR timeouts as fatal.
///
/// Production call sites outside [`StatusSpinnerRenderer::draw_full`] (reconnect loops,
/// disconnected-event redraws, debug capture) used bare `terminal.draw()?`, which still
/// exited the client when crossterm's CSI `6n` round-trip timed out. Route those through
/// this helper so the session stays attached.
pub(in crate::tui::app) fn draw_ui_frame<B>(
    terminal: &mut ratatui::Terminal<B>,
    app: &dyn crate::tui::TuiState,
) -> Result<()>
where
    B: Backend,
    B::Error: std::fmt::Display,
{
    match terminal.draw(|frame| crate::tui::ui::draw(frame, app)) {
        Ok(_) => Ok(()),
        Err(e) if is_cursor_position_timeout_msg(&e.to_string()) => {
            crate::logging::warn(&format!(
                "Skipping frame after cursor-position timeout during draw ({e})"
            ));
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("terminal draw failed: {e}")),
    }
}

fn render_status_spinner_into_buffer(buffer: &Buffer, area: Rect, symbol: &str) -> bool {
    area.width > 0
        && area.height > 0
        && buffer.cell((area.x, area.y)).is_some()
        && !symbol.is_empty()
}

fn render_status_spinner_into_buffer_mut(buffer: &mut Buffer, area: Rect, symbol: &str) {
    buffer.set_stringn(
        area.x,
        area.y,
        symbol,
        1,
        // The spinner cell is patched outside the full-frame draw, so apply
        // light-theme adaptation here explicitly (no-op on dark themes).
        Style::default().fg(next_code_tui_style::adapt_color_for_theme(
            next_code_tui_style::theme::ai_color(),
        )),
    );
}

impl App {
    /// Run the TUI application
    /// Returns Some(session_id) if hot-reload was requested
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<RunResult> {
        let mut event_stream = EventStream::new();
        let mut redraw_period = crate::tui::redraw_interval(&self);
        let mut redraw_interval = redraw_timer(redraw_period);
        let mut status_spinner_interval = status_spinner_interval();
        let mut status_spinner_renderer = StatusSpinnerRenderer::default();
        let mut needs_redraw = true;
        let mut handterm_native_scroll =
            super::handterm_native_scroll::HandtermNativeScrollClient::connect_from_env();
        // Subscribe to bus for background task completion notifications
        let mut bus_receiver = Bus::global().subscribe();
        if let Some(status) = Bus::global().latest_update_status() {
            self.handle_update_status(status);
        }

        loop {
            self.sync_sleep_guard();
            let desired_redraw = crate::tui::redraw_interval(&self);
            if desired_redraw != redraw_period {
                redraw_period = desired_redraw;
                redraw_interval = redraw_timer(redraw_period);
            }

            if needs_redraw {
                status_spinner_renderer.draw_full(&mut self, &mut terminal)?;
                reset_status_spinner_interval(&mut status_spinner_interval, &self);
                if let Some(native) = handterm_native_scroll.as_mut() {
                    native.sync_from_app(&self);
                }
                needs_redraw = false;
            }

            // First-time agent snapshot check (runs once after initial draw)
            if !self.agent_snapshot_checked {
                super::inline_interactive::openers::check_agent_snapshots(&mut self);
                needs_redraw = true;
            }

            if self.should_quit {
                break;
            }

            // Process pending turn OR wait for input/redraw
            if self.pending_turn {
                self.pending_turn = false;
                // Process turn while still handling input
                self.process_turn_with_input(&mut terminal, &mut event_stream, &mut bus_receiver)
                    .await;
                needs_redraw = true;
            } else if self.pending_queued_dispatch {
                self.pending_queued_dispatch = false;
                self.process_queued_messages(&mut terminal, &mut event_stream)
                    .await;
                local::finish_turn(&mut self);
                needs_redraw = true;
            } else {
                // Wait for input or redraw tick
                tokio::select! {
                    _ = status_spinner_interval.tick(), if status_spinner_renderer.spinner_only_available(&self) => {
                        if !status_spinner_renderer.draw_status_spinner_only(&self, &mut terminal)? {
                            needs_redraw = true;
                        }
                    }
                    _ = redraw_interval.tick() => {
                        needs_redraw |= local::handle_tick(&mut self);
                    }
                    event = event_stream.next() => {
                        if event.is_some() {
                            needs_redraw |= local::handle_terminal_event(&mut self, &mut terminal, event)?;
                        } else {
                            tokio::time::sleep(redraw_period).await;
                        }
                    }
                    command = async {
                        match handterm_native_scroll.as_mut() {
                            Some(native) => native.recv().await,
                            None => futures::future::pending::<Option<super::handterm_native_scroll::HostToApp>>().await,
                        }
                    } => {
                        if let Some(command) = command {
                            self.apply_handterm_native_scroll(command);
                            self.request_full_redraw();
                            needs_redraw = true;
                        } else {
                            handterm_native_scroll = None;
                        }
                    }
                    // Handle background task completion notifications
                    bus_event = bus_receiver.recv() => {
                        needs_redraw |= local::handle_bus_event(&mut self, bus_event);
                    }
                }
            }
        }

        self.extract_session_memories().await;

        Ok(RunResult {
            reload_session: self.reload_requested.take(),
            rebuild_session: self.rebuild_requested.take(),
            update_session: self.update_requested.take(),
            restart_session: self.restart_requested.take(),
            exit_code: self.requested_exit_code,
            session_id: Some(self.session.id.clone()),
        })
    }

    /// Run the TUI in remote mode, connecting to a server
    pub async fn run_remote(
        mut self,
        mut terminal: DefaultTerminal,
        remote_working_dir: Option<String>,
    ) -> Result<RunResult> {
        let mut event_stream = EventStream::new();
        let mut redraw_period = crate::tui::redraw_interval(&self);
        let mut redraw_interval = redraw_timer(redraw_period);
        let mut status_spinner_interval = status_spinner_interval();
        let mut status_spinner_renderer = StatusSpinnerRenderer::default();
        let mut needs_redraw = true;
        let mut handterm_native_scroll =
            super::handterm_native_scroll::HandtermNativeScrollClient::connect_from_env();
        let mut remote_state = remote::RemoteRunState::default();

        'outer: loop {
            if self.display_messages.is_empty() {
                if self.server_spawning {
                    self.set_remote_startup_phase(super::RemoteStartupPhase::StartingServer);
                } else {
                    self.set_remote_startup_phase(super::RemoteStartupPhase::Connecting);
                }
            }
            if needs_redraw {
                status_spinner_renderer.draw_full(&mut self, &mut terminal)?;
                // Close the startup-profile gap: `pre_run_remote` is the last
                // pre-loop mark, so the first completed paint here is the real
                // process-to-first-frame point. Logged once via a static guard so
                // the end-to-end launch cost (including the ~5ms first draw) is
                // visible in the startup profile without re-marking every frame.
                {
                    use std::sync::atomic::{AtomicBool, Ordering};
                    static FIRST_FRAME_MARKED: AtomicBool = AtomicBool::new(false);
                    if !FIRST_FRAME_MARKED.swap(true, Ordering::Relaxed) {
                        crate::startup_profile::mark("first_frame");
                        crate::startup_profile::report_to_log();
                    }
                }
                reset_status_spinner_interval(&mut status_spinner_interval, &self);
                needs_redraw = false;
            }

            let session_to_resume = self.reconnect_target_session_id();

            let mut remote_conn = match remote::connect_with_retry(
                &mut self,
                &mut terminal,
                &mut event_stream,
                &mut remote_state,
                session_to_resume.as_deref(),
                remote_working_dir.as_deref(),
            )
            .await?
            {
                remote::ConnectOutcome::Connected(remote) => remote,
                remote::ConnectOutcome::Retry => continue,
                remote::ConnectOutcome::Quit => break 'outer,
            };
            status_spinner_renderer.invalidate();

            match remote::handle_post_connect(
                &mut self,
                &mut terminal,
                &mut remote_conn,
                &mut remote_state,
                session_to_resume.as_deref(),
            )
            .await?
            {
                remote::PostConnectOutcome::Ready => {}
                remote::PostConnectOutcome::Quit => break 'outer,
            }
            status_spinner_renderer.invalidate();
            needs_redraw = true;

            let mut bus_receiver_remote = Bus::global().subscribe();
            if let Some(status) = Bus::global().latest_update_status() {
                self.handle_update_status(status);
                needs_redraw = true;
            }

            // Main event loop
            loop {
                self.sync_sleep_guard();
                let desired_redraw = crate::tui::redraw_interval(&self);
                if desired_redraw != redraw_period {
                    redraw_period = desired_redraw;
                    redraw_interval = redraw_timer(redraw_period);
                }

                if needs_redraw {
                    status_spinner_renderer.draw_full(&mut self, &mut terminal)?;
                    reset_status_spinner_interval(&mut status_spinner_interval, &self);
                    if let Some(native) = handterm_native_scroll.as_mut() {
                        native.sync_from_app(&self);
                    }
                    needs_redraw = false;
                }

                if self.should_quit {
                    break 'outer;
                }

                if self.pending_queued_dispatch {
                    self.pending_queued_dispatch = false;
                    remote::process_remote_followups(&mut self, &mut remote_conn).await;
                    needs_redraw = true;
                    continue;
                }

                tokio::select! {
                    _ = status_spinner_interval.tick(), if status_spinner_renderer.spinner_only_available(&self) => {
                        if !status_spinner_renderer.draw_status_spinner_only(&self, &mut terminal)? {
                            needs_redraw = true;
                        }
                    }
                    _ = redraw_interval.tick() => {
                        needs_redraw |= remote::handle_tick(&mut self, &mut remote_conn).await;
                    }
                    event = remote_conn.next_event() => {
                        let (outcome, event_redraw) = remote::handle_remote_event(
                            &mut self,
                            &mut terminal,
                            &mut remote_conn,
                            &mut remote_state,
                            event,
                        )
                        .await?;
                        needs_redraw |= event_redraw;
                        match outcome {
                            remote::RemoteEventOutcome::Continue => {}
                            remote::RemoteEventOutcome::Reconnect => continue 'outer,
                            remote::RemoteEventOutcome::Quit => break 'outer,
                        }
                    }
                    event = event_stream.next() => {
                        if event.is_some() {
                            needs_redraw |= remote::handle_terminal_event(&mut self, &mut terminal, &mut remote_conn, event).await?;
                        } else {
                            tokio::time::sleep(redraw_period).await;
                        }
                    }
                    command = async {
                        match handterm_native_scroll.as_mut() {
                            Some(native) => native.recv().await,
                            None => futures::future::pending::<Option<super::handterm_native_scroll::HostToApp>>().await,
                        }
                    } => {
                        if let Some(command) = command {
                            self.apply_handterm_native_scroll(command);
                            self.request_full_redraw();
                            needs_redraw = true;
                        } else {
                            handterm_native_scroll = None;
                        }
                    }
                    bus_event = bus_receiver_remote.recv() => {
                        needs_redraw |= remote::handle_bus_event(&mut self, &mut remote_conn, bus_event).await;
                    }
                }
            }
        }

        Ok(RunResult {
            reload_session: self.reload_requested.take(),
            rebuild_session: self.rebuild_requested.take(),
            update_session: self.update_requested.take(),
            restart_session: self.restart_requested.take(),
            exit_code: self.requested_exit_code,
            session_id: if self.is_remote {
                self.remote_session_id.clone()
            } else {
                Some(self.session.id.clone())
            },
        })
    }

    /// Run the TUI in replay mode, playing back a timeline of events.
    pub async fn run_replay(
        self,
        terminal: DefaultTerminal,
        timeline: Vec<crate::replay::TimelineEvent>,
        speed: f64,
    ) -> Result<RunResult> {
        replay::run_replay(self, terminal, timeline, speed).await
    }

    /// Run an interactive swarm replay, rendering multiple sessions in tiled panes.
    pub async fn run_swarm_replay(
        terminal: DefaultTerminal,
        panes: Vec<crate::replay::PaneReplayInput>,
        speed: f64,
        centered_override: Option<bool>,
    ) -> Result<()> {
        replay::run_swarm_replay(terminal, panes, speed, centered_override).await
    }

    /// Run replay headlessly, rendering each frame to an in-memory buffer.
    /// Returns a list of (timestamp_secs, Buffer) pairs for video export.
    pub async fn run_headless_replay(
        mut self,
        timeline: &[crate::replay::TimelineEvent],
        speed: f64,
        width: u16,
        height: u16,
        fps: u32,
    ) -> Result<Vec<(f64, ratatui::buffer::Buffer)>> {
        use crate::replay::ReplayEvent;
        use ratatui::backend::TestBackend;

        let replay_events = crate::replay::timeline_to_replay_events(timeline);
        if replay_events.is_empty() {
            anyhow::bail!("No replay events to export");
        }

        let backend = TestBackend::new(width, height);
        let mut terminal = ratatui::Terminal::new(backend)?;
        let mut remote = crate::tui::backend::ReplayRemoteState::default();

        let frame_duration_ms: f64 = 1000.0 / fps as f64;
        let mut frames: Vec<(f64, ratatui::buffer::Buffer)> = Vec::new();
        let mut sim_time_ms: f64 = 0.0;
        let mut next_frame_at: f64 = 0.0;

        let total_duration_ms: f64 = replay_events.iter().map(|(d, _)| *d as f64 / speed).sum();

        let mut event_schedule: Vec<(f64, &ReplayEvent)> = Vec::new();
        {
            let mut abs_time: f64 = 0.0;
            for (delay_ms, evt) in &replay_events {
                abs_time += *delay_ms as f64 / speed;
                event_schedule.push((abs_time, evt));
            }
        }

        let mut event_cursor: usize = 0;
        let mut replay_turn_id: u64 = 0;

        terminal.draw(|f| crate::tui::render_frame(f, &self))?;
        frames.push((0.0, terminal.backend().buffer().clone()));

        let progress_interval = (total_duration_ms / 20.0).max(1000.0);
        let mut next_progress = progress_interval;

        while sim_time_ms <= total_duration_ms + frame_duration_ms {
            while event_cursor < event_schedule.len()
                && event_schedule[event_cursor].0 <= sim_time_ms
            {
                let (_t, event) = event_schedule[event_cursor];
                replay::apply_replay_event(
                    &mut self,
                    &mut remote,
                    event,
                    &mut replay_turn_id,
                    Some(sim_time_ms),
                );
                event_cursor += 1;
            }

            if sim_time_ms >= next_frame_at {
                replay::update_replay_elapsed_override(&mut self, sim_time_ms);
                terminal.draw(|f| crate::tui::render_frame(f, &self))?;
                frames.push((sim_time_ms / 1000.0, terminal.backend().buffer().clone()));
                next_frame_at = sim_time_ms + frame_duration_ms;
            }

            if sim_time_ms >= next_progress {
                let pct = (sim_time_ms / total_duration_ms * 100.0).min(100.0);
                eprint!("\r  Rendering... {:.0}%", pct);
                next_progress += progress_interval;
            }

            sim_time_ms += frame_duration_ms;
        }

        eprintln!("\r  Rendering... 100%  ({} frames captured)", frames.len());

        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn cursor_position_timeout_is_detected() {
        let err = std::io::Error::new(
            std::io::ErrorKind::Other,
            "The cursor position could not be read within a normal duration",
        );
        assert!(is_cursor_position_timeout(&err));
        assert!(is_cursor_position_timeout_msg(
            "The cursor position could not be read within a normal duration"
        ));
        // Case-insensitive: some wrappers rephrase / lowercase the message.
        assert!(is_cursor_position_timeout_msg(
            "error: the cursor position could not be read within a normal duration"
        ));
        let other = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Broken pipe");
        assert!(!is_cursor_position_timeout(&other));
        assert!(!is_cursor_position_timeout_msg("Broken pipe"));
    }

    #[test]
    fn force_full_buffer_redraw_empties_diff_base() {
        use ratatui::backend::TestBackend;
        use ratatui::widgets::Paragraph;

        let backend = TestBackend::new(10, 4);
        let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("hello"), frame.area());
            })
            .expect("draw");

        // Same helpers as production, but for TestBackend (no DSR).
        terminal.current_buffer_mut().reset();
        terminal.swap_buffers();

        // Previous buffer is blank, so a redraw of the same content still emits cells.
        let completed = terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("hello"), frame.area());
            })
            .expect("draw after reset");
        assert!(
            completed
                .buffer
                .content
                .iter()
                .any(|cell| cell.symbol() == "h"),
            "content should be present after buffer-forced full redraw"
        );
    }

    #[tokio::test]
    async fn redraw_timer_waits_one_period_and_skips_missed_ticks() {
        let mut timer = redraw_timer(Duration::from_millis(250));
        assert!(
            tokio::time::timeout(Duration::from_millis(20), timer.tick())
                .await
                .is_err(),
            "the first redraw tick must not fire immediately"
        );
        assert_eq!(
            timer.missed_tick_behavior(),
            tokio::time::MissedTickBehavior::Skip
        );
    }

    fn assert_duration_close(actual: Duration, expected: Duration) {
        let actual_ms = actual.as_millis() as i128;
        let expected_ms = expected.as_millis() as i128;
        assert!(
            (actual_ms - expected_ms).abs() <= 1,
            "expected {actual:?} to be within 1ms of {expected:?}"
        );
    }

    #[test]
    fn status_spinner_fast_path_uses_status_elapsed_clock() {
        let full_status_elapsed = 0.0;
        let app_lifetime_elapsed = 0.24;

        let full_status_symbol = next_code_tui_style::theme::activity_indicator(
            full_status_elapsed,
            STATUS_SPINNER_FPS,
            true,
        );
        let old_app_lifetime_symbol = next_code_tui_style::theme::activity_indicator(
            app_lifetime_elapsed,
            STATUS_SPINNER_FPS,
            true,
        );
        let fast_path_symbol = next_code_tui_style::theme::activity_indicator(
            status_spinner_elapsed_for_sources(Some(full_status_elapsed)),
            STATUS_SPINNER_FPS,
            true,
        );

        assert_ne!(
            old_app_lifetime_symbol, full_status_symbol,
            "the app lifetime clock can be on a different spinner frame than the status clock"
        );
        assert_eq!(fast_path_symbol, full_status_symbol);
    }

    #[test]
    fn primary_spinner_statuses_are_explicit() {
        // Status-bar primary spinner retired — conversation chrome owns activity.
        assert!(!status_uses_primary_spinner(&ProcessingStatus::Sending));
        assert!(!status_uses_primary_spinner(&ProcessingStatus::Streaming));
        assert!(!status_uses_primary_spinner(
            &ProcessingStatus::RunningTool("bash".to_string())
        ));
        assert!(!status_uses_primary_spinner(&ProcessingStatus::Idle));
        assert!(!status_uses_primary_spinner(
            &ProcessingStatus::WaitingForNetwork {
                listener: "network".to_string(),
            }
        ));
    }

    #[test]
    fn status_spinner_reset_targets_next_frame_boundary() {
        assert_duration_close(
            status_spinner_delay_until_next_frame(0.0),
            STATUS_SPINNER_ONLY_INTERVAL,
        );
        assert_duration_close(
            status_spinner_delay_until_next_frame(0.040),
            Duration::from_millis(40),
        );
        assert_duration_close(
            status_spinner_delay_until_next_frame(1.0),
            Duration::from_millis(40),
        );
        assert_duration_close(
            status_spinner_delay_until_next_frame(f32::NAN),
            STATUS_SPINNER_ONLY_INTERVAL,
        );
    }

    #[test]
    fn status_spinner_partial_mutates_only_status_cell() {
        let area = Rect::new(0, 0, 8, 2);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(0, 0, "abcdefgh", Style::default().fg(Color::White));
        buffer.set_string(0, 1, "ABCDEFGH", Style::default().fg(Color::Blue));
        let before = buffer.clone();

        let status_area = Rect::new(2, 1, 6, 1);
        assert!(render_status_spinner_into_buffer(&buffer, status_area, "⠂"));
        render_status_spinner_into_buffer_mut(&mut buffer, status_area, "⠂");

        for y in 0..2 {
            for x in 0..8 {
                if (x, y) == (2, 1) {
                    assert_eq!(buffer.cell((x, y)).unwrap().symbol(), "⠂");
                    assert_eq!(
                        buffer.cell((x, y)).unwrap().fg,
                        next_code_tui_style::theme::ai_color()
                    );
                } else {
                    assert_eq!(buffer.cell((x, y)), before.cell((x, y)));
                }
            }
        }
    }
}
