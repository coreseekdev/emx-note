use std::io;
use emx_note::ResolveContext;

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_name: String,
    action: emx_note::FrontMatterAction,
) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    println!("Frontmatter for note: {}", note_name);
    println!("  in capsa: {}", capsa_ref.name);
    match action {
        emx_note::FrontMatterAction::Print => println!("  Action: print"),
        emx_note::FrontMatterAction::Edit { key, value } => println!("  Action: edit {} = {}", key, value),
        emx_note::FrontMatterAction::Delete { key } => println!("  Action: delete {}", key),
    }
    println!("(FrontMatter command not yet implemented)");
    Ok(())
}
