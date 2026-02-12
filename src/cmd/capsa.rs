use std::io;
use crate::ResolveContext;

pub fn run(ctx: &ResolveContext, cmd: emx_note::CapssaCommand) -> io::Result<()> {
    match cmd {
        emx_note::CapssaCommand::List => {
            println!("Listing all capsae...");
            let capsas = ctx.list_capsas();
            if capsas.is_empty() {
                println!("  (none)");
            } else {
                for name in capsas {
                    println!("  - {}", name);
                }
            }
        }
        emx_note::CapssaCommand::Create { name } => {
            println!("Creating capsa: {}", name);
            println!("(Capsa create not yet implemented)");
        }
        emx_note::CapssaCommand::Info { name } => {
            println!("Capsa info: {}", name);
            if let Some(cap_ref) = ctx.resolve_capsa(&name) {
                println!("  Path: {}", cap_ref.path.display());
                println!("  Is link: {}", cap_ref.is_link);
            } else {
                println!("  Not found");
            }
        }
        emx_note::CapssaCommand::Delete { name } => {
            println!("Deleting capsa: {}", name);
            println!("(Capsa delete not yet implemented)");
        }
    }
    Ok(())
}
