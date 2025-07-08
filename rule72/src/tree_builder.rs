use crate::types::{CatLine, Category, ContChunk, Document, ListItem, ListNode};

/// Build hierarchical document structure from classified lines
pub fn build_document(lines: Vec<CatLine>) -> Document {
    let mut document = Document {
        headline: None,
        body_chunks: Vec::new(),
        footers: Vec::new(),
    };

    let mut current_chunk: Option<ContChunk> = None;
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];

        match line.final_category {
            Category::Footer => {
                // Finish current chunk and add footers
                if let Some(chunk) = current_chunk.take() {
                    document.body_chunks.push(chunk);
                }
                // Collect all remaining lines as footers
                for footer_line in &lines[i..] {
                    document.footers.push(footer_line.clone());
                }
                break;
            }
            _ => {
                // Handle first line as potential headline
                if i == 0 && line.final_category == Category::ProseGeneral {
                    document.headline = Some(line.clone());
                    i += 1;
                    continue;
                }

                match line.final_category {
                    Category::Empty => {
                        // Finish current chunk
                        if let Some(chunk) = current_chunk.take() {
                            document.body_chunks.push(chunk);
                        }
                        // Add empty line as a paragraph chunk
                        document
                            .body_chunks
                            .push(ContChunk::Paragraph(vec![line.clone()]));
                        i += 1;
                    }
                    Category::List => {
                        // Check if we can merge the last paragraph chunk as introduction to this list
                        let mut list_introduction = Vec::new();

                        // Check if the last chunk is a single-line paragraph ending with ":"
                        if let Some(ContChunk::Paragraph(para_lines)) = document.body_chunks.last()
                        {
                            if para_lines.len() == 1
                                && para_lines[0].text.trim().ends_with(':')
                                && (para_lines[0].final_category == Category::ProseGeneral
                                    || para_lines[0].final_category == Category::ProseIntroduction)
                            {
                                // Remove the last paragraph chunk and use it as introduction
                                if let Some(ContChunk::Paragraph(intro_lines)) =
                                    document.body_chunks.pop()
                                {
                                    list_introduction.extend(intro_lines);
                                }
                            }
                        }

                        // Finish current chunk if any
                        if let Some(chunk) = current_chunk.take() {
                            document.body_chunks.push(chunk);
                        }

                        // Parse list but with our pre-determined introduction
                        let (mut list_node, consumed) = parse_list_simple(&lines, i);
                        list_node.introduction = list_introduction;
                        document.body_chunks.push(ContChunk::List(list_node));
                        i += consumed;
                    }
                    Category::Code => {
                        match &mut current_chunk {
                            Some(ContChunk::Code(ref mut code_lines)) => {
                                code_lines.push(line.clone());
                            }
                            _ => {
                                if let Some(chunk) = current_chunk.take() {
                                    document.body_chunks.push(chunk);
                                }
                                current_chunk = Some(ContChunk::Code(vec![line.clone()]));
                            }
                        }
                        i += 1;
                    }
                    Category::Table => {
                        match &mut current_chunk {
                            Some(ContChunk::Table(ref mut table_lines)) => {
                                table_lines.push(line.clone());
                            }
                            _ => {
                                if let Some(chunk) = current_chunk.take() {
                                    document.body_chunks.push(chunk);
                                }
                                current_chunk = Some(ContChunk::Table(vec![line.clone()]));
                            }
                        }
                        i += 1;
                    }
                    Category::Comment => {
                        match &mut current_chunk {
                            Some(ContChunk::Comment(ref mut comment_lines)) => {
                                comment_lines.push(line.clone());
                            }
                            _ => {
                                if let Some(chunk) = current_chunk.take() {
                                    document.body_chunks.push(chunk);
                                }
                                current_chunk = Some(ContChunk::Comment(vec![line.clone()]));
                            }
                        }
                        i += 1;
                    }
                    _ => {
                        // ProseGeneral, ProseIntroduction, URL -> paragraph
                        match &mut current_chunk {
                            Some(ContChunk::Paragraph(ref mut para_lines)) => {
                                para_lines.push(line.clone());
                            }
                            _ => {
                                if let Some(chunk) = current_chunk.take() {
                                    document.body_chunks.push(chunk);
                                }
                                current_chunk = Some(ContChunk::Paragraph(vec![line.clone()]));
                            }
                        }
                        i += 1;
                    }
                }
            }
        }
    }

    // Finish any remaining chunk
    if let Some(chunk) = current_chunk {
        document.body_chunks.push(chunk);
    }

    document
}

/// Parse a list without looking for introduction lines
fn parse_list_simple(lines: &[CatLine], start: usize) -> (ListNode, usize) {
    let mut items = Vec::new();
    let mut i = start;

    while i < lines.len() && lines[i].final_category == Category::List {
        let bullet_line = lines[i].clone();
        i += 1;

        // Collect continuation lines
        let mut continuation = Vec::new();
        while i < lines.len() {
            match lines[i].final_category {
                Category::ProseGeneral | Category::Code => {
                    // Check if this is a continuation (indented relative to bullet)
                    if lines[i].indent > bullet_line.indent {
                        continuation.push(lines[i].clone());
                        i += 1;
                    } else {
                        break;
                    }
                }
                Category::List => {
                    // Check if this is a nested list
                    if lines[i].indent > bullet_line.indent {
                        let (nested_list, consumed) = parse_list_simple(lines, i);
                        items.push(ListItem {
                            bullet_line: bullet_line.clone(),
                            continuation: continuation.clone(),
                            nested: Some(Box::new(nested_list)),
                        });
                        i += consumed;
                        break;
                    } else {
                        // Same or lesser indentation - end of current item
                        break;
                    }
                }
                _ => break,
            }
        }

        if items.is_empty() || items.last().unwrap().nested.is_none() {
            items.push(ListItem {
                bullet_line,
                continuation,
                nested: None,
            });
        }
    }

    let consumed = i - start;
    (
        ListNode {
            introduction: Vec::new(),
            items,
        },
        consumed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::classify_with_context;
    use crate::lexer::lex_lines;

    #[test]
    fn test_document_building() {
        let lines = vec![
            "Subject line",
            "",
            "Introduction:",
            "- First item",
            "- Second item",
            "",
            "Final paragraph",
        ];

        let lexed = lex_lines(&lines);
        let classified = classify_with_context(lexed);
        let document = build_document(classified);

        assert!(document.headline.is_some());
        assert!(document.body_chunks.len() >= 3); // At least empty, list, paragraph

        // Find the list chunk (it may not be at index 1 due to different parsing)
        let has_list = document
            .body_chunks
            .iter()
            .any(|chunk| matches!(chunk, ContChunk::List(_)));
        assert!(has_list, "Document should contain a list chunk");
    }
}
