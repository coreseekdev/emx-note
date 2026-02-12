use std::path::Path;

pub fn run(home: &Path, name: &str) -> std::io::Result<()> {
    let capsa_path = home.join(name);

    if capsa_path.exists() {
        eprintln!("Capsa '{}' already exists", name);
        std::process::exit(1);
    }

    std::fs::create_dir_all(&capsa_path)?;
    println!("Created capsa: {}", name);
    println!("Path: {}", capsa_path.display());

    Ok(())
}
