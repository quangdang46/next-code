//! Shared stubs for xai-grok-* crate compatibility.
//!
//! These are minimal stubs that let dependent crates (especially
//! `next-code-grok-render`) compile without the full Grok shell.

// ===========================================================================
// Top-level
// ===========================================================================

pub fn is_wsl() -> bool {
    false
}

pub fn dup_tui_stderr() -> Result<std::fs::File, std::io::Error> {
    std::fs::File::open("/dev/null")
}

/// Extension trait adding [`success()`] to `i32` (exit-code check).
pub trait ExitStatusExt {
    /// Returns `true` when the exit code indicates success (`== 0`).
    fn success(&self) -> bool;
}

impl ExitStatusExt for i32 {
    fn success(&self) -> bool {
        *self == 0
    }
}

/// Extension trait adding [`is_terminal()`] to `&mut dyn Write`.
///
/// The standard `std::io::IsTerminal` trait requires `AsFd`/`AsHandle`,
/// which `dyn Write` does not implement. This trait bridges the gap so the
/// render crate can query terminal status on the locked-stderr handle.
pub trait StderrIsTerminal {
    /// Returns `true` when the handle is a terminal / TTY.
    fn is_terminal(&self) -> bool;
}

impl StderrIsTerminal for dyn std::io::Write + '_ {
    fn is_terminal(&self) -> bool {
        false
    }
}

// ===========================================================================
// clipboard
// ===========================================================================

pub mod clipboard {
    /// Outcome of a native clipboard write attempt.
    ///
    /// Returned by [`set_text_with_outcome`] instead of a bare `Result`,
    /// so the render crate can inspect per-backend success/failure.
    #[derive(Debug, Clone)]
    pub struct ClipboardOutcome {
        /// At least one CLI tool succeeded.
        pub cli_ok: bool,
        /// The arboard (Rust native) backend succeeded.
        pub arboard_ok: bool,
        /// Wayland data-control protocol was available and succeeded.
        pub data_control: bool,
        /// Names of CLI tools that were tried (e.g. `["wl-copy"]`).
        pub cli_ok_tools: Vec<&'static str>,
        /// Names of CLI tools that were attempted.
        pub cli_tools_tried: Vec<&'static str>,
        /// Any backend succeeded.
        pub any_ok: bool,
    }

    /// Exit status of a child process.
    ///
    /// Returned by [`wait_with_deadline`] so callers can use `status_ok()`
    /// (analogous to `std::process::ExitStatus::success()`) and format it
    /// with `Display` without importing an extension trait.
    #[derive(Debug, Clone, Copy)]
    pub struct ProcessStatus(pub i32);

    impl ProcessStatus {
        /// Returns `true` when the exit code indicates success (`== 0`).
        pub fn success(&self) -> bool {
            self.0 == 0
        }
    }

