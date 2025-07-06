use regex::Regex;
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;

/// Formatting options
#[derive(Debug, Clone)]
pub struct Options {
    pub width: usize,
    pub headline_width: usize,
    pub strip_ansi: bool,
}

/// Public API: reflow an entire commit message
pub fn reflow(input: &str, opts: &Options) -> String {
    let mut lines: Vec<&str> = input.lines().collect();

    // Remove any trailing carriage returns
    for l in &mut lines {
        *l = l.trim_end_matches('\r');
    }

    let mut output: Vec<String> = Vec::new();

    // Separate comment lines (#) - we keep them in output but ignore for parsing headline/body
    let mut idx = 0;
    while idx < lines.len() && (lines[idx].starts_with('#') || lines[idx].trim().is_empty()) {
        output.push(lines[idx].to_string());
        idx += 1;
    }

    if idx >= lines.len() {
        return output.join("\n") + "\n"; // nothing else
    }

    // Headline
    let headline = lines[idx];
    output.push(headline.trim_end().to_string());
    idx += 1;

    // Ensure exactly one blank line after headline
    if idx < lines.len() && !lines[idx].trim().is_empty() {
        output.push(String::new());
    } else {
        // skip additional blank lines, keep just one
        while idx < lines.len() && lines[idx].trim().is_empty() {
            idx += 1;
        }
        output.push(String::new());
    }

    // Detect footer start
    let footer_re = Regex::new(r"^[A-Za-z][A-Za-z0-9-]*:").unwrap();
    let mut footer_index = lines.len();
    for (i, line) in lines.iter().enumerate().skip(idx) {
        if footer_re.is_match(line) {
            footer_index = i;
            break;
        }
    }

    let body_lines = &lines[idx..footer_index];
    let footers = &lines[footer_index..];

    // Process body
    let processed_body = process_body(body_lines, opts);
    output.extend(processed_body);

    if !footers.is_empty() {
        if !output.last().map_or(false, |l| l.trim().is_empty()) {
            output.push(String::new());
        }
        output.extend(footers.iter().map(|s| (*s).to_string()));
    }

    output.join("\n") + "\n"
}

fn process_body(lines: &[&str], opts: &Options) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        // Blank line passthrough
        if line.trim().is_empty() {
            out.push(String::new());
            i += 1;
            continue;
        }

        // Code fence detection
        if line.trim_start().starts_with("```") {
            let fence = line.trim_start();
            out.push(line.to_string());
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with(fence) {
                out.push(lines[i].to_string());
                i += 1;
            }
            if i < lines.len() {
                out.push(lines[i].to_string());
                i += 1;
            }
            continue;
        }

        // Indented code block (4+ spaces or tab)
        if line.starts_with("    ") || line.starts_with('\t') {
            while i < lines.len() && (lines[i].starts_with("    ") || lines[i].starts_with('\t')) {
                out.push(lines[i].to_string());
                i += 1;
            }
            continue;
        }

        // Table detection (line contains at least one '|' char and not a list)
        if line.contains('|') {
            while i < lines.len() && lines[i].contains('|') {
                out.push(lines[i].to_string());
                i += 1;
            }
            continue;
        }

        // List detection
        if is_list_item(line) {
            let start_indent = leading_spaces(line);
            while i < lines.len() {
                let cur = lines[i];
                if cur.trim().is_empty() {
                    out.push(String::new());
                    i += 1;
                    break;
                }
                if !is_list_item(cur) && leading_spaces(cur) <= start_indent {
                    break;
                }
                // gather list item lines until next item or less indent
                let mut item_lines = Vec::new();
                item_lines.push(cur);
                i += 1;
                while i < lines.len()
                    && !lines[i].trim().is_empty()
                    && !(is_list_item(lines[i]))
                {
                    // treat as continuation paragraph of current list item
                    item_lines.push(lines[i]);
                    i += 1;
                }
                let bullet_prefix = extract_bullet_prefix(item_lines[0]);
                let text_lines: Vec<&str> = if bullet_prefix.is_empty() {
                    item_lines.clone()
                } else {
                    let mut tl = Vec::new();
                    tl.push(item_lines[0][bullet_prefix.len()..].trim_start());
                    tl.extend(item_lines[1..].iter().map(|s| s.trim_start()));
                    tl
                };
                let wrapped = wrap_text(&text_lines.join(" "), opts.width - bullet_prefix.len());
                for (j, w) in wrapped.iter().enumerate() {
                    if j == 0 {
                        out.push(format!("{}{}", bullet_prefix, w));
                    } else {
                        out.push(format!("{:indent$}{}", "", w, indent = bullet_prefix.len()));
                    }
                }
            }
            continue;
        }

        // URL line (unwrapped if exceeds width)
        let is_url =
            line.trim_start().starts_with("http://") || line.trim_start().starts_with("https://");
        if is_url {
            out.push(line.to_string());
            i += 1;
            continue;
        }

        // Paragraph
        let mut para = Vec::new();
        para.push(line.trim());
        i += 1;
        while i < lines.len() && !lines[i].trim().is_empty() && !is_block_start(lines[i]) {
            para.push(lines[i].trim());
            i += 1;
        }
        let joined = para.join(" ");
        let wrapped = wrap_text(&joined, opts.width);
        out.extend(wrapped);
    }
    out
}

fn is_block_start(line: &str) -> bool {
    line.starts_with("    ")
        || line.starts_with('\t')
        || line.trim_start().starts_with("```")
        || line.contains('|')
        || is_list_item(line)
}

fn is_list_item(line: &str) -> bool {
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

fn extract_bullet_prefix(line: &str) -> &str {
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

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![String::new()];
    }
    let mut out_lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }
        let candidate = format!("{} {}", current, word);
        if display_width(&candidate) > width {
            out_lines.push(current);
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        out_lines.push(current);
    }
    out_lines
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wrap_simple() {
        let options = Options {
            width: 20,
            headline_width: 50,
            strip_ansi: false,
        };
        let input =
            "Subject line\n\nThis is a very long paragraph that should wrap nicely at twenty cols.";
        let expected = "Subject line\n\nThis is a very long\nparagraph that\nshould wrap nicely\nat twenty cols.\n";
        let out = reflow(input, &options);
        assert_eq!(out, expected);
    }
}
