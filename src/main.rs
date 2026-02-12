use clap::Parser;
use emx_note::{capsa::Capsa, Cli, Command, default_notes_path};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let notes_path = default_notes_path().unwrap_or_else(|| ".".into());
    let capsa = Capsa::new(cli.caps, &notes_path);

    match cli.command {
        Command::List => {
            println!("Capsae (note collections):");
            match Capsa::list_all(&notes_path) {
                Ok(list) => {
                    if list.is_empty() {
                        println!("  (none)");
                    } else {
                        for name in list {
                            println!("  - {}", name);
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::Create { name } => {
            let new_capsa = Capsa::new(name, &notes_path);
            if new_capsa.exists() {
                eprintln!("Capsa '{}' already exists", new_capsa.name);
                std::process::exit(1);
            }
            new_capsa.create()?;
            println!("Created capsa: {}", new_capsa.name);
            Ok(())
        }

        Command::Info => {
            println!("Capsa: {}", capsa.name);
            println!("Path: {}", capsa.path().display());
            println!("Exists: {}", capsa.exists());
            Ok(())
        }
    }
}
