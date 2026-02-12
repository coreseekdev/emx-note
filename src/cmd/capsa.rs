use std::io;
use std::fs;
use emx_note::ResolveContext;
use emx_note::util;

pub fn run(ctx: &emx_note::ResolveContext, cmd: emx_note::CapsaCommand) -> io::Result<()> {
    match cmd {
        emx_note::CapsaCommand::List => {
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
        emx_note::CapsaCommand::Create { name } => {
            create_capsa(ctx, &name)?;
        }
        emx_note::CapsaCommand::Info { name } => {
            println!("Capsa info: {}", name);
            if let Some(cap_ref) = ctx.resolve_capsa(&name) {
                println!("  Path: {}", util::display_path(&cap_ref.path));
                println!("  Is link: {}", cap_ref.is_link);
                println!("  Exists: true");
            } else {
                println!("  Exists: false");
            }
        }
        emx_note::CapsaCommand::Delete { name } => {
            delete_capsa(ctx, &name)?;
        }
    }
    Ok(())
}

fn create_capsa(ctx: &ResolveContext, name: &str) -> io::Result<()> {
    // Validate name doesn't start with '.' (reserved for system)
    if name.starts_with('.') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Capsa name cannot start with '.' (reserved for system)",
        ));
    }

    // Check if already exists
    if ctx.resolve_capsa(name).is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Capsa '{}' already exists", name),
        ));
    }

    // Get the prefixed name (with agent prefix if applicable)
    let prefixed_name = ctx.apply_agent_prefix(name);
    let capsa_path = ctx.home.join(&prefixed_name);

    // Create the directory
    fs::create_dir_all(&capsa_path)?;

    // Create default subdirectories
    fs::create_dir_all(capsa_path.join("#daily"))?;

    println!("Created capsa: {}", name);
    println!("  Path: {}", util::display_path(&capsa_path));

    Ok(())
}

fn delete_capsa(ctx: &ResolveContext, name: &str) -> io::Result<()> {
    // Cannot delete system default
    if name == emx_note::DEFAULT_CAPSA_NAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot delete system default capsa",
        ));
    }

    // Check if exists
    let cap_ref = ctx.resolve_capsa(name).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Capsa '{}' not found", name))
    })?;

    // Cannot delete linked capsae
    if cap_ref.is_link {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot delete linked capsa (delete the link file instead)",
        ));
    }

    // Delete the directory
    fs::remove_dir_all(&cap_ref.path)?;

    println!("Deleted capsa: {}", name);

    Ok(())
}
