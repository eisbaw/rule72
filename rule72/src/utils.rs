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
        assert_eq!(result, vec!["short", "verylongwordthatexceedslimit", "more"]);
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
}
