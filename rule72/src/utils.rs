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

pub fn count_indent(line: &str) -> usize {
    line.chars()
        .take_while(|&c| c == ' ' || c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

pub fn count_special_chars(s: &str) -> usize {
    s.chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count()
}

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

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        let word_width = display_width(word);
        let current_width = display_width(&current_line);

        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_width + 1 + word_width <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}
