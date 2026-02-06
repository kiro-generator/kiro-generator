use std::fmt::Write;

/// Escape a filesystem path for use as a systemd unit instance name.
///
/// Implements the same algorithm as `systemd-escape --path`:
/// - Leading/trailing slashes are stripped
/// - Consecutive slashes are collapsed to a single dash
/// - `/` becomes `-`
/// - Any character outside `[a-zA-Z0-9:_.]` is hex-escaped as `\xHH`
/// - The root path `/` becomes `-`
///
/// See systemd.unit(5) for the full specification.
pub fn escape_path(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return "-".to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_was_slash = false;
    for ch in trimmed.chars() {
        if ch == '/' {
            if !prev_was_slash {
                out.push('-');
                prev_was_slash = true;
            }
        } else if ch.is_ascii_alphanumeric() || ch == ':' || ch == '_' || ch == '.' {
            out.push(ch);
            prev_was_slash = false;
        } else {
            write!(out, "\\x{:02x}", ch as u32).unwrap();
            prev_was_slash = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_simple_path() {
        assert_eq!(escape_path("/home/user/project"), "home-user-project");
    }

    #[test]
    fn test_escape_root() {
        assert_eq!(escape_path("/"), "-");
    }

    #[test]
    fn test_escape_trailing_slashes() {
        assert_eq!(escape_path("/home/user/"), "home-user");
    }

    #[test]
    fn test_escape_consecutive_slashes() {
        assert_eq!(escape_path("/home//user"), "home-user");
        assert_eq!(escape_path("/home///user///project"), "home-user-project");
    }

    #[test]
    fn test_escape_special_chars() {
        assert_eq!(
            escape_path("/home/user/my project"),
            "home-user-my\\x20project"
        );
    }
}
