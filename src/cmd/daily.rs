use std::io::{self, Write};
use std::fs::{self, OpenOptions};
use chrono::Local;
use emx_note::CapssaRef;

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, title: Option<String>) -> io::Result<()> {
    let capsa_ref = super::open::resolve_capsa(ctx, caps)?;
    let now = Local::now();
    let date_str = now.format("%Y%m%d").to_string();
    let time_str = now.format("%H%M%S").to_string();
    let date_display = now.format("%Y-%m-%d").to_string();
    let datetime_display = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Use provided title or default
    let title = title.unwrap_or_else(|| "Daily Note".to_string());

    // Create slug (empty if default title)
    let slug = if title == "Daily Note" {
        String::new()
    } else {
        format!("-{}", slugify(&title))
    };

    // Generate filename: HHmmSS[-title].md
    let filename = format!("{}{}.md", time_str, slug);

    // Create daily subdirectory
    let daily_dir = capsa_ref.path.join("daily").join(&date_str);
    fs::create_dir_all(&daily_dir)?;

    // Create note file
    let note_path = daily_dir.join(&filename);

    // Check if template exists and use it
    let template_path = capsa_ref.path.join(".template").join("DAILY.md");
    let content = if template_path.exists() {
        let template = fs::read_to_string(&template_path)?;
        // Replace placeholders
        template
            .replace("{{title}}", &title)
            .replace("{{date}}", &date_display)
            .replace("{{datetime}}", &datetime_display)
    } else {
        format!("# {}\n\n_Created: {}_\n\n", title, datetime_display)
    };

    fs::write(&note_path, content)?;

    // Update daily link file (YYYYMMDD.md)
    update_daily_link(&capsa_ref, &date_str, &date_display, &filename, &title)?;

    println!("Created daily note: {}", filename);
    println!("  in: {}", note_path.display());

    Ok(())
}

/// Update the daily link file (YYYYMMDD.md) with new note link
fn update_daily_link(
    capsa_ref: &CapssaRef,
    date_str: &str,
    date_display: &str,
    filename: &str,
    title: &str,
) -> io::Result<()> {
    let daily_link_path = capsa_ref.path.join("daily").join(format!("{}.md", date_str));

    // Create or append to the daily link file
    let mut file = if daily_link_path.exists() {
        OpenOptions::new().append(true).open(&daily_link_path)?
    } else {
        // Create new file with date header
        let mut file = fs::File::create(&daily_link_path)?;
        writeln!(file, "# {}\n", date_display)?;
        file
    };

    // Add link to the new note (relative path from daily/ directory)
    // Link format: [title](./YYYYMMDD/filename)
    writeln!(file, "- [{}]({}/{})", title, date_str, filename)?;

    Ok(())
}

/// Convert title to slug (lowercase, replace spaces with hyphens)
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
