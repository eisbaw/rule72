//! Core data structures for commit message parsing and classification.
//!
//! This module defines the fundamental types used throughout the rule72 pipeline:
//! - Configuration options
//! - Line categories and classification data
//! - Document structure representation

use std::collections::HashMap;

/// Formatting options for commit message reflow
#[derive(Debug, Clone)]
pub struct Options {
    pub width: usize,
    pub headline_width: usize,
    pub debug_svg: Option<String>,
    pub debug_trace: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 72,
            headline_width: 50,
            debug_svg: None,
            debug_trace: false,
        }
    }
}

/// Line categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    ProseIntroduction,
    ProseGeneral,
    List,
    Code,
    Table,
    URL,
    Empty,
    Comment,
    Footer,
}

/// Categorical line with classification probabilities
#[derive(Debug, Clone)]
pub struct CatLine {
    pub text: String,
    pub line_number: usize,
    pub indent: usize,
    pub probabilities: HashMap<Category, f32>,
    pub final_category: Category,
}

/// Contiguous chunk types in the tree structure
#[derive(Debug)]
pub enum ContChunk {
    Table(Vec<CatLine>),
    Paragraph(Vec<CatLine>),
    List(ListNode),
    Code(Vec<CatLine>),
    Comment(Vec<CatLine>),
}

#[derive(Debug)]
pub struct ListNode {
    pub introduction: Vec<CatLine>, // Introduction lines that precede the list
    pub items: Vec<ListItem>,
}

#[derive(Debug)]
pub struct ListItem {
    pub bullet_line: CatLine,
    pub continuation: Vec<CatLine>,
    pub nested: Option<Box<ListNode>>,
}

