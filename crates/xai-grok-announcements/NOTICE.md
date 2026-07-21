# NOTICE — xai-grok-announcements

Vendored (near-verbatim) from `xai-org/grok-build` `xai-grok-announcements` (Apache-2.0)
for the next-code Grok Face migration (PR6).

Upstream: https://github.com/xai-org/grok-build
Upstream path: crates/codegen/xai-grok-announcements

## Role in next-code

Shared announcement types, persistence, and formatting for Grok CLI apps
(`RemoteAnnouncement`, `AnnouncementCta`, `AnnouncementsRefreshed`, hide-key logic,
`filter_expired`/`prune_hidden_announcement_ids`, hidden-ids read/write). All pure
logic is kept byte-for-byte where practical; the only adaptation is persistence:
`announcements_state_path()` resolves through `xai_grok_config::grok_home` (the PR3
grok_home shim) instead of upstream's `xai_grok_tools::util::grok_home`. The `ts-rs`
binding-generation feature and its export test are dropped — nothing in this
workspace consumes TS bindings.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
