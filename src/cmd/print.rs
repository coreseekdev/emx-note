use std::io;
use crate::ResolveContext;

pub fn run(ctx: &ResolveContext, caps: Option<&str>, note_name: String, _mentions: bool) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Printing note: {}", note_name);
    println!("  in capsa: {}", capsa_ref.name);
    println!("(Print command not yet implemented)");
    Ok(())
}
