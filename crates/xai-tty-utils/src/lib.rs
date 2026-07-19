pub fn detach_std_command(_: &mut std::process::Command) {}
pub fn dup_tui_stderr() -> Result<std::fs::File, std::io::Error> { std::fs::File::open("/dev/null") }
