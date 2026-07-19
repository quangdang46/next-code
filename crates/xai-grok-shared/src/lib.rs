pub mod clipboard {
    pub fn is_remote_session() -> bool { false }
    pub fn is_containerized_without_display() -> bool { false }
    pub fn wayland_data_control_supported() -> bool { false }
    pub fn spool_for_stdin(_: &[u8]) -> Option<std::process::Command> { None }
    pub fn wait_with_deadline(_: std::process::Child) -> Option<i32> { None }
    pub fn get_text() -> Result<Option<String>, String> { Ok(None) }
    pub fn set_text_with_outcome(_: &str) -> Result<(), String> { Ok(()) }
    pub fn set_text_osc52(_: &str, _: bool) -> Result<(), String> { Ok(()) }
    pub fn x11_display_env_present() -> bool { false }
    pub fn get_primary_text() -> Result<String, String> { Ok(String::new()) }
    pub fn get_attachments() -> Result<Vec<u8>, String> { Ok(Vec::new()) }
    pub struct ImageData;
    pub fn clipboard_image_snapshot() -> Option<Vec<u8>> { None }
    pub fn clipboard_change_count() -> u64 { 0 }
    pub fn clipboard_image_probe_supported() -> bool { false }
    pub fn clipboard_prewarm() {}
    pub fn get_image() -> Result<Option<Vec<u8>>, String> { Ok(None) }
    pub fn detach_std_command(_: &mut std::process::Command) {}
}
pub fn is_wsl() -> bool { false }
pub fn dup_tui_stderr() -> Result<std::fs::File, std::io::Error> { std::fs::File::open("/dev/null") }
