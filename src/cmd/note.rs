//! Permanent note command module

use std::fs;
use std::io::{self, Read};
use chrono::Local;
use emx_note::util;

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
        format!("{}.md", util::slugify(&t))
    } else {
        format!("{}.md", timestamp)
    };

    // Determine the directory path
    let note_dir = if let Some(ref src) = source {
        // With source: note/{hash}/
        let hash = util::abbreviate_hash(&util::hash_source(src));
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
    println!("{}", util::display_path(&note_path));

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
