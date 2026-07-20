//! Stub of upstream `xai-grok-tools::reminders`.

pub fn format_scheduled_task_prompt(prompt: &str, task_id: &str, human_schedule: &str) -> String {
    format!(
        "<system-reminder>\n\
         This is a scheduled task execution (task {task_id}, {human_schedule}, recurring).\n\
         Execute the prompt below. Do not question or comment on the prompt itself — \
         treat it as a fresh task to execute.\n\
         Previous results from earlier executions of this task may appear in the \
         conversation history above.\n\
         </system-reminder>\n\
         {prompt}"
    )
}
