use std::io;
use emx_note::ResolveContext;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Interactive search in capsa: {}", capsa_ref.name);
    println!("(Search command not yet implemented)");
    Ok(())
}
