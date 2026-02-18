//! Task add command

use std::io;
use emx_note::{EditOp, apply_edits, note_resolver};
use emx_note::CapsaEngine;
use super::TaskFileReader;

/// Add a new task
pub fn run(capsa: &CapsaEngine, node_ref: &str) -> io::Result<()> {
    // Validate that node_ref resolves to an existing note
    let extensions = ["md", "txt"];
    note_resolver::resolve_note_or_error(&capsa.path, node_ref, &extensions)?;

    let task_file = capsa.task_file();
    let path = task_file.file();

    let content = if path.exists() {
        task_file.load()?
    } else {
        // Default empty TASK.md content
        r#"---
PREFIX: TASK-
---

---

"#.to_string()
    };

    let reader = if path.exists() {
        TaskFileReader::load(&path)?
    } else {
        TaskFileReader::new(content.clone())
    };

    // Check if node_ref already exists
    if let Some(existing_id) = reader.find_by_node_ref(node_ref) {
        println!("{}", existing_id);
        return Ok(());
    }

    // Generate new task ID
    let task_id = reader.next_task_id();

    // Add reference at the end of the file (after the last reference separator)
    let new_ref = format!("[{}]: {}", task_id, node_ref);
    let append_point = reader.find_reference_append_point();

    let edits = vec![
        EditOp::insert_at_line(append_point, new_ref),
    ];

    let new_content = apply_edits(&content, edits)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    task_file.save(&new_content)?;

    println!("{}", task_id);
    Ok(())
}
