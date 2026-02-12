//! Delete note command module

use std::fs;
use std::io;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, note_path: String) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Securely resolve note path
    let full_path = emx_note::secure_path(&capsa_ref.path, &note_path)?;

    // Ensure .md extension if not present
    let full_path = if full_path.extension().is_none() {
        full_path.with_extension("md")
    } else {
        full_path
    };

    if !full_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Note '{}' not found", note_path)
        ));
    }

    if full_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is a directory, not a note", note_path)
        ));
    }

    // Delete the file
    fs::remove_file(&full_path)?;

    println!("Deleted note: {}", note_path);
    println!("  from: {}", full_path.display());

    Ok(())
}
