pub mod cli;
pub mod commands;

pub use cli::{Cli, Command};

/// Default notes directory name (relative to home)
pub const DEFAULT_NOTES_DIR: &str = ".emx-notes";

/// Get the notes directory path.
/// Prefers: --home flag > $EMX_NOTE_HOME > ~/.emx-notes
pub fn notes_path(home: Option<&str>) -> std::path::PathBuf {
    if let Some(h) = home {
        return std::path::PathBuf::from(h);
    }
    if let Ok(env) = std::env::var("EMX_NOTE_HOME") {
        return std::path::PathBuf::from(env);
    }
    dirs::home_dir()
        .unwrap_or_else(|| ".".into())
        .join(DEFAULT_NOTES_DIR)
}

