//! Task release command

use std::io;
use std::path::Path;
use emx_note::{EditOp, apply_edits};
use super::{TaskFileReader, load_task_content, save_task_content, get_agent_name, get_timestamp};
use super::log;

/// Release task(s)
pub fn run(
    capsa_path: &Path,
    task_ids: &[String],
    done: bool,
    force: bool,
    dry_run: bool,
) -> io::Result<()> {
    // If no agent name set and not marking done, behave like log for single task
    let agent_marker = get_agent_name();
    if agent_marker.is_none() && !done && task_ids.len() == 1 {
        return log::run(capsa_path, &task_ids[0]);
    }

    // Check --force with multiple tasks
    if force && task_ids.len() > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--force only works with single task"
        ));
    }

    let content = load_task_content(capsa_path)?;
    let reader = TaskFileReader::new(content.clone());

    if dry_run {
        println!("--- TASK.md changes ---");
        for task_id in task_ids {
            let task = reader.get_task(task_id).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("Task '{}' not found", task_id))
            })?;

            let new_checkbox = if done { "[x]" } else { "[ ]" };
            let old_owner = task.owner.as_deref().unwrap_or("(none)");
            println!("{}: [ ] -> {}, {} -> (none)", task_id, new_checkbox, old_owner);
        }
        println!("---");
        println!("Would release {} task(s)", task_ids.len());
        return Ok(());
    }

    let mut current_content = content;
    let mut released_count = 0;

    for task_id in task_ids {
        let task = reader.get_task(task_id).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("Task '{}' not found", task_id))
        })?;

        // Skip if no owner (idempotent - already released) unless --force or --done
        // With --done, anyone can mark as done (public owned)
        if !force && !done && task.owner.is_none() {
            continue;
        }

        // Find and update task entry
        let current_reader = TaskFileReader::new(current_content.clone());

        if let Some((line_num, task_line)) = current_reader.find_task_entry_line(task_id) {
            // Task is in body section - update existing entry
            // task_line is the unique source locator (contains unique task-id like [TASK-01])
            // Step 1: Remove owner from the end
            let updated = if let Some(at_pos) = task_line.find('@') {
                task_line[..at_pos].trim_end().to_string()
            } else {
                task_line.clone()
            };

            // Step 2: Update checkbox based on --done flag
            // The checkbox format is always "- [x]" or "- [ ]" at the start
            // We need to replace the exact checkbox pattern, not just any [x] or [ ]
            let final_line = if done {
                // Set to done: change "- [ ]" to "- [x]"
                // Use regex or precise string manipulation
                if updated.starts_with("- [ ]") {
                    format!("- [x]{}", &updated[5..])
                } else {
                    updated // Already [x] or different format
                }
            } else {
                // Reset to not done: change "- [x]" to "- [ ]"
                if updated.starts_with("- [x]") {
                    format!("- [ ]{}", &updated[5..])
                } else {
                    updated // Already [ ] or different format
                }
            };

            // Step 3: Use EditOp::replace with the unique source (task_line)
            let mut edits = vec![EditOp::replace(task_line, &final_line)];

            // Add completion comment if --done and no agent
            if done && agent_marker.is_none() {
                let timestamp = get_timestamp();
                let comment = format!("  - {} Completed by @anonymous", timestamp);
                // Find insert point after the task line
                let lines: Vec<&str> = current_content.lines().collect();
                let mut insert_at = line_num + 1;
                for (i, line) in lines.iter().enumerate().skip(line_num + 1) {
                    if line.trim().starts_with("  - ") {
                        insert_at = i + 1;
                    } else if line.trim().starts_with("- [") || (!line.trim().is_empty() && !line.trim().starts_with("  ")) {
                        break;
                    }
                }
                edits.push(EditOp::insert_at_line(insert_at, comment));
            }

            current_content = apply_edits(&current_content, edits)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            released_count += 1;
        } else if done {
            // Task is in backlog - create new entry in body with [x] checkbox
            let task_title = task.title.as_ref().unwrap_or(&task.node_ref);
            let task_entry = format!("- [x] [{}][{}]", task_title, task_id);

            let (insert_point, needs_header) = current_reader.find_body_insert_point(None)
                .unwrap_or((0, false));

            let mut edits = Vec::new();
            let lines_inserted = if needs_header {
                let header_text = "## Done\n\n".to_string();
                edits.push(EditOp::insert_at_line(insert_point, format!("{}{}", header_text, task_entry)));
                3
            } else {
                edits.push(EditOp::insert_at_line(insert_point, task_entry.clone()));
                1
            };

            // Add blank line after task entry
            edits.push(EditOp::insert_at_line(insert_point + lines_inserted, String::new()));

            // Add completion comment if no agent
            if agent_marker.is_none() {
                let timestamp = get_timestamp();
                let comment = format!("  - {} Completed by @anonymous", timestamp);
                edits.push(EditOp::insert_at_line(insert_point + lines_inserted + 1, comment));
            }

            current_content = apply_edits(&current_content, edits)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            released_count += 1;
        }
    }

    if released_count > 0 {
        save_task_content(capsa_path, &current_content)?;
    }

    // Show log after marking done for single task
    if done && task_ids.len() == 1 {
        return log::run(capsa_path, &task_ids[0]);
    }

    for task_id in task_ids {
        println!("{}", task_id);
    }
    Ok(())
}
