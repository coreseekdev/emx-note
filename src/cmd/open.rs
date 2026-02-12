use std::io;
use crate::ResolveContext;

pub fn run(ctx: &ResolveContext, caps: Option<&str>, note_name: String) -> io::Result<()> {
    let capsa_ref = resolve_capsa(ctx, caps)?;
    println!("Opening note: {}", note_name);
    println!("  in capsa: {} at {}", capsa_ref.name, capsa_ref.path.display());
    println!("(Open command not yet implemented)");
    Ok(())
}

pub fn resolve_capsa(ctx: &ResolveContext, caps: Option<&str>) -> io::Result<emx_note::CapssaRef> {
    let default = ctx.default_capsa_name();
    let capsa_name = caps.unwrap_or(&default);
    ctx.resolve_capsa(capsa_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Capsa not found"))
}
