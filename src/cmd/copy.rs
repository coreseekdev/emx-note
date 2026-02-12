//! Copy note command module

use std::fs;
use std::io;

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    source: String,
    dest: String,
    overwrite: bool,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Securely resolve source path
    let source_path = emx_note::secure_path(&capsa_ref.path, &source)?;
    let source_path = if source_path.extension().is_none() {
        source_path.with_extension("md")
    } else {
        source_path
    };

    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source note '{}' not found", source)
        ));
    }

    // Securely resolve destination path
    let dest_path = emx_note::secure_path(&capsa_ref.path, &dest)?;
    let dest_path = if dest_path.extension().is_none() {
        dest_path.with_extension("md")
    } else {
        dest_path
    };

    if dest_path.exists() && !overwrite {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Destination '{}' already exists. Use --overwrite to replace.", dest)
        ));
    }

    // Create parent directories if needed
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file
    fs::copy(&source_path, &dest_path)?;

    // Output full path for shell pipeline compatibility
    println!("{}", dest_path.display());

    Ok(())
}
