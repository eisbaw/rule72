//! Pretty printing: Format document chunks with appropriate wrapping and spacing.
//!
//! This module handles the final formatting stage, applying content-aware
//! formatting rules to each chunk type (greedy wrap for prose, verbatim for
//! code, proper indentation for lists, etc.).

use crate::types::{Category, ContChunk, Document, ListNode, Options};
use crate::utils::{display_width, extract_bullet_prefix, wrap_text};

/// Pretty print the document structure into formatted text
pub fn pretty_print(doc: &Document, opts: &Options) -> String {
    let mut output = Vec::new();

    // Print headline as-is (no wrapping)
    if let Some(headline) = &doc.headline {
        output.push(headline.text.trim_end().to_string());
    }

    // Print body chunks
    for chunk in &doc.body_chunks {
        match chunk {
            ContChunk::Code(lines) | ContChunk::Comment(lines) | ContChunk::Table(lines) => {
                for line in lines {
                    output.push(line.text.trim_end().to_string());
                }
            }
            ContChunk::Paragraph(lines) => {
                // Check if this is just an empty line
                if lines.len() == 1 && lines[0].final_category == Category::Empty {
                    output.push(String::new());
                } else {
                    let needs_wrap = lines.iter().any(|l| display_width(&l.text) > opts.width);
                    if needs_wrap {
                        let text = lines
                            .iter()
                            .map(|l| l.text.trim())
                            .collect::<Vec<_>>()
                            .join(" ");
                        let wrapped = wrap_text(&text, opts.width);
                        output.extend(wrapped);
                    } else {
                        for line in lines {
                            output.push(line.text.trim_end().to_string());
                        }
                    }
                }
            }
            ContChunk::List(list_node) => {
                output.extend(pretty_print_list(list_node, opts, 0));
            }
        }
    }

    // Print footers
    if !doc.footers.is_empty() {
        output.push(String::new()); // Blank line before footers
        for footer in &doc.footers {
            output.push(footer.text.trim_end().to_string());
        }
    }

    output.join("\n") + "\n"
}

