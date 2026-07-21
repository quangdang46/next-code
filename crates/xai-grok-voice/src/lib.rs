//! Compile-stub of `xai-org/grok-build` `xai-grok-voice` (Apache-2.0) for the
//! next-code Grok Face migration (PR6).
//!
//! Upstream is a real xAI streaming-STT voice dictation pipeline (mic → cpal
//! capture → xAI STT WebSocket → transcript events). next-code has no mic
//! capture, no STT network client, and no `audio` feature/`cpal` dependency —
//! this crate only reproduces the type/function *shapes* the pager needs to
//! compile: `VoiceConfig`, `VoiceCommand`, `VoiceEvent`, `SharedVoiceAuth`,
//! `run_voice_pipeline` (a no-op that drains commands until `Shutdown`), and
//! the (pure, data-only) STT language catalog from upstream `language.rs`.
//!
//! [`AUDIO_SUPPORTED`] is hard-pinned to `false` so Face never advertises a
//! microphone it cannot open.

pub mod auth;
pub mod config;
pub mod error;
pub mod event;
pub mod language;
pub mod pipeline;

pub use auth::{SharedVoiceAuth, StaticVoiceAuth, VoiceAuthProvider};
pub use config::VoiceConfig;
pub use error::VoiceError;
pub use event::VoiceEvent;
pub use language::{
    STT_LANGUAGE_AUTO, STT_LANGUAGE_DEFAULT, STT_LANGUAGES, SttLanguage, canonicalize_stt_language,
    language_for_api, stt_language_by_code,
};
pub use pipeline::{VoiceCommand, run_voice_pipeline};

/// Whether this build can capture microphone audio. Always `false`: next-code
/// ships no `audio` feature, no `cpal` dependency, and no mic/STT network
/// calls anywhere in this crate. Consumers gate voice on this so the pager
/// never advertises a mic it can't open.
pub const AUDIO_SUPPORTED: bool = false;
