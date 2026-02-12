pub mod capsa;
pub mod cli;

pub use capsa::Capsa;
pub use cli::{Cli, Command};

/// Default notes directory path
pub const DEFAULT_NOTES_DIR: &str = ".emx-notes";

/// Get the default notes directory path in user's home directory
pub fn default_notes_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|p| p.join(DEFAULT_NOTES_DIR))
}
