//! Task comment command

use std::io;
use std::path::Path;
use emx_note::{EditOp, apply_edits};
use super::{TaskFileReader, TaskStatus, load_task_content, save_task_content, get_timestamp};
use super::log;

/// Add comment to task
pub fn run(
    capsa_path: &Path,
    task_id: &str,
    message: &str,
    git: Option<&str>,
    dry_run: bool,
) -> io::Result<()> {
    let content = load_task_content(capsa_path)?;
    let reader = TaskFileReader::new(content.clone());

    // Find task
    let task = reader.get_task(task_id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Task '{}' not found", task_id))
    })?;

    // Check task is in body (taken)
    if task.status == TaskStatus::Backlog {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Task '{}' not in body section\nHint: Use 'task take {}' first", task_id, task_id)
        ));
    }

    // Format comment
    let timestamp = get_timestamp();
    let comment = if let Some(hash) = git {
        format!("  - {} {} [{}]", timestamp, message, hash)
    } else {
        format!("  - {} {}", timestamp, message)
    };

    if dry_run {
        println!("--- TASK.md (append) ---");
        println!("{}", comment);
        println!("---");
        println!("Would append to: {}", task_id);
        return Ok(());
    }

    // Find task line and append comment
    if let Some((task_line_num, _task_line)) = reader.find_task_entry_line(task_id) {
        // Find where to insert the comment (after task line and any existing comments)
        let lines: Vec<&str> = content.lines().collect();
        let mut insert_at = task_line_num + 1;

        for (i, line) in lines.iter().enumerate().skip(task_line_num + 1) {
            if line.trim().starts_with("  - ") {
                insert_at = i + 1;
            } else if line.trim().starts_with("- [") {
                // Next task starts
                break;
            } else if !line.trim().is_empty() && !line.trim().starts_with("  ") {
                // Non-comment content
                break;
            }
        }

        let edits = vec![EditOp::insert_at_line(insert_at, comment)];
        let new_content = apply_edits(&content, edits)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        save_task_content(capsa_path, &new_content)?;
    }

    // Show log after adding comment
    log::run(capsa_path, task_id)
}
