//! Lexical analysis: Convert raw text lines into classified Category Lines.
//!
//! This module performs the first stage of processing, analyzing each line
//! individually to assign initial probability scores to different categories
//! (prose, list, code, table, etc.) based on content patterns and indentation.

use std::collections::HashMap;

use crate::types::{CatLine, Category, Options};
use crate::utils::{count_indent, count_special_chars, debug_trace, is_footer_line, is_list_item};

/// Lexer: convert raw lines to CatLines with initial probabilities
pub fn lex_lines(lines: &[&str], opts: &Options) -> Vec<CatLine> {
    debug_trace!(opts, "=== LEXER PHASE ===");
    debug_trace!(opts, "Processing {} input lines", lines.len());

    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            debug_trace!(opts, "Line {}: {:?}", idx + 1, line);
            let mut probabilities = HashMap::new();
            let indent = count_indent(line);
            let trimmed = line.trim();
            debug_trace!(opts, "  Indent: {}, Trimmed: {:?}", indent, trimmed);

            // Initial probabilities based on content patterns
            if trimmed.is_empty() {
                probabilities.insert(Category::Empty, 1.0);
            } else if trimmed.starts_with('#') || trimmed.starts_with("//") {
                probabilities.insert(Category::Comment, 0.9);
                probabilities.insert(Category::ProseGeneral, 0.1);
            } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
                probabilities.insert(Category::Table, 0.8);
                probabilities.insert(Category::Code, 0.2);
            } else if trimmed.starts_with("http") || trimmed.contains("://") {
                probabilities.insert(Category::URL, 0.9);
                probabilities.insert(Category::ProseGeneral, 0.1);
            } else if is_footer_line(trimmed) {
                probabilities.insert(Category::Footer, 0.9);
                probabilities.insert(Category::ProseGeneral, 0.1);
            } else if is_list_item(trimmed) {
                probabilities.insert(Category::List, 0.92);
                probabilities.insert(Category::ProseGeneral, 0.08);
            } else if indent >= 4
                || (!trimmed.is_empty()
                    && count_special_chars(trimmed) as f32 / trimmed.len() as f32 > 0.3)
            {
                probabilities.insert(Category::Code, 0.77);
                probabilities.insert(Category::ProseGeneral, 0.23);
            } else if idx == 0 {
                // First line is likely a headline/subject
                probabilities.insert(Category::ProseGeneral, 0.94);
                probabilities.insert(Category::Code, 0.06);
            } else {
                // Default prose classification
                probabilities.insert(Category::ProseGeneral, 0.8);
                probabilities.insert(Category::ProseIntroduction, 0.2);
            }

            // Find the most likely category
            let final_category = probabilities
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(cat, _)| *cat)
                .unwrap_or(Category::ProseGeneral);

            debug_trace!(opts, "  → Final classification: {:?}", final_category);

            CatLine {
                text: line.to_string(),
                line_number: idx,
                indent,
                probabilities,
                final_category,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let lines = vec![
            "# Comment",
            "Subject line",
            "",
            "Body paragraph",
            "- List item",
            "  continuation",
            "    code block",
            "| table | row |",
            "Signed-off-by: Author <email>",
        ];

        let opts = Options {
            width: 72,
            headline_width: 50,
            debug_svg: None,
            debug_trace: false,
        };
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines.len(), 9);
        assert_eq!(cat_lines[0].final_category, Category::Comment);
        assert_eq!(cat_lines[1].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[2].final_category, Category::Empty);
        assert_eq!(cat_lines[4].final_category, Category::List);
        assert_eq!(cat_lines[6].final_category, Category::Code);
        assert_eq!(cat_lines[7].final_category, Category::Table);
        assert_eq!(cat_lines[8].final_category, Category::Footer);
    }

    #[test]
    fn test_lexer_urls() {
        let lines = vec![
            "Subject line",
            "Check out https://example.com",
            "See http://github.com/user/repo",
            "Visit ftp://files.example.org",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        // First line is subject, others should be URL
        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[1].final_category, Category::URL);
        assert_eq!(cat_lines[2].final_category, Category::URL);
        assert_eq!(cat_lines[3].final_category, Category::URL);
    }

    #[test]
    fn test_lexer_comments() {
        let lines = vec![
            "Subject line",
            "# Hash comment",
            "// Double slash comment",
            "/* Block comment start",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[1].final_category, Category::Comment);
        assert_eq!(cat_lines[2].final_category, Category::Comment);
        // Block comment should be prose or code, not comment (our pattern is specific)
        assert_ne!(cat_lines[3].final_category, Category::Comment);
    }

    #[test]
    fn test_lexer_code_detection() {
        let lines = vec![
            "Subject line",
            "        heavily indented",
            "    function() {",
            "lots!@#$%^&*()of{}special[]chars",
            "normal text with some punctuation.",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[1].final_category, Category::Code); // 8 spaces
        assert_eq!(cat_lines[2].final_category, Category::Code); // 4 spaces
        assert_eq!(cat_lines[3].final_category, Category::Code); // high special char ratio
        assert_eq!(cat_lines[4].final_category, Category::ProseGeneral); // normal text
    }

    #[test]
    fn test_lexer_list_items() {
        let lines = vec![
            "Subject line",
            "* Bullet item",
            "- Dash item",
            "  * Indented bullet",
            "1. Numbered item",
            "2) Paren numbered",
            "10. Double digit",
            "🔥 Emoji bullet",
            "✅ Check emoji",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        for i in 1..cat_lines.len() {
            assert_eq!(cat_lines[i].final_category, Category::List);
        }
    }

    #[test]
    fn test_lexer_tables() {
        let lines = vec![
            "Subject line",
            "| Name | Value |",
            "| foo  | bar   |",
            "|left|right|",
            "| with | spaces |",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        for i in 1..cat_lines.len() {
            assert_eq!(cat_lines[i].final_category, Category::Table);
        }
    }

    #[test]
    fn test_lexer_footers() {
        let lines = vec![
            "Subject line",
            "Signed-off-by: Author <email>",
            "Co-authored-by: Contributor <contrib@example.com>",
            "Reviewed-by: Reviewer <reviewer@example.com>",
            "Fixes: #123",
            "Closes: #456",
            "Resolves: #789",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        for i in 1..cat_lines.len() {
            assert_eq!(cat_lines[i].final_category, Category::Footer);
        }
    }

    #[test]
    fn test_lexer_empty_lines() {
        let lines = vec!["Subject line", "", "   ", "\t", "Body text"];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[1].final_category, Category::Empty);
        assert_eq!(cat_lines[2].final_category, Category::Empty);
        assert_eq!(cat_lines[3].final_category, Category::Empty);
        assert_eq!(cat_lines[4].final_category, Category::ProseGeneral);
    }

    #[test]
    fn test_lexer_indentation_tracking() {
        let lines = vec![
            "Subject line",
            "  two spaces",
            "    four spaces",
            "\tone tab",
            "\t\ttwo tabs",
            "  \tmixed",
        ];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].indent, 0);
        assert_eq!(cat_lines[1].indent, 2);
        assert_eq!(cat_lines[2].indent, 4);
        assert_eq!(cat_lines[3].indent, 4); // tab = 4 spaces
        assert_eq!(cat_lines[4].indent, 8); // 2 tabs = 8 spaces
        assert_eq!(cat_lines[5].indent, 6); // 2 spaces + 1 tab = 6
    }

    #[test]
    fn test_lexer_line_numbers() {
        let lines = vec!["First line", "Second line", "Third line"];

        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines[0].line_number, 0);
        assert_eq!(cat_lines[1].line_number, 1);
        assert_eq!(cat_lines[2].line_number, 2);
    }

    #[test]
    fn test_lexer_empty_input() {
        let lines: Vec<&str> = vec![];
        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert!(cat_lines.is_empty());
    }

    #[test]
    fn test_lexer_probabilities() {
        let lines = vec!["Subject line"];
        let opts = Options::default();
        let cat_lines = lex_lines(&lines, &opts);

        assert_eq!(cat_lines.len(), 1);
        let probabilities = &cat_lines[0].probabilities;

        // Should have probabilities for the classified category
        assert!(probabilities.contains_key(&Category::ProseGeneral));
        assert!(probabilities.get(&Category::ProseGeneral).unwrap() > &0.0);

        // Sum of probabilities should be reasonable (not necessarily 1.0)
        let total: f32 = probabilities.values().sum();
        assert!(total > 0.0);
    }
}
