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
    pretty_print(&document, opts)
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

}