/// Document structure
#[derive(Debug)]
pub struct Document {
    pub headline: Option<CatLine>,
    pub body_chunks: Vec<ContChunk>,
    pub footers: Vec<CatLine>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let opts = Options::default();
        assert_eq!(opts.width, 72);
        assert_eq!(opts.headline_width, 50);
        assert_eq!(opts.debug_svg, None);
        assert_eq!(opts.debug_trace, false);
    }

    #[test]
    fn test_options_clone() {
        let opts1 = Options {
            width: 80,
            headline_width: 60,
            debug_svg: Some("test.svg".to_string()),
            debug_trace: true,
        };
        let opts2 = opts1.clone();

        assert_eq!(opts1.width, opts2.width);
        assert_eq!(opts1.headline_width, opts2.headline_width);
        assert_eq!(opts1.debug_svg, opts2.debug_svg);
        assert_eq!(opts1.debug_trace, opts2.debug_trace);
    }

    #[test]
    fn test_category_equality() {
        assert_eq!(Category::ProseGeneral, Category::ProseGeneral);
        assert_ne!(Category::ProseGeneral, Category::List);
        assert_ne!(Category::Code, Category::Comment);
    }

    #[test]
    fn test_category_hash() {
        let mut map = HashMap::new();
        map.insert(Category::ProseGeneral, 1.0);
        map.insert(Category::List, 0.8);
        map.insert(Category::Code, 0.6);

        assert_eq!(map.get(&Category::ProseGeneral), Some(&1.0));
        assert_eq!(map.get(&Category::List), Some(&0.8));
        assert_eq!(map.get(&Category::Code), Some(&0.6));
        assert_eq!(map.get(&Category::Footer), None);
    }

    #[test]
    fn test_catline_creation() {
        let mut probabilities = HashMap::new();
        probabilities.insert(Category::ProseGeneral, 0.8);
        probabilities.insert(Category::List, 0.2);

        let cat_line = CatLine {
            text: "Test line".to_string(),
            line_number: 0,
            indent: 2,
            probabilities,
            final_category: Category::ProseGeneral,
        };

        assert_eq!(cat_line.text, "Test line");
        assert_eq!(cat_line.line_number, 0);
        assert_eq!(cat_line.indent, 2);
        assert_eq!(cat_line.final_category, Category::ProseGeneral);
        assert_eq!(
            cat_line.probabilities.get(&Category::ProseGeneral),
            Some(&0.8)
        );
    }

    #[test]
    fn test_catline_clone() {
        let mut probabilities = HashMap::new();
        probabilities.insert(Category::ProseGeneral, 0.8);

        let cat_line1 = CatLine {
            text: "Test line".to_string(),
            line_number: 0,
            indent: 2,
            probabilities,
            final_category: Category::ProseGeneral,
        };

        let cat_line2 = cat_line1.clone();

        assert_eq!(cat_line1.text, cat_line2.text);
        assert_eq!(cat_line1.line_number, cat_line2.line_number);
        assert_eq!(cat_line1.indent, cat_line2.indent);
        assert_eq!(cat_line1.final_category, cat_line2.final_category);
    }

    #[test]
    fn test_document_creation() {
        let mut probabilities = HashMap::new();
        probabilities.insert(Category::ProseGeneral, 1.0);

        let headline = CatLine {
            text: "Test subject".to_string(),
            line_number: 0,
            indent: 0,
            probabilities: probabilities.clone(),
            final_category: Category::ProseGeneral,
        };

        let body_line = CatLine {
            text: "Body text".to_string(),
            line_number: 1,
            indent: 0,
            probabilities,
            final_category: Category::ProseGeneral,
        };

        let document = Document {
            headline: Some(headline),
            body_chunks: vec![ContChunk::Paragraph(vec![body_line])],
            footers: vec![],
        };

        assert!(document.headline.is_some());
        assert_eq!(document.body_chunks.len(), 1);
        assert_eq!(document.footers.len(), 0);

        if let Some(ref headline) = document.headline {
            assert_eq!(headline.text, "Test subject");
        }
    }

    #[test]
    fn test_list_node_creation() {
        let mut probabilities = HashMap::new();
        probabilities.insert(Category::List, 1.0);

        let bullet_line = CatLine {
            text: "- First item".to_string(),
            line_number: 0,
            indent: 0,
            probabilities,
            final_category: Category::List,
        };

        let list_item = ListItem {
            bullet_line,
            continuation: vec![],
            nested: None,
        };

        let list_node = ListNode {
            introduction: vec![],
            items: vec![list_item],
        };

        assert_eq!(list_node.introduction.len(), 0);
        assert_eq!(list_node.items.len(), 1);
        assert_eq!(list_node.items[0].bullet_line.text, "- First item");
        assert_eq!(list_node.items[0].continuation.len(), 0);
        assert!(list_node.items[0].nested.is_none());
    }

    #[test]
    fn test_nested_list_item() {
        let mut probabilities = HashMap::new();
        probabilities.insert(Category::List, 1.0);

        let bullet_line = CatLine {
            text: "- Parent item".to_string(),
            line_number: 0,
            indent: 0,
            probabilities: probabilities.clone(),
            final_category: Category::List,
        };

        let nested_bullet = CatLine {
            text: "  - Nested item".to_string(),
            line_number: 1,
            indent: 2,
            probabilities,
            final_category: Category::List,
        };

        let nested_item = ListItem {
            bullet_line: nested_bullet,
            continuation: vec![],
            nested: None,
        };

        let nested_node = ListNode {
            introduction: vec![],
            items: vec![nested_item],
        };

        let parent_item = ListItem {
            bullet_line,
            continuation: vec![],
            nested: Some(Box::new(nested_node)),
        };

        assert!(parent_item.nested.is_some());

        if let Some(ref nested) = parent_item.nested {
            assert_eq!(nested.items.len(), 1);
            assert_eq!(nested.items[0].bullet_line.text, "  - Nested item");
        }
    }

    #[test]
    fn test_cont_chunk_variants() {
        let mut probabilities = HashMap::new();
        probabilities.insert(Category::ProseGeneral, 1.0);

        let line = CatLine {
            text: "Test line".to_string(),
            line_number: 0,
            indent: 0,
            probabilities,
            final_category: Category::ProseGeneral,
        };

        // Test different chunk types
        let paragraph = ContChunk::Paragraph(vec![line.clone()]);
        let code = ContChunk::Code(vec![line.clone()]);
        let comment = ContChunk::Comment(vec![line.clone()]);
        let table = ContChunk::Table(vec![line]);

        match paragraph {
            ContChunk::Paragraph(lines) => assert_eq!(lines.len(), 1),
            _ => panic!("Expected Paragraph chunk"),
        }

        match code {
            ContChunk::Code(lines) => assert_eq!(lines.len(), 1),
            _ => panic!("Expected Code chunk"),
        }

        match comment {
            ContChunk::Comment(lines) => assert_eq!(lines.len(), 1),
            _ => panic!("Expected Comment chunk"),
        }

        match table {
            ContChunk::Table(lines) => assert_eq!(lines.len(), 1),
            _ => panic!("Expected Table chunk"),
        }
    }
}
