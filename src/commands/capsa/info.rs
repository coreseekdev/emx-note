use std::path::Path;

pub fn run(home: &Path, name: &str) {
    let capsa_path = home.join(name);

    println!("Capsa: {}", name);
    println!("Path: {}", capsa_path.display());
    println!("Exists: {}", capsa_path.exists());

    if capsa_path.exists() {
        if let Ok(notes) = std::fs::read_dir(&capsa_path)
            .map(|entries| entries.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count())
        {
            println!("Notes: {}", notes);
        }
    }
}
