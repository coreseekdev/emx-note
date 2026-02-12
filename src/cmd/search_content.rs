use std::io;
use crate::ResolveContext;

pub fn run(ctx: &ResolveContext, caps: Option<&str>, search_term: String) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Searching content for: {}", search_term);
    println!("  in capsa: {}", capsa_ref.name);
    println!("(SearchContent command not yet implemented)");
    Ok(())
}
