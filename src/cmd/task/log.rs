//! Task log command

use std::io;
use std::path::Path;
use super::{TaskFileReader, TaskStatus, task_file_path};

/// Show execution log
pub fn run(capsa_path: &Path, task_id: &str) -> io::Result<()> {
    let path = task_file_path(capsa_path);

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task file not found")
        ));
    }

    let reader = TaskFileReader::load(&path)?;
    let task = reader.get_task(task_id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Task '{}' not found", task_id))
    })?;

    let status_str = match task.status {
        TaskStatus::Backlog => "backlog",
        TaskStatus::Doing => "doing",
        TaskStatus::Done => "done",
    };

    println!("{}: {}", task.id, task.title.as_deref().unwrap_or(&task.node_ref));
    println!("Status: {} | Owner: {}", status_str, task.owner.as_deref().unwrap_or("(none)"));
    println!("---");

    if task.comments.is_empty() {
        println!("(no comments)");
    } else {
        for comment in &task.comments {
            println!("- {}", comment);
        }
    }

    Ok(())
}
