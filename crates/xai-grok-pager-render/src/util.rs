//! Shared utility functions.

use std::borrow::Cow;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub use xai_grok_config::grok_home;

/// Outcome of [`write_text_resilient`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenText {
    /// Path that actually received the bytes (may differ from the request
    /// when the destination was locked on Windows).
    pub path: PathBuf,
    /// `true` when we had to pick a sibling name because the requested path
    /// could not be replaced (ERROR_SHARING_VIOLATION / lock).
    pub redirected: bool,
}

/// Options for [`write_text_resilient`].
#[derive(Debug, Clone, Copy, Default)]
pub struct WriteTextOptions {
    /// On Unix, create/tighten the final file to mode `0600`.
    pub owner_only: bool,
}

/// Write `text` without truncating the destination in place.
///
/// Windows antivirus, Search Indexer, OneDrive, and editors often hold a share
/// lock on a path Face just exported (or on the always-rewritten
/// `last-copy.txt`). [`std::fs::write`] opens with truncate and fails with
/// `ERROR_SHARING_VIOLATION` (os error 32). This helper:
/// 1. writes a unique temp sibling in the same directory (handle closed)
/// 2. renames onto `path`, retrying briefly on transient lock errors
/// 3. if the destination stays locked, renames to a unique sibling instead
pub fn write_text_resilient(
    path: &Path,
    text: &str,
    opts: WriteTextOptions,
) -> io::Result<WrittenText> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = temp_sibling(path);
    write_temp_file(&tmp, text.as_bytes(), opts.owner_only).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e
    })?;

    const MAX_ATTEMPTS: u32 = 8;
    for attempt in 0..MAX_ATTEMPTS {
        match std::fs::rename(&tmp, path) {
            Ok(()) => {
                harden_owner_only(path, opts.owner_only)?;
                return Ok(WrittenText {
                    path: path.to_path_buf(),
                    redirected: false,
                });
            }
            Err(e) if is_sharing_or_lock_error(&e) && attempt + 1 < MAX_ATTEMPTS => {
                std::thread::sleep(Duration::from_millis(15 + 15 * u64::from(attempt)));
            }
            Err(e) if is_sharing_or_lock_error(&e) => {
                break;
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                return Err(e);
            }
        }
    }

    // Destination stayed locked — land on a unique sibling so export still
    // succeeds (and the toast/scrollback can name the real path).
    let alt = unique_sibling(path);
    match std::fs::rename(&tmp, &alt) {
        Ok(()) => {
            harden_owner_only(&alt, opts.owner_only)?;
            Ok(WrittenText {
                path: alt,
                redirected: true,
            })
        }
        Err(rename_err) => {
            // Last resort: rewrite to the alternate path (temp may still be
            // usable if rename failed for a non-lock reason on alt).
            let _ = std::fs::remove_file(&tmp);
            write_temp_file(&alt, text.as_bytes(), opts.owner_only).map_err(|_| rename_err)?;
            harden_owner_only(&alt, opts.owner_only)?;
            Ok(WrittenText {
                path: alt,
                redirected: true,
            })
        }
    }
}

fn temp_sibling(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let id = uuid::Uuid::new_v4();
    path.with_file_name(format!(".{name}.{id}.tmp"))
}

fn unique_sibling(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let id = &uuid::Uuid::new_v4().to_string()[..8];
    let name = match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => format!("{stem}-{id}.{ext}"),
        None => format!("{stem}-{id}"),
    };
    path.with_file_name(name)
}

fn write_temp_file(path: &Path, bytes: &[u8], owner_only: bool) -> io::Result<()> {
    {
        #[cfg(unix)]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create_new(true);
            if owner_only {
                opts.mode(0o600);
            }
            opts.open(path)?
        };
        #[cfg(not(unix))]
        let mut file = {
            let _ = owner_only;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)?
        };
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    Ok(())
}

