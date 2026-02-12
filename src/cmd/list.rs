//! List notes command module

use std::fs;
use std::io;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, filter: Option<String>) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    if let Some(f) = filter {
        // Handle filter
        if f.starts_with('#') {
            // Tag filter: list tag file contents
            list_tag(&capsa_ref, &f)?;
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
fn list_tag(capsa_ref: &emx_note::CapsaRef, tag: &str) -> io::Result<()> {
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
        return list_daily_subdirs(&daily_dir);
    }

    let tag_file = capsa_ref.path.join(format!("#{}.md", tag_name));

    if !tag_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Tag '{}' not found", tag),
        ));
    }

    let content = fs::read_to_string(&tag_file)?;

    // Print the tag file content
    println!("{}", content);

    Ok(())
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
        let path = entry.path();

        // Skip hidden files/directories
        if name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            println!("{}/", name_str);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            println!("{}", name_str);
        }
    }

    Ok(())
}

/// List only subdirectory names (for #daily - dates only, no trailing slash)
fn list_daily_subdirs(path: &std::path::Path) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort alphabetically
    entries.sort_by(|a, b| {
        a.file_name().cmp(&b.file_name())
    });

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

    Ok(())
}