    impl std::fmt::Display for ProcessStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }

    pub fn is_remote_session() -> bool {
        false
    }
    pub fn is_containerized_without_display() -> bool {
        false
    }
    pub fn wayland_data_control_supported() -> bool {
        false
    }
    pub fn spool_for_stdin(_: &[u8]) -> std::io::Result<std::fs::File> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "spool not available in stub",
        ))
    }
    pub fn wait_with_deadline(
        _: &mut std::process::Child,
        _: std::time::Duration,
    ) -> std::io::Result<ProcessStatus> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "wait_with_deadline not available in stub",
        ))
    }
    pub fn get_text() -> Result<Option<String>, String> {
        Ok(None)
    }
    pub fn set_text_with_outcome(_: &str) -> ClipboardOutcome {
        ClipboardOutcome {
            cli_ok: false,
            arboard_ok: false,
            data_control: false,
            cli_ok_tools: Vec::new(),
            cli_tools_tried: Vec::new(),
            any_ok: false,
        }
    }
    pub fn set_text_osc52(_: &str, _: bool) -> Result<(), String> {
        Ok(())
    }
    pub fn x11_display_env_present() -> bool {
        false
    }
    pub fn get_primary_text() -> Result<String, String> {
        Ok(String::new())
    }

    pub fn get_attachments() -> Result<AttachmentsProbeResult, String> {
        Ok(AttachmentsProbeResult {
            image: None,
            file_urls: None,
        })
    }

    /// Result of a clipboard attachments probe.
    pub struct AttachmentsProbeResult {
        pub image: Option<ImageData>,
        pub file_urls: Option<String>,
    }

    /// In-memory image from the clipboard.
    #[derive(Debug, Clone)]
    pub struct ImageData {
        pub data: Vec<u8>,
        pub mime_type: String,
    }

    /// Snapshot of clipboard state: `(change_count, has_pasteable_image)`.
    /// Off-macOS returns `(None, false)`.
    pub fn clipboard_image_snapshot() -> (Option<u64>, bool) {
        (None, false)
    }

    /// Cheap pasteboard `changeCount` read. `None` off-macOS.
    pub fn clipboard_change_count() -> Option<u64> {
        None
    }

    /// Whether the fast image probe exists on this platform.
    pub fn clipboard_image_probe_supported() -> bool {
        false
    }

    /// Prime the macOS AppKit dlopen on a background thread (no-op stub).
    pub fn clipboard_prewarm() {}

    /// Read an image from the system clipboard.
    pub fn get_image() -> Result<Option<ImageData>, String> {
        Ok(None)
    }

    /// Detach a std::process::Command from the process group (no-op stub).
    pub fn detach_std_command(_: &mut std::process::Command) {}

    /// Derive a file extension from a MIME type.
    pub fn mime_to_extension(_mime: &str) -> &'static str {
        ".bin"
    }

    /// Guess MIME type from raw bytes.
    pub fn mime_from_bytes(_data: &[u8]) -> &'static str {
        "application/octet-stream"
    }

    /// Name of the native clipboard tool (e.g. `wl-paste`, `pbpaste`).
    pub fn native_tool_name() -> String {
        String::new()
    }

    /// OSC 52 clipboard sink — tracks whether the terminal supports
    /// the escape-sequence clipboard protocol.
    pub enum OSC52Sink {
        /// Terminal supports OSC 52 writes.
        Active(Box<dyn std::io::Write>),
        /// Terminal does not support OSC 52.
        Inactive,
    }

    impl OSC52Sink {
        /// Returns `true` when the sink is active (terminal supports OSC 52).
        pub fn sink_active(&self) -> bool {
            matches!(self, OSC52Sink::Active(_))
        }
    }
}

// ===========================================================================
// session
// ===========================================================================

pub mod session {
    use std::path::PathBuf;

    pub struct SessionHandle;

    pub fn current_session() -> Option<SessionHandle> {
        None
    }

    /// Session identity information.
    pub mod info {
        /// Identity for a single session.
        pub struct Info<I = String> {
            pub id: I,
            pub cwd: String,
        }
    }

    /// Derive the session directory on disk from session identity.
    ///
    /// Receives `&info::Info` — the stub returns a `.grok`-style path
    /// relative to `cwd` when the `id` can be stringified, or `None`
    /// when it cannot.
    pub fn session_dir<I>(info: &info::Info<I>) -> PathBuf
    where
        I: std::fmt::Display,
    {
        // Real logic would be: cwd/.grok/sessions/<id>
        PathBuf::from(".")
            .join(".grok")
            .join("sessions")
            .join(format!("{}", info.id))
    }

    /// Terminal info for feedback submissions.
    pub struct FeedbackTerminalInfo {
        pub brand: String,
        pub multiplexer: String,
        pub is_ssh: bool,
        pub is_byobu: bool,
        pub term_var: String,
        pub tmux_version: Option<String>,
        pub hyperlink_osc8_support: Option<String>,
        pub clipboard_route: Option<String>,
        pub clipboard_native_tool: Option<String>,
        pub display_server: Option<String>,
    }
}

// ===========================================================================
// stderr
// ===========================================================================

pub mod stderr {
    use std::io::Write;

