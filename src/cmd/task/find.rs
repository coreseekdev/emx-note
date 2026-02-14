//! Task find command

use std::io;
use std::path::Path;
use super::{TaskFileReader, TaskStatus, task_file_path};

/// Find tasks by note reference
pub fn run(capsa_path: &Path, node_ref: &str) -> io::Result<()> {
    let path = task_file_path(capsa_path);

    if !path.exists() {
        eprintln!("No tasks found matching '{}'", node_ref);
        return Ok(());
    }

    let reader = TaskFileReader::load(&path)?;
    let tasks = reader.all_tasks();

    let matching: Vec<_> = tasks.iter()
        .filter(|t| t.node_ref.contains(node_ref))
        .collect();

    if matching.is_empty() {
        eprintln!("No tasks found matching '{}'", node_ref);
        return Ok(());
    }

    for task in matching {
        let status_str = match task.status {
            TaskStatus::Backlog => "backlog",
            TaskStatus::Doing => "doing",
            TaskStatus::Done => "done",
        };
        let owner = task.owner.as_deref().unwrap_or("(none)");
        println!("{:<10} {:<24} {:<10} {}",
                 task.id,
                 task.title.as_deref().unwrap_or("-"),
                 status_str,
                 owner);
    }

    Ok(())
}
