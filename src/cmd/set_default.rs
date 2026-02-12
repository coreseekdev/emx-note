use std::io;
use crate::ResolveContext;

pub fn run(_ctx: &ResolveContext, caps: String) -> io::Result<()> {
    println!("Setting default capsa to: {}", caps);
    println!("(SetDefault command not yet implemented)");
    println!("Would set EMX_NOTE_DEFAULT={}", caps);
    Ok(())
}
