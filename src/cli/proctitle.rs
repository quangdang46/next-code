//! Mapping from parsed CLI arguments to an initial process title.
//!
//! This logic depends on the clap `Args`/`Command` types defined in `cli`, so
//! it lives in the CLI layer. The low-level title-setting primitives it uses
//! (`compact_process_title`, `session_name`, `set_title`) live in the
//! `process_title` core module.

use crate::cli::args::{AmbientCommand, Args, Command};
use crate::process_title::{compact_process_title, session_name, set_title};

pub(crate) fn initial_title(args: &Args) -> String {
    match &args.command {
        Some(Command::Serve { .. }) => "nc:server".to_string(),
        Some(Command::Acp) => "next-code acp".to_string(),
        Some(Command::Server { .. }) => "next-code server".to_string(),
        Some(Command::Connect) => "nc:client".to_string(),
        Some(Command::Run { .. }) => "next-code run".to_string(),
        Some(Command::Login { .. }) => "next-code login".to_string(),
        Some(Command::Repl) => "next-code repl".to_string(),
        Some(Command::Update) => "next-code update".to_string(),
        Some(Command::Version { .. }) => "next-code version".to_string(),
        Some(Command::Usage { .. }) => "next-code usage".to_string(),
        Some(Command::Plugin(..)) => "next-code plugin".to_string(),
        Some(Command::SelfDev { .. }) => "nc:selfdev".to_string(),
        Some(Command::Debug { .. }) => "next-code debug".to_string(),
        Some(Command::Auth(_)) => "next-code auth".to_string(),
        Some(Command::Provider(_)) => "next-code provider".to_string(),
        Some(Command::Memory(_)) => "next-code memory".to_string(),
        Some(Command::Session(_)) => "next-code session".to_string(),
        Some(Command::Secrets(_)) => "next-code secrets".to_string(),
        Some(Command::Ambient(subcommand)) => match subcommand {
            AmbientCommand::RunVisible => "next-code ambient visible".to_string(),
            _ => "next-code ambient".to_string(),
        },
        Some(Command::Cloud(_)) => "next-code cloud".to_string(),
        Some(Command::Pair { .. }) => "next-code pair".to_string(),
        Some(Command::Permissions) => "next-code permissions".to_string(),
        Some(Command::Permission(_)) => "next-code permission".to_string(),
        Some(Command::Transcript { .. }) => "next-code transcript".to_string(),
        Some(Command::Dictate { .. }) => "next-code dictate".to_string(),
        Some(Command::SetupHotkey {
            listen_macos_hotkey,
            notify_cli_launch,
        }) => {
            if *listen_macos_hotkey {
                "next-code hotkey listener".to_string()
            } else if notify_cli_launch.is_some() {
                "next-code shortcut reminder".to_string()
            } else {
                "next-code hotkey setup".to_string()
            }
        }
        Some(Command::Browser { .. }) => "next-code browser".to_string(),
        Some(Command::Replay { .. }) => "next-code replay".to_string(),
        Some(Command::Model(_)) => "next-code model".to_string(),
        Some(Command::ProviderTestCoverage { .. }) => "next-code provider-test-coverage".to_string(),
        Some(Command::ProviderDoctor { .. }) => "next-code provider-doctor".to_string(),
        Some(Command::Doctor { .. }) => "next-code doctor".to_string(),
        Some(Command::AuthTest { .. }) => "next-code auth-test".to_string(),
        Some(Command::Restart { .. }) => "next-code restart".to_string(),
        // Menubar is handled via Command::Ambient(AmbientCommand::Menubar)
        Some(Command::SetupLauncher) => "next-code setup-launcher".to_string(),
        None => {
            if let Some(resume) = args.resume.as_deref().filter(|resume| !resume.is_empty()) {
                let prefix = if crate::cli::selfdev::client_selfdev_requested() {
                    "nc:d:"
                } else {
                    "nc:c:"
                };
                compact_process_title(prefix, Some(&session_name(resume)))
            } else if crate::cli::selfdev::client_selfdev_requested() {
                "nc:selfdev".to_string()
            } else {
                "nc:client".to_string()
            }
        }
    }
}

pub(crate) fn set_initial_title(args: &Args) {
    set_title(initial_title(args));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::lock_test_env;
    use clap::Parser;

    const SELFDEV_ENV: &str = next_code_selfdev_types::CLIENT_SELFDEV_ENV;

    fn with_selfdev_env_removed<T>(f: impl FnOnce() -> T) -> T {
        let _guard = lock_test_env();
        let previous = std::env::var_os(SELFDEV_ENV);
        crate::env::remove_var(SELFDEV_ENV);
        let result = f();
        if let Some(value) = previous {
            crate::env::set_var(SELFDEV_ENV, value);
        }
        result
    }

    #[test]
    fn initial_title_labels_server() {
        with_selfdev_env_removed(|| {
            let args = Args::parse_from(["next-code", "serve"]);
            assert_eq!(initial_title(&args), "nc:server");
        });
    }

    #[test]
    fn initial_title_labels_resume_client_with_short_name() {
        with_selfdev_env_removed(|| {
            let args = Args::parse_from(["next-code", "--resume", "session_fox_123"]);
            assert_eq!(initial_title(&args), "nc:c:fox");
        });
    }

    #[test]
    fn initial_title_labels_selfdev_command() {
        with_selfdev_env_removed(|| {
            let args = Args::parse_from(["next-code", "self-dev"]);
            assert_eq!(initial_title(&args), "nc:selfdev");
        });
    }
}
