//! Print note command module

use std::fs;
use std::io;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, note_name: String, mentions: bool) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Securely resolve note path
    let full_path = emx_note::secure_path(&capsa_ref.path, &note_name)?;

    // Ensure .md extension if not present
    let full_path = if full_path.extension().is_none() {
        full_path.with_extension("md")
    } else {
        full_path
    };

    if !full_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Note '{}' not found", note_name)
        ));
    }

    // Read and print note content
    let content = fs::read_to_string(&full_path)?;
    println!("{}", content);

    // If mentions requested, find backlinks
    if mentions {
        println!("\n--- Backlinks ---");
        let backlinks = find_backlinks(&capsa_ref.path, &note_name)?;

        if backlinks.is_empty() {
            println!("No backlinks found.");
        } else {
            for (source, context) in backlinks {
                println!("  {} - {}", source, context);
            }
        }
    }

    Ok(())
}

/// Find all notes that link to the given note
fn find_backlinks(capsa_path: &std::path::PathBuf, note_name: &str) -> io::Result<Vec<(String, String)>> {
    use pulldown_cmark::{Parser, Event, Tag};

    let mut backlinks = Vec::new();
    let note_stem = note_name.trim_end_matches(".md");

    // Find all markdown files
    let mut notes: Vec<std::path::PathBuf> = Vec::new();
    find_notes_recursive(capsa_path, capsa_path, &mut notes)?;

    for note_path in &notes {
        let source_relative = note_path.strip_prefix(capsa_path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        // Skip the note itself
        if source_relative.trim_end_matches(".md") == note_stem {
            continue;
        }

        let content = match fs::read_to_string(note_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Check for wiki links and markdown links
        let mut found = false;
        let context: String;

        // Extract wiki links from raw content
        if content.contains(&format!("[[{}]]", note_stem)) ||
           content.contains(&format!("[[{}|", note_stem)) ||
           content.contains(&format!("[[{}#", note_stem)) {
            found = true;
        }

        // Also check markdown links using pulldown-cmark
        let parser = Parser::new(&content);
        for event in parser {
            if let Event::Start(Tag::Link { dest_url, .. }) = event {
                let url = dest_url.to_string();
                if url.trim_end_matches(".md") == note_stem ||
                   url.starts_with(&format!("{}#", note_stem)) {
                    found = true;
                    break;
                }
            }
        }

        if found {
            // Get first line of content as context
            context = content.lines()
                .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .map(|l| l.chars().take(50).collect::<String>())
                .unwrap_or_default();

            backlinks.push((source_relative, context));
        }
    }

    Ok(backlinks)
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
