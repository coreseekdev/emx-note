//! Task add command

use std::fs;
use std::io;
use std::path::Path;
use emx_note::{EditOp, apply_edits};
use emx_note::note_resolver;
use super::{TaskFileReader, task_file_path, save_task_content};

/// Add a new task
pub fn run(capsa_path: &Path, node_ref: &str) -> io::Result<()> {
    // Validate that node_ref resolves to an existing note
    let extensions = ["md", "txt"];
    note_resolver::resolve_note_or_error(capsa_path, node_ref, &extensions)?;

    let path = task_file_path(capsa_path);

    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        // Default empty TASK.md content
        // Format: frontmatter, blank line, body separator, blank line for references
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

    save_task_content(capsa_path, &new_content)?;

    println!("{}", task_id);
    Ok(())
}
