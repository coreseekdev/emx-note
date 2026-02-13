//! Print note command module

use std::fs;
use std::io;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, note_name: String) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Resolve note using helper function
    let note_path = emx_note::resolve_note_or_error(
        &capsa_ref.path,
        &note_name,
        emx_note::DEFAULT_EXTENSIONS
    )?;

    // Read and print note content
    let content = fs::read_to_string(&note_path)?;
    println!("{}", content);
    Ok(())
}
