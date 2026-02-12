use std::io;
use emx_note::ResolveContext;

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_name: String,
    content: Option<String>,
    _append: bool,
    _overwrite: bool,
    _open: bool,
) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Creating note: {}", note_name);
    println!("  in capsa: {}", capsa_ref.name);
    if let Some(c) = content {
        println!("  content: {}", c);
    }
    println!("(Create command not yet implemented)");
    Ok(())
}
