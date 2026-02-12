use std::path::Path;

pub fn run(home: &Path) -> std::io::Result<()> {
    println!("Capsae (note collections):");
    if !home.exists() {
        println!("  (none)");
        return Ok(());
    }

    let mut capsae: Vec<String> = std::fs::read_dir(home)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    capsae.sort();

    if capsae.is_empty() {
        println!("  (none)");
    } else {
        for name in capsae {
            println!("  - {}", name);
        }
    }

    Ok(())
}
