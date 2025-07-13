//! Utility functions for text analysis, formatting, and debug tracing.
//!
//! This module provides helper functions used throughout the codebase for
//! indentation counting, text wrapping, list detection, footer recognition,
//! and debug output with automatic file:line prefixes.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Debug trace macro that includes file and line information
macro_rules! debug_trace {
    ($opts:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        if $opts.debug_trace {
            eprintln!("[{}:{}] {}", file!(), line!(), format!($fmt $(, $($arg)*)?));
        }
    };
}

pub(crate) use debug_trace;

/// Count leading whitespace characters (spaces and tabs) in a line.
/// Tabs are treated as four spaces when measuring indentation.
pub fn count_indent(line: &str) -> usize {
    line.chars()
        .take_while(|&c| c == ' ' || c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

/// Count special characters that might indicate code content.
/// Returns the number of characters that are typically found in code
/// (symbols, punctuation, etc.) rather than natural language.
pub fn count_special_chars(s: &str) -> usize {
    s.chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count()
}

/// Check if a line matches Git footer patterns (tag: value format).
/// Recognizes common Git trailers like "Signed-off-by:", "Co-authored-by:", etc.
pub fn is_footer_line(line: &str) -> bool {
    // Common footer tags - be very specific about what we consider footers
    let footer_tags = [
        "Signed-off-by:",
        "Co-authored-by:",
        "Reviewed-by:",
        "Acked-by:",
        "Tested-by:",
        "Reported-by:",
        "Suggested-by:",
        "Fixes:",
        "Closes:",
        "Resolves:",
        "See-also:",
        "Ref:",
        "References:",
    ];

    // Check if line starts with a known footer tag
    for tag in &footer_tags {
        if line.starts_with(tag) {
            return true;
        }
    }

    // Don't use generic pattern matching - it's too broad and catches regular content
    // like "EN: something broke" which are clearly not footers
    false
}

/// Detect if a line is a list item (bullet, numbered, or emoji).
/// Recognizes common list markers including markdown bullets, numbers, and emoji.
pub fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
        return true;
    }

    // Numbered list (e.g., "1." or "2)")
    let digits = trimmed.chars().take_while(|c| c.is_ascii_digit());
    let digit_count = digits.clone().count();
    if digit_count > 0 {
        let rest = &trimmed[digit_count..];
        if rest.starts_with(") ") || rest.starts_with(". ") {
            return true;
        }
    }

    // Emoji or other grapheme cluster bullet followed by space
    let mut graphemes = trimmed.graphemes(true);
    if let Some(first_cluster) = graphemes.next() {
        if !first_cluster.is_ascii() {
            if let Some(rest) = trimmed.get(first_cluster.len()..first_cluster.len() + 1) {
                return rest == " ";
            }
        }
    }
    false
}

/// Extract the bullet prefix from a list item line.
/// Returns the bullet marker (including trailing space) that should be
/// preserved when wrapping list content.
pub fn extract_bullet_prefix(line: &str) -> &str {
    let trimmed_start = line.trim_start_matches(' ');
    let offset = line.len() - trimmed_start.len();

    // Identify grapheme cluster or ascii bullet
    let mut idx = offset;
    for (byte_idx, ch) in trimmed_start.char_indices() {
        idx = offset + byte_idx;
        if ch == ' ' {
            // include the space and any following spaces
            idx += 1;
            break;
        }
    }
    while idx < line.len() && &line[idx..idx + 1] == " " {
        idx += 1;
    }
    &line[..idx]
}