fn harden_owner_only(path: &Path, owner_only: bool) -> io::Result<()> {
    #[cfg(unix)]
    if owner_only {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = (path, owner_only);
    Ok(())
}

/// Windows `ERROR_SHARING_VIOLATION` (32) / `ERROR_LOCK_VIOLATION` (33), plus
/// rename-over-open `ERROR_ACCESS_DENIED` (5). Also treat
/// [`io::ErrorKind::PermissionDenied`] as retryable — some std mappings lose
/// the raw code.
fn is_sharing_or_lock_error(err: &io::Error) -> bool {
    match err.raw_os_error() {
        Some(32 | 33) => true,
        #[cfg(windows)]
        Some(5) => true,
        _ => err.kind() == io::ErrorKind::PermissionDenied,
    }
}

/// Path to `$GROK_HOME` / `$NEXT_CODE_HOME` / `~/.next-code` `pager.toml`.
pub fn pager_toml_path() -> PathBuf {
    grok_home().join("pager.toml")
}

/// User-facing label for the Face home directory.
///
/// Derived from resolved [`grok_home()`] vs `xai_grok_config::default_grok_home()`,
/// not solely from whether an env override is set.
///
/// - Default install: `~/.next-code`
/// - `$GROK_HOME` override: `$GROK_HOME`
/// - `$NEXT_CODE_HOME` override (and not equal to default path): `$NEXT_CODE_HOME`
pub fn display_grok_home_prefix() -> String {
    let resolved = grok_home();
    if resolved == xai_grok_config::default_grok_home() {
        return "~/.next-code".to_string();
    }
    if std::env::var_os("GROK_HOME").is_some() {
        return "$GROK_HOME".to_string();
    }
    if std::env::var_os("NEXT_CODE_HOME").is_some() {
        return "$NEXT_CODE_HOME".to_string();
    }
    // Resolved path differs from default without a known env (e.g. canonicalize
    // drift) — still prefer the product label.
    "~/.next-code".to_string()
}

/// User-facing path under [`grok_home()`], e.g. ``~/.next-code/config.toml``.
pub fn display_user_grok_path(relative: impl AsRef<Path>) -> String {
    let rel = relative.as_ref();
    let prefix = display_grok_home_prefix();
    if rel.as_os_str().is_empty() {
        return prefix;
    }
    format!("{prefix}/{}", rel.display())
}

/// Abbreviate an absolute path for display: prefer [`grok_home()`], then `$HOME`.
pub fn abbreviate_path(path: &str) -> Cow<'_, str> {
    let path_buf = Path::new(path);
    let grok = grok_home();
    if let Ok(rest) = path_buf.strip_prefix(&grok) {
        let prefix = display_grok_home_prefix();
        if rest.as_os_str().is_empty() {
            return Cow::Owned(prefix);
        }
        return Cow::Owned(format!("{prefix}/{}", rest.display()));
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
        && let Some(rest) = path.strip_prefix(&home)
    {
        if rest.is_empty() {
            return Cow::Borrowed("~");
        }
        if rest.starts_with('/') {
            return Cow::Owned(format!("~{rest}"));
        }
    }
    Cow::Borrowed(path)
}

/// True when `path` is under user [`grok_home()`] (not project `{cwd}/.grok`).
pub fn is_under_user_grok_home(path: &Path) -> bool {
    path.starts_with(grok_home())
}

/// Format a duration as a compact human-friendly string.
///
/// Uses consistent rounding for visual stability:
/// - Under 10s: `"5.2s"` (one decimal for granularity)
/// - 10-59s: `"32s"` (no decimal)
/// - 1m-59m: `"2m5s"`
/// - 1h+: `"1h2m"`
pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    if total_secs < 10 {
        return format!("{:.1}s", d.as_secs_f64());
    }
    if total_secs < 60 {
        return format!("{total_secs}s");
    }
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    if mins < 60 {
        return format!("{mins}m{secs}s");
    }
    let hours = mins / 60;
    let remaining_mins = mins % 60;
    format!("{hours}h{remaining_mins}m")
}

/// Format a duration as a coarse recency string for "time ago" / age
/// displays (e.g. dashboard row age column and peek panel prefix).
///
/// Buckets chosen for the agent dashboard so the column stays compact
/// and doesn't distract with second-level churn:
/// - < 1 minute: `"just now"`
/// - minutes: `"1m"` … `"59m"`
/// - hours: `"1h"` … `"23h"`
/// - days: `"1d"` … `"29d"`
/// - months (≈30d+): `"1mo"` … `"11mo"`
/// - years (≈365d+): `"1y"` …
pub fn format_time_ago(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        return "just now".to_string();
    }
    if secs < 3600 {
        let mins = secs / 60;
        return format!("{mins}m");
    }
    if secs < 86400 {
        let hours = secs / 3600;
        return format!("{hours}h");
    }
    let days = secs / 86400;
    if days < 30 {
        return format!("{days}d");
    }
    if days < 365 {
        let months = days / 30;
        return format!("{months}mo");
    }
    let years = days / 365;
    format!("{years}y")
}