/// Pretty print a list node with proper indentation and wrapping
pub fn pretty_print_list(list: &ListNode, opts: &Options, _depth: usize) -> Vec<String> {
    let mut output = Vec::new();

    // Print introduction lines first
    for intro_line in &list.introduction {
        if intro_line.final_category == Category::Empty {
            output.push(String::new());
        } else {
            output.push(intro_line.text.trim_end().to_string());
        }
    }

    for item in &list.items {
        let bullet_prefix = extract_bullet_prefix(&item.bullet_line.text);
        let bullet_width = display_width(bullet_prefix);
        let text_start = item.bullet_line.text[bullet_prefix.len()..].trim_start();

        // Combine bullet line and continuation
        let mut full_text = text_start.to_string();
        for cont in &item.continuation {
            full_text.push(' ');
            full_text.push_str(cont.text.trim());
        }

        // Check if wrapping is needed
        let first_line = format!("{}{}", bullet_prefix, text_start);
        if display_width(&first_line) > opts.width
            || item
                .continuation
                .iter()
                .any(|l| display_width(&l.text) > opts.width)
        {
            let wrapped = wrap_text(&full_text, opts.width - bullet_width);
            for (i, line) in wrapped.iter().enumerate() {
                if i == 0 {
                    output.push(format!("{}{}", bullet_prefix, line));
                } else {
                    let padding = " ".repeat(bullet_width);
                    output.push(format!("{}{}", padding, line));
                }
            }
        } else {
            // Keep original formatting if within width
            output.push(item.bullet_line.text.trim_end().to_string());
            for cont in &item.continuation {
                output.push(cont.text.trim_end().to_string());
            }
        }

        // Handle nested list
        if let Some(nested) = &item.nested {
            output.extend(pretty_print_list(nested, opts, _depth + 1));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::classify_with_context;
    use crate::lexer::lex_lines;
    use crate::tree_builder::build_document;

    #[test]
    fn test_wrap_simple() {
        let text = "This is a long line that should be wrapped at some reasonable point to fit within the specified width limit";
        let wrapped = wrap_text(text, 20);

        assert!(wrapped.len() > 1);
        for line in wrapped {
            assert!(display_width(&line) <= 20);
        }
    }

    #[test]
    fn test_pretty_print_integration() {
        let lines = vec![
            "Short subject",
            "",
            "- A very long list item that should definitely be wrapped when it exceeds the specified width limit for the document",
            "- Short item",
        ];

        let opts = Options {
            width: 50,
            headline_width: 50,
            debug_svg: None,
            debug_trace: false,
        };

        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        assert!(output.contains("Short subject"));
        assert!(output.contains("- A very long list item"));
    }

    #[test]
    fn test_pretty_print_code_blocks() {
        let lines = vec![
            "Subject line",
            "",
            "Example:",
            "    function test() {",
            "        return true;",
            "    }",
        ];

        let opts = Options::default();
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // Code should be preserved as-is
        assert!(output.contains("    function test() {"));
        assert!(output.contains("        return true;"));
        assert!(output.contains("    }"));
    }

    #[test]
    fn test_pretty_print_tables() {
        let lines = vec![
            "Subject line",
            "",
            "Data:",
            "| Name | Value |",
            "| foo  | bar   |",
        ];

        let opts = Options::default();
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // Tables should be preserved as-is
        assert!(output.contains("| Name | Value |"));
        assert!(output.contains("| foo  | bar   |"));
    }

    #[test]
    fn test_pretty_print_comments() {
        let lines = vec![
            "Subject line",
            "",
            "# This is a comment",
            "// Another comment",
        ];

        let opts = Options::default();
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // Comments should be preserved as-is
        assert!(output.contains("# This is a comment"));
        assert!(output.contains("// Another comment"));
    }

    #[test]
    fn test_pretty_print_footers() {
        let lines = vec![
            "Subject line",
            "",
            "Body text",
            "",
            "Signed-off-by: Author <email>",
            "Co-authored-by: Contributor <contrib@example.com>",
        ];

        let opts = Options::default();
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // Footers should be separated by blank line
        assert!(output.contains("Signed-off-by: Author <email>"));
        assert!(output.contains("Co-authored-by: Contributor <contrib@example.com>"));

        // Should have blank line before footers
        let lines: Vec<&str> = output.trim().split('\n').collect();
        let signed_off_idx = lines
            .iter()
            .position(|&l| l.contains("Signed-off-by"))
            .unwrap();
        assert!(signed_off_idx > 0);
        assert_eq!(lines[signed_off_idx - 1], "");
    }

    #[test]
    fn test_pretty_print_empty_lines() {
        let lines = vec!["Subject line", "", "", "Body text"];

        let opts = Options::default();
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // Empty lines should be preserved
        assert!(output.contains("Subject line"));
        assert!(output.contains("Body text"));

        // Should have empty lines in output
        let output_lines: Vec<&str> = output.trim().split('\n').collect();
        assert!(output_lines.iter().any(|&l| l.is_empty()));
    }

    #[test]
    fn test_pretty_print_paragraph_wrapping() {
        let lines = vec![
            "Subject line",
            "",
            "This is a very long paragraph that should definitely be wrapped when it exceeds the specified width limit for the document formatting",
        ];

        let opts = Options {
            width: 50,
            headline_width: 50,
            debug_svg: None,
            debug_trace: false,
        };

        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // Paragraph should be wrapped
        let output_lines: Vec<&str> = output.trim().split('\n').collect();
        let body_lines: Vec<&str> = output_lines
            .iter()
            .filter(|&&l| !l.is_empty() && l != "Subject line")
            .cloned()
            .collect();

        // Should have multiple lines from wrapping
        assert!(body_lines.len() > 1);

        // Each line should be within width limit
        for line in body_lines {
            assert!(display_width(line) <= 50);
        }
    }

    #[test]
    fn test_pretty_print_mixed_content() {
        let lines = vec![
            "Subject line",
            "",
            "Introduction paragraph",
            "",
            "- List item one",
            "- List item two",
            "",
            "    code block",
            "",
            "Final paragraph",
            "",
            "Signed-off-by: Author <email>",
        ];

        let opts = Options::default();
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);
        let output = pretty_print(&document, &opts);

        // All content types should be present
        assert!(output.contains("Subject line"));
        assert!(output.contains("Introduction paragraph"));
        assert!(output.contains("- List item one"));
        assert!(output.contains("    code block"));
        assert!(output.contains("Final paragraph"));
        assert!(output.contains("Signed-off-by: Author <email>"));
    }

    #[test]
    fn test_pretty_print_empty_document() {
        let document = Document {
            headline: None,
            body_chunks: Vec::new(),
            footers: Vec::new(),
        };

        let opts = Options::default();
        let output = pretty_print(&document, &opts);

        // Should just be a newline
        assert_eq!(output, "\n");
    }
}
