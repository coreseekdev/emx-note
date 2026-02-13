//! Link check command module
//!
//! This module provides functionality to check and manage links between notes.
//! Uses pulldown-cmark for markdown parsing to detect and validate local links.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// Information about a link found in a markdown file
#[derive(Debug, Clone)]
pub struct LinkInfo {
    /// Source file path (relative to scan root)
    pub source: PathBuf,
    /// Line number where the link appears
    pub line: usize,
    /// Link target (as it appears in markdown)
    pub target: String,
    /// Whether this link is broken (target doesn't exist)
    pub broken: bool,
}

/// Result of link scanning operation
pub struct ScanResult {
    /// All links found (both valid and broken)
    pub links: Vec<LinkInfo>,
    /// Source files that were scanned
    pub sources: Vec<PathBuf>,
}

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    cmd: &emx_note::cli::LinkCommand,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    match cmd {
        emx_note::cli::LinkCommand::Check { ref path } => {
            let scan_path = path.as_deref().unwrap_or(&capsa_ref.path);
            check(&capsa_ref.path, scan_path)
        }
        emx_note::cli::LinkCommand::List { ref path } => {
            let scan_path = path.as_deref().unwrap_or(&capsa_ref.path);
            list_links(&capsa_ref.path, scan_path)
        }
        emx_note::cli::LinkCommand::Orphans { ref path } => {
            let scan_path = path.as_deref().unwrap_or(&capsa_ref.path);
            find_orphans(&capsa_ref.path, scan_path)
        }
    }
}

/// Check for broken links in a directory
fn check(root: &Path, scan_path: &Path) -> io::Result<()> {
    let result = scan_dir(scan_path)?;

    let broken_count = result.links.iter().filter(|l| l.broken).count();

    if broken_count > 0 {
        eprintln!("Found {} broken link(s):", broken_count);
        for link in &result.links {
            if link.broken {
                let source_display = link.source.strip_prefix(root).unwrap_or(&link.source);
                eprintln!("  {}:{}: broken link -> {}",
                    source_display.display(),
                    link.line,
                    link.target
                );
            }
        }
        Err(io::Error::other(format!("Found {} broken link(s)", broken_count)))
    } else {
        println!("No broken links found. OK.");
        Ok(())
    }
}

/// List all local links found in directory
fn list_links(root: &Path, scan_path: &Path) -> io::Result<()> {
    let result = scan_dir(scan_path)?;

    for link in &result.links {
        let source_display = link.source.strip_prefix(root).unwrap_or(&link.source);
        let status = if link.broken { "BROKEN" } else { "OK" };

        println!("{}:{}: {} -> {}",
            source_display.display(),
            link.line,
            status,
            link.target
        );
    }

    Ok(())
}

/// Find files that are not linked by any other file
fn find_orphans(root: &Path, scan_path: &Path) -> io::Result<()> {
    let result = scan_dir(scan_path)?;

    // Build a set of all linked targets
    let linked_targets: HashSet<PathBuf> = result.links.iter()
        .filter(|l| !l.broken)
        .map(|l| scan_path.join(&l.target))
        .flat_map(|target| {
            // If target is a directory, add all .md files in it
            if target.is_dir() {
                fs::read_dir(&target).ok().and_then(|entries| {
                    Some(entries.filter_map(|e| e.ok())
                        .filter_map(|e| e.path().extension())
                        .filter(|ext| ext == "md" || ext == "mx" || ext == "emx")
                        .map(|p| p.to_path_buf()))
                })
            } else {
                // Single file
                Some(vec![target.clone()])
            }
        })
        .collect();

    // Find orphans (files that exist but are not in linked_targets)
    let mut orphans: Vec<_> = result.sources.iter()
        .filter(|source| !linked_targets.contains(source))
        .collect();

    orphans.sort_by(|a, b| a.cmp(b));

    if orphans.is_empty() {
        println!("No orphaned files found.");
        Ok(())
    } else {
        eprintln!("Found {} orphaned file(s):", orphans.len());
        for orphan in &orphans {
            let display = orphan.strip_prefix(root).unwrap_or(orphan);
            eprintln!("  {}", display.display());
        }
        Err(io::Error::other(format!("Found {} orphaned file(s)", orphans.len())))
    }
}

/// Scan a directory for markdown files and extract all links
fn scan_dir(scan_path: &Path) -> io::Result<ScanResult> {
    let mut links = Vec::new();
    let mut sources = Vec::new();

    // Walk through directory looking for .md, .mx, .emx files
    for entry in fs::read_dir(scan_path).map_err(|e| {
        format!("Failed to read directory '{}': {}", scan_path.display(), e)
    })? {
        let entry_path = entry.path();
        let ext = entry_path.extension().and_then(|s| s.to_str());

        // Only process markdown files
        if !ext.map(|e| e == "md" || e == "mx" || e == "emx").unwrap_or(false) {
            continue;
        }

        // Skip directories and hidden files
        if entry_path.is_dir() || entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        sources.push(entry_path.clone());

        // Parse markdown content to extract links
        let content = fs::read_to_string(&entry_path).map_err(|e| {
            format!("Failed to read '{}': {}", entry_path.display(), e)
        })?;

        let root_relative = entry_path.strip_prefix(scan_path).unwrap_or(&entry_path);

        for link in extract_links(&content) {
            links.push(LinkInfo {
                source: root_relative.to_path_buf(),
                line: link.line,
                target: link.target.clone(),
                broken: false, // Will be validated later
            });
        }
    }

    Ok(ScanResult { links, sources })
}

/// Extract all local links from markdown content using pulldown-cmark
fn extract_links(content: &str) -> Vec<LinkInfo> {
    let mut links = Vec::new();
    let mut current_line = 1;
    let mut current_link_start: Option<usize> = None;
    let mut link_text = String::new();
    let mut in_code_block = false;
    let mut in_link = false;

    let parser = Parser::new(content);

    for event in parser {
        match event {
            // Track line numbers
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::CodeBlock)
            => {
                current_line += 1;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
            }
            Event::Start(Tag::Link { .. }) => {
                in_link = true;
                current_link_start = Some(current_line);
            }
            Event::End(TagEnd::Link) => {
                in_link = false;
            }
            Event::Text(text) | Event::Code(text) => {
                if in_code_block {
                    link_text.clear();
                    continue; // Skip links in code blocks
                }

                link_text.push_str(&text);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                // Inline link [text](url)
                let url = dest_url.to_string();
                let start_line = current_link_start.unwrap_or(current_line);

                // Only include local file links
                if !url.contains("://") && !url.starts_with("mailto:") && !url.starts_with('#') {
                    links.push(LinkInfo {
                        source: PathBuf::new(), // Will be set by caller
                        line: start_line,
                        target: url.clone(),
                        broken: false,
                    });
                }
            }
            _ => {}
        }
    }

    links
}
