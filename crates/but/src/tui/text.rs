//! Shared utilities for terminal-width detection and text truncation.
//!
//! Every function here is Unicode-width-aware (CJK, emoji, combining marks)
//! and ANSI-escape-aware so that colored strings are measured and truncated
//! correctly.

use terminal_size::Width;
use unicode_width::UnicodeWidthChar;

/// Returns the current terminal width in columns, defaulting to 80
/// when detection fails (e.g. when stdout is not a TTY).
pub fn terminal_width() -> usize {
    terminal_size::terminal_size().map_or(80, |(Width(w), _)| w as usize)
}

/// Truncate `text` to fit within `max_width` display columns.
///
/// Uses [`unicode_width`] so that CJK / emoji characters (which occupy
/// two terminal columns each) are measured correctly. ANSI escape
/// sequences (e.g. color codes) are passed through without counting
/// toward the width.
///
/// When truncation occurs an `…` character (1 column wide) is appended
/// and the total result is guaranteed to be ≤ `max_width` columns.
pub fn truncate_text(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut width = 0;
    let mut result = String::new();
    let mut in_ansi = false;
    let mut ansi_buffer = String::new();

    for ch in text.chars() {
        // Start of an ANSI escape sequence — buffer it.
        if ch == '\x1b' {
            in_ansi = true;
            ansi_buffer.push(ch);
            continue;
        }

        // Inside an escape sequence — keep buffering until the
        // terminating 'm'.
        if in_ansi {
            ansi_buffer.push(ch);
            if ch == 'm' {
                // Flush the whole escape sequence into the result
                // without counting toward display width.
                result.push_str(&ansi_buffer);
                ansi_buffer.clear();
                in_ansi = false;
            }
            continue;
        }

        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width > max_width {
            // Text will be truncated – reserve 1 column for '…'.
            // Walk back if needed so the ellipsis still fits.
            while width >= max_width {
                if let Some(last) = result.pop() {
                    // If we popped an ANSI terminator we need to
                    // discard the whole escape sequence we just
                    // partially undid.  In practice this is unlikely
                    // since escapes are zero-width, but be safe.
                    if last == 'm' {
                        // Pop back to the ESC that started this sequence.
                        while let Some(c) = result.pop() {
                            if c == '\x1b' {
                                break;
                            }
                        }
                        // The escape was zero-width, keep walking.
                        continue;
                    }
                    width -= last.width().unwrap_or(0);
                } else {
                    break;
                }
            }
            result.push('…');
            return result;
        }
        result.push(ch);
        width += ch_width;
    }

    // No truncation needed.
    result
}

/// Remove all ANSI escape sequences from `s`, returning plaintext.
///
/// Useful when you need to measure the *display* width of a string
/// that may contain color / style codes.
pub fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;

    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }

        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }

        result.push(ch);
    }

    result
}

#[cfg(test)]
mod tests {
    use unicode_width::UnicodeWidthStr;

    use super::{strip_ansi_codes, truncate_text};

    // ── Plain-text truncation ────────────────────────────────────

    #[test]
    fn short_text_is_not_truncated() {
        assert_eq!(truncate_text("hello", 10), "hello");
    }

    #[test]
    fn text_at_exact_limit_is_not_truncated() {
        assert_eq!(truncate_text("hello", 5), "hello");
    }

    #[test]
    fn text_exceeding_limit_is_truncated_with_ellipsis() {
        assert_eq!(truncate_text("hello world", 5), "hell…");
    }

    #[test]
    fn empty_text_stays_empty() {
        assert_eq!(truncate_text("", 10), "");
    }

    #[test]
    fn max_width_of_zero_gives_empty_string() {
        assert_eq!(truncate_text("hello", 0), "");
    }

    #[test]
    fn max_width_of_one_gives_ellipsis_only() {
        assert_eq!(truncate_text("hello", 1), "…");
    }

    #[test]
    fn unicode_single_width_characters_are_handled() {
        // ü is a single-width character (1 display column)
        assert_eq!(truncate_text("über-cool", 5), "über…");
    }

    #[test]
    fn cjk_double_width_characters_are_handled() {
        // Each CJK character occupies 2 display columns.
        // 你(2) + 好(2) = 4 cols, + …(1) = 5 cols total.
        assert_eq!(truncate_text("你好世界", 5), "你好…");
        assert_eq!(truncate_text("你好世界", 5).width(), 5);
    }

    #[test]
    fn cjk_truncation_does_not_exceed_max_width() {
        // With max_width 4, a second CJK char (2 cols) leaves no room
        // for the ellipsis alongside it, so only the first char + … fits.
        // 你(2) + …(1) = 3 cols ≤ 4
        let result = truncate_text("你好世界", 4);
        assert!(result.width() <= 4);
        assert_eq!(result, "你…");
    }

    #[test]
    fn truncation_preserves_exact_boundary() {
        let msg = "this is a overly long commit message to demonstrate truncation";
        let result = truncate_text(msg, 60);
        assert!(result.ends_with('…'));
        // For ASCII text, display width == char count
        assert_eq!(result.width(), 60);
    }

    #[test]
    fn emoji_characters_are_handled() {
        // Many emoji are wide characters; ensure we respect their display width.
        let single = "🙂";
        let single_width = UnicodeWidthStr::width(single);
        assert!(single_width >= 1);
        // A single emoji that fits within max_width should not be truncated.
        assert_eq!(truncate_text(single, single_width), single);
        // Repeated emoji should be truncated without exceeding max_width.
        let repeated = "🙂🙂🙂";
        let result = truncate_text(repeated, single_width * 2);
        assert!(result.width() <= single_width * 2);
    }

    #[test]
    fn zero_width_combining_characters_are_handled() {
        // "a" + COMBINING ACUTE ACCENT; display width should be 1.
        let text = "a\u{0301}";
        assert_eq!(UnicodeWidthStr::width(text), 1);
        // With max_width equal to the display width, no truncation should occur.
        assert_eq!(truncate_text(text, 1), text);
    }

    // ── ANSI-aware truncation ────────────────────────────────────

    #[test]
    fn ansi_colored_text_is_truncated_without_counting_escapes() {
        // "\x1b[31m" = red, "\x1b[0m" = reset — both zero-width.
        let colored = "\x1b[31mhello world\x1b[0m";
        let result = truncate_text(colored, 5);
        // The ANSI prefix should be preserved and the visible text
        // truncated to 4 chars + ellipsis.
        assert!(result.starts_with("\x1b[31m"));
        let plain = strip_ansi_codes(&result);
        assert_eq!(plain, "hell…");
    }

    #[test]
    fn ansi_codes_only_produce_no_visible_output_within_width() {
        let just_color = "\x1b[31m\x1b[0m";
        assert_eq!(truncate_text(just_color, 5), just_color);
    }

    #[test]
    fn plain_text_is_unchanged_by_strip_ansi() {
        assert_eq!(strip_ansi_codes("hello world"), "hello world");
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        let colored = "\x1b[31mhello\x1b[0m world";
        assert_eq!(strip_ansi_codes(colored), "hello world");
    }
}
