//! Permanent note command module

use std::io::{self, Read};
use emx_note::{CapsaEngine, util};

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    title: Option<String>,
    source: Option<String>,
) -> io::Result<()> {
    let capsa = CapsaEngine::new(super::resolve::resolve_capsa(ctx, caps)?);

    // Read content from stdin (empty if no data)
    let content = read_stdin_content()?;

    // Create the permanent note
    let note_path = capsa.create_permanent_note(
        title.as_deref(),
        source.as_deref(),
        &content,
    )?;

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
