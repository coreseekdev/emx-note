use std::io;

pub fn run(ctx: &emx_note::ResolveContext, path_only: bool) -> io::Result<()> {
    let default_name = ctx.default_capsa_name();
    if path_only {
        if let Some(cap_ref) = ctx.resolve_default() {
            println!("{}", cap_ref.path.display());
        } else {
            eprintln!("Default capsa '{}' not found", default_name);
            std::process::exit(1);
        }
    } else {
        println!("Default capsa: {}", default_name);
        if let Some(cap_ref) = ctx.resolve_default() {
            println!("Path: {}", cap_ref.path.display());
            println!("Is link: {}", cap_ref.is_link);
        } else {
            println!("(not found)");
        }
    }
    Ok(())
}
