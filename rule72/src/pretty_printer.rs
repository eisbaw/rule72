use crate::types::{Category, ContChunk, Document, ListNode, Options};
use crate::utils::{display_width, extract_bullet_prefix, wrap_text};

/// Pretty print the document structure into formatted text
pub fn pretty_print(doc: &Document, opts: &Options) -> String {
    let mut output = Vec::new();

    // Print headline as-is (no wrapping)
    if let Some(headline) = &doc.headline {
        output.push(headline.text.clone());
    }

    // Print body chunks
    for chunk in &doc.body_chunks {
        match chunk {
            ContChunk::Code(lines) | ContChunk::Comment(lines) | ContChunk::Table(lines) => {
                for line in lines {
                    output.push(line.text.clone());
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
                            output.push(line.text.clone());
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
            output.push(footer.text.clone());
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
            output.push(intro_line.text.clone());
        }
    }

    for item in &list.items {
        let bullet_prefix = extract_bullet_prefix(&item.bullet_line.text);
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
            let wrapped = wrap_text(&full_text, opts.width - bullet_prefix.len());
            for (i, line) in wrapped.iter().enumerate() {
                if i == 0 {
                    output.push(format!("{}{}", bullet_prefix, line));
                } else {
                    output.push(format!(
                        "{:width$}{}",
                        "",
                        line,
                        width = bullet_prefix.len()
                    ));
                }
            }
        } else {
            // Keep original formatting if within width
            output.push(item.bullet_line.text.clone());
            for cont in &item.continuation {
                output.push(cont.text.clone());
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
            strip_ansi: false,
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
}