    /// Run a closure while holding the TUI stderr lock.
    ///
    /// The closure receives a `&mut std::io::Stderr` pointing to the render
    /// backend's stderr (which may be a dup'd fd, not `std::io::stderr()`).
    /// The concrete `Stderr` type lets callers use `IsTerminal::is_terminal()`
    /// (via `std::io::IsTerminal`) which is not available on `dyn Write`.
    pub fn with_locked_stderr<R>(f: impl FnOnce(&mut std::io::Stderr) -> R) -> R {
        f(&mut std::io::stderr())
    }

    /// Acquire the stderr lock, returning a guard that releases the lock
    /// when dropped.
    ///
    /// Returns `None` when locking fails or the lock is not available.
    pub fn stderr_lock() -> Option<Box<dyn Write>> {
        Some(Box::new(std::io::stderr()))
    }
}

// ===========================================================================
// placeholder_images
// ===========================================================================

pub mod placeholder_images {
    use std::path::{Path, PathBuf};

    /// Cap on total bytes loaded from orphan placeholders in one pass.
    pub const MAX_PLACEHOLDER_AGGREGATE_BYTES: usize = 10 * 1024 * 1024;

    /// A single `[Image #N: <path>]` placeholder parsed from text.
    #[derive(Debug, Clone)]
    pub struct Placeholder {
        /// The display number (`N` in `[Image #N]`).
        pub display_number: usize,
        /// The file path extracted from the placeholder.
        pub path: String,
        /// Byte-range span `(start, end)` in the original text.
        pub span: (usize, usize),
    }

    /// Result of loading a placeholder image from disk.
    #[derive(Debug, Clone)]
    pub struct LoadedPlaceholderImage {
        /// Raw image bytes.
        pub data: Vec<u8>,
        /// MIME type of the image.
        pub mime_type: String,
    }

    /// Errors during placeholder image loading.
    #[derive(Debug)]
    pub struct PlaceholderError(pub String);

    impl std::fmt::Display for PlaceholderError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for PlaceholderError {}

    /// Build a `[Image #N]` meta string for the given display number.
    ///
    /// The returned string is stored as the `meta` field on the wire
    /// `ImageContent` so the server can resolve the token by number.
    pub fn display_number_meta(display_number: usize) -> String {
        format!("[Image #{}]", display_number)
    }

    /// Extract a display number from a wire-format `meta` string.
    ///
    /// Returns `None` when the meta is absent or does not match the
    /// `[Image #N]` format produced by [`display_number_meta`].
    pub fn display_number_from_meta(meta: Option<&String>) -> Option<usize> {
        let s = meta?;
        let s = s.strip_prefix("[Image #")?;
        let s = s.strip_suffix(']')?;
        s.parse().ok()
    }

    /// Compute the set of allowed path prefixes from a workspace CWD.
    ///
    /// Returns the workspace directory itself plus common parent paths
    /// (home dir, etc.) so placeholders for user-owned files resolve.
    pub fn default_allowed_prefixes(workspace_cwd: &Path) -> Vec<PathBuf> {
        let mut prefixes = Vec::new();
        prefixes.push(workspace_cwd.to_path_buf());
        if let Ok(home) = std::env::var("HOME") {
            prefixes.push(PathBuf::from(home));
        }
        prefixes
    }

    /// Remove file paths from `[Image #N: <path>]` placeholders, leaving
    /// just `[Image #N]`.
    ///
    /// This prevents the model from reading the path directly when it
    /// already has the image bytes.
    pub fn strip_paths_from_image_placeholders(text: String) -> String {
        // Simple regex-free stub: keep everything unchanged.
        // Real impl strips the `: <path>` suffix from `[Image #N: <path>]`.
        text
    }

    /// Parse all `[Image #N: <path>]` placeholders from text.
    ///
    /// Returns a list of [`Placeholder`] structs.
    pub fn extract_placeholders(text: &str) -> Vec<Placeholder> {
        // Stub: no actual parsing. Returns empty vec.
        // Real impl scans for `[Image #N: <path>]` patterns.
        Vec::new()
    }

