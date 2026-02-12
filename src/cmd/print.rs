//! Print note command module

use std::fs;
use std::io;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, note_name: String) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Default extensions to search
    let extensions = vec![".md", ".mx", ".emx"];

    // Resolve note using new resolver
    let resolved = emx_note::resolve_note(&capsa_ref.path, &note_name, &extensions.as_slice())?;

    match resolved {
        emx_note::ResolvedNote::Found(path) => {
            // Read and print note content
            let content = fs::read_to_string(&path)?;
            println!("{}", content);
            Ok(())
        }
        emx_note::ResolvedNote::Ambiguous(candidates) => {
            // Print conflict message to stderr
            eprintln!("Error: Ambiguous note reference '{}'", note_name);
            eprintln!("Found {} matching notes:", candidates.len());
            for (i, path) in candidates.iter().enumerate() {
                // Get relative path for cleaner output
                let relative = path.strip_prefix(&capsa_ref.path)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                eprintln!("  {}. {}", i + 1, relative);
            }

            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Ambiguous note reference: {} candidates found", candidates.len())
            ))
        }
        emx_note::ResolvedNote::NotFound => {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Note '{}' not found", note_name)
            ))
        }
    }
}
