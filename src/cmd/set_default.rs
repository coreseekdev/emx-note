use std::io;

pub fn run(_ctx: &emx_note::ResolveContext, caps: String) -> io::Result<()> {
    println!("Setting default capsa to: {}", caps);
    println!("(SetDefault command not yet implemented)");
    println!("Would set EMX_NOTE_DEFAULT={}", caps);
    Ok(())
}
