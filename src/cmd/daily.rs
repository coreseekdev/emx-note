use std::io;
use std::fs;
use chrono::Local;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    let now = Local::now();
    let date_str = now.format("%Y%m%d");
    let time_str = now.format("%H%M%S");

    // Parse args or use default
    let title = "Daily Note";

    // Create slug
    let slug = slugify(&title);
    let filename = format!("{}-{}.md", time_str, slug);

    // Create daily subdirectory
    let daily_dir = capsa_ref.path.join(&format!("daily/{}", date_str));
    fs::create_dir_all(&daily_dir)?;

    // Create note file
    let note_path = daily_dir.join(&filename);

    // Check if template exists and use it
    let template_path = capsa_ref.path.join(".template").join("DAILY.md");
    let content = if template_path.exists() {
        fs::read_to_string(&template_path)?
    } else {
        format!("# {}\n\n_Created: {}_\n\n", title, now.format("%Y-%m-%d %H:%M:%S"))
    };

    fs::write(&note_path, content)?;

    println!("Created daily note: {}", filename);
    println!("  in: {}", note_path.display());

    Ok(())
}

/// Convert title to slug (lowercase, replace spaces with hyphens)
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}
