//! Permanent note command module

use std::fs;
use std::io::{self, Read};
use chrono::Local;
use sha2::{Sha256, Digest};

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    title: Option<String>,
    source: Option<String>,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;
    let now = Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();

    // Generate filename
    let filename = if let Some(t) = title {
        format!("{}.md", slugify(&t))
    } else {
        format!("{}.md", timestamp)
    };

    // Determine the directory path
    let note_dir = if let Some(ref src) = source {
        // With source: note/{hash}/
        let hash = abbreviate_hash(&hash_source(src));
        capsa_ref.path.join("note").join(&hash)
    } else {
        // Without source: note/
        capsa_ref.path.join("note")
    };

    // Create directory and note file
    fs::create_dir_all(&note_dir)?;
    let note_path = note_dir.join(&filename);

    // Read content from stdin (empty if no data)
    let content = read_stdin_content()?;

    // Write the file
    fs::write(&note_path, content)?;

    // If source is provided, create a .source file with the original source string
    if let Some(src) = source {
        let source_file = note_dir.join(".source");
        fs::write(&source_file, &src)?;
    }

    // Output full path for shell pipeline compatibility
    println!("{}", note_path.display());

    Ok(())
}

/// Read content from stdin, returns empty string if no data
fn read_stdin_content() -> io::Result<String> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(0) => Ok(String::new()),
        Ok(_) => Ok(buffer),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(String::new()),
        Err(e) => Err(e),
    }
}

/// Convert title to slug (lowercase, replace spaces with hyphens)
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Hash source string using SHA256
fn hash_source(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

/// Abbreviate hash to git-style length (12 characters for SHA256)
/// Git uses 7 characters for SHA1 (160 bits), for SHA256 we use 12
fn abbreviate_hash(full_hash: &str) -> String {
    full_hash.chars().take(12).collect()
}
