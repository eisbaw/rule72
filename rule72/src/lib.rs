use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Formatting options
#[derive(Debug, Clone)]
pub struct Options {
    pub width: usize,
    pub headline_width: usize,
    pub strip_ansi: bool,
    pub debug_svg: Option<String>,
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

/// Public API: reflow an entire commit message
pub fn reflow(input: &str, opts: &Options) -> String {
    let lines: Vec<&str> = input.lines().map(|l| l.trim_end_matches('\r')).collect();

    // Lex lines into CatLines
    let cat_lines = lex_lines(&lines);

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

/// Lexer: convert raw lines to CatLines with initial probabilities
fn lex_lines(lines: &[&str]) -> Vec<CatLine> {
    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let mut probabilities = HashMap::new();
            let indent = count_indent(line);
            let trimmed = line.trim();

            // Initial probability assignment
            if trimmed.is_empty() {
                probabilities.insert(Category::Empty, 1.0);
            } else if line.starts_with('#') {
                probabilities.insert(Category::Comment, 1.0);
            } else if idx > 0 && is_footer_line(line) {
                // Only check for footers after first line
                probabilities.insert(Category::Footer, 0.9);
                probabilities.insert(Category::ProseGeneral, 0.1);
            } else {
                // Analyze line characteristics
                let special_char_ratio =
                    count_special_chars(trimmed) as f32 / trimmed.len().max(1) as f32;
                let has_url = trimmed.starts_with("http://") || trimmed.starts_with("https://");
                let has_table_markers =
                    trimmed.contains('|') && trimmed.chars().filter(|&c| c == '|').count() > 1;
                let has_list_marker = is_list_item(line);
                let ends_with_colon = trimmed.ends_with(':') && !trimmed.contains("://");

                if has_url {
                    probabilities.insert(Category::URL, 0.9);
                    probabilities.insert(Category::ProseGeneral, 0.1);
                } else if has_table_markers {
                    probabilities.insert(Category::Table, 0.8);
                    probabilities.insert(Category::Code, 0.2);
                } else if has_list_marker {
                    probabilities.insert(Category::List, 0.9);
                    probabilities.insert(Category::ProseGeneral, 0.1);
                } else if special_char_ratio > 0.3 || indent >= 4 {
                    probabilities.insert(Category::Code, 0.7);
                    probabilities.insert(Category::ProseGeneral, 0.3);
                } else if ends_with_colon && idx > 0 {
                    // Don't treat headline as introduction
                    probabilities.insert(Category::ProseIntroduction, 0.7);
                    probabilities.insert(Category::ProseGeneral, 0.3);
                } else {
                    probabilities.insert(Category::ProseGeneral, 0.8);
                    probabilities.insert(Category::Code, special_char_ratio);
                }
            }

            // Determine initial category (highest probability)
            let final_category = probabilities
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(cat, _)| *cat)
                .unwrap_or(Category::ProseGeneral);

            CatLine {
                text: (*line).to_string(),
                line_number: idx,
                indent,
                probabilities,
                final_category,
            }
        })
        .collect()
}

/// Apply context-aware classification using neighboring lines
fn classify_with_context(mut lines: Vec<CatLine>) -> Vec<CatLine> {
    let context_window = 2; // ±2 lines

    for i in 0..lines.len() {
        let mut context_probs = HashMap::new();

        // Gather context from neighboring lines
        let start = i.saturating_sub(context_window);
        let end = (i + context_window).min(lines.len() - 1);
        for (j, line) in lines.iter().enumerate().take(end + 1).skip(start) {
            if i == j {
                continue;
            }

            let weight = 1.0 / (1.0 + (i as f32 - j as f32).abs());
            for (cat, prob) in &line.probabilities {
                *context_probs.entry(*cat).or_insert(0.0) += prob * weight;
            }
        }

        // Adjust probabilities based on context
        let mut adjusted_probs = lines[i].probabilities.clone();

        // Special rules for context-dependent classification
        if context_probs.get(&Category::Code).unwrap_or(&0.0) > &0.5 {
            // Boost code probability if surrounded by code
            *adjusted_probs.entry(Category::Code).or_insert(0.0) += 0.3;
        }

        if context_probs.get(&Category::List).unwrap_or(&0.0) > &0.5 {
            // Boost list probability if surrounded by lists
            *adjusted_probs.entry(Category::List).or_insert(0.0) += 0.2;
        }

        // Normalize probabilities
        let total: f32 = adjusted_probs.values().sum();
        if total > 0.0 {
            for prob in adjusted_probs.values_mut() {
                *prob /= total;
            }
        }

        // Update final category
        lines[i].final_category = adjusted_probs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(cat, _)| *cat)
            .unwrap_or(Category::ProseGeneral);

        lines[i].probabilities = adjusted_probs;
    }

    lines
}

