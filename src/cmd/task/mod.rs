//! Task management command module
//!
//! Implements task tracking in TASK.md files with agent coordination.
//! Uses EditOp pattern for file modifications.

mod reader;
mod add;
mod take;
mod comment;
mod release;
mod list;
mod show;
mod log;
mod find;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use chrono::Local;

pub use reader::{TaskFileReader, TaskStatus};

/// Get task file path
pub fn task_file_path(capsa_path: &Path) -> PathBuf {
    // Allow override via environment variable
    if let Ok(filename) = std::env::var("EMX_TASKFILE") {
        capsa_path.join(filename)
    } else {
        capsa_path.join("TASK.md")
    }
}

/// Get current timestamp for comments
pub fn get_timestamp() -> String {
    // Allow override via environment variable for reproducible tests
    if let Ok(ts) = std::env::var("EMX_TASK_TIMESTAMP") {
        ts
    } else {
        Local::now().format("%Y-%m-%d %H:%M").to_string()
    }
}

/// Get agent name from environment
pub fn get_agent_name() -> Option<String> {
    std::env::var("EMX_AGENT_NAME").ok()
        .filter(|s| !s.is_empty())
        .map(|s| format!("@{}", s))
}

/// Load or create TASK.md content
pub fn load_task_content(capsa_path: &Path) -> io::Result<String> {
    let path = task_file_path(capsa_path);

    if path.exists() {
        fs::read_to_string(&path)
    } else {
        // Return default empty TASK.md content
        // Format: frontmatter, blank line, body separator, blank line for references
        Ok(
r#"---
PREFIX: TASK-
---

---

"#.to_string())
    }
}

/// Save content to TASK.md
pub fn save_task_content(capsa_path: &Path, content: &str) -> io::Result<()> {
    let path = task_file_path(capsa_path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, content)
}

/// Main entry point
pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, cmd: emx_note::TaskCommand) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    match cmd {
        emx_note::TaskCommand::Add { node_ref } => add::run(&capsa_ref.path, &node_ref),
        emx_note::TaskCommand::Take { task_id, title, header, dry_run } => {
            take::run(&capsa_ref.path, &task_id, title.as_deref(), header.as_deref(), dry_run)
        }
        emx_note::TaskCommand::Comment { task_id, message, git, dry_run } => {
            comment::run(&capsa_ref.path, &task_id, &message, git.as_deref(), dry_run)
        }
        emx_note::TaskCommand::Release { task_ids, done, force, dry_run } => {
            release::run(&capsa_ref.path, &task_ids, done, force, dry_run)
        }
        emx_note::TaskCommand::List { status, oneline, owner } => {
            list::run(&capsa_ref.path, status.as_deref(), oneline, owner.as_deref())
        }
        emx_note::TaskCommand::Show { task_id } => show::run(&capsa_ref.path, &task_id),
        emx_note::TaskCommand::Log { task_id } => log::run(&capsa_ref.path, &task_id),
        emx_note::TaskCommand::Find { node_ref } => find::run(&capsa_ref.path, &node_ref),
    }
}
