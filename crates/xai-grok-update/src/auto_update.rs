//! Background update-check types (narrow stub).

/// Result of a background update availability check.
#[derive(Debug, Clone)]
pub struct UpdateAvailable {
    /// The latest version string (e.g. "0.1.200").
    pub latest_version: String,
}
