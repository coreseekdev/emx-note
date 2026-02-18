pub mod cli;
pub mod edit;
pub mod markdown;
pub mod resolve;
pub mod util;
pub mod note_resolver;
pub mod engine;

pub use cli::{Cli, Command, CapsaCommand, TagCommand, LinkCommand, TaskCommand};
pub use edit::{EditOp, ValidationError, apply_edits};
pub use engine::{CapsaEngine, Tags, Tag, TaskFile};
pub use markdown::{
    MarkdownHeading, MarkdownLink,
    extract_references, extract_headings, extract_links,
    has_reference, get_reference_dest, find_heading_line, extract_frontmatter_prefix,
};
pub use resolve::{ResolveContext, CapsaRef, DEFAULT_CAPSA_NAME, GLOBAL_NAMESPACE_MARKER, SHARED_NAMESPACE};
pub use util::{secure_path, validate_link_target, extract_note_title, slugify, hash_source, abbreviate_hash, MAX_FRONTMATTER_SIZE};
pub use note_resolver::{ResolvedNote, resolve_note, resolve_note_or_error, resolve_note_with_force};

/// Default notes directory name (relative to home)
pub const DEFAULT_NOTES_DIR: &str = ".emx-notes";

/// Default file extensions for notes
pub const DEFAULT_EXTENSIONS: &[&str] = &[".md", ".mx", ".emx"];

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

