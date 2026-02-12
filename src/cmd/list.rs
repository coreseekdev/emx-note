//! List notes command module

use std::fs;
use std::io;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, path: Option<String>) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Determine the directory to list
    let list_path = if let Some(ref p) = path {
        emx_note::secure_path(&capsa_ref.path, p)?
    } else {
        capsa_ref.path.clone()
    };

    if !list_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path '{}' not found", path.unwrap_or_default())
        ));
    }

    if !list_path.is_dir() {
        // If it's a file, just print it
        println!("{}", list_path.display());
        return Ok(());
    }

    // List directory contents
    let mut entries: Vec<_> = fs::read_dir(&list_path)?
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
