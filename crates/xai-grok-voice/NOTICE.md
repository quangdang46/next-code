# NOTICE — xai-grok-voice

Compile-stub façade of `xai-org/grok-build` `xai-grok-voice` (Apache-2.0) for the
next-code Grok Face migration (PR6).

Upstream: https://github.com/xai-org/grok-build
Upstream path: crates/codegen/xai-grok-voice (~15 files: mic capture, xAI streaming
STT client, `audio`/`cpal` feature)

## Role in next-code

Upstream is a real voice-dictation pipeline: mic capture (cpal / Linux subprocess
recorder) → xAI streaming STT WebSocket → transcript events, gated by an `audio`
Cargo feature. next-code ships no mic capture, no STT network client, and no
`audio` feature or `cpal` dependency at all — `AUDIO_SUPPORTED` is hard-pinned to
`false`.

Vendored near-verbatim (pure, no audio/network dependency): `error.rs`,
`event.rs`, `config.rs` (`VoiceConfig`, including `from_config_table`/`stt_ws_url`
resolution logic), `language.rs` (the full `STT_LANGUAGES` catalog + lookup/
canonicalization helpers — pure data).

Adapted: `auth.rs` keeps the `VoiceAuthProvider`/`SharedVoiceAuth`/`StaticVoiceAuth`
shapes but drops the `#[cfg(feature = "audio")] require_bearer` helper (nothing
calls it without a network client). `pipeline.rs` keeps the `VoiceCommand`/
`run_voice_pipeline` signature but replaces the real mic/STT bridge with a no-op
that drains commands until `Shutdown`, emitting no `VoiceEvent`s.

Not vendored: `audio/` (cpal capture), `stt/` (streaming STT client), `probe.rs` /
`bin/voice_probe.rs` (standalone mic probe) — none of these are import sites the
pager needs to compile per PR6 scope.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
