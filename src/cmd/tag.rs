//! Tag management module
//!
//! Tags are stored as #xxxx.md files in the capsa root directory.
//! Notes added to tags are grouped by date.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use chrono::Local;
use emx_note::TagCommand;
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

    // Create or append to tag file
    let mut file = if tag_file.exists() {
        // Check if note already tagged
        let content = fs::read_to_string(&tag_file)?;
        if content.contains(&format!("]({})", note_relative)) {
            return Ok(()); // Already tagged, silently skip
        }
        OpenOptions::new().append(true).open(&tag_file)?
    } else {
        // Create new tag file with heading
        let mut file = fs::File::create(&tag_file)?;
        writeln!(file, "# {}\n", tag.trim_start_matches('#'))?;
        file
    };

    // Add date group header and link
    let content = fs::read_to_string(&tag_file)?;
    let date_header = format!("## {}", date_display);

    if !content.contains(&date_header) {
        writeln!(file)?;
        writeln!(file, "## {}", date_display)?;
    }

    writeln!(file, "- [{}]({})", note_title, note_relative)?;

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
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = Vec::new();
    let mut removed = false;
    let mut current_date_idx: Option<usize> = None;

    for line in lines.iter() {
        // Track date headers
        if line.starts_with("## ") {
            // Check if previous date section was empty
            if let Some(idx) = current_date_idx {
                let section_content: Vec<&String> = new_lines[idx..].iter()
                    .filter(|l| !l.trim().is_empty())
                    .collect();
                if section_content.len() == 1 {
                    new_lines.truncate(idx);
                }
            }
            current_date_idx = Some(new_lines.len());
            new_lines.push(line.to_string());
            continue;
        }

        // Check for the note link
        if line.contains(&format!("]({})", note_relative)) {
            removed = true;
            continue; // Skip this line
        }

        new_lines.push(line.to_string());
    }

    // Check final date section
    if let Some(idx) = current_date_idx {
        let section_content: Vec<&String> = new_lines[idx..].iter()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if section_content.len() == 1 {
            new_lines.truncate(idx);
        }
    }

    if !removed {
        // Note wasn't in this tag, silently succeed
        return Ok(());
    }

    // If tag file only has main heading left, delete it
    let non_empty: Vec<&String> = new_lines.iter().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.is_empty() || (non_empty.len() == 1 && non_empty[0].starts_with('#')) {
        fs::remove_file(&tag_file)?;
    } else {
        fs::write(&tag_file, new_lines.join("\n") + "\n")?;
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
