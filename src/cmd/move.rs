use std::io;
use crate::ResolveContext;

pub fn run(
    ctx: &ResolveContext,
    caps: Option<&str>,
    current: String,
    new: String,
    _open: bool,
) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Moving note: {} -> {}", current, new);
    println!("  in capsa: {}", capsa_ref.name);
    println!("(Move command not yet implemented)");
    Ok(())
}
