//! Context-aware classification refinement using neighboring line analysis.
//!
//! This module implements the second stage of processing, using a 4-point
//! FIR-like kernel to examine surrounding context and refine initial
//! classifications for improved accuracy.

use crate::types::{CatLine, Category};

/// Apply context-aware classification to refine initial probabilities
///
/// Uses a 4-point FIR-like kernel examining ±2 neighboring lines to adjust
/// classification probabilities. Center line is excluded to avoid circular
/// reinforcement - we use surrounding context as independent evidence.
pub fn classify_with_context(mut cat_lines: Vec<CatLine>) -> Vec<CatLine> {
    let len = cat_lines.len();

    for i in 0..len {
        let mut new_probabilities = cat_lines[i].probabilities.clone();

        // Look at surrounding context (±2 lines)
        for offset in -2i32..=2i32 {
            if offset == 0 {
                continue;
            }
            let neighbor_idx = i as i32 + offset;
            if neighbor_idx < 0 || neighbor_idx >= len as i32 {
                continue;
            }
            let neighbor = &cat_lines[neighbor_idx as usize];

            // Context-based adjustments
            match neighbor.final_category {
                Category::List => {
                    // Lines near lists are more likely to be lists or prose
                    if cat_lines[i].indent > 0 && cat_lines[i].final_category != Category::Code {
                        *new_probabilities.entry(Category::List).or_insert(0.0) += 0.1;
                        *new_probabilities
                            .entry(Category::ProseGeneral)
                            .or_insert(0.0) += 0.05;
                    }
                }
                Category::Code => {
                    // Lines near code blocks with similar indentation are likely code
                    if cat_lines[i].indent >= 4
                        && cat_lines[i].indent.abs_diff(neighbor.indent) <= 2
                    {
                        *new_probabilities.entry(Category::Code).or_insert(0.0) += 0.15;
                    }
                }
                Category::Table => {
                    // Lines near tables that look table-like get boosted
                    if cat_lines[i].text.contains('|') {
                        *new_probabilities.entry(Category::Table).or_insert(0.0) += 0.2;
                    }
                }
                Category::ProseIntroduction => {
                    // After introduction, next lines are often lists or prose
                    if offset == 1 {
                        *new_probabilities.entry(Category::List).or_insert(0.0) += 0.1;
                        *new_probabilities
                            .entry(Category::ProseGeneral)
                            .or_insert(0.0) += 0.1;
                    }
                }
                _ => {}
            }
        }

        // Special case: lines that end with ":" are often introductions
        if cat_lines[i].text.trim().ends_with(':') && !cat_lines[i].text.contains("http") {
            *new_probabilities
                .entry(Category::ProseIntroduction)
                .or_insert(0.0) += 0.3;
        }

        // Normalize probabilities
        let total: f32 = new_probabilities.values().sum();
        if total > 0.0 {
            for prob in new_probabilities.values_mut() {
                *prob /= total;
            }
        }

        // Update final category based on new probabilities
        let final_category = new_probabilities
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(cat, _)| *cat)
            .unwrap_or(Category::ProseGeneral);

        cat_lines[i].probabilities = new_probabilities;
        cat_lines[i].final_category = final_category;
    }

    cat_lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex_lines;
    use crate::types::Options;

    #[test]
    fn test_context_classification() {
        let lines = vec![
            "Subject line",
            "",
            "This is an introduction:",
            "- First item",
            "- Second item",
            "  continuation",
        ];

        let opts = Options {
            width: 72,
            headline_width: 50,
            strip_ansi: false,
            debug_svg: None,
            debug_trace: false,
        };
        let lexed = lex_lines(&lines, &opts);
        let classified = classify_with_context(lexed);

        // Introduction line should be detected (may be ProseGeneral or ProseIntroduction)
        assert!(matches!(
            classified[2].final_category,
            Category::ProseIntroduction | Category::ProseGeneral
        ));

        // List items should be properly classified
        assert_eq!(classified[3].final_category, Category::List);
        assert_eq!(classified[4].final_category, Category::List);
    }
}
