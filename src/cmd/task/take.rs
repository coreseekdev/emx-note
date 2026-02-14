//! Task take command

use std::io;
use std::path::Path;
use emx_note::{EditOp, apply_edits};
use super::{TaskFileReader, TaskStatus, load_task_content, save_task_content, get_agent_name};

/// Take ownership of a task
pub fn run(
    capsa_path: &Path,
    task_id: &str,
    title: Option<&str>,
    header: Option<&str>,
    dry_run: bool,
) -> io::Result<()> {
    let content = load_task_content(capsa_path)?;
    let reader = TaskFileReader::new(content.clone());

    // Find task in references
    let task = reader.get_task(task_id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Task '{}' not found", task_id))
    })?;

    // Get agent name first (needed for ownership check)
    let agent_marker = get_agent_name();

    // Check if already taken (only when agent name is set)
    if agent_marker.is_some() {
        if let Some(ref owner) = task.owner {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Task '{}' already taken by {}\nHint: Use 'task release {}' if you are {}, or wait for release",
                        task_id, owner, task_id, owner)
            ));
        }
    }

    // Determine title
    let task_title = title.map(|s| s.to_string())
        .or(task.title)
        .unwrap_or_else(|| task.node_ref.clone());

    // Create task entry
    let checkbox = if task.status == TaskStatus::Done { "[x]" } else { "[ ]" };
    let owner_str = agent_marker.as_ref()
        .map(|a| format!(" {}", a))
        .unwrap_or_default();
    let task_entry = format!("- {} [{}][{}]{}", checkbox, task_title, task_id, owner_str);

    if dry_run {
        println!("--- TASK.md (new entry) ---");
        println!("{}", task_entry);
        println!("---");
        if let Some(ref h) = header {
            println!("Would insert under header: {}", h);
        } else {
            println!("Would insert before first header or reference section");
        }
        return Ok(());
    }

    // Check if task already in body
    if let Some((_line_num, existing_line)) = reader.find_task_entry_line(task_id) {
        // Update existing entry based on agent_marker
        let updated_line = if let Some(at_pos) = existing_line.find('@') {
            // Has owner marker
            if let Some(ref a) = agent_marker {
                // Replace with new owner
                format!("{} {}", existing_line[..at_pos].trim_end(), a)
            } else {
                // Remove owner
                existing_line[..at_pos].trim_end().to_string()
            }
        } else {
            // No owner marker
            if let Some(ref a) = agent_marker {
                // Add owner
                format!("{} {}", existing_line.trim_end(), a)
            } else {
                // No change needed
                existing_line.clone()
            }
        };

        if updated_line != existing_line {
            let edits = vec![EditOp::replace(&existing_line, &updated_line)];
            let new_content = apply_edits(&content, edits)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            save_task_content(capsa_path, &new_content)?;
        }
    } else {
        // Insert new task entry
        let (insert_point, needs_header) = reader.find_body_insert_point(header)
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;

        let mut edits = Vec::new();

        let lines_inserted = if needs_header {
            let header_text = match header {
                Some(h) if h.starts_with("##") => format!("{}\n\n", h),
                Some(h) => format!("## {}\n\n", h),
                None => String::new(),
            };
            // Insert header + blank line + task entry
            edits.push(EditOp::insert_at_line(insert_point, format!("{}{}", header_text, task_entry)));
            3
        } else {
            edits.push(EditOp::insert_at_line(insert_point, task_entry.clone()));
            1
        };

        // Add blank line after task entry
        edits.push(EditOp::insert_at_line(insert_point + lines_inserted, String::new()));

        let new_content = apply_edits(&content, edits)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        save_task_content(capsa_path, &new_content)?;
    }

    println!("{}", task_id);
    Ok(())
}
