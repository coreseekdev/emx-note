use std::io;
use emx_note::util;

pub fn run(ctx: &emx_note::ResolveContext, path_only: bool) -> io::Result<()> {
    let default_name = ctx.default_capsa_name();
    if path_only {
        if let Some(cap_ref) = ctx.resolve_default() {
            println!("{}", util::display_path(&cap_ref.path));
        } else {
            eprintln!("Default capsa '{}' not found", default_name);
            std::process::exit(1);
        }
    } else {
        println!("Default capsa: {}", default_name);
        if let Some(cap_ref) = ctx.resolve_default() {
            println!("Path: {}", util::display_path(&cap_ref.path));
            println!("Is link: {}", cap_ref.is_link);
        } else {
            println!("(not found)");
        }
    }
    Ok(())
}
