//! Facade of `xai-org/grok-build` `xai-grok-update` (Apache-2.0) for the
//! next-code Grok Face migration (PR7).
//!
//! Upstream performs version checks and binary downloads. This stub only
//! reproduces [`channel_label`] and [`auto_update::UpdateAvailable`].

pub mod auto_update;

/// Channel label derived from the cached stable pointer.
///
/// Upstream returns `" [alpha]"`, `" [stable]"`, or `""`. Stub: always `""`.
pub fn channel_label() -> &'static str {
    ""
}
