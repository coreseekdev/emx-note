//! List notes command module

use std::fs;
use std::io;
use std::collections::HashMap;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde_json::json;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, filter: Option<String>) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    if let Some(f) = filter {
        // Handle filter
        if f.starts_with('#') {
            // Tag filter: list tag file contents
            list_tag(&capsa_ref, &f, ctx.json)?;
        } else {
            // Try as source hash in note/ (12 chars) or date in #daily/ (8 chars YYYYMMDD)
            // First, hash the input to get the directory name
            use emx_note::{hash_source, abbreviate_hash};
            let hash = abbreviate_hash(&hash_source(&f));
            let note_hash_dir = capsa_ref.path.join("note").join(&hash);
            if note_hash_dir.is_dir() {
                list_directory(&note_hash_dir)?;
            } else {
                // Try as date in #daily/
                let daily_dir = capsa_ref.path.join("#daily").join(&f);
                if daily_dir.is_dir() {
                    list_directory(&daily_dir)?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Not found: note/{}/ or #daily/{}", hash, f),
                    ));
                }
            }
        }
    } else {
        // No filter: list top-level notes in note/
        let note_dir = capsa_ref.path.join("note");
        if !note_dir.exists() {
            return Ok(()); // Empty, no error
        }

        // List only top-level .md files (not in hash subdirectories)
        let mut entries: Vec<_> = fs::read_dir(&note_dir)?
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by(|a, b| {
            a.file_name().cmp(&b.file_name())
        });

        for entry in entries {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip directories (hash subdirectories)
            if path.is_dir() {
                continue;
            }

            // Only show .md files
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                println!("{}", name_str);
            }
        }
    }

    Ok(())
}

/// List tag file contents
fn list_tag(capsa_ref: &emx_note::CapsaRef, tag: &str, json: bool) -> io::Result<()> {
    let tag_name = tag.trim_start_matches('#');

    // Special case: #daily lists only the subdirectory names (dates) in #daily/
    if tag_name == "daily" {
        let daily_dir = capsa_ref.path.join("#daily");
        if !daily_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Daily directory not found. Create a daily note first.",
            ));
        }
        return list_daily_subdirs(&daily_dir, json);
    }

    let tag_file = capsa_ref.path.join(format!("#{}.md", tag_name));

    if !tag_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Tag '{}' not found", tag),
        ));
    }

    let content = fs::read_to_string(&tag_file)?;

    // Strip YAML frontmatter if present
    let content = strip_yaml_frontmatter(&content);

    // Parse markdown with pulldown-cmark to extract links grouped by date
    let mut grouped_links: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_date: Option<String> = None;
    let mut in_h2 = false;
    let mut heading_text = String::new();
    let mut link_dest: String = String::new();

    for event in Parser::new(&content) {
        match event {
            // Start of a heading - check if it's H2 (date header)
            Event::Start(Tag::Heading { level: pulldown_cmark::HeadingLevel::H2, .. }) => {
                in_h2 = true;
                heading_text.clear();
            }
            // Text content - capture if we're in a heading
            Event::Text(text) => {
                if in_h2 {
                    heading_text.push_str(&text);
                }
            }
            // End of heading - check if it's a date
            Event::End(TagEnd::Heading(pulldown_cmark::HeadingLevel::H2)) => {
                in_h2 = false;
                // Check if heading text matches date format YYYY-MM-DD
                if is_date_format(&heading_text) {
                    current_date = Some(heading_text.clone());
                }
            }
            // Start of a link
            Event::Start(Tag::Link { link_type: _, dest_url, .. }) => {
                link_dest = dest_url.to_string();
            }
            // End of link - add to grouped links if it's a local file
            Event::End(TagEnd::Link) => {
                // Only include local file links (not http://, not # anchors)
                if !link_dest.contains("://") && !link_dest.starts_with('#') {
                    let formatted = format_link_target(&link_dest);
                    let date = current_date.as_ref()
                        .cloned()
                        .unwrap_or_else(|| "_uncategorized".to_string());

                    grouped_links.entry(date).or_insert_with(Vec::new).push(formatted);
                }
                link_dest.clear();
            }
            _ => {}
        }
    }

    // Output
    if json {
        // Output as JSON object with date groups
        println!("{}", json!(grouped_links));
    } else {
        // Output as plain text (links grouped by date)
        let mut dates: Vec<String> = grouped_links.keys().cloned().collect();
        dates.sort();

        // Put uncategorized first
        if grouped_links.contains_key("_uncategorized") {
            if let Some(pos) = dates.iter().position(|x| x == "_uncategorized") {
                dates.remove(pos);
                dates.insert(0, "_uncategorized".to_string());
            }
        }

        for date in dates {
            if let Some(links) = grouped_links.get(&date) {
                println!("## {}", date);
                for link in links {
                    println!("{}", link);
                }
            }
        }
    }

    Ok(())
}

