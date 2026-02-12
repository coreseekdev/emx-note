//! Create note command module

use std::fs;
use std::io::{self, Read};

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_name: String,
    content: Option<String>,
    overwrite: bool,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Securely resolve note path (prevents path traversal)
    let note_path = emx_note::secure_path(&capsa_ref.path, &note_name)?;

    // Ensure .md extension
    let note_path = if note_path.extension().is_none() {
        note_path.with_extension("md")
    } else {
        note_path
    };

    // Read content from stdin or argument
    let final_content = read_content(content)?;

    // Check if note already exists
    if note_path.exists() && !overwrite {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Note '{}' already exists. Use --overwrite to replace.", note_name)
        ));
    }

    // Create parent directories if needed
    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the file
    fs::write(&note_path, &final_content)?;

    // Output full path for shell pipeline compatibility
    println!("{}", note_path.display());

    Ok(())
}

/// Read content from argument or stdin.
/// - Some("-") -> read from stdin
/// - Some(text) -> use the text directly
/// - None -> read from stdin (empty string if no data)
fn read_content(content: Option<String>) -> io::Result<String> {
    match content {
        Some(arg) if arg == "-" => read_from_stdin(),
        Some(text) => Ok(text),
        None => {
            // Try to read from stdin, return empty if no data
            match read_from_stdin() {
                Ok(s) if !s.is_empty() => Ok(s),
                _ => Ok(String::new()),
            }
        }
    }
}

/// Read all content from stdin
fn read_from_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
