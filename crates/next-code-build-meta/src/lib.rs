//! Compile-time build/version metadata for next_code.
//!
//! The build script (`build.rs`) computes git- and version-derived values and
//! emits them via `cargo:rustc-env`. This module re-exposes them as `pub const`
//! so any workspace crate can read identical values through e.g.
//! `next_code_build_meta::VERSION` instead of `env!("NEXT_CODE_VERSION")`.

/// Human-readable version string, e.g. `v0.14.6-dev (abc1234)`.
pub const VERSION: &str = env!("NEXT_CODE_VERSION");
/// Short git hash of the build commit, e.g. `abc1234` (or `unknown`).
pub const GIT_HASH: &str = env!("NEXT_CODE_GIT_HASH");
/// Commit date/time of the build commit (or `unknown`).
pub const GIT_DATE: &str = env!("NEXT_CODE_GIT_DATE");
/// `git describe --tags --always` output (may be empty).
pub const GIT_TAG: &str = env!("NEXT_CODE_GIT_TAG");
/// Auto-incrementing build semver (dev) or explicit release semver.
pub const SEMVER: &str = env!("NEXT_CODE_SEMVER");
/// Base semver taken from the root `Cargo.toml` package version.
pub const BASE_SEMVER: &str = env!("NEXT_CODE_BASE_SEMVER");
/// Semver used for update comparisons.
pub const UPDATE_SEMVER: &str = env!("NEXT_CODE_UPDATE_SEMVER");
/// Encoded changelog (record/unit separated). See build.rs for the format.
pub const CHANGELOG: &str = env!("NEXT_CODE_CHANGELOG");
/// Root crate package version (mirrors the historical `CARGO_PKG_VERSION`).
pub const PKG_VERSION: &str = env!("NEXT_CODE_PKG_VERSION");

/// Whether this binary was built as a release build (`NEXT_CODE_RELEASE_BUILD=1`;
/// dual-read: legacy `NEXT_CODE_RELEASE_BUILD=1`).
pub const fn is_release_build() -> bool {
    option_env!("NEXT_CODE_RELEASE_BUILD").is_some()
}
