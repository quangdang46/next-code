//! Canonical slash-command wording (from upstream xai-grok-tools-api).

pub const SCHEDULER_CREATE_TOOL_NAME: &str = "scheduler_create";
pub const IMAGE_GEN_TOOL_NAME: &str = "image_gen";
pub const IMAGINE_COMMAND_NAME: &str = "imagine";
pub const IMAGE_TO_VIDEO_TOOL_NAME: &str = "image_to_video";
pub const IMAGINE_VIDEO_COMMAND_NAME: &str = "imagine-video";

pub fn loop_usage_message() -> &'static str {
    "Usage: /loop [interval] <prompt>\n     Example: /loop 30m check deploy status\n     Example: /loop check deploy status every hour\n\n     Tell me how often it should run (e.g. 30m, 1 hour, every 2 days)."
}

pub fn loop_schedule_instruction(args: &str) -> String {
    format!(
        "# /loop -- schedule a recurring prompt\n\n         Parse the input below into an interval and a prompt, then schedule it with scheduler_create.\n\n         ## Input\n         {args}"
    )
}

pub fn imagine_usage_message() -> &'static str {
    "Usage: /imagine <description>\n     Provide a text description to generate an image."
}

pub fn imagine_instruction(prompt: &str) -> String {
    format!(
        "Call the image_gen tool immediately, passing the user's prompt below          verbatim. After the tool completes, briefly acknowledge and mention          where the image was saved.\n\nPrompt: {prompt}"
    )
}

pub fn imagine_video_usage_message() -> &'static str {
    "Usage: /imagine-video <description>\n     Provide a text description to generate a video."
}

pub fn imagine_video_instruction(prompt: &str) -> String {
    format!("Generate a video for the user prompt.\n\nUser prompt: {prompt}")
}
