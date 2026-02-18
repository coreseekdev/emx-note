//! Task find command

use std::io;
use emx_note::CapsaEngine;
use super::{TaskFileReader, TaskStatus};

/// Find tasks by note reference
pub fn run(capsa: &CapsaEngine, node_ref: &str) -> io::Result<()> {
    let task_file = capsa.task_file();
    let path = task_file.file();

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