/// Build document tree structure from classified lines
fn build_document(lines: Vec<CatLine>) -> Document {
    let mut headline = None;
    let mut body_chunks = Vec::new();
    let mut footers = Vec::new();

    let mut i = 0;

    // Skip initial comments and empty lines
    while i < lines.len()
        && (lines[i].final_category == Category::Comment
            || lines[i].final_category == Category::Empty)
    {
        if lines[i].final_category == Category::Comment {
            body_chunks.push(ContChunk::Comment(vec![lines[i].clone()]));
        }
        i += 1;
    }

    // Extract headline - but check if it's actually an introduction to a list
    if i < lines.len() && lines[i].final_category != Category::Footer {
        // Check if the next line (after any empty lines) is a list
        let mut next_idx = i + 1;
        while next_idx < lines.len() && lines[next_idx].final_category == Category::Empty {
            next_idx += 1;
        }

        let next_is_list =
            next_idx < lines.len() && lines[next_idx].final_category == Category::List;

        // If current line ends with colon and next line is a list, treat as introduction + list
        if lines[i].text.trim().ends_with(':') && next_is_list {
            // This is an introduction line, not a headline
            // Include any empty lines between introduction and list
            let mut intro_chunk = vec![lines[i].clone()];
            i += 1;
            while i < lines.len() && lines[i].final_category == Category::Empty {
                intro_chunk.push(lines[i].clone());
                i += 1;
            }

            // Now parse the list
            if i < lines.len() && lines[i].final_category == Category::List {
                let list_node = parse_list_with_intro(&lines, &mut i, intro_chunk);
                body_chunks.push(ContChunk::List(list_node));
            }
        } else {
            // Regular headline
            headline = Some(lines[i].clone());
            i += 1;

            // Preserve empty line after headline as a body chunk
            if i < lines.len() && lines[i].final_category == Category::Empty {
                body_chunks.push(ContChunk::Paragraph(vec![lines[i].clone()]));
                i += 1;
            }
        }
    }

    // Process remaining body chunks
    while i < lines.len() && lines[i].final_category != Category::Footer {
        match lines[i].final_category {
            Category::Empty => {
                // Preserve empty lines between chunks
                body_chunks.push(ContChunk::Paragraph(vec![lines[i].clone()]));
                i += 1;
            }
            Category::Comment => {
                let mut comment_lines = vec![lines[i].clone()];
                i += 1;
                while i < lines.len() && lines[i].final_category == Category::Comment {
                    comment_lines.push(lines[i].clone());
                    i += 1;
                }
                body_chunks.push(ContChunk::Comment(comment_lines));
            }
            Category::Table => {
                let mut table_lines = vec![lines[i].clone()];
                i += 1;
                while i < lines.len() && lines[i].final_category == Category::Table {
                    table_lines.push(lines[i].clone());
                    i += 1;
                }
                body_chunks.push(ContChunk::Table(table_lines));
            }
            Category::Code => {
                // Check if this is an introduction line (ends with colon) followed by a list
                let is_intro = lines[i].text.trim().ends_with(':');
                let next_is_list =
                    i + 1 < lines.len() && lines[i + 1].final_category == Category::List;

                if is_intro && next_is_list {
                    // This is an introduction line for the following list
                    let intro_lines = vec![lines[i].clone()];
                    i += 1;

                    // Parse the list with the introduction
                    if i < lines.len() && lines[i].final_category == Category::List {
                        let list_node = parse_list_with_intro(&lines, &mut i, intro_lines);
                        body_chunks.push(ContChunk::List(list_node));
                    }
                } else {
                    // Regular code block
                    let mut code_lines = vec![lines[i].clone()];
                    i += 1;
                    while i < lines.len() && lines[i].final_category == Category::Code {
                        code_lines.push(lines[i].clone());
                        i += 1;
                    }
                    body_chunks.push(ContChunk::Code(code_lines));
                }
            }
            Category::List => {
                let list_node = parse_list(&lines, &mut i);
                body_chunks.push(ContChunk::List(list_node));
            }
            Category::ProseIntroduction => {
                // Check if this is followed by code block
                let next_is_code =
                    i + 1 < lines.len() && lines[i + 1].final_category == Category::Code;
                if next_is_code {
                    // Include the introduction line with the code block
                    let mut code_lines = vec![lines[i].clone()];
                    i += 1;
                    while i < lines.len() && lines[i].final_category == Category::Code {
                        code_lines.push(lines[i].clone());
                        i += 1;
                    }
                    body_chunks.push(ContChunk::Code(code_lines));
                } else {
                    // Treat as regular paragraph
                    let mut para_lines = vec![lines[i].clone()];
                    i += 1;
                    while i < lines.len()
                        && (lines[i].final_category == Category::ProseGeneral
                            || lines[i].final_category == Category::ProseIntroduction
                            || lines[i].final_category == Category::URL)
                        && lines[i].final_category != Category::Empty
                    {
                        para_lines.push(lines[i].clone());
                        i += 1;
                    }
                    body_chunks.push(ContChunk::Paragraph(para_lines));
                }
            }
            Category::ProseGeneral | Category::URL => {
                let mut para_lines = vec![lines[i].clone()];
                i += 1;
                while i < lines.len()
                    && (lines[i].final_category == Category::ProseGeneral
                        || lines[i].final_category == Category::ProseIntroduction
                        || lines[i].final_category == Category::URL)
                    && lines[i].final_category != Category::Empty
                {
                    para_lines.push(lines[i].clone());
                    i += 1;
                }
                body_chunks.push(ContChunk::Paragraph(para_lines));
            }
            Category::Footer => break,
        }
    }

    // Collect footers
    while i < lines.len() {
        if lines[i].final_category == Category::Footer {
            footers.push(lines[i].clone());
        }
        i += 1;
    }

    Document {
        headline,
        body_chunks,
        footers,
    }
}

