//! Product telemetry event types used by the pager (stub shapes).

use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementCtaSurface {
    #[default]
    Banner,
    Welcome,
    Header,
    Dashboard,
    Keyboard,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextualTipKind {
    #[default]
    Undo,
    PlanMode,
    ImageInput,
    SendNow,
    SmallScreen,
    WordSelect,
    SshWrap,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextualTipAction {
    #[default]
    Shown,
    Accepted,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptSuggestionAction {
    #[default]
    Shown,
    Accepted,
    Dismissed,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum YoloTrigger {
    #[default]
    SlashCommand,
    ClientMeta,
    Pager,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CreditLimitChoice {
    #[default]
    UpgradeTier,
    PayAsYouGo,
    PurchaseCredits,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CreditLimitUpsellSurface {
    #[default]
    QuestionModal,
    InlineCard,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionsInputMethod {
    #[default]
    Keyboard,
    Mouse,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionsModalTrigger {
    #[default]
    SlashCommand,
    KeyboardShortcut,
    CommandPalette,
    AuthHandoff,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionsModalTab {
    #[default]
    Hooks,
    Plugins,
    Marketplace,
    Skills,
    McpServers,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InstallKind {
    #[default]
    Git,
    Local,
}

impl InstallKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SuperGrokUpsell {
    #[default]
    WelcomeScreen,
    RateLimitError,
    FreeUsagePaywall,
    RestrictedCommand,
}

#[derive(Debug, Clone, Default, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct AnnouncementCtaShown {
    pub id: Option<String>,
    pub source: AnnouncementCtaSurface,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnouncementCtaClicked {
    pub id: Option<String>,
    pub source: AnnouncementCtaSurface,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextualTip {
    pub tip: ContextualTipKind,
    pub action: ContextualTipAction,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptSuggestion {
    pub action: PromptSuggestionAction,
    pub chars: usize,
    pub words: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditLimitUpsellShown {
    pub surface: CreditLimitUpsellSurface,
    pub max_tier: bool,
    pub pay_as_you_go: bool,
    #[serde(default)]
    pub unified_billing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditLimitUpsellClicked {
    pub surface: CreditLimitUpsellSurface,
    pub choice: CreditLimitChoice,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionActivated {
    pub auth_method: Option<String>,
    pub upsell_shown_this_session: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionsModalOpened {
    pub trigger: ExtensionsModalTrigger,
    pub tab: ExtensionsModalTab,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionsModalAction {
    pub tab: ExtensionsModalTab,
    pub action: String,
    pub input_method: ExtensionsInputMethod,
    pub target: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginInstalled {
    pub install_kind: InstallKind,
    pub success: bool,
    pub trust: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginUninstalled {
    pub confirmed: bool,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginCtaImpression {
    pub plugin_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginCtaDismissed {
    pub plugin_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginCtaConnectClicked {
    pub plugin_name: String,
    pub is_retry: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginCtaInstalled {
    pub plugin_name: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationEmitted {
    pub protocol: &'static str,
    pub event_kind: &'static str,
    pub was_focused: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayRefreshProbe {
    #[serde(flatten)]
    pub terminal: TerminalTelemetry,
    pub outcome: String,
    pub hz: Option<i64>,
    pub source: String,
    pub skip_reason: String,
    pub duration_ms: i64,
    pub auto_cadence_enabled: bool,
    pub auto_cadence_applied: bool,
    pub effective_min_draw_ms: i64,
    pub effective_scroll_cadence_ms: i64,
    pub auto_cadence_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuperGrokUpsellShown {
    pub source: SuperGrokUpsell,
    pub auth_method: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuperGrokUpsellClicked {
    pub source: SuperGrokUpsell,
    pub auth_method: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YoloToggled {
    pub enabled: bool,
    pub previous_state: bool,
    pub trigger: YoloTrigger,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanSubmit {
    pub action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitHit {
    pub model_id: String,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditLimitHit {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardOpened {
    pub agents: usize,
    pub subagents: usize,
    pub leader_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardClosed {
    pub agents: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardAgentAttached {
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardAgentLaunched {
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackspaceNoEffect {
    #[serde(flatten)]
    pub terminal: TerminalTelemetry,
    pub key_code: String,
    pub key_modifiers: String,
    pub key_kind: String,
    pub cursor_pos: usize,
    pub text_len: usize,
    pub has_selection: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardCopy {
    pub terminal: TerminalTelemetry,
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

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardImagePaste {
    pub terminal: TerminalTelemetry,
    pub probe: String,
    pub outcome: String,
    pub image_mime: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasteKeyEmptyHostClipboard {
    pub terminal: TerminalTelemetry,
    pub surface: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PagerCommandSource {
    Builtin,
    NonBuiltin,
}

#[derive(Debug, Clone, Serialize)]
pub struct PagerSlashCommand {
    pub command_name: String,
    pub source: PagerCommandSource,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPickerOutcome {
    RecentProject,
    CustomPath,
    CurrentDir,
    DontAskAgain,
    Dismissed,
}

impl ProjectPickerOutcome {
    pub fn picked_project(self) -> bool {
        matches!(self, Self::RecentProject | Self::CustomPath)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectPickerSelected {
    pub outcome: ProjectPickerOutcome,
    pub picked_project: bool,
    pub project_dir_options: usize,
}
