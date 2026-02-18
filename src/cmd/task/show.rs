//! Task show command

use std::io;
use emx_note::CapsaEngine;
use super::{TaskFileReader, TaskStatus};

/// Show task details
pub fn run(capsa: &CapsaEngine, task_id: &str) -> io::Result<()> {
    let task_file = capsa.task_file();
    let path = task_file.file();

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

    println!("ID:       {}", task.id);
    println!("Title:    {}", task.title.as_deref().unwrap_or("-"));
    println!("Status:   {}", status_str);
    println!("Owner:    {}", task.owner.as_deref().unwrap_or("(none)"));
    println!("File:     {}", task.node_ref);
    println!("Comments: {}", task.comments.len());

    Ok(())
}
