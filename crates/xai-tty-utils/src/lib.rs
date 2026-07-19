/// Detach a command from the terminal (setsid / new process group).
pub fn detach_std_command(_: &mut std::process::Command) {}

/// Duplicate the TUI stderr file descriptor so the fallback logger can write
/// to the original stderr even after the terminal is taken over.
pub fn dup_tui_stderr() -> Result<std::fs::File, std::io::Error> {
    std::fs::File::open("/dev/null")
}

/// Detect whether the process is running under WSL (Windows Subsystem for Linux).
pub fn is_wsl() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/version")
            .ok()
            .map(|v| v.to_lowercase().contains("microsoft") || v.to_lowercase().contains("wsl"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}
