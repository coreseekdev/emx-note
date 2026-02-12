use std::path::Path;

use crate::cli::NoteCommand;

pub fn run(home: &Path, cmd: NoteCommand) -> std::io::Result<()> {
    match cmd {
        NoteCommand::List { capsa } => list(home, &capsa),
        NoteCommand::Create { capsa, title } => create(home, &capsa, &title),
        NoteCommand::Edit { id } => edit(&id),
        NoteCommand::View { id } => view(&id),
    }
}

fn list(home: &Path, capsa: &str) -> std::io::Result<()> {
    let capsa_path = home.join(capsa);

    if !capsa_path.exists() {
        eprintln!("Capsa '{}' does not exist", capsa);
        std::process::exit(1);
    }

    println!("Notes in capsa '{}':", capsa);

    let notes: Vec<String> = std::fs::read_dir(&capsa_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    if notes.is_empty() {
        println!("  (none)");
    } else {
        for note in notes {
            println!("  - {}", note);
        }
    }

    Ok(())
}

fn create(home: &Path, capsa: &str, title: &str) -> std::io::Result<()> {
    let capsa_path = home.join(capsa);

    if !capsa_path.exists() {
        eprintln!("Capsa '{}' does not exist", capsa);
        eprintln!("Create it first with: emx-note capsa create {}", capsa);
        std::process::exit(1);
    }

    // Generate filename from title (simple implementation)
    let filename = format!("{}.md", title.replace(' ', "_").to_lowercase());
    let note_path = capsa_path.join(&filename);

    if note_path.exists() {
        eprintln!("Note '{}' already exists", title);
        std::process::exit(1);
    }

    // Create note with frontmatter
    let content = format!(
        "# {}\n\n_Created: {}_\n\n",
        title,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    std::fs::write(&note_path, content)?;
    println!("Created note: {}", filename);
    println!("Path: {}", note_path.display());

    Ok(())
}

fn edit(id: &str) -> std::io::Result<()> {
    println!("Edit note: {}", id);
    println!("(Editor integration not implemented yet)");
    Ok(())
}

fn view(id: &str) -> std::io::Result<()> {
    println!("View note: {}", id);
    println!("(View command not implemented yet)");
    Ok(())
}