    /// Load an image from `path`, validating that it falls within one of
    /// the `allowed_prefixes`.
    ///
    /// Returns the raw bytes and the detected MIME type on success.
    pub fn load_placeholder_image(
        path: &str,
        allowed_prefixes: &[PathBuf],
    ) -> Result<LoadedPlaceholderImage, PlaceholderError> {
        let _ = allowed_prefixes; // unused in stub
        Err(PlaceholderError(format!(
            "stub: cannot load {}",
            path
        )))
    }
}

// ===========================================================================
// ui_config
// ===========================================================================

pub mod ui_config {
    use std::path::PathBuf;

    /// UI configuration for the pager.
    ///
    /// This stub matches the fields accessed by `next-code-grok-render`.
    /// Real values come from `config.toml` on disk.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct UiConfig {
        /// Compact mode (single-line turn headers).
        pub compact_mode: bool,
        /// Show timestamps on turn headers.
        pub show_timestamps: Option<bool>,
        /// Simple mode (no decoration).
        pub simple_mode: Option<bool>,
        /// Keep text selection behaviour (`"hold"`, `"flash"`, `"word_select"`).
        pub keep_text_selection: Option<String>,
        /// Vim keybindings.
        pub vim_mode: Option<bool>,
        /// Show thinking blocks.
        pub show_thinking_blocks: Option<bool>,
        /// Group tool verbs.
        pub group_tool_verbs: Option<bool>,
        /// Collapsed edit blocks (rollout flag).
        pub collapsed_edit_blocks: Option<bool>,
        /// Prompt autocomplete suggestions.
        pub prompt_suggestions: Option<bool>,
        /// Scroll mode string (e.g. `"auto"`, `"per_turn"`).
        pub scroll_mode: Option<String>,
        /// Invert scroll direction.
        pub invert_scroll: Option<bool>,
        /// Scroll lines per tick (0 = unset = per-terminal profile default).
        pub scroll_lines: Option<u8>,
        /// Selection highlight duration in ms (0 = hold).
        pub selection_highlight_duration_ms: Option<u64>,
        /// Double-click action.
        pub double_click_action: Option<String>,
        /// Scroll speed (1-100).
        pub scroll_speed: Option<u8>,
    }

    impl UiConfig {
        /// Default for the timeline sidebar toggle.
        pub const SHOW_TIMELINE_DEFAULT: bool = true;
        /// Default for page-flip-on-send toggle.
        pub const PAGE_FLIP_ON_SEND_DEFAULT: bool = false;

        /// Whether the timeline is enabled.
        pub fn show_timeline_enabled(&self) -> bool {
            Self::SHOW_TIMELINE_DEFAULT
        }

        /// Whether page-flip-on-send is enabled.
        pub fn page_flip_on_send_enabled(&self) -> bool {
            Self::PAGE_FLIP_ON_SEND_DEFAULT
        }

        /// Whether keep-text-selection is enabled (legacy bool path).
        pub fn keep_text_selection_enabled(&self) -> bool {
            // Legacy: when `keep_text_selection` is not set, default to flash
            // (not hold), unless `selection_highlight_duration_ms == 0` means
            // hold. The stub returns `false` (flash).
            self.selection_highlight_duration_ms == Some(0)
        }

        /// Directory for Grok configuration files.
        pub fn grok_config_dir() -> PathBuf {
            std::env::var("HOME")
                .ok()
                .map(PathBuf::from)
                .map(|h| h.join(".grok"))
                .unwrap_or_else(|| PathBuf::from(".grok"))
        }

        /// Whether this workspace context is worktree-aware.
        pub fn is_worktree_aware() -> bool {
            false
        }
    }

    impl Default for UiConfig {
        fn default() -> Self {
            Self {
                compact_mode: false,
                show_timestamps: Some(true),
                simple_mode: Some(true),
                keep_text_selection: None,
                vim_mode: Some(false),
                show_thinking_blocks: Some(true),
                group_tool_verbs: Some(true),
                collapsed_edit_blocks: Some(false),
                prompt_suggestions: Some(true),
                scroll_mode: None,
                invert_scroll: Some(false),
                scroll_lines: None,
                selection_highlight_duration_ms: None,
                double_click_action: None,
                scroll_speed: Some(50),
            }
        }
    }

}
