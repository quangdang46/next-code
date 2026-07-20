# Plan Report

## Summary (read this first)
- **You asked:** Continue to the next brick after PR5 merge (PR6).
- **What is going on:** SUMMARY said PR6 is three tiny stubs (~4 files / &lt;50 LOC). Upstream reality is larger: voice (~15 rs), announcements (1 solid lib), `xai-file-utils` (~15 rs / GCS+S3). Pager imports are still **narrow** — we stub only what PR7 needs to compile, not vendor upload stacks.
- **We recommend:** **Option C (narrow)** — three workspace crates with Cargo names matching pager deps: `xai-grok-voice`, `xai-grok-announcements`, `xai-file-utils`. Keep no-op / Default convention. Do **not** enable real mic/STT or GCS uploads.
- **Risk:** Low–Medium (voice `AUDIO_SUPPORTED` + STT catalog must match settings registry expectations)
- **Status:** Implemented — reviewed vs grok-build; `upload_bytes` signature fixed to match pager `trace_cmd`

## Feature planning
- **Recommended approach:**
  1. **`xai-grok-announcements`** — vendor/adapt the single upstream `lib.rs` (types + pure helpers). Point hidden-id persistence at `xai_grok_tools::util::grok_home` / `xai_grok_config::grok_home` (already `~/.next-code`).
  2. **`xai-grok-voice`** — compile stub: `VoiceConfig`, `VoiceCommand`/`VoiceEvent`, `SharedVoiceAuth`, copy `STT_LANGUAGES` catalog + helpers from upstream `language.rs`, `run_voice_pipeline` = drain-until-Shutdown no-op. Default **`AUDIO_SUPPORTED = false`** (no `audio` feature / no cpal) so Face never advertises a dead mic.
  3. **`xai-file-utils`** — façade only for pager hits: `workspace_classifier::is_project_dir`, `gcs::upload_bytes` → `Err`, `trace_context::span_from_meta_traceparent` → no-op/`tracing::Span::none()` equivalent. Do **not** vendor S3/GCS clients or `xai-grok-auth`.
- **Prior art:** Local `grok-build` ba69d70. SUMMARY names `xai-grok-file-utils` / `xai_grok_file_util` are wrong — pager Cargo dep is **`xai-file-utils`** / `xai_file_utils::`.
- **Integration points:** new crates + workspace members; Face packages stay green; no brain wiring.
- **Sub-agents used:** skipped (small brick, import list verified directly)
- **Option A:** skip — blocks PR7 voice/settings/announcements/project_picker/trace paths
- **Option B (avoid):** full vendor `xai-file-utils` / real voice audio — pulls auth, AWS, cpal
- **Open questions (defaults if go ahead):**
  1. Cargo names `xai-grok-voice` / `xai-grok-announcements` / `xai-file-utils` ✅
  2. `AUDIO_SUPPORTED = false` ✅
  3. Announcements: keep real pure logic + disk under grok_home ✅
  4. No real GCS/S3 ✅

## Evidence
1. **Voice pager uses:** `VoiceConfig`, `VoiceCommand::{PttPress,PttRelease,Shutdown}`, `VoiceEvent::{Interim,Final,Error}`, `SharedVoiceAuth`, `run_voice_pipeline`, `AUDIO_SUPPORTED`, `STT_LANGUAGES` / `stt_language_by_code` / `language_for_api` / `STT_LANGUAGE_*` (`app_view`, `event_loop`, `settings/*`, dispatch/tests).
2. **Announcements:** `RemoteAnnouncement`, `AnnouncementCta`, `AnnouncementsRefreshed`, hide-key / filter / prune / read/write hidden ids (`acp_handler/settings`, views, effects).
3. **file-utils:** only `workspace_classifier::is_project_dir`, `gcs::upload_bytes`, `trace_context::span_from_meta_traceparent` (project_picker, trace_cmd, leader_cluster).
4. SUMMARY “&lt;50 LOC each” is outdated — correct the table in the same PR.

## Steps
1. [ ] Branch `pr-6-grok-voice-announcements-file-utils` from `dev`
2. [ ] Add `xai-grok-announcements` (adapt upstream lib)
3. [ ] Add `xai-grok-voice` stub (+ language catalog; no audio)
4. [ ] Add `xai-file-utils` minimal façade
5. [ ] Workspace members; `cargo check -p xai-grok-voice -p xai-grok-announcements -p xai-file-utils -p xai-grok-pager-render`
6. [ ] Update SUMMARY naming; PR → `dev`, Refs #35

## Files to touch
- `crates/xai-grok-voice/**`
- `crates/xai-grok-announcements/**`
- `crates/xai-file-utils/**`
- `Cargo.toml`
- `docs/grok-migration-SUMMARY.md`
- `docs/plans/PLAN-20260720-grok-pr6-voice-announcements-file-utils.md` (this file)
