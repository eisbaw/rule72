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
    pub strip_ansi: bool,
    pub debug_svg: Option<String>,
    pub debug_trace: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 72,
            headline_width: 50,
            strip_ansi: false,
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
