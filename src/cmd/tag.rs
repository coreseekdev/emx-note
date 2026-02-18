//! Tag management module
//!
//! Tags are stored as #xxxx.md files in the capsa root directory.
//! Notes added to tags are grouped by date.

use std::io;
use emx_note::{CapsaEngine, TagCommand};

pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, cmd: TagCommand) -> io::Result<()> {
    match cmd {
        TagCommand::Add { note_ref, tags, force } => {
            add_tags(ctx, caps, &note_ref, &tags, force)
        }
        TagCommand::Remove { note_ref, tags, force } => {
            remove_tags(ctx, caps, &note_ref, &tags, force)
        }
    }
}

/// Add tags to a note
fn add_tags(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_ref: &str,
    tags: &[String],
    force: bool,
) -> io::Result<()> {
    let capsa = CapsaEngine::new(super::resolve::resolve_capsa(ctx, caps)?);

    // Resolve note reference with force support
    let note_paths = capsa.resolve_note(note_ref, force)?;

    // Add each tag to each note
    for note_path in &note_paths {
        for tag_name in tags {
            capsa.tags().get(tag_name).add_note(note_path)?;
        }
    }

    Ok(())
}

/// Remove tags from a note
fn remove_tags(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_ref: &str,
    tags: &[String],
    force: bool,
) -> io::Result<()> {
    let capsa = CapsaEngine::new(super::resolve::resolve_capsa(ctx, caps)?);

    // Resolve note reference with force support
    let note_paths = capsa.resolve_note(note_ref, force)?;

    // Remove each tag from each note
    for note_path in &note_paths {
        let relative = note_path.strip_prefix(&capsa.path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        for tag_name in tags {
            capsa.tags().get(tag_name).remove_note(&relative)?;
        }
    }

    Ok(())
}
