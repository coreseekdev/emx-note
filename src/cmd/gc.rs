//! Garbage collection module for orphaned notes
//!
//! Finds notes that:
//! 1. Are older than N days
//! 2. Have no incoming links from other notes
//!
//! Default mode is dry-run (list only), use --execute to actually delete.
//! Use --force to skip confirmation prompt.

use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Local};
use pulldown_cmark::{Parser, Event, Tag};

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    days: u32,
    execute: bool,
    force: bool,
    verbose: bool,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    println!("Scanning for orphaned notes in capsa: {}", capsa_ref.name);
    println!("Minimum age: {} days", days);
    println!();

    // Find all markdown files
    let all_notes = find_all_notes(&capsa_ref.path)?;

    if verbose {
        println!("Found {} notes total", all_notes.len());
    }

    // Build a map of all links (target -> sources)
    let link_map = build_link_map(&capsa_ref.path, &all_notes, verbose)?;

    // Find orphaned notes older than N days
    let min_age = Duration::from_secs(days as u64 * 24 * 60 * 60);
    let now = SystemTime::now();
    let mut orphaned: Vec<(PathBuf, String)> = Vec::new();

    for note_path in &all_notes {
        // Get relative path
        let relative = note_path.strip_prefix(&capsa_ref.path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        // Skip files in special directories
        if relative.starts_with(".template/") || relative.starts_with("daily/") {
            continue;
        }

        // Check age
        let metadata = fs::metadata(note_path)?;
        let created = metadata.created()
            .or_else(|_| metadata.modified())?;
        let age = now.duration_since(created).unwrap_or_default();

        if age < min_age {
            if verbose {
                let created_dt: DateTime<Local> = created.into();
                println!("  [skipping] {} (too recent: {})", relative, created_dt.format("%Y-%m-%d"));
            }
            continue;
        }

        // Check if has incoming links
        let has_links = link_map.contains_key(&relative);

        if !has_links {
            orphaned.push((note_path.clone(), relative));
        } else if verbose {
            let sources = link_map.get(&relative).unwrap();
            println!("  [linked] {} (from {} note(s))", relative, sources.len());
        }
    }

    if orphaned.is_empty() {
        println!("No orphaned notes found.");
        return Ok(());
    }

    println!("Found {} orphaned note(s):", orphaned.len());
    println!();

    for (path, relative) in &orphaned {
        let metadata = fs::metadata(path)?;
        let created = metadata.created()
            .or_else(|_| metadata.modified())?;
        let created_dt: DateTime<Local> = created.into();

        println!("  {} (created: {})", relative, created_dt.format("%Y-%m-%d"));
    }

    println!();

    if execute {
        // Require confirmation unless --force is specified
        if !force {
            print!("Delete {} note(s)? [y/N] ", orphaned.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        println!("Deleting {} orphaned note(s)...", orphaned.len());
        for (path, relative) in &orphaned {
            fs::remove_file(path)?;
            println!("  Deleted: {}", relative);
        }
        println!("Done.");
    } else {
        println!("Dry-run mode. Run with --execute to actually delete these notes.");
        println!("Tip: Use --execute --force to skip confirmation prompt.");
    }

    Ok(())
}

/// Find all markdown files in the capsa
fn find_all_notes(capsa_path: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut notes = Vec::new();
    find_notes_recursive(capsa_path, &capsa_path, &mut notes)?;
    Ok(notes)
}

fn find_notes_recursive(base: &PathBuf, current: &PathBuf, notes: &mut Vec<PathBuf>) -> io::Result<()> {
    if !current.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories
            if let Some(name) = path.file_name() {
                if !name.to_string_lossy().starts_with('.') {
                    find_notes_recursive(base, &path, notes)?;
                }
            }
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            notes.push(path);
        }
    }

    Ok(())
}

/// Build a map of all links (target -> list of source files) using pulldown-cmark
fn build_link_map(
    capsa_path: &PathBuf,
    notes: &[PathBuf],
    verbose: bool,
) -> io::Result<std::collections::HashMap<String, Vec<String>>> {
    let mut link_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for note_path in notes {
        let source_relative = note_path.strip_prefix(capsa_path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        let content = match fs::read_to_string(note_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Extract links using pulldown-cmark (for standard markdown links)
        let mut links = extract_links(&content);

        // Also extract wiki links from raw content (pulldown-cmark may not handle them as text)
        extract_wiki_links(&content, &mut links);

        for target in links.iter() {
            let normalized = normalize_link(target);
            link_map.entry(normalized).or_default().push(source_relative.clone());
        }
    }

    if verbose {
        let total_links: usize = link_map.values().map(|v| v.len()).sum();
        println!("Found {} incoming links across all notes", total_links);
    }

    Ok(link_map)
}

/// Extract all links from markdown content using pulldown-cmark
fn extract_links(content: &str) -> HashSet<String> {
    let mut links = HashSet::new();
    let parser = Parser::new(content);

    for event in parser {
        match event {
            // Standard markdown links [text](url)
            Event::Start(Tag::Link { dest_url, .. }) => {
                // Only track internal .md links
                let url_str = dest_url.to_string();
                if is_internal_link(&url_str) {
                    links.insert(url_str);
                }
            }
            // Wiki-style links [[note]] are not directly supported by pulldown-cmark
            // but we can check for them in text events
            Event::Text(text) => {
                extract_wiki_links(&text, &mut links);
            }
            Event::InlineHtml(html) | Event::Html(html) => {
                // Also check HTML blocks for wiki links (some parsers handle them this way)
                extract_wiki_links(&html, &mut links);
            }
            _ => {}
        }
    }

    links
}

/// Check if a link is an internal markdown link
fn is_internal_link(url: &str) -> bool {
    // Skip external URLs
    if url.starts_with("http://") || url.starts_with("https://") ||
       url.starts_with("ftp://") || url.starts_with("mailto:") ||
       url.starts_with("tel:") {
        return false;
    }

    // Check if it's a markdown file or has no extension (wiki-style)
    url.ends_with(".md") || !url.contains('.')
}

/// Extract wiki-style [[links]] from text
fn extract_wiki_links(text: &str, links: &mut HashSet<String>) {
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' && chars.peek() == Some(&'[') {
            chars.next(); // consume second '['

            let mut link = String::new();
            let mut closed = false;

            while let Some(c) = chars.next() {
                if c == ']' && chars.peek() == Some(&']') {
                    chars.next(); // consume second ']'
                    closed = true;
                    break;
                }
                // Handle display text separator
                if c == '|' {
                    break;
                }
                // Handle anchor
                if c == '#' && !link.is_empty() {
                    // Keep anchor as part of link for now, normalize_link will strip it
                }
                link.push(c);
            }

            if closed && !link.is_empty() {
                links.insert(link);
            }
        }
    }
}

/// Normalize a link target to a consistent format
fn normalize_link(link: &str) -> String {
    let mut normalized = link.to_string();

    // Remove anchor part
    if let Some(idx) = normalized.find('#') {
        normalized.truncate(idx);
    }

    // Ensure .md extension
    if !normalized.ends_with(".md") {
        normalized.push_str(".md");
    }

    // Normalize path separators
    normalized.replace('\\', "/")
}
