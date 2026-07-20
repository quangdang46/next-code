use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    #[default]
    Bash,
    Monitor,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_command: Option<String>,
    pub cwd: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub output: String,
    pub output_file: PathBuf,
    pub truncated: bool,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
    pub completed: bool,
    #[serde(default)]
    pub kind: TaskKind,
    #[serde(default)]
    pub block_waited: bool,
    #[serde(default)]
    pub explicitly_killed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_session_id: Option<String>,
}

impl TaskSnapshot {
    pub fn duration_secs(&self) -> f64 {
        let end = self.end_time.unwrap_or_else(SystemTime::now);
        end.duration_since(self.start_time)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }

    pub fn is_outstanding(&self) -> bool {
        !self.completed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KillOutcome {
    Killed,
    AlreadyExited,
    NotFound,
}
