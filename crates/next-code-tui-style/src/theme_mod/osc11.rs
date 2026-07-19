//! OSC 11 terminal background detection.
//! Copied/adapted from grok-build osc11.rs.
//!
//! Queries the terminal's background color:
//!   Query: `\x1b]11;?\x07`
//!   Reply: `\x1b]11;rgb:RRRR/GGGG/BBBB\x07`

use super::system_appearance::SystemAppearance;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::io::RawFd;

const OSC_11_QUERY: &[u8] = b"\x1b]11;?\x07";
const OSC_11_PREFIX: &[u8] = b"\x1b]11;rgb:";
const OSC_TERMINATORS: &[u8] = b"\x07\x1b\\";
const OSC_11_TIMEOUT: Duration = Duration::from_millis(500);

/// Detect terminal background via OSC 11.
/// Must be called BEFORE crossterm EventStream init.
pub fn detect_via_osc11() -> Option<SystemAppearance> {
    let response = query_osc11()?;
    let rgb = parse_osc11_rgb(&response)?;
    classify_luminance(rgb)
}

fn query_osc11() -> Option<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::io::Read;
        let mut stdin_fd: RawFd = 0; // stdin
        // Write query to stderr (safer than stdout during TUI init)
        let stderr_fd = 2;
        let result = unsafe {
            libc::write(stderr_fd, OSC_11_QUERY.as_ptr() as *const _, OSC_11_QUERY.len())
        };
        if result < 0 {
            return None;
        }
        // Read response from stdin
        let mut buf = [0u8; 512];
        let mut read_buf = Vec::new();
        // Non-blocking read with timeout
        let start = std::time::Instant::now();
        while start.elapsed() < OSC_11_TIMEOUT {
            // Set stdin to non-blocking
            unsafe {
                let flags = libc::fcntl(stdin_fd, libc::F_GETFL);
                if flags < 0 { break; }
                libc::fcntl(stdin_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
            match std::io::stdin().read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    read_buf.extend_from_slice(&buf[..n]);
                    // Check for OSC terminator
                    if read_buf.iter().any(|&b| b == 0x07 || b == 0x1c) {
                        break;
                    }
                    // Restore blocking
                    unsafe {
                        let flags = libc::fcntl(stdin_fd, libc::F_GETFL);
                        if flags >= 0 {
                            libc::fcntl(stdin_fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(_) => break,
            }
        }
        if read_buf.is_empty() { None } else { Some(read_buf) }
    }
    #[cfg(not(unix))]
    { None }
}

fn parse_osc11_rgb(data: &[u8]) -> Option<(u8, u8, u8)> {
    // Find OSC 11 prefix
    let start = data.windows(OSC_11_PREFIX.len()).position(|w| w == OSC_11_PREFIX)?;
    let rgb_data = &data[start + OSC_11_PREFIX.len()..];
    // Find terminator
    let end = rgb_data.iter().position(|&b| OSC_TERMINATORS.contains(&b)).unwrap_or(rgb_data.len());
    let rgb_str = std::str::from_utf8(&rgb_data[..end]).ok()?;
    // Parse "RRRR/GGGG/BBBB" (4-digit) or "RR/GG/BB" (2-digit)
    let parts: Vec<&str> = rgb_str.split('/').collect();
    if parts.len() != 3 { return None; }
    let parse_channel = |s: &str| -> Option<u8> {
        if s.len() == 4 {
            // 4-digit hex: extract high byte
            u8::from_str_radix(&s[..2], 16).ok()
        } else if s.len() == 2 {
            u8::from_str_radix(s, 16).ok()
        } else {
            None
        }
    };
    Some((parse_channel(parts[0])?, parse_channel(parts[1])?, parse_channel(parts[2])?))
}

/// Classify luminance using ITU-R BT.709.
fn classify_luminance((r, g, b): (u8, u8, u8)) -> Option<SystemAppearance> {
    let srgb_to_linear = |c: f32| -> f32 {
        let c = c / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    let r_lin = srgb_to_linear(r as f32);
    let g_lin = srgb_to_linear(g as f32);
    let b_lin = srgb_to_linear(b as f32);
    let y = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
    Some(if y < 0.5 { SystemAppearance::Dark } else { SystemAppearance::Light })
}
