//! Move/rename note command module

use std::fs;
use std::io;
use emx_note::util;

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    current: String,
    new: String,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Securely resolve source path
    let source_path = emx_note::secure_path(&capsa_ref.path, &current)?;
    let source_path = if source_path.extension().is_none() {
        source_path.with_extension("md")
    } else {
        source_path
    };

    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Note '{}' not found", current)
        ));
    }

    // Securely resolve destination path
    let dest_path = emx_note::secure_path(&capsa_ref.path, &new)?;
    let dest_path = if dest_path.extension().is_none() {
        dest_path.with_extension("md")
    } else {
        dest_path
    };

    if dest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Destination '{}' already exists", new)
        ));
    }

    // Create parent directories if needed
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Move the file
    fs::rename(&source_path, &dest_path)?;

    // Update links in other notes
    let updated = update_links(&capsa_ref.path, &current, &new)?;

    println!("Moved: {} -> {}", current, new);
    println!("  from: {}", util::display_path(&source_path));
    println!("  to: {}", util::display_path(&dest_path));
    if updated > 0 {
        println!("  Updated {} link(s) in other notes", updated);
    }

    Ok(())
}

/// Update links in all notes that point to the old name
fn update_links(capsa_path: &std::path::PathBuf, old_name: &str, new_name: &str) -> io::Result<usize> {
    let old_stem = old_name.trim_end_matches(".md");
    let new_stem = new_name.trim_end_matches(".md");

    let mut notes: Vec<std::path::PathBuf> = Vec::new();
    find_notes_recursive(capsa_path, capsa_path, &mut notes)?;

    let mut updated_count = 0;

    for note_path in &notes {
        let content = match fs::read_to_string(note_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut new_content = content.clone();
        let mut changed = false;

        // Update wiki links [[old-name]]
        let wiki_link = format!("[[{}]]", old_stem);
        let new_wiki_link = format!("[[{}]]", new_stem);
        if new_content.contains(&wiki_link) {
            new_content = new_content.replace(&wiki_link, &new_wiki_link);
            changed = true;
        }

        // Update wiki links with display text [[old-name|display]]
        let wiki_with_display = format!("[[{}|", old_stem);
        let new_wiki_with_display = format!("[[{}|", new_stem);
        if new_content.contains(&wiki_with_display) {
            new_content = new_content.replace(&wiki_with_display, &new_wiki_with_display);
            changed = true;
        }

        // Update wiki links with anchor [[old-name#anchor]]
        let wiki_with_anchor = format!("[[{}#", old_stem);
        let new_wiki_with_anchor = format!("[[{}#", new_stem);
        if new_content.contains(&wiki_with_anchor) {
            new_content = new_content.replace(&wiki_with_anchor, &new_wiki_with_anchor);
            changed = true;
        }

        // Update markdown links [text](old-name.md)
        let md_link = format!("]({})", old_name);
        let new_md_link = format!("]({})", new_name);
        if new_content.contains(&md_link) {
            new_content = new_content.replace(&md_link, &new_md_link);
            changed = true;
        }

        if changed {
            fs::write(note_path, &new_content)?;
            updated_count += 1;
        }
    }

    Ok(updated_count)
}

fn find_notes_recursive(base: &std::path::PathBuf, current: &std::path::PathBuf, notes: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
    if !current.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
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
