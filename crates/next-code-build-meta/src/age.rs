//! Binary / commit age formatting for UI status chrome.

use chrono::{DateTime, Utc};

/// Human-readable age from a duration in seconds.
pub fn format_age(secs: i64) -> String {
    if secs < 0 {
        "future?".to_string()
    } else if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Format build age from clock + optional binary mtime + embedded git date.
///
/// When git commit age and binary mtime differ by more than 5 minutes, returns
/// `"built_age, code git_age"` (without the `built` prefix — callers add that).
pub fn binary_age_from(
    now: DateTime<Utc>,
    build_mtime: Option<DateTime<Utc>>,
    git_date: &str,
) -> Option<String> {
    let build_date = build_mtime?;
    let build_secs = now.signed_duration_since(build_date).num_seconds();
    let build_age = format_age(build_secs);

    let git_commit_date = DateTime::parse_from_str(git_date, "%Y-%m-%d %H:%M:%S %z")
        .ok()
        .map(|dt| dt.with_timezone(&Utc));
    let git_secs = git_commit_date.map(|d| now.signed_duration_since(d).num_seconds());

    if let Some(git_secs) = git_secs {
        let diff = (git_secs - build_secs).abs();
        if diff > 300 {
            let git_age = format_age(git_secs);
            return Some(format!("{build_age}, code {git_age}"));
        }
    }

    Some(build_age)
}

/// Age of the running binary relative to now (and optionally vs embedded git date).
pub fn binary_age() -> Option<String> {
    let modified = std::env::current_exe()
        .ok()
        .and_then(|path| std::fs::metadata(path).ok())
        .and_then(|meta| meta.modified().ok())
        .map(DateTime::<Utc>::from);
    binary_age_from(Utc::now(), modified, crate::GIT_DATE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn format_age_buckets() {
        assert_eq!(format_age(-1), "future?");
        assert_eq!(format_age(0), "just now");
        assert_eq!(format_age(59), "just now");
        assert_eq!(format_age(60), "1m ago");
        assert_eq!(format_age(3599), "59m ago");
        assert_eq!(format_age(3600), "1h ago");
        assert_eq!(format_age(86399), "23h ago");
        assert_eq!(format_age(86400), "1d ago");
    }

    #[test]
    fn binary_age_omits_code_when_close() {
        let now = Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap();
        let build = now - chrono::Duration::hours(2);
        let git = (build + chrono::Duration::minutes(2))
            .format("%Y-%m-%d %H:%M:%S %z")
            .to_string();
        assert_eq!(
            binary_age_from(now, Some(build), &git).as_deref(),
            Some("2h ago")
        );
    }

    #[test]
    fn binary_age_includes_code_when_far() {
        let now = Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap();
        let build = now - chrono::Duration::hours(2);
        let git_dt = now - chrono::Duration::hours(5);
        let git = git_dt.format("%Y-%m-%d %H:%M:%S %z").to_string();
        assert_eq!(
            binary_age_from(now, Some(build), &git).as_deref(),
            Some("2h ago, code 5h ago")
        );
    }

    #[test]
    fn binary_age_none_without_mtime() {
        let now = Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap();
        assert!(binary_age_from(now, None, "unused").is_none());
    }
}