/// Parse a list structure (handles nesting)
fn parse_list(lines: &[CatLine], i: &mut usize) -> ListNode {
    parse_list_with_intro(lines, i, Vec::new())
}

fn parse_list_with_intro(lines: &[CatLine], i: &mut usize, introduction: Vec<CatLine>) -> ListNode {
    let mut items: Vec<ListItem> = Vec::new();
    let base_indent = lines[*i].indent;

    while *i < lines.len()
        && lines[*i].final_category == Category::List
        && lines[*i].indent >= base_indent
    {
        if lines[*i].indent > base_indent {
            // This is a nested list item
            if let Some(last_item) = items.last_mut() {
                let nested = parse_list_with_intro(lines, i, Vec::new());
                last_item.nested = Some(Box::new(nested));
            }
        } else {
            // Regular list item at current level
            let bullet_line = lines[*i].clone();
            *i += 1;

            let mut continuation = Vec::new();
            while *i < lines.len()
                && lines[*i].final_category != Category::List
                && lines[*i].final_category != Category::Empty
                && lines[*i].indent > base_indent
            {
                continuation.push(lines[*i].clone());
                *i += 1;
            }

            items.push(ListItem {
                bullet_line,
                continuation,
                nested: None,
            });
        }
    }

    ListNode {
        introduction,
        items,
    }
}

