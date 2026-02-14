//! Markdown parsing utilities using pulldown-cmark
//!
//! Provides unified markdown parsing functions for the codebase.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Represents a markdown heading
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownHeading {
    /// Heading level (1-6)
    pub level: u8,
    /// Heading text content
    pub text: String,
}

/// Represents a markdown link
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownLink {
    /// Link text (visible text)
    pub text: String,
    /// Link destination URL
    pub dest: String,
    /// Whether this is a reference-style link (vs inline)
    pub is_reference: bool,
}

/// Extract link reference definitions from markdown content
///
/// Reference definitions are in the format `[id]: dest "optional title"`
/// Returns a list of (id, dest) pairs.
pub fn extract_references(content: &str) -> Vec<(String, String)> {
    let mut references = Vec::new();
    let mut in_ref_def = false;
    let mut current_id = String::new();
    let mut current_dest = String::new();

    for event in Parser::new(content) {
        match event {
            Event::Start(Tag::Link { link_type, dest_url, .. }) => {
                // Check if this is a reference definition
                if matches!(link_type, pulldown_cmark::LinkType::Reference | pulldown_cmark::LinkType::ReferenceUnknown) {
                    in_ref_def = true;
                    current_dest = dest_url.to_string();
                }
            }
            Event::End(TagEnd::Link) => {
                if in_ref_def && !current_id.is_empty() {
                    references.push((current_id.clone(), current_dest.clone()));
                }
                in_ref_def = false;
                current_id.clear();
                current_dest.clear();
            }
            _ => {}
        }
    }

    // Also parse raw reference definitions manually since pulldown-cmark
    // might not expose them as link events in all cases
    for line in content.lines() {
        let trimmed = line.trim();
        // Match: [id]: dest or [id]:dest
        if trimmed.starts_with('[') && trimmed.contains("]:") {
            if let Some(end_bracket) = trimmed.find(']') {
                let id = trimmed[1..end_bracket].to_string();
                let after_bracket = &trimmed[end_bracket + 1..];
                if let Some(rest) = after_bracket.strip_prefix(':') {
                    let dest = rest.trim().split_whitespace().next().unwrap_or("");
                    if !dest.is_empty() && !references.iter().any(|(ref_id, _)| ref_id == &id) {
                        references.push((id, dest.to_string()));
                    }
                }
            }
        }
    }

    references
}

/// Extract all headings from markdown content
pub fn extract_headings(content: &str) -> Vec<MarkdownHeading> {
    let mut headings = Vec::new();
    let mut in_heading = false;
    let mut current_level = 1;
    let mut current_text = String::new();

    for event in Parser::new(content) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                current_text.clear();
            }
            Event::Text(text) => {
                if in_heading {
                    current_text.push_str(&text);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    headings.push(MarkdownHeading {
                        level: current_level,
                        text: current_text.trim().to_string(),
                    });
                }
                in_heading = false;
            }
            _ => {}
        }
    }

    headings
}

/// Extract all links from markdown content (inline links only, not reference definitions)
pub fn extract_links(content: &str) -> Vec<MarkdownLink> {
    let mut links = Vec::new();
    let mut in_link = false;
    let mut current_text = String::new();
    let mut current_dest = String::new();
    let mut current_is_ref = false;

    for event in Parser::new(content) {
        match event {
            Event::Start(Tag::Link { link_type, dest_url, .. }) => {
                in_link = true;
                current_dest = dest_url.to_string();
                current_is_ref = matches!(
                    link_type,
                    pulldown_cmark::LinkType::Reference
                        | pulldown_cmark::LinkType::ReferenceUnknown
                        | pulldown_cmark::LinkType::Collapsed
                        | pulldown_cmark::LinkType::CollapsedUnknown
                        | pulldown_cmark::LinkType::Shortcut
                        | pulldown_cmark::LinkType::ShortcutUnknown
                );
                current_text.clear();
            }
            Event::Text(text) => {
                if in_link {
                    current_text.push_str(&text);
                }
            }
            Event::End(TagEnd::Link) => {
                if in_link && !current_dest.is_empty() {
                    links.push(MarkdownLink {
                        text: current_text.clone(),
                        dest: current_dest.clone(),
                        is_reference: current_is_ref,
                    });
                }
                in_link = false;
            }
            _ => {}
        }
    }

    links
}