/// Wrap text to specified width using greedy wrapping algorithm.
/// Preserves word boundaries and handles Unicode characters correctly.
/// Words longer than the width limit are placed on their own line.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = display_width(word);

        // Handle words longer than width limit
        if word_width > width {
            // If current line has content, finish it first
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }
            // Add the long word as its own line
            lines.push(word.to_string());
            continue;
        }

        if current_line.is_empty() {
            current_line.push_str(word);
            current_width = word_width;
        } else if current_width + 1 + word_width <= width {
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Calculate the display width of text, handling Unicode characters properly.
/// Returns the number of columns the text would occupy in a terminal,
/// accounting for wide characters, combining marks, etc.
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_basic() {
        let result = wrap_text("hello world", 15);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn test_wrap_text_wrapping() {
        let result = wrap_text("hello world this is a test", 10);
        assert_eq!(result, vec!["hello", "world this", "is a test"]);
    }

    #[test]
    fn test_wrap_text_long_word() {
        let result = wrap_text("short verylongwordthatexceedslimit more", 10);
        assert_eq!(
            result,
            vec!["short", "verylongwordthatexceedslimit", "more"]
        );
    }

    #[test]
    fn test_wrap_text_empty() {
        let result = wrap_text("", 10);
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_wrap_text_unicode() {
        let result = wrap_text("🔥 hello 世界", 10);
        assert_eq!(result, vec!["🔥 hello", "世界"]);
    }

    #[test]
    fn test_count_indent() {
        assert_eq!(count_indent("hello"), 0);
        assert_eq!(count_indent("  hello"), 2);
        assert_eq!(count_indent("    hello"), 4);
        assert_eq!(count_indent("\thello"), 4);
        assert_eq!(count_indent("\t\thello"), 8);
        assert_eq!(count_indent("  \thello"), 6);
        assert_eq!(count_indent(""), 0);
        assert_eq!(count_indent("   "), 3);
    }

    #[test]
    fn test_count_special_chars() {
        assert_eq!(count_special_chars("hello world"), 0); // space is whitespace, not special
        assert_eq!(count_special_chars("hello-world"), 1); // hyphen
        assert_eq!(count_special_chars("foo(bar)"), 2); // parentheses
        assert_eq!(count_special_chars("abc123"), 0); // alphanumeric only
        assert_eq!(count_special_chars("!@#$%^&*()"), 10); // all special
        assert_eq!(count_special_chars(""), 0); // empty
        assert_eq!(count_special_chars("   "), 0); // whitespace only
    }

    #[test]
    fn test_is_footer_line() {
        assert!(is_footer_line("Signed-off-by: John Doe <john@example.com>"));
        assert!(is_footer_line(
            "Co-authored-by: Jane Smith <jane@example.com>"
        ));
        assert!(is_footer_line("Reviewed-by: Bob Wilson"));
        assert!(is_footer_line("Acked-by: Alice Brown"));
        assert!(is_footer_line("Tested-by: Charlie Davis"));
        assert!(is_footer_line("Fixes: #123"));
        assert!(is_footer_line("Closes: #456"));
        assert!(is_footer_line("Resolves: #789"));

        // Should not match non-footer lines
        assert!(!is_footer_line("This is a regular line"));
        assert!(!is_footer_line("EN: something broke")); // not a git footer
        assert!(!is_footer_line("Random: text"));
        assert!(!is_footer_line(""));
        assert!(!is_footer_line("Subject: this is not a footer"));
    }

    #[test]
    fn test_is_list_item() {
        // Bullet lists
        assert!(is_list_item("* First item"));
        assert!(is_list_item("- Second item"));
        assert!(is_list_item("  * Indented item"));
        assert!(is_list_item("    - More indented"));

        // Numbered lists
        assert!(is_list_item("1. First numbered"));
        assert!(is_list_item("2) Second numbered"));
        assert!(is_list_item("10. Double digit"));
        assert!(is_list_item("  3. Indented numbered"));

        // Emoji bullets
        assert!(is_list_item("🔥 Fire bullet"));
        assert!(is_list_item("✅ Check bullet"));
        assert!(is_list_item("  🚀 Indented emoji"));

        // Should not match non-list items
        assert!(!is_list_item("Regular text"));
        assert!(!is_list_item("*no space after asterisk"));
        assert!(!is_list_item("-no space after dash"));
        assert!(!is_list_item("1.no space after number"));
        assert!(!is_list_item(""));
        assert!(!is_list_item("   "));
    }

    #[test]
    fn test_extract_bullet_prefix() {
        assert_eq!(extract_bullet_prefix("* First item"), "* ");
        assert_eq!(extract_bullet_prefix("- Second item"), "- ");
        assert_eq!(extract_bullet_prefix("  * Indented item"), "  * ");
        assert_eq!(extract_bullet_prefix("    - More indented"), "    - ");
        assert_eq!(extract_bullet_prefix("1. First numbered"), "1. ");
        assert_eq!(extract_bullet_prefix("2) Second numbered"), "2) ");
        assert_eq!(extract_bullet_prefix("10. Double digit"), "10. ");
        assert_eq!(extract_bullet_prefix("  3. Indented numbered"), "  3. ");
        assert_eq!(extract_bullet_prefix("🔥 Fire bullet"), "🔥 ");
        assert_eq!(extract_bullet_prefix("✅ Check bullet"), "✅ ");
        assert_eq!(extract_bullet_prefix("  🚀 Indented emoji"), "  🚀 ");

        // Edge cases
        assert_eq!(extract_bullet_prefix("*  Multiple spaces"), "*  ");
        assert_eq!(extract_bullet_prefix("1.   Extra spaces"), "1.   ");
    }

    #[test]
    fn test_display_width() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("世界"), 4); // Wide characters
        assert_eq!(display_width("🔥"), 2); // Emoji
        assert_eq!(display_width(""), 0);
        assert_eq!(display_width("a🔥b"), 4); // Mixed
        assert_eq!(display_width("héllo"), 5); // Accented characters
    }
}
