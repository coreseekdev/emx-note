use std::io;
use crate::ResolveContext;

pub fn run(ctx: &ResolveContext, caps: Option<&str>, note_path: String) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Deleting note: {}", note_path);
    println!("  in capsa: {}", capsa_ref.name);
    println!("(Delete command not yet implemented)");
    Ok(())
}
