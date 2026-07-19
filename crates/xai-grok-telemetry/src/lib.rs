pub mod client {
    pub fn is_enabled() -> bool { false }
}

pub mod session_ctx {
    use crate::events::TelemetryEvent;

    pub fn log_event<T: TelemetryEvent>(_: T) {}
}

pub mod events {
    pub trait TelemetryEvent {}

    /// Clipboard copy event.
    pub struct ClipboardCopy {
        pub terminal: crate::events::TerminalTelemetry,
        pub source: &'static str,
        pub text_len: u64,
        pub route_native: bool,
        pub route_tmux: bool,
        pub route_osc52: bool,
        pub route_label: String,
        pub cli_tools_tried: String,
        pub cli_ok_tools: String,
        pub cli_ok: bool,
        pub arboard_ok: bool,
        pub data_control: bool,
        pub tmux_ok: bool,
        pub osc52_ok: bool,
        pub delivery: &'static str,
        pub osc52_sink: bool,
        pub container_no_display: bool,
        pub reported_success: bool,
        pub toast_kind: &'static str,
        pub duration_ms: u64,
    }
    impl TelemetryEvent for ClipboardCopy {}

    /// Paste key with empty host clipboard event.
    pub struct PasteKeyEmptyHostClipboard {
        pub terminal: crate::events::TerminalTelemetry,
        pub surface: String,
    }
    impl TelemetryEvent for PasteKeyEmptyHostClipboard {}

    /// Clipboard image paste event.
    pub struct ClipboardImagePaste {
        pub terminal: crate::events::TerminalTelemetry,
        pub probe: String,
        pub outcome: String,
        pub image_mime: String,
        pub duration_ms: u64,
    }
    impl TelemetryEvent for ClipboardImagePaste {}

    /// Flat snapshot of terminal details for telemetry submission.
    #[derive(Debug, Clone, Default)]
    pub struct TerminalTelemetry {
        pub brand: String,
        pub multiplexer: String,
        pub is_ssh: bool,
        pub is_byobu: bool,
        pub term_var: String,
        pub host_os: String,
        pub display_server: String,
        pub modifier_cmd_fate: String,
        pub modifier_opt_fate: String,
        pub enter_modifier_fate: String,
        pub tmux_version: String,
        pub xtversion: String,
        pub hyperlink_osc8: String,
        pub hyperlink_skip_reason: String,
        pub clipboard_route: String,
        pub clipboard_native_tool: String,
        pub clipboard_data_control: String,
    }
}
