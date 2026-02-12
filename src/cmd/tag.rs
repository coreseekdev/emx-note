//! Tag/Label management module
//!
//! Tags are stored as #xxxx.md files in the capsa root.
//! Notes added to tags are grouped by date.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use chrono::Local;
use emx_note::TagCommand;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, cmd: TagCommand) -> io::Result<()> {
    match cmd {
        TagCommand::Add { tag, note } => add_tag(ctx, caps, &tag, &note),
        TagCommand::Remove { tag, note } => remove_tag(ctx, caps, &tag, &note),
        TagCommand::List { tag } => list_tags(ctx, caps, tag.as_deref()),
        TagCommand::Delete { tag } => delete_tag(ctx, caps, &tag),
    }
}

/// Get the tag file path
fn tag_path(capsa_path: &PathBuf, tag: &str) -> PathBuf {
    // Normalize tag name (remove # prefix if present)
    let tag_name = tag.trim_start_matches('#');
    capsa_path.join(format!("#{}.md", tag_name))
}

/// Add a note to a tag
fn add_tag(ctx: &emx_note::ResolveContext, caps: Option<&str>, tag: &str, note: &str) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Verify note exists
    let note_path = capsa_ref.path.join(note);
    if !note_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Note '{}' not found", note)
        ));
    }

    let tag_file = tag_path(&capsa_ref.path, tag);
    let now = Local::now();
    let date_display = now.format("%Y-%m-%d").to_string();

    // Get note title (first heading or filename)
    let note_title = get_note_title(&note_path)?;

    // Create or append to tag file
    let mut file = if tag_file.exists() {
        // Check if note already tagged
        let content = fs::read_to_string(&tag_file)?;
        if content.contains(&format!("]({})", note)) {
            println!("Note '{}' already tagged with #{}", note, tag.trim_start_matches('#'));
            return Ok(());
        }
        OpenOptions::new().append(true).open(&tag_file)?
    } else {
        // Create new tag file with heading
        let mut file = fs::File::create(&tag_file)?;
        writeln!(file, "# {}\n", tag.trim_start_matches('#'))?;
        file
    };

    // Add date group header and link
    // Check if today's date header already exists
    let content = fs::read_to_string(&tag_file)?;
    let date_header = format!("## {}", date_display);

    if !content.contains(&date_header) {
        writeln!(file)?;
        writeln!(file, "## {}", date_display)?;
    }

    writeln!(file, "- [{}]({})", note_title, note)?;

    let tag_name = tag.trim_start_matches('#');
    println!("Added '{}' to #{}", note, tag_name);
    println!("  in: {}", tag_file.display());

    Ok(())
}

/// Remove a note from a tag
fn remove_tag(ctx: &emx_note::ResolveContext, caps: Option<&str>, tag: &str, note: &str) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;
    let tag_file = tag_path(&capsa_ref.path, tag);

    if !tag_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Tag '#{}' not found", tag.trim_start_matches('#'))
        ));
    }

    let content = fs::read_to_string(&tag_file)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = Vec::new();
    let mut removed = false;
    let mut current_date_idx: Option<usize> = None;

    for line in lines.iter() {
        // Track date headers
        if line.starts_with("## ") {
            // Check if previous date section was empty (only had the header)
            if let Some(idx) = current_date_idx {
                // Check if section only has the date header
                let section_content: Vec<&String> = new_lines[idx..].iter()
                    .filter(|l| !l.trim().is_empty())
                    .collect();
                if section_content.len() == 1 {
                    // Only date header, remove it
                    new_lines.truncate(idx);
                }
            }
            current_date_idx = Some(new_lines.len());
            new_lines.push(line.to_string());
            continue;
        }

        // Check for the note link
        if line.contains(&format!("]({})", note)) {
            removed = true;
            continue; // Skip this line (remove it)
        }

        new_lines.push(line.to_string());
    }

    // Check final date section
    if let Some(idx) = current_date_idx {
        let section_content: Vec<&String> = new_lines[idx..].iter()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if section_content.len() == 1 {
            // Only date header, remove it
            new_lines.truncate(idx);
        }
    }

    if !removed {
        println!("Note '{}' not found in #{}", note, tag.trim_start_matches('#'));
        return Ok(());
    }

    // If tag file only has main heading left, delete it
    let non_empty: Vec<&String> = new_lines.iter().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.is_empty() {
        fs::remove_file(&tag_file)?;
        println!("Removed '{}' from #{} (tag is now empty, deleted)", note, tag.trim_start_matches('#'));
    } else if non_empty.len() == 1 && non_empty[0].starts_with('#') {
        fs::remove_file(&tag_file)?;
        println!("Removed '{}' from #{} (tag is now empty, deleted)", note, tag.trim_start_matches('#'));
    } else {
        fs::write(&tag_file, new_lines.join("\n") + "\n")?;
        println!("Removed '{}' from #{}", note, tag.trim_start_matches('#'));
    }

    Ok(())
}

/// List all tags or notes in a specific tag
fn list_tags(ctx: &emx_note::ResolveContext, caps: Option<&str>, tag: Option<&str>) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    if let Some(tag_name) = tag {
        // List notes in a specific tag
        let tag_file = tag_path(&capsa_ref.path, tag_name);
        if !tag_file.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Tag '#{}' not found", tag_name.trim_start_matches('#'))
            ));
        }

        let content = fs::read_to_string(&tag_file)?;
        println!("# {}:", tag_name.trim_start_matches('#'));
        for line in content.lines() {
            if line.starts_with("- [") {
                // Extract note info
                println!("  {}", line);
            } else if line.starts_with("## ") {
                println!("{}", line);
            }
        }
    } else {
        // List all tags
        let mut tags: Vec<String> = Vec::new();

        if let Ok(entries) = fs::read_dir(&capsa_ref.path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if name_str.starts_with('#') && name_str.ends_with(".md") {
                    // Extract tag name (remove # prefix and .md suffix)
                    let tag = name_str.trim_start_matches('#').trim_end_matches(".md");
                    tags.push(tag.to_string());
                }
            }
        }

        if tags.is_empty() {
            println!("No tags found.");
        } else {
            tags.sort();
            println!("Tags:");
            for tag in tags {
                println!("  #{}", tag);
            }
        }
    }

    Ok(())
}

/// Delete a tag file
fn delete_tag(ctx: &emx_note::ResolveContext, caps: Option<&str>, tag: &str) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;
    let tag_file = tag_path(&capsa_ref.path, tag);

    if !tag_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Tag '#{}' not found", tag.trim_start_matches('#'))
        ));
    }

    fs::remove_file(&tag_file)?;
    println!("Deleted tag #{}", tag.trim_start_matches('#'));

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
