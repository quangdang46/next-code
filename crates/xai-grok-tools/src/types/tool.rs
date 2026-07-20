#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumCount,
    strum::EnumIter,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ToolKind {
    Read,
    Edit,
    Delete,
    ListDir,
    Write,
    Move,
    Search,
    Lsp,
    Execute,
    Plan,
    WebSearch,
    WebFetch,
    BackgroundTaskAction,
    WaitTasksAction,
    KillTaskAction,
    List,
    Skill,
    MemorySearch,
    MemoryGet,
    Task,
    EnterPlan,
    ExitPlan,
    AskUser,
    ImageGen,
    VideoGen,
    ImageToVideo,
    ReferenceToVideo,
    DeployApp,
    SearchTool,
    UseTool,
    Monitor,
    GoalUpdate,
    #[serde(other)]
    Other,
}

impl ToolKind {
    pub const VARIANT_COUNT: usize = <Self as strum::EnumCount>::COUNT;

    pub fn as_key(self) -> &'static str {
        self.into()
    }
}