/// Convert unix-epoch millis into a wall-clock [`SystemTime`].
///
/// Used for dashboard recency that originates as a wall-clock timestamp (the
/// leader roster's `last_change_unix_ms`). A non-positive value — the
/// `#[serde(default)]` `0` sentinel for a missing roster timestamp — falls
/// back to "now".
pub fn system_time_from_unix_ms(unix_ms: i64) -> SystemTime {
    if unix_ms <= 0 {
        return SystemTime::now();
    }
    UNIX_EPOCH
        .checked_add(Duration::from_millis(unix_ms as u64))
        .unwrap_or_else(SystemTime::now)
}

/// Project a monotonic [`Instant`] onto the wall clock as the [`SystemTime`]
/// it corresponds to (`SystemTime::now() - instant.elapsed()`).
///
/// The dashboard stores row recency as a wall-clock `SystemTime` so on-disk
/// roster timestamps (which can predate this process — even the machine's
/// boot — and so are unrepresentable as a monotonic `Instant`) sit in the same
/// comparable space as local rows. Local rows hold live `Instant` anchors;
/// this maps them across. A fixed anchor ages correctly because only `now`
/// advances, and the sub-millisecond skew between the two `now()` samples is
/// invisible to the minute-granularity [`format_time_ago`] buckets.
pub fn system_time_from_instant(instant: Instant) -> SystemTime {
    SystemTime::now()
        .checked_sub(instant.elapsed())
        .unwrap_or_else(SystemTime::now)
}

/// Decode common HTML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`)
/// that may appear in LLM-generated session summaries.
pub fn decode_html_entities(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains('&') {
        return std::borrow::Cow::Borrowed(s);
    }
    let mut out = s.to_string();
    out = out.replace("&amp;", "&");
    out = out.replace("&lt;", "<");
    out = out.replace("&gt;", ">");
    out = out.replace("&quot;", "\"");
    out = out.replace("&#39;", "'");
    out = out.replace("&#x27;", "'");
    out = out.replace("&apos;", "'");
    std::borrow::Cow::Owned(out)
}

