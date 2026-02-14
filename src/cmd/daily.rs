//! Daily note command module

use std::io::{self, Write, Read};
use std::fs::{self, OpenOptions};
use chrono::{Local, DateTime, NaiveDateTime, TimeZone};
use emx_note::CapsaRef;
use emx_note::util;

/// Get current timestamp, allowing override via EMX_TASK_TIMESTAMP for testing
fn get_timestamp() -> DateTime<Local> {
    if let Ok(ts) = std::env::var("EMX_TASK_TIMESTAMP") {
        // Parse "YYYY-MM-DD HH:MM" format
        if let Ok(naive) = NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M") {
            // Use from_local_datetime to treat the input as local time, not UTC
            return Local.from_local_datetime(&naive).single().unwrap_or_else(|| Local::now());
        }
    }
    Local::now()
}

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, title: Option<String>) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;
    let now = get_timestamp();
    let date_str = now.format("%Y%m%d").to_string();
    let time_str = now.format("%H%M%S").to_string();
    let date_display = now.format("%Y-%m-%d").to_string();

    // Use provided title or default
    let title = title.unwrap_or_else(|| "Daily Note".to_string());

    // Create slug (empty if default title)
    let slug = if title == "Daily Note" {
        String::new()
    } else {
        format!("-{}", util::slugify(&title))
    };

    // Generate filename: HHmmSS[-title].md
    let filename = format!("{}{}.md", time_str, slug);

    // Create daily subdirectory: #daily/YYYYMMDD/
    let daily_dir = capsa_ref.path.join("#daily").join(&date_str);
    fs::create_dir_all(&daily_dir)?;

    // Create note file
    let note_path = daily_dir.join(&filename);

    // Read content from stdin (empty if no data)
    let content = read_stdin_content()?;

    // Write the file
    fs::write(&note_path, content)?;

    // Update daily link file (note/#daily.md)
    update_daily_link(&capsa_ref, &date_str, &date_display, &filename, &title)?;

    // Output full path for shell pipeline compatibility
    println!("{}", util::display_path(&note_path));

    Ok(())
}

/// Read content from stdin, returns empty string if no data
fn read_stdin_content() -> io::Result<String> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(0) => Ok(String::new()),
        Ok(_) => Ok(buffer),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(String::new()),
        Err(e) => Err(e),
    }
}

/// Update the daily link file (note/#daily.md) with new note link
fn update_daily_link(
    capsa_ref: &CapsaRef,
    date_str: &str,
    _date_display: &str,
    filename: &str,
    title: &str,
) -> io::Result<()> {
    let note_dir = capsa_ref.path.join("note");
    let daily_link_path = note_dir.join("#daily.md");

    // Ensure note/ directory exists
    fs::create_dir_all(&note_dir)?;

    // Create or append to the daily link file
    let mut file = if daily_link_path.exists() {
        OpenOptions::new().append(true).open(&daily_link_path)?
    } else {
        // Create new file with title
        let mut file = fs::File::create(&daily_link_path)?;
        writeln!(file, "# Daily Notes\n")?;
        file
    };

    // Add link to the new note
    // Link format: - [title](#daily/YYYYMMDD/filename)
    writeln!(file, "- [{}](#daily/{}/{})", title, date_str, filename)?;

    Ok(())
}