/// Check if a reference definition exists
pub fn has_reference(content: &str, ref_id: &str) -> bool {
    get_reference_dest(content, ref_id).is_some()
}

/// Get the destination of a reference definition
pub fn get_reference_dest(content: &str, ref_id: &str) -> Option<String> {
    // Parse manually to get reference definitions
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.contains("]:") {
            if let Some(end_bracket) = trimmed.find(']') {
                let id = &trimmed[1..end_bracket];
                if id.eq_ignore_ascii_case(ref_id) {
                    let after_bracket = &trimmed[end_bracket + 1..];
                    if let Some(rest) = after_bracket.strip_prefix(':') {
                        let dest = rest.trim().split_whitespace().next()?;
                        return Some(dest.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Find the line number of a specific heading
pub fn find_heading_line(content: &str, heading_text: &str, level: Option<u8>) -> Option<usize> {
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Check if this is a heading
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count() as u8;

            // Check level match
            if let Some(l) = level {
                if hash_count != l {
                    continue;
                }
            }

            // Extract heading text
            let heading = trimmed[hash_count as usize..].trim();
            if heading == heading_text {
                return Some(line_num);
            }
        }
    }
    None
}

/// Extract the prefix from a frontmatter PREFIX field
pub fn extract_frontmatter_prefix(content: &str) -> Option<String> {
    let mut found_prefix = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    // Look for opening ---
    while i < lines.len() {
        if lines[i].trim() == "---" {
            i += 1;
            // Read frontmatter content
            while i < lines.len() && lines[i].trim() != "---" {
                let line = lines[i].trim();
                if let Some(value) = line.strip_prefix("PREFIX:") {
                    found_prefix = Some(value.trim().to_string());
                }
                i += 1;
            }
            break;
        }
        i += 1;
    }

    found_prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_references() {
        let content = r#"
# Task File

---
[task-01]: notes/note1.md
[task-02]: notes/note2.md
"#;
        let refs = extract_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], ("task-01".to_string(), "notes/note1.md".to_string()));
        assert_eq!(refs[1], ("task-02".to_string(), "notes/note2.md".to_string()));
    }

    #[test]
    fn test_extract_headings() {
        let content = r#"
# Main Title
## Section 1
### Subsection
"#;
        let headings = extract_headings(content);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "Main Title");
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[2].level, 3);
    }

    #[test]
    fn test_extract_links() {
        let content = "See [my note](notes/my-note.md) for details.";
        let links = extract_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "my note");
        assert_eq!(links[0].dest, "notes/my-note.md");
    }

    #[test]
    fn test_has_reference() {
        let content = "[task-01]: notes/note1.md\n";
        assert!(has_reference(content, "task-01"));
        assert!(!has_reference(content, "task-02"));
    }

    #[test]
    fn test_get_reference_dest() {
        let content = "[task-01]: notes/note1.md\n";
        assert_eq!(get_reference_dest(content, "task-01"), Some("notes/note1.md".to_string()));
        assert_eq!(get_reference_dest(content, "task-02"), None);
    }

    #[test]
    fn test_find_heading_line() {
        let content = "# Title\n\n## Section\n\nContent";
        assert_eq!(find_heading_line(content, "Title", Some(1)), Some(0));
        assert_eq!(find_heading_line(content, "Section", Some(2)), Some(2));
        assert_eq!(find_heading_line(content, "Section", Some(1)), None);
    }

    #[test]
    fn test_extract_frontmatter_prefix() {
        let content = "---\nPREFIX: task-\n---\n\nContent";
        assert_eq!(extract_frontmatter_prefix(content), Some("task-".to_string()));
    }

    #[test]
    fn test_extract_frontmatter_prefix_none() {
        let content = "# No frontmatter\n\nContent";
        assert_eq!(extract_frontmatter_prefix(content), None);
    }
}