pub fn parse_schedule_interval_secs(human: &str) -> Option<u64> {
    let s = human.trim_start();
    if !s.starts_with("every ") {
        return None;
    }
    let rest = s[6..].trim_start();
    let (num_str, unit) = if let Some(sp) = rest.find(char::is_whitespace) {
        (&rest[..sp], &rest[sp + 1..])
    } else if rest.len() >= 2 {
        let (d, u) = rest.split_at(rest.len() - 1);
        (d, u)
    } else {
        return None;
    };
    let n: u64 = num_str.parse().ok()?;
    let unit = unit.trim();
    let secs_per = match unit {
        "s" | "second" | "seconds" => 1,
        "m" | "minute" | "minutes" => 60,
        "h" | "hour" | "hours" => 3600,
        "d" | "day" | "days" => 86400,
        _ => return None,
    };
    Some(n * secs_per)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsecond() {
        assert_eq!(format_duration(Duration::from_millis(500)), "0.5s");
    }

    #[test]
    fn under_ten_seconds() {
        assert_eq!(format_duration(Duration::from_secs_f64(5.23)), "5.2s");
    }

    #[test]
    fn ten_seconds_no_decimal() {
        assert_eq!(format_duration(Duration::from_secs(10)), "10s");
    }

    #[test]
    fn seconds_no_decimal() {
        assert_eq!(format_duration(Duration::from_secs_f64(12.3)), "12s");
    }

    #[test]
    fn thirty_seconds() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
    }

    #[test]
    fn minutes() {
        assert_eq!(format_duration(Duration::from_secs(125)), "2m5s");
    }

    #[test]
    fn hours() {
        assert_eq!(format_duration(Duration::from_secs(3725)), "1h2m");
    }

    #[test]
    fn time_ago_just_now() {
        assert_eq!(format_time_ago(Duration::from_secs(0)), "just now");
        assert_eq!(format_time_ago(Duration::from_secs(30)), "just now");
        assert_eq!(format_time_ago(Duration::from_secs(59)), "just now");
    }

    #[test]
    fn time_ago_minutes() {
        assert_eq!(format_time_ago(Duration::from_secs(60)), "1m");
        assert_eq!(format_time_ago(Duration::from_secs(125)), "2m");
        assert_eq!(format_time_ago(Duration::from_secs(3599)), "59m");
    }

    #[test]
    fn time_ago_hours() {
        assert_eq!(format_time_ago(Duration::from_secs(3600)), "1h");
        assert_eq!(format_time_ago(Duration::from_secs(7200)), "2h");
        assert_eq!(format_time_ago(Duration::from_secs(86399)), "23h");
    }

    #[test]
    fn time_ago_days() {
        assert_eq!(format_time_ago(Duration::from_secs(86400)), "1d");
        assert_eq!(format_time_ago(Duration::from_secs(172800)), "2d");
        assert_eq!(format_time_ago(Duration::from_secs(2_592_000 - 1)), "29d"); // just under 30d
    }

    #[test]
    fn time_ago_months() {
        assert_eq!(format_time_ago(Duration::from_secs(2_592_000)), "1mo"); // 30d
        assert_eq!(format_time_ago(Duration::from_secs(5_184_000)), "2mo");
        // 359d is still 11mo (359/30=11); 360d would be 12mo.
        assert_eq!(format_time_ago(Duration::from_secs(359 * 86400)), "11mo");
    }

    #[test]
    fn time_ago_years() {
        assert_eq!(format_time_ago(Duration::from_secs(31_536_000)), "1y"); // 365d
        assert_eq!(format_time_ago(Duration::from_secs(63_072_000)), "2y");
    }

    fn now_unix_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    /// A real past timestamp survives the round-trip and renders its true age —
    /// including ages beyond the machine's uptime, which a monotonic `Instant`
    /// could not represent (its floor is system boot).
    #[test]
    fn system_time_from_unix_ms_renders_real_age() {
        let two_hours_ago = now_unix_ms() - 2 * 3_600_000;
        let elapsed = system_time_from_unix_ms(two_hours_ago)
            .elapsed()
            .unwrap_or_default();
        assert_eq!(format_time_ago(elapsed), "2h");

        let forty_five_days_ago = now_unix_ms() - 45 * 86_400_000;
        let elapsed = system_time_from_unix_ms(forty_five_days_ago)
            .elapsed()
            .unwrap_or_default();
        assert_eq!(format_time_ago(elapsed), "1mo");
    }

    /// A zero / missing timestamp (the `#[serde(default)]` sentinel) falls back
    /// to "now" rather than the unix epoch (1970).
    #[test]
    fn system_time_from_unix_ms_zero_falls_back_to_now() {
        let elapsed = system_time_from_unix_ms(0).elapsed().unwrap_or_default();
        assert!(elapsed.as_secs() < 5, "zero sentinel must fall back to now");
    }

    /// A future timestamp (clock skew) renders as "just now": `elapsed()` errors
    /// on a future `SystemTime`, and callers default that to a zero duration.
    #[test]
    fn system_time_from_unix_ms_future_renders_just_now() {
        let future = now_unix_ms() + 10_000_000;
        let elapsed = system_time_from_unix_ms(future)
            .elapsed()
            .unwrap_or_default();
        assert_eq!(format_time_ago(elapsed), "just now");
    }

    /// A fixed `Instant` projects to a stable wall-clock moment, so its age
    /// reflects time-since-anchor (here ~10m) rather than re-anchoring to now.
    #[test]
    fn system_time_from_instant_reflects_elapsed() {
        let ten_min_ago = Instant::now() - Duration::from_secs(600);
        let elapsed = system_time_from_instant(ten_min_ago)
            .elapsed()
            .unwrap_or_default();
        assert_eq!(format_time_ago(elapsed), "10m");
    }

    #[test]
    fn parses_every_5_minutes() {
        assert_eq!(parse_schedule_interval_secs("every 5 minutes"), Some(300));
    }

    #[test]
    fn parses_every_5m_short() {
        assert_eq!(parse_schedule_interval_secs("every 5m"), Some(300));
    }

    #[test]
    fn parses_every_10s() {
        assert_eq!(parse_schedule_interval_secs("every 10s"), Some(10));
    }

    #[test]
    fn parses_every_1_hour() {
        assert_eq!(parse_schedule_interval_secs("every 1 hour"), Some(3600));
    }

    #[test]
    fn parses_every_1_day() {
        assert_eq!(parse_schedule_interval_secs("every 1 day"), Some(86400));
    }

    #[test]
    fn decode_html_entities_no_entities() {
        let s = "hello world";
        let out = decode_html_entities(s);
        assert!(matches!(out, std::borrow::Cow::Borrowed(_)));
        assert_eq!(out.as_ref(), s);
    }

    #[test]
    fn decode_html_entities_amp() {
        assert_eq!(decode_html_entities("foo &amp; bar").as_ref(), "foo & bar");
    }

    #[test]
    fn decode_html_entities_multiple() {
        assert_eq!(
            decode_html_entities("1 &lt; 2 &amp;&amp; 3 &gt; 2").as_ref(),
            "1 < 2 && 3 > 2"
        );
    }

    #[test]
    fn decode_html_entities_quotes() {
        assert_eq!(
            decode_html_entities("&quot;hello&quot; &amp; &#39;world&#39;").as_ref(),
            "\"hello\" & 'world'"
        );
    }

    #[test]
    fn unknown_schedule_returns_none() {
        assert_eq!(parse_schedule_interval_secs("foo bar"), None);
        assert_eq!(parse_schedule_interval_secs("every foo"), None);
        assert_eq!(parse_schedule_interval_secs("every 5x"), None);
    }

    #[test]
    fn display_grok_home_prefix_default_install() {
        if std::env::var("GROK_HOME").is_ok() || std::env::var("NEXT_CODE_HOME").is_ok() {
            return;
        }
        assert_eq!(display_grok_home_prefix(), "~/.next-code");
    }

    #[test]
    fn display_user_grok_path_joins_relative() {
        let path = display_user_grok_path("config.toml");
        assert!(path.ends_with("/config.toml") || path.ends_with("\\config.toml"));
        assert!(
            path.contains(".next-code")
                || path.contains("$GROK_HOME")
                || path.contains("$NEXT_CODE_HOME")
        );
    }

    #[test]
    fn abbreviate_path_uses_home_when_under_default_grok() {
        if std::env::var("GROK_HOME").is_ok() || std::env::var("NEXT_CODE_HOME").is_ok() {
            return;
        }
        if let Ok(home) = std::env::var("HOME") {
            if home.is_empty() {
                return;
            }
            let full = format!("{home}/.next-code/memory/MEMORY.md");
            let abbreviated = abbreviate_path(&full);
            assert!(
                abbreviated.contains("memory/MEMORY.md"),
                "got {abbreviated}"
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn abbreviate_path_empty_home_does_not_fake_tilde() {
        let prev = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", "");
        }
        assert_eq!(abbreviate_path("/foo").as_ref(), "/foo");

        match prev {
            Some(home) => unsafe { std::env::set_var("HOME", home) },
            None => unsafe { std::env::remove_var("HOME") },
        }
    }

    #[test]
    fn write_text_resilient_creates_parent_and_overwrites() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("out.md");
        let written = write_text_resilient(&path, "hello", WriteTextOptions::default())
            .expect("first write");
        assert_eq!(written.path, path);
        assert!(!written.redirected);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");

        let written = write_text_resilient(&path, "world", WriteTextOptions::default())
            .expect("overwrite");
        assert_eq!(written.path, path);
        assert!(!written.redirected);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "world");
        // Temp siblings must not linger after a successful rename.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path().join("nested"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains(".tmp")
            })
            .collect();
        assert!(leftovers.is_empty(), "leftover temps: {leftovers:?}");
    }

    #[cfg(windows)]
    #[test]
    fn write_text_resilient_redirects_when_destination_exclusively_locked() {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("locked.md");
        std::fs::write(&path, b"old").expect("seed");

        // share_mode(0) = exclusive; blocks rename-replace (ERROR_SHARING_VIOLATION).
        let guard = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .expect("exclusive open");

        let written = write_text_resilient(&path, "fresh export", WriteTextOptions::default())
            .expect("redirected write");
        assert!(written.redirected, "expected unique sibling path");
        assert_ne!(written.path, path);
        assert_eq!(
            std::fs::read_to_string(&written.path).unwrap(),
            "fresh export"
        );

        drop(guard);
        // Original locked file must be untouched.
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "old");
    }
}
