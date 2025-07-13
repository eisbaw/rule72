//! # rule72 - Git commit message reflow tool
//!
//! A smart command-line formatter that rewraps Git commit messages while preserving
//! structure (headlines, paragraphs, lists, code blocks, footers, etc.).
//!
//! ## Algorithm
//! 1. **Lexical Analysis**: Classify each line with probability scores
//! 2. **Context Refinement**: Use 4-point FIR-like kernel on neighbors
//! 3. **Document Building**: Group lines into semantic chunks
//! 4. **Pretty Printing**: Format each chunk type appropriately
//!
//! ## Example
//! ```rust
//! use rule72::{reflow, Options};
//!
//! let input = "Very long commit message that needs to be wrapped...";
//! let opts = Options::default();
//! let output = reflow(input, &opts);
//! ```

// Public modules
pub mod classifier;
pub mod debug;
pub mod lexer;
pub mod pretty_printer;
pub mod tree_builder;
pub mod types;
pub mod utils;

// Re-export public API types
pub use types::{CatLine, Category, ContChunk, Document, ListItem, ListNode, Options};

// Re-export main functions
pub use classifier::classify_with_context;
pub use debug::generate_debug_svg;
pub use lexer::lex_lines;
pub use pretty_printer::pretty_print;
pub use tree_builder::build_document;

/// Remove trailing whitespace from all lines in the text
fn remove_trailing_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        + if text.ends_with('\n') { "\n" } else { "" }
}

/// Public API: reflow an entire commit message
pub fn reflow(input: &str, opts: &Options) -> String {
    let lines: Vec<&str> = input.lines().map(|l| l.trim_end_matches('\r')).collect();

    // Lex lines into CatLines
    let cat_lines = lex_lines(&lines, opts);

    // Apply context-aware classification
    let classified_lines = classify_with_context(cat_lines);

    // Build document structure
    let document = build_document(classified_lines);

    // Generate debug SVG if requested
    if let Some(svg_path) = &opts.debug_svg {
        generate_debug_svg(&document, svg_path);
    }

    // Pretty print the document
    let output = pretty_print(&document, opts);

    // Post-processing: remove trailing whitespace from all lines
    remove_trailing_whitespace(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() {
        let input =
            "Subject line\n\n- First item\n- Second item\n\nSigned-off-by: Test <test@example.com>";
        let opts = Options {
            width: 72,
            headline_width: 50,
            debug_svg: None,
            debug_trace: false,
        };

        let output = reflow(input, &opts);
        assert!(output.contains("Subject line"));
        assert!(output.contains("- First item"));
        assert!(output.contains("Signed-off-by:"));
    }

    #[test]
    fn test_remove_trailing_whitespace() {
        // Test basic trailing whitespace removal
        let input = "line one  \nline two\t\nline three   \n";
        let expected = "line one\nline two\nline three\n";
        assert_eq!(remove_trailing_whitespace(input), expected);

        // Test preserving final newline
        let input_with_newline = "content  \n";
        let expected_with_newline = "content\n";
        assert_eq!(
            remove_trailing_whitespace(input_with_newline),
            expected_with_newline
        );

        // Test without final newline
        let input_no_newline = "content  ";
        let expected_no_newline = "content";
        assert_eq!(
            remove_trailing_whitespace(input_no_newline),
            expected_no_newline
        );

        // Test empty string
        assert_eq!(remove_trailing_whitespace(""), "");

        // Test string with only whitespace
        assert_eq!(remove_trailing_whitespace("   \n  \t\n"), "\n\n");

        // Test mixed content
        let mixed = "normal line\n  line with spaces  \n\ttabbed line\t\n  \n";
        let expected_mixed = "normal line\n  line with spaces\n\ttabbed line\n\n";
        assert_eq!(remove_trailing_whitespace(mixed), expected_mixed);
    }

    #[test]
    fn test_trailing_whitespace_integration() {
        // Test that the full reflow pipeline removes trailing whitespace
        let input = "Subject with trailing spaces  \n\n- List item with spaces   \n  continuation with spaces  \n\nSigned-off-by: Author  ";
        let opts = Options::default();

        let output = reflow(input, &opts);

        // Verify no trailing whitespace on any line
        for line in output.lines() {
            assert_eq!(
                line,
                line.trim_end(),
                "Line should not have trailing whitespace: '{}'",
                line
            );
        }

        // Verify content is preserved
        assert!(output.contains("Subject with trailing spaces"));
        assert!(output.contains("- List item with spaces"));
        assert!(output.contains("Signed-off-by: Author"));
    }

    #[test]
    fn test_trailing_whitespace_preserves_indentation() {
        // Test that leading whitespace is preserved while trailing is removed
        let input = "  indented line with trailing  \n    more indented  \n";
        let expected = "  indented line with trailing\n    more indented\n";
        assert_eq!(remove_trailing_whitespace(input), expected);
    }
}
