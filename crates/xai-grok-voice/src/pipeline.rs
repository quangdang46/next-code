//! Compile stub of upstream `xai-grok-voice` `pipeline.rs`.
//!
//! Upstream drives a real mic → STT bridge keyed off `PttPress`/`PttRelease`;
//! this build has no capture/STT backend at all (`AUDIO_SUPPORTED = false`),
//! so [`run_voice_pipeline`] only drains the command channel until
//! [`VoiceCommand::Shutdown`], emitting nothing. The command/type shapes are
//! kept identical so the pager compiles unchanged.

use tokio::sync::mpsc;

use crate::auth::SharedVoiceAuth;
use crate::config::VoiceConfig;
use crate::event::VoiceEvent;

/// Commands from the pager event loop (toggle start/stop, or push-to-talk).
#[derive(Debug)]
pub enum VoiceCommand {
    /// Begin streaming audio to STT (mic open until [`VoiceCommand::PttRelease`]).
    PttPress,
    /// End the current capture session.
    PttRelease,
    /// Tear down the pipeline task.
    Shutdown,
}

/// Stub: drains `cmd_rx` until [`VoiceCommand::Shutdown`] (or the channel
/// closes), never emitting a [`VoiceEvent`] — there is no audio backend to
/// drive one from. `config`/`auth`/`event_tx` are accepted only to match the
/// upstream signature the pager calls.
pub async fn run_voice_pipeline(
    _config: VoiceConfig,
    _auth: SharedVoiceAuth,
    mut cmd_rx: mpsc::Receiver<VoiceCommand>,
    _event_tx: mpsc::Sender<VoiceEvent>,
) {
    while let Some(cmd) = cmd_rx.recv().await {
        if matches!(cmd, VoiceCommand::Shutdown) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::StaticVoiceAuth;

    #[tokio::test]
    async fn drains_until_shutdown() {
        let (cmd_tx, cmd_rx) = mpsc::channel(4);
        let (event_tx, mut event_rx) = mpsc::channel(4);
        let auth = StaticVoiceAuth::shared("k").unwrap();

        let handle = tokio::spawn(run_voice_pipeline(
            VoiceConfig::default(),
            auth,
            cmd_rx,
            event_tx,
        ));

        cmd_tx.send(VoiceCommand::PttPress).await.unwrap();
        cmd_tx.send(VoiceCommand::PttRelease).await.unwrap();
        cmd_tx.send(VoiceCommand::Shutdown).await.unwrap();

        handle.await.unwrap();
        assert!(event_rx.recv().await.is_none());
    }
}