/// Check if a string matches date format YYYY-MM-DD
fn is_date_format(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    // Format: YYYY-MM-DD
    bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

/// Format a link target for display
/// - For daily notes (daily/YYYYMMDD/HHmmSS-xxx.md): YYYYMMDD/HHmmSS-xxx
/// - For notes with hash (note/hash/title.md): hash/title
/// - For top-level notes (note/title.md): title
fn format_link_target(link: &str) -> String {
    // Remove .md extension if present
    let without_ext = link.strip_suffix(".md").unwrap_or(link);

    // Handle daily notes: daily/YYYYMMDD/HHmmSS-xxx
    if without_ext.starts_with("daily/") {
        // Remove "daily/" prefix
        return without_ext[6..].to_string();
    }

    // Handle regular notes: note/... or note/hash/...
    if without_ext.starts_with("note/") {
        // Remove "note/" prefix
        return without_ext[5..].to_string();
    }

    // Return as-is for other cases
    without_ext.to_string()
}

/// Strip YAML frontmatter from content (if present)
/// YAML frontmatter starts with --- and ends with --- on its own line
fn strip_yaml_frontmatter(content: &str) -> &str {
    let content_trimmed = content.trim_start();

    // Check if content starts with YAML frontmatter delimiter
    if !content_trimmed.starts_with("---") {
        return content;
    }

    // Find closing delimiter
    let after_first_delimiter = &content_trimmed[3..];
    if let Some(end_pos) = after_first_delimiter.find("\n---") {
        let end_line_pos = end_pos + 4; // +4 for "\n---"
        let remaining = &after_first_delimiter[end_line_pos..];

        // Trim leading whitespace/newlines after the frontmatter
        return remaining.trim_start_matches(|c| c == '\n' || c == '\r');
    }

    // No closing delimiter found, return original
    content
}

/// List directory contents
fn list_directory(path: &std::path::Path) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_name: String = a.file_name().to_string_lossy().into_owned();
                let b_name: String = b.file_name().to_string_lossy().into_owned();
                a_name.cmp(&b_name)
            }
        }
    });

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden files/directories
        if name_str.starts_with('.') {
            continue;
        }

        if entry.path().is_dir() {
            println!("{}/", name_str);
        } else if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
            println!("{}", name_str);
        }
    }

    Ok(())
}

/// List only subdirectory names (for #daily - dates only, no trailing slash)
fn list_daily_subdirs(path: &std::path::Path, json: bool) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort alphabetically
    entries.sort_by(|a, b| {
        a.file_name().cmp(&b.file_name())
    });

    if json {
        // Output as JSON array
        let names: Vec<String> = entries.iter()
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .filter(|n| !n.starts_with('.'))
                } else {
                    None
                }
            })
            .collect();
        println!("{}", json!(names));
    } else {
        // Output as plain text
        for entry in entries {
            let path = entry.path();

            // Only list subdirectories, no trailing slash
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    // Skip hidden directories
                    if !name_str.starts_with('.') {
                        println!("{}", name_str);
                    }
                }
            }
        }
    }

    Ok(())
}
