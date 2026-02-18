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
        emx_note::CapsaCommand::Create { name, path } => {
            create_capsa(ctx, &name, path)?;
        }
        emx_note::CapsaCommand::Resolve { name } => {
            resolve_capsa(ctx, &name)?;
        }
    }
    Ok(())
}

fn create_capsa(ctx: &ResolveContext, name: &str, path: Option<String>) -> io::Result<()> {
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

    // Get the namespaced name (with agent namespace if applicable)
    let namespaced_name = ctx.apply_agent_namespace(name);
    let capsa_path = ctx.home.join(&namespaced_name);

    if let Some(target_path) = path {
        // Create a link capsa
        // Validate the target path exists
        let target = std::path::Path::new(&target_path);
        if !target.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Target path does not exist: {}", target_path),
            ));
        }

        // Convert to absolute path (dunce handles UNC prefix on Windows)
        let absolute = dunce::canonicalize(target)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid target path: {}", e)))?;

        // Create link file with INI-style content
        let link_content = format!("[link]\ntarget = {}", util::display_path(&absolute));
        fs::write(&capsa_path, link_content)?;

        // Output the link file path
        println!("{}", util::display_path(&capsa_path));
    } else {
        // Create a regular directory capsa
        // Create the directory
        fs::create_dir_all(&capsa_path)?;

        // Create default subdirectories
        fs::create_dir_all(capsa_path.join("#daily"))?;

        // Output the created path
        println!("{}", util::display_path(&capsa_path));
    }

    Ok(())
}

fn resolve_capsa(ctx: &ResolveContext, name: &str) -> io::Result<()> {
    let cap_ref = ctx.resolve_capsa(name).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Capsa '{}' not found", name))
    })?;

    println!("{}", util::display_path(&cap_ref.path));
    Ok(())
}
