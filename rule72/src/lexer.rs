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
                || count_special_chars(trimmed) as f32 / trimmed.len() as f32 > 0.3
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
            strip_ansi: false,
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
}
