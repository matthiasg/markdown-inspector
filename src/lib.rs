//! Markdown Inspector - parse and navigate markdown document structure
//!
//! This library provides functions to parse markdown headings and extract
//! sections from documents based on their outline structure.

/// A markdown heading with its location and level
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Heading level (1-6, corresponding to # through ######)
    pub level: u8,
    /// The heading text (without the # prefix)
    pub text: String,
}

/// Parse all headings from markdown content
///
/// Returns a list of headings in document order with their line numbers and levels.
/// Skips headings inside fenced code blocks.
pub fn parse_headings(content: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim_start();

        // Toggle code block state on fence markers
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('#') {
            let mut level = 1_u8;
            let mut remaining = rest;

            while let Some(r) = remaining.strip_prefix('#') {
                level += 1;
                remaining = r;
                if level >= 6 {
                    break;
                }
            }

            // Must have space after #'s
            if let Some(text) = remaining.strip_prefix(' ') {
                headings.push(Heading {
                    line_number,
                    level,
                    text: text.trim().to_string(),
                });
            }
        }
    }

    headings
}

/// Find a section by line number or heading text
///
/// Searches in this order:
/// 1. If `section` parses as a number, find heading at that line
/// 2. Exact text match
/// 3. Case-insensitive substring match
pub fn find_section<'a>(headings: &'a [Heading], section: &str) -> Option<&'a Heading> {
    // Try parsing as line number first
    if let Ok(line_num) = section.parse::<usize>() {
        return headings.iter().find(|h| h.line_number == line_num);
    }

    // Try exact match first
    if let Some(h) = headings.iter().find(|h| h.text == section) {
        return Some(h);
    }

    // Try case-insensitive contains
    let section_lower = section.to_lowercase();
    headings
        .iter()
        .find(|h| h.text.to_lowercase().contains(&section_lower))
}

/// Get the line range for a section (start line, end line)
///
/// The end line is the line before the next heading at the same or higher level,
/// or None if this section extends to the end of the document.
pub fn get_section_range(headings: &[Heading], heading: &Heading) -> (usize, Option<usize>) {
    let start = heading.line_number;

    // Find next heading at same or higher level (lower number)
    let end = headings
        .iter()
        .filter(|h| h.line_number > start && h.level <= heading.level)
        .map(|h| h.line_number)
        .next();

    (start, end)
}

/// Extract a section's content from the document
///
/// Returns the text from `start` line to `end` line (exclusive),
/// or to the end of the document if `end` is None.
pub fn extract_section(content: &str, start: usize, end: Option<usize>) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = start.saturating_sub(1);
    let end_idx = end.map(|e| e.saturating_sub(1)).unwrap_or(lines.len());

    lines[start_idx..end_idx].join("\n")
}

/// Format a heading as an outline entry with line number and indentation
pub fn format_outline_entry(heading: &Heading) -> String {
    let indent = "  ".repeat((heading.level - 1) as usize);
    format!("{:>4}:{}{}", heading.line_number, indent, heading.text)
}

/// Get subsection headings within a section's range
pub fn get_subsections(
    headings: &[Heading],
    start: usize,
    end: Option<usize>,
    max_depth: u8,
) -> Vec<&Heading> {
    headings
        .iter()
        .filter(|h| {
            h.line_number >= start && end.is_none_or(|e| h.line_number < e) && h.level <= max_depth
        })
        .collect()
}

/// Get the first subsection within a section (if any)
pub fn get_first_subsection<'a>(headings: &'a [Heading], heading: &Heading) -> Option<&'a Heading> {
    let start = heading.line_number;
    headings
        .iter()
        .find(|h| h.line_number > start && h.level > heading.level)
}

/// Extract section summary: intro text up to first subsection
///
/// Returns the text from the section start to the first subsection heading,
/// or the full section if there are no subsections.
///
/// `section_end` is the line number where this section ends (next sibling/parent heading).
pub fn extract_section_intro(
    content: &str,
    heading: &Heading,
    first_subsection: Option<&Heading>,
    section_end: Option<usize>,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = heading.line_number.saturating_sub(1);
    let end_idx = first_subsection
        .map(|h| h.line_number.saturating_sub(1))
        .or(section_end.map(|e| e.saturating_sub(1)))
        .unwrap_or(lines.len());

    lines[start_idx..end_idx].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headings() {
        let content = "# Title\n\nSome text\n\n## Section 1\n\nMore text\n\n### Subsection\n";
        let headings = parse_headings(content);

        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "Title");
        assert_eq!(headings[0].line_number, 1);

        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].text, "Section 1");
        assert_eq!(headings[1].line_number, 5);

        assert_eq!(headings[2].level, 3);
        assert_eq!(headings[2].text, "Subsection");
        assert_eq!(headings[2].line_number, 9);
    }

    #[test]
    fn test_find_section_by_line() {
        let headings = vec![
            Heading {
                line_number: 1,
                level: 1,
                text: "Title".into(),
            },
            Heading {
                line_number: 5,
                level: 2,
                text: "Section".into(),
            },
        ];

        let found = find_section(&headings, "5");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "Section");
    }

    #[test]
    fn test_find_section_by_text() {
        let headings = vec![
            Heading {
                line_number: 1,
                level: 1,
                text: "Title".into(),
            },
            Heading {
                line_number: 5,
                level: 2,
                text: "My Section".into(),
            },
        ];

        // Exact match
        let found = find_section(&headings, "My Section");
        assert!(found.is_some());

        // Partial match
        let found = find_section(&headings, "section");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "My Section");
    }

    #[test]
    fn test_section_range() {
        let headings = vec![
            Heading {
                line_number: 1,
                level: 1,
                text: "Title".into(),
            },
            Heading {
                line_number: 5,
                level: 2,
                text: "Section 1".into(),
            },
            Heading {
                line_number: 10,
                level: 2,
                text: "Section 2".into(),
            },
        ];

        let (start, end) = get_section_range(&headings, &headings[1]);
        assert_eq!(start, 5);
        assert_eq!(end, Some(10));

        let (start, end) = get_section_range(&headings, &headings[2]);
        assert_eq!(start, 10);
        assert_eq!(end, None);
    }

    #[test]
    fn test_skip_code_blocks() {
        let content = r#"# Title

```bash
# This is a comment, not a heading
echo "hello"
```

## Real Section
"#;
        let headings = parse_headings(content);

        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].text, "Title");
        assert_eq!(headings[1].text, "Real Section");
    }
}
