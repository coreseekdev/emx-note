use std::path::Path;

pub fn run(home: &Path, name: &str) -> std::io::Result<()> {
    let capsa_path = home.join(name);

    if !capsa_path.exists() {
        eprintln!("Capsa '{}' does not exist", name);
        std::process::exit(1);
    }

    // Check if directory is not empty
    let has_notes = std::fs::read_dir(&capsa_path)?
        .filter_map(|e| e.ok())
        .any(|e| e.path().is_file());

    if has_notes {
        eprintln!("Cannot delete non-empty capsa '{}'", name);
        eprintln!("Use --force to delete anyway (not implemented yet)");
        std::process::exit(1);
    }

    std::fs::remove_dir(&capsa_path)?;
    println!("Deleted capsa: {}", name);

    Ok(())
}
