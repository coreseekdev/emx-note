//! Task list command

use std::io;
use emx_note::CapsaEngine;
use super::{TaskFileReader, TaskStatus};

/// List tasks
pub fn run(
    capsa: &CapsaEngine,
    status: Option<&str>,
    oneline: bool,
    owner: Option<&str>,
) -> io::Result<()> {
    let task_file = capsa.task_file();
    let path = task_file.file();

    if !path.exists() {
        return Ok(()); // No tasks file, empty result
    }

    let reader = TaskFileReader::load(&path)?;
    let tasks = reader.all_tasks();

    // Filter by status
    let filtered: Vec<_> = tasks.iter().filter(|t| {
        let status_match = match status {
            None | Some("all") => true,
            Some("backlog") => t.status == TaskStatus::Backlog,
            Some("doing") => t.status == TaskStatus::Doing,
            Some("done") => t.status == TaskStatus::Done,
            Some(s) => {
                eprintln!("Warning: Unknown status filter '{}', showing all", s);
                true
            }
        };

        let owner_match = match owner {
            None => true,
            Some("(none)") => t.owner.is_none(),
            Some(o) => t.owner.as_deref() == Some(o),
        };

        status_match && owner_match
    }).collect();

    if oneline {
        for task in filtered {
            println!("{}", task.id);
        }
    } else {
        // Print table header
        println!("{:<10} {:<24} {:<22} {:<10} {}",
                 "ID", "TITLE", "FILE", "STATUS", "OWNER");

        for task in filtered {
            let title = task.title.as_deref().unwrap_or("-");
            let owner = task.owner.as_deref().unwrap_or("(none)");
            let status_str = match task.status {
                TaskStatus::Backlog => "backlog",
                TaskStatus::Doing => "doing",
                TaskStatus::Done => "done",
            };
            println!("{:<10} {:<24} {:<22} {:<10} {}",
                     task.id,
                     title.chars().take(24).collect::<String>(),
                     task.node_ref.chars().take(22).collect::<String>(),
                     status_str,
                     owner);
        }
    }

    Ok(())
}