/// Pretty print the document structure
fn pretty_print(doc: &Document, opts: &Options) -> String {
    let mut output = Vec::new();

    // Print headline
    if let Some(headline) = &doc.headline {
        output.push(headline.text.clone());
        // Don't automatically add blank line - let the body chunks handle spacing
    }

    // Print body chunks
    for (idx, chunk) in doc.body_chunks.iter().enumerate() {
        // Add blank line between headline and first body chunk if needed
        if idx == 0 && doc.headline.is_some() {
            let curr_is_empty = matches!(chunk, ContChunk::Paragraph(lines) if lines.len() == 1 && lines[0].final_category == Category::Empty);
            if !curr_is_empty {
                output.push(String::new()); // Add blank line between headline and body
            }
        }

        // Add blank line between non-empty chunks (but not before the first chunk)
        if idx > 0 {
            let prev_chunk = &doc.body_chunks[idx - 1];
            let curr_chunk = chunk;

            let prev_is_empty = matches!(prev_chunk, ContChunk::Paragraph(lines) if lines.len() == 1 && lines[0].final_category == Category::Empty);
            let curr_is_empty = matches!(chunk, ContChunk::Paragraph(lines) if lines.len() == 1 && lines[0].final_category == Category::Empty);

            // Check if previous chunk is a ProseIntroduction that should connect to current list
            let prev_is_intro = matches!(prev_chunk, ContChunk::Paragraph(lines) if lines.len() == 1 && lines[0].final_category == Category::ProseIntroduction);
            let curr_is_list = matches!(curr_chunk, ContChunk::List(_));

            // Check if previous chunk is a single-line Code chunk ending with colon (introduction) and current is a list
            let prev_is_code_intro = matches!(prev_chunk, ContChunk::Code(lines) if lines.len() == 1 && lines[0].text.trim().ends_with(':'));

            // Don't add extra blank line if:
            // - Either chunk is an empty line
            // - Previous chunk is a comment
            // - Previous chunk is a ProseIntroduction and current is a List (they should be connected)
            // - Previous chunk is a single-line Code ending with colon and current is a List
            if !(prev_is_empty
                || curr_is_empty
                || matches!(prev_chunk, ContChunk::Comment(_))
                || (prev_is_intro && curr_is_list)
                || (prev_is_code_intro && curr_is_list))
            {
                output.push(String::new());
            }
        }

        match chunk {
            ContChunk::Comment(lines) => {
                for line in lines {
                    output.push(line.text.clone());
                }
            }
            ContChunk::Table(lines) | ContChunk::Code(lines) => {
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

/// Pretty print a list node
fn pretty_print_list(list: &ListNode, opts: &Options, _depth: usize) -> Vec<String> {
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

// Helper functions

fn count_indent(line: &str) -> usize {
    line.chars()
        .take_while(|&c| c == ' ' || c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

fn count_special_chars(s: &str) -> usize {
    s.chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count()
}

fn is_footer_line(line: &str) -> bool {
    // Common footer tags - be very specific about what we consider footers
    let footer_tags = [
        "Signed-off-by:",
        "Co-authored-by:",
        "Reviewed-by:",
        "Acked-by:",
        "Tested-by:",
        "Reported-by:",
        "Suggested-by:",
        "Fixes:",
        "Closes:",
        "Resolves:",
        "See-also:",
        "Ref:",
        "References:",
    ];

    // Check if line starts with a known footer tag
    for tag in &footer_tags {
        if line.starts_with(tag) {
            return true;
        }
    }

    // Don't use generic pattern matching - it's too broad and catches regular content
    // like "EN: something broke" which are clearly not footers
    false
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

/// Generate debug SVG visualization
fn generate_debug_svg(doc: &Document, path: &str) {
    use std::fs::File;
    use std::io::Write;

    let font_size = 14;
    let line_height = 20;
    let char_width = 8;
    let margin = 20;

    // First, collect all the actual lines from the document
    let mut doc_lines: Vec<CatLine> = Vec::new();

    // Add headline
    if let Some(headline) = &doc.headline {
        doc_lines.push(headline.clone());
    }

    // Add body chunks
    for chunk in &doc.body_chunks {
        match chunk {
            ContChunk::Comment(lines)
            | ContChunk::Table(lines)
            | ContChunk::Code(lines)
            | ContChunk::Paragraph(lines) => {
                doc_lines.extend(lines.iter().cloned());
            }
            ContChunk::List(list_node) => {
                collect_list_lines_for_svg(&mut doc_lines, list_node);
            }
        }
    }

    // Add footers
    doc_lines.extend(doc.footers.iter().cloned());

    // Now create the visualization data
    let mut all_lines = Vec::new();

    if let Some(headline) = &doc.headline {
        all_lines.push((headline.clone(), 0, "headline"));
    }

    for chunk in &doc.body_chunks {
        match chunk {
            ContChunk::Comment(lines) => {
                for line in lines {
                    all_lines.push((line.clone(), 1, "comment"));
                }
            }
            ContChunk::Table(lines) => {
                for line in lines {
                    all_lines.push((line.clone(), 1, "table"));
                }
            }
            ContChunk::Code(lines) => {
                for line in lines {
                    all_lines.push((line.clone(), 1, "code"));
                }
            }
            ContChunk::Paragraph(lines) => {
                for line in lines {
                    if line.final_category == Category::Empty {
                        all_lines.push((line.clone(), 1, "empty"));
                    } else {
                        all_lines.push((line.clone(), 1, "paragraph"));
                    }
                }
            }
            ContChunk::List(list_node) => {
                collect_list_lines_owned(&mut all_lines, list_node, 1);
            }
        }
    }

    for footer in &doc.footers {
        all_lines.push((footer.clone(), 0, "footer"));
    }

    let max_width = all_lines
        .iter()
        .map(|(line, _, _)| display_width(&line.text))
        .max()
        .unwrap_or(0);

    let svg_width = margin * 2 + max_width * char_width;
    let svg_height = margin * 2 + all_lines.len() * line_height;

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        svg_width, svg_height, svg_width, svg_height
    ));

    svg.push_str("\n<style>\n");
    svg.push_str(&format!(
        "    text {{ font-family: monospace; font-size: {}px; }}\n",
        font_size
    ));
    svg.push_str("    .headline { fill: #2e3440; }\n");
    svg.push_str("    .comment { fill: #616e88; }\n");
    svg.push_str("    .table { fill: #5e81ac; }\n");
    svg.push_str("    .code { fill: #b48ead; }\n");
    svg.push_str("    .paragraph { fill: #2e3440; }\n");
    svg.push_str("    .list { fill: #2e3440; }\n");
    svg.push_str("    .footer { fill: #4c566a; }\n");
    svg.push_str("    .empty { fill: #d8dee9; }\n");
    svg.push_str("    .chunk-rect { fill: none; stroke-width: 2; opacity: 0.5; }\n");
    svg.push_str("    .chunk-label { font-size: 10px; fill: #4c566a; }\n");
    svg.push_str("    .prob-tooltip { font-size: 10px; fill: #2e3440; }\n");
    svg.push_str("    .ruler-dots { fill: #c3e88d; opacity: 1.0; font-family: monospace; font-size: 14px; }\n");
    svg.push_str("</style>\n");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#eceff4\"/>");
    svg.push('\n');

    // Draw ruler dots for each line (at bottom z-order)
    let mut ruler_y = margin;
    let mut prev_chunk_type = "";
    for (line, _depth, chunk_type) in &all_lines {
        // Skip dots for empty lines that come directly after headline
        let is_empty_after_headline =
            line.final_category == Category::Empty && prev_chunk_type == "headline";

        if !is_empty_after_headline {
            let dots_count = if chunk_type == &"headline" { 50 } else { 72 };
            let dots = "·".repeat(dots_count);
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" class="ruler-dots">{}</text>"#,
                margin, ruler_y, dots
            ));
        }
        prev_chunk_type = chunk_type;
        ruler_y += line_height;
    }

    // Draw lines and collect chunk boundaries
    let mut y = margin;
    let mut chunk_boundaries = Vec::new();
    let mut current_chunk_start = 0;
    let mut current_chunk_type = "";

    for (idx, (line, _depth, chunk_type)) in all_lines.iter().enumerate() {
        if idx == 0 || chunk_type != &current_chunk_type {
            if idx > 0 {
                chunk_boundaries.push((current_chunk_start, idx - 1, current_chunk_type));
            }
            current_chunk_start = idx;
            current_chunk_type = chunk_type;
        }

        // Category color based on final classification
        let category_color = match line.final_category {
            Category::ProseIntroduction => "#d08770",
            Category::ProseGeneral => "#2e3440",
            Category::List => "#5e81ac",
            Category::Code => "#b48ead",
            Category::Table => "#88c0d0",
            Category::URL => "#81a1c1",
            Category::Empty => "#d8dee9",
            Category::Comment => "#616e88",
            Category::Footer => "#4c566a",
        };

        // Background rect for category - transparent so ruler dots show through
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.05"/>"#,
            margin,
            y - font_size,
            max_width * char_width,
            line_height,
            category_color
        ));

        // Text line
        let escaped_text = line
            .text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        let prob_text = line
            .probabilities
            .iter()
            .map(|(cat, prob)| format!("  {:?}: {:.2}", cat, prob))
            .collect::<Vec<_>>()
            .join("\n");

        svg.push_str(&format!(
            r#"<text x="{}" y="{}" class="{}">"#,
            margin + line.indent * char_width,
            y,
            chunk_type
        ));

        svg.push_str(&format!(
            r#"<title>Line {}: {:?}
Probabilities:
{}</title>"#,
            line.line_number + 1,
            line.final_category,
            prob_text
        ));

        // For empty lines, show a placeholder
        if line.final_category == Category::Empty {
            svg.push_str("[empty line]");
        } else {
            svg.push_str(&escaped_text);
        }
        svg.push_str("</text>");

        y += line_height;
    }

    // Add last chunk
    if !all_lines.is_empty() {
        chunk_boundaries.push((current_chunk_start, all_lines.len() - 1, current_chunk_type));
    }

    // Draw chunk boundaries
    for (start_idx, end_idx, chunk_type) in chunk_boundaries {
        let chunk_y = margin + start_idx * line_height - font_size;
        let chunk_height = (end_idx - start_idx + 1) * line_height;

        let chunk_color = match chunk_type {
            "headline" => "#5e81ac",
            "comment" => "#616e88",
            "table" => "#88c0d0",
            "code" => "#b48ead",
            "paragraph" => "#a3be8c",
            "list" => "#81a1c1",
            "footer" => "#bf616a",
            "empty" => "#d8dee9",
            _ => "#4c566a",
        };

        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" class="chunk-rect" stroke="{}"/>"#,
            margin - 5,
            chunk_y,
            max_width * char_width + 10,
            chunk_height,
            chunk_color
        ));

        // Chunk label positioned in bottom right corner
        let label_x = margin + max_width * char_width - 5; // Near right edge
        let label_y = chunk_y + chunk_height - 3; // Near bottom edge
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" class="chunk-label" text-anchor="end">{}</text>"#,
            label_x, label_y, chunk_type
        ));
    }

    svg.push_str("</svg>");

    // Write to file
    if let Ok(mut file) = File::create(path) {
        let _ = file.write_all(svg.as_bytes());
        eprintln!("Debug SVG written to: {}", path);
    } else {
        eprintln!("Failed to create SVG file: {}", path);
    }
}

