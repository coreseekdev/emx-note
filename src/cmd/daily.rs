//! Daily note command module

use std::io;
use emx_note::{CapsaEngine, util, read_stdin_content};

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, title: Option<String>) -> io::Result<()> {
    let capsa = CapsaEngine::new(super::resolve::resolve_capsa(ctx, caps)?);

    // Read content from stdin (empty if no data)
    let content = read_stdin_content()?;

    // Create the daily note
    let note_path = capsa.create_daily_note(title.as_deref(), &content)?;

    // Output full path for shell pipeline compatibility
    println!("{}", util::display_path(&note_path));

    Ok(())
}
