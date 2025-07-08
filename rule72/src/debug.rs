use std::fs::File;
use std::io::Write;

use crate::types::{CatLine, Category, ContChunk, Document, ListNode};
use crate::utils::display_width;

/// Generate SVG debug visualization of document structure
pub fn generate_debug_svg(doc: &Document, path: &str) {
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

        // Category color based on final classification - brighter colors for better visibility
        let category_color = match line.final_category {
            Category::ProseIntroduction => "#ff8c00", // bright orange
            Category::ProseGeneral => "#1e1e1e",      // dark gray
            Category::List => "#0080ff",              // bright blue
            Category::Code => "#ff40ff",              // bright magenta
            Category::Table => "#00cccc",             // bright cyan
            Category::URL => "#40a0ff",               // light blue
            Category::Empty => "#e0e0e0",             // light gray
            Category::Comment => "#808080",           // medium gray
            Category::Footer => "#606060",            // dark gray
        };

        // Background rect for category - increased opacity for better visibility
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.15"/>"#,
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