fn collect_list_lines_owned(
    all_lines: &mut Vec<(CatLine, usize, &'static str)>,
    list: &ListNode,
    depth: usize,
) {
    // Add introduction lines
    for intro in &list.introduction {
        if intro.final_category == Category::Empty {
            all_lines.push((intro.clone(), depth, "empty"));
        } else {
            all_lines.push((intro.clone(), depth, "list"));
        }
    }

    for item in &list.items {
        all_lines.push((item.bullet_line.clone(), depth, "list"));
        for cont in &item.continuation {
            all_lines.push((cont.clone(), depth + 1, "list"));
        }
        if let Some(nested) = &item.nested {
            collect_list_lines_owned(all_lines, nested, depth + 1);
        }
    }
}

fn collect_list_lines_for_svg(doc_lines: &mut Vec<CatLine>, list: &ListNode) {
    // Add introduction lines
    doc_lines.extend(list.introduction.iter().cloned());

    for item in &list.items {
        doc_lines.push(item.bullet_line.clone());
        doc_lines.extend(item.continuation.iter().cloned());
        if let Some(nested) = &item.nested {
            collect_list_lines_for_svg(doc_lines, nested);
        }
    }
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
            "This is prose.",
            "- List item",
        ];
        let cat_lines = lex_lines(&lines);

        assert_eq!(cat_lines[0].final_category, Category::Comment);
        assert_eq!(cat_lines[1].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[2].final_category, Category::Empty);
        assert_eq!(cat_lines[3].final_category, Category::ProseGeneral);
        assert_eq!(cat_lines[4].final_category, Category::List);
    }

    #[test]
    fn test_wrap_simple() {
        let options = Options {
            width: 20,
            headline_width: 50,
            strip_ansi: false,
            debug_svg: None,
        };
        let input =
            "Subject line\n\nThis is a very long paragraph that should wrap nicely at twenty cols.";
        let expected = "Subject line\n\nThis is a very long\nparagraph that\nshould wrap nicely\nat twenty cols.\n";
        let out = reflow(input, &options);
        assert_eq!(out, expected);
    }
}
