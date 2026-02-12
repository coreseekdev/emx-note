use std::io;
use emx_note::ResolveContext;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, path: Option<String>) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Listing notes in capsa: {}", capsa_ref.name);
    if let Some(p) = path {
        println!("  path: {}", p);
    }
    println!("(List command not yet implemented)");
    Ok(())
}
