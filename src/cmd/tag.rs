//! Tag management module
//!
//! Tags are stored as #xxxx.md files in the capsa root directory.
//! Notes added to tags are grouped by date.

use std::fs;
use std::io;
use std::path::PathBuf;
use chrono::Local;
use emx_note::{EditOp, apply_edits, TagCommand};
use emx_note::util;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, cmd: TagCommand) -> io::Result<()> {
    match cmd {
        TagCommand::Add { note_ref, tags, force } => {
            add_tags(ctx, caps, &note_ref, &tags, force)
        }
        TagCommand::Remove { note_ref, tags, force } => {
            remove_tags(ctx, caps, &note_ref, &tags, force)
        }
    }
}

/// Get the tag file path (in capsa root directory)
fn tag_path(capsa_path: &PathBuf, tag: &str) -> PathBuf {
    let tag_name = tag.trim_start_matches('#');
    capsa_path.join(format!("#{}.md", tag_name))
}

/// Add tags to a note
fn add_tags(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_ref: &str,
    tags: &[String],
    force: bool,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Resolve note reference with force support
    let note_paths = emx_note::resolve_note_with_force(
        &capsa_ref.path,
        note_ref,
        emx_note::DEFAULT_EXTENSIONS,
        force,
        "add tags to"
    )?;

    // Add each tag to each note
    for note_path in &note_paths {
        let relative = note_path.strip_prefix(&capsa_ref.path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        for tag in tags {
            add_single_tag(&capsa_ref.path, &relative, note_path, tag)?;
        }
    }

    Ok(())
}

/// Add a single tag to a single note
fn add_single_tag(
    capsa_path: &PathBuf,
    note_relative: &str,
    note_path: &PathBuf,
    tag: &str,
) -> io::Result<()> {
    let tag_file = tag_path(capsa_path, tag);
    let now = Local::now();
    let date_display = now.format("%Y-%m-%d").to_string();

    // Get note title
    let note_title = get_note_title(note_path)?;

    // The link line is our unique source locator for checking duplicates
    let link_line = format!("- [{}]({})", note_title, note_relative);

    if tag_file.exists() {
        // Check if note already tagged
        let content = fs::read_to_string(&tag_file)?;
        if content.lines().any(|line| line.contains(&format!("]({})", note_relative))) {
            return Ok(()); // Already tagged, silently skip
        }

        // Build edits to add date header (if needed) and link
        let date_header = format!("## {}", date_display);
        let has_date_header = content.lines().any(|line| line == date_header);

        let mut edits = Vec::new();

        if !has_date_header {
            // Need to add date header first
            edits.push(EditOp::append(format!("\n{}", date_header)));
        }

        // Add the link line (unique source locator is the link content)
        edits.push(EditOp::append(link_line));

        let new_content = apply_edits(&content, edits)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        fs::write(&tag_file, new_content)?;
    } else {
        // Create new tag file with heading, date header, and link
        let content = format!("# {}\n\n{}\n{}", tag.trim_start_matches('#'), date_display, link_line);
        fs::write(&tag_file, content)?;
    }

    // Output tag file path
    println!("{}", util::display_path(&tag_file));

    Ok(())
}

/// Remove tags from a note
fn remove_tags(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_ref: &str,
    tags: &[String],
    force: bool,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Resolve note reference with force support
    let note_paths = emx_note::resolve_note_with_force(
        &capsa_ref.path,
        note_ref,
        emx_note::DEFAULT_EXTENSIONS,
        force,
        "remove tags from"
    )?;

    // Remove each tag from each note
    for note_path in &note_paths {
        let relative = note_path.strip_prefix(&capsa_ref.path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        for tag in tags {
            remove_single_tag(&capsa_ref.path, &relative, tag)?;
        }
    }

    Ok(())
}

/// Remove a single tag from a single note
fn remove_single_tag(capsa_path: &PathBuf, note_relative: &str, tag: &str) -> io::Result<()> {
    let tag_file = tag_path(capsa_path, tag);

    if !tag_file.exists() {
        // Tag doesn't exist, silently succeed (idempotent)
        return Ok(());
    }

    let content = fs::read_to_string(&tag_file)?;

    // Step 1: Find the exact line to delete (unique source locator)
    // The line contains ](note_relative) which should be unique
    let line_to_delete = content.lines()
        .find(|line| line.contains(&format!("]({})", note_relative)));

    let link_line = match line_to_delete {
        Some(line) => line.to_string(),
        None => return Ok(()), // Note wasn't in this tag, silently succeed
    };

    // Step 2: Find the date header for this link (to check if section becomes empty)
    let mut date_to_delete: Option<String> = None;
    let mut current_date_header: Option<String> = None;
    let mut links_count = 0;

    for line in content.lines() {
        if line.starts_with("## ") {
            // Check previous section
            if links_count == 0 && current_date_header.is_some() {
                // Previous section was already empty (shouldn't happen but handle it)
            }
            current_date_header = Some(line.to_string());
            links_count = 0;
        } else if line.starts_with("- [") {
            if line == link_line {
                // This is the line we're deleting, don't count it
            } else {
                links_count += 1;
            }
        }
    }

    // After the loop, check if current section becomes empty
    if links_count == 0 {
        date_to_delete = current_date_header;
    }

    // Step 3: Build edits - collect all lines to delete
    let mut edits = Vec::new();

    // Delete the link line (unique source locator)
    edits.push(EditOp::delete_line(&link_line));

    // Add date header deletion if needed
    if let Some(ref date_line) = date_to_delete {
        edits.push(EditOp::delete_line(date_line));
    }

    // Step 4: Apply edits
    let new_content = apply_edits(&content, edits)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Step 5: Check if file should be deleted (only main heading left)
    let non_empty: Vec<&str> = new_content.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if non_empty.is_empty() || (non_empty.len() == 1 && non_empty[0].starts_with('#')) {
        fs::remove_file(&tag_file)?;
    } else {
        fs::write(&tag_file, &new_content)?;
    }

    // Output tag file path
    println!("{}", util::display_path(&tag_file));

    Ok(())
}

/// Extract title from note (first H1 heading or filename)
fn get_note_title(note_path: &PathBuf) -> io::Result<String> {
    let content = fs::read_to_string(note_path)?;

    // Look for first # heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Ok(trimmed.trim_start_matches("# ").to_string());
        }
    }

    // Fallback to filename
    Ok(note_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string()))
}
