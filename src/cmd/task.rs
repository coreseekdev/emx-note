//! Task management command module
//!
//! Implements task tracking in TASK.md files with agent coordination.
//! Uses EditOp pattern for file modifications.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use chrono::Local;
use emx_note::{EditOp, apply_edits, extract_references, extract_frontmatter_prefix, get_reference_dest};
use emx_note::note_resolver;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Backlog,
    Doing,
    Done,
}

/// Task information
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: Option<String>,
    pub node_ref: String,
    pub status: TaskStatus,
    pub owner: Option<String>,
    pub comments: Vec<String>,
}

/// Read-only TASK.md file reader
pub struct TaskFileReader {
    content: String,
    prefix: String,
}

impl TaskFileReader {
    /// Load TASK.md file
    pub fn load(path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let prefix = extract_frontmatter_prefix(&content)
            .unwrap_or_else(|| "task-".to_string());
        Ok(TaskFileReader { content, prefix })
    }

    /// Get the task ID prefix
    pub fn get_prefix(&self) -> &str {
        &self.prefix
    }

    /// Get the raw content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get all reference definitions (task-id -> node_ref)
    pub fn get_references(&self) -> Vec<(String, String)> {
        extract_references(&self.content)
    }

    /// Get the next task ID
    pub fn next_task_id(&self) -> String {
        let refs = self.get_references();
        let mut max_num = 0;
        for (task_id, _) in &refs {
            if let Some(num_str) = task_id.strip_prefix(&self.prefix) {
                if let Ok(num) = num_str.parse::<u32>() {
                    max_num = max_num.max(num);
                }
            }
        }
        format!("{}{:02}", self.prefix, max_num + 1)
    }

    /// Find task ID by node reference
    pub fn find_by_node_ref(&self, node_ref: &str) -> Option<String> {
        self.get_references()
            .iter()
            .find(|(_, r)| r == node_ref)
            .map(|(id, _)| id.clone())
    }

    /// Get the node reference for a task ID
    pub fn get_node_ref(&self, task_id: &str) -> Option<String> {
        get_reference_dest(&self.content, task_id)
    }

    /// Find a task entry line in the body
    pub fn find_task_entry_line(&self, task_id: &str) -> Option<(usize, String)> {
        for (i, line) in self.content.lines().enumerate() {
            if line.contains(&format!("[{}]", task_id)) && line.trim().starts_with("- [") {
                return Some((i, line.to_string()));
            }
        }
        None
    }

    /// Get task details
    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        let node_ref = self.get_node_ref(task_id)?;

        // Check if task is in body
        let mut title: Option<String> = None;
        let mut owner: Option<String> = None;
        let mut is_done = false;
        let mut comments = Vec::new();
        let mut in_task = false;

        for line in self.content.lines() {
            let trimmed = line.trim();

            // Check for task entry: - [ ] [title][task-id] @owner
            if trimmed.starts_with("- [") {
                if line.contains(&format!("[{}]", task_id)) {
                    // Extract checkbox state: "- [x]" vs "- [ ]"
                    // trimmed[2] is '[', trimmed[3] is 'x' or ' '
                    if trimmed.len() > 3 {
                        is_done = trimmed.as_bytes()[3] == b'x';
                    }

                    // Extract title: after "] " and before "[task-id]"
                    // Format: - [ ] [title][task-id] @owner
                    if let Some(start) = trimmed.find("] [") {
                        let rest = &trimmed[start + 3..];
                        if let Some(end_title) = rest.find(&format!("][{}", task_id)) {
                            title = Some(rest[..end_title].to_string());
                        }

                        // Check for owner
                        if let Some(at_pos) = trimmed.find('@') {
                            owner = Some(trimmed[at_pos..].trim().to_string());
                        }
                    }
                    in_task = true;
                } else {
                    in_task = false;
                }
            } else if in_task && line.starts_with("  - ") {
                // Collect comments (indented list items - check original line, not trimmed)
                comments.push(trimmed[2..].to_string()); // Skip "- " from trimmed version
            } else if in_task && !trimmed.is_empty() && !line.starts_with("  ") {
                // End of task (new non-indented content)
                in_task = false;
            }
        }

        // Determine status
        let in_body = self.content.lines().any(|l| l.contains(&format!("[{}]", task_id)) && l.trim().starts_with("- ["));
        let status = if in_body {
            if is_done { TaskStatus::Done } else { TaskStatus::Doing }
        } else {
            TaskStatus::Backlog
        };

        Some(Task {
            id: task_id.to_string(),
            title,
            node_ref,
            status,
            owner,
            comments,
        })
    }

    /// Get all tasks
    pub fn all_tasks(&self) -> Vec<Task> {
        self.get_references()
            .iter()
            .filter_map(|(id, _)| self.get_task(id))
            .collect()
    }

    /// Find the line number for inserting a new section header + task entry
    /// Returns Ok((insert_at, needs_header)) where needs_header is true if header must be created
    /// Returns Err if header is specified but not found
    pub fn find_body_insert_point(&self, header: Option<&str>) -> Result<(usize, bool), String> {
        let lines: Vec<&str> = self.content.lines().collect();

        // Find the body section - it's after frontmatter (second ---) and before the reference separator (third ---)
        // Format:
        // ---          (line 0, frontmatter start)
        // PREFIX: ...  (line 1)
        // ---          (line 2, frontmatter end)
        //              (line 3, empty) <- Body content goes here
        // ---          (line 4, body separator)
        //              (line 5, empty)
        // [refs]       (line 6+)

        // Find the frontmatter end (second ---) to know where body starts
        let mut frontmatter_end = None;
        let mut body_separator = None;
        let mut dash_count = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "---" {
                dash_count += 1;
                if dash_count == 2 {
                    frontmatter_end = Some(i);
                } else if dash_count == 3 {
                    body_separator = Some(i);
                    break;
                }
            }
        }

        // Body starts after frontmatter end (skip blank line if present)
        let body_start = match frontmatter_end {
            Some(idx) => {
                let mut start = idx + 1;
                // Skip blank line after frontmatter
                while start < lines.len() && lines[start].trim().is_empty() {
                    start += 1;
                }
                start
            }
            None => 3, // Default: after frontmatter
        };

        if let Some(hdr) = header {
            let header_line = if hdr.starts_with("##") {
                hdr.to_string()
            } else {
                format!("## {}", hdr)
            };

            // Check if header already exists (search within body section)
            let search_end = body_separator.unwrap_or(lines.len());
            for i in body_start..search_end {
                if lines[i].trim() == header_line {
                    // Found header, insert after it
                    return Ok((i + 1, false));
                }
            }

            // Header doesn't exist - check if any headers exist
            let has_any_header = lines[body_start..search_end]
                .iter()
                .any(|l| l.trim().starts_with("##"));

            if has_any_header {
                // Other headers exist but not this one - error
                return Err(format!("Header '{}' not found", header_line));
            } else {
                // No headers exist at all - create the new header
                return Ok((body_start, true));
            }
        }

        // No header specified, insert at body_start
        Ok((body_start, false))
    }

    /// Find the line after the last reference definition
    pub fn find_reference_append_point(&self) -> usize {
        let lines: Vec<&str> = self.content.lines().collect();

        // Find the last reference line
        for (i, line) in lines.iter().enumerate().rev() {
            if line.trim().starts_with('[') && line.contains("]:") {
                return i + 1;
            }
        }

        // No references found, find the body separator (third ---)
        // Format: --- frontmatter --- \n --- \n references
        // The reference section starts after the body separator "---"
        let mut dash_count = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "---" {
                dash_count += 1;
                if dash_count == 3 {
                    // Skip blank lines after the separator
                    let mut result = i + 1;
                    while result < lines.len() && lines[result].trim().is_empty() {
                        result += 1;
                    }
                    return result;
                }
            }
        }

        lines.len()
    }
}

/// Get task file path
fn task_file_path(capsa_path: &Path) -> PathBuf {
    // Allow override via environment variable
    if let Ok(filename) = std::env::var("EMX_TASKFILE") {
        capsa_path.join(filename)
    } else {
        capsa_path.join("TASK.md")
    }
}

/// Get current timestamp for comments
fn get_timestamp() -> String {
    // Allow override via environment variable for reproducible tests
    if let Ok(ts) = std::env::var("EMX_TASK_TIMESTAMP") {
        ts
    } else {
        Local::now().format("%Y-%m-%d %H:%M").to_string()
    }
}

/// Get agent name from environment
fn get_agent_name() -> Option<String> {
    std::env::var("EMX_AGENT_NAME").ok()
        .filter(|s| !s.is_empty())
        .map(|s| format!("@{}", s))
}

/// Load or create TASK.md content
fn load_task_content(capsa_path: &Path) -> io::Result<String> {
    let path = task_file_path(capsa_path);

    if path.exists() {
        fs::read_to_string(&path)
    } else {
        // Return default empty TASK.md content
        // Format: frontmatter, blank line, body separator, blank line for references
        Ok(
r#"---
PREFIX: task-
---

---

"#.to_string())
    }
}

/// Save content to TASK.md
fn save_task_content(capsa_path: &Path, content: &str) -> io::Result<()> {
    let path = task_file_path(capsa_path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, content)
}

/// Main entry point
pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, cmd: emx_note::TaskCommand) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    match cmd {
        emx_note::TaskCommand::Add { node_ref } => cmd_add(&capsa_ref.path, &node_ref),
        emx_note::TaskCommand::Take { task_id, title, header, dry_run } => {
            cmd_take(&capsa_ref.path, &task_id, title.as_deref(), header.as_deref(), dry_run)
        }
        emx_note::TaskCommand::Comment { task_id, message, git, dry_run } => {
            cmd_comment(&capsa_ref.path, &task_id, &message, git.as_deref(), dry_run)
        }
        emx_note::TaskCommand::Release { task_ids, done, force, dry_run } => {
            cmd_release(&capsa_ref.path, &task_ids, done, force, dry_run)
        }
        emx_note::TaskCommand::List { status, oneline, owner } => {
            cmd_list(&capsa_ref.path, status.as_deref(), oneline, owner.as_deref())
        }
        emx_note::TaskCommand::Show { task_id } => cmd_show(&capsa_ref.path, &task_id),
        emx_note::TaskCommand::Log { task_id } => cmd_log(&capsa_ref.path, &task_id),
        emx_note::TaskCommand::Find { node_ref } => cmd_find(&capsa_ref.path, &node_ref),
    }
}

/// Add a new task
fn cmd_add(capsa_path: &Path, node_ref: &str) -> io::Result<()> {
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
PREFIX: task-
---

---

"#.to_string()
    };

    let reader = if path.exists() {
        TaskFileReader::load(&path)?
    } else {
        TaskFileReader {
            content: content.clone(),
            prefix: "task-".to_string(),
        }
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

/// Take ownership of a task
fn cmd_take(
    capsa_path: &Path,
    task_id: &str,
    title: Option<&str>,
    header: Option<&str>,
    dry_run: bool,
) -> io::Result<()> {
    let content = load_task_content(capsa_path)?;
    let reader = TaskFileReader {
        content: content.clone(),
        prefix: "task-".to_string(),
    };

    // Find task in references
    let task = reader.get_task(task_id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Task '{}' not found", task_id))
    })?;

    // Check if already taken
    if let Some(ref owner) = task.owner {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Task '{}' already taken by {}\nHint: Use 'task release {}' if you are {}, or wait for release",
                    task_id, owner, task_id, owner)
        ));
    }

    // Get agent name
    let agent_marker = get_agent_name();

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
        // Update existing entry (just add owner if needed)
        if !existing_line.contains('@') {
            if let Some(ref a) = agent_marker {
                let updated_line = format!("{} {}", existing_line.trim_end(), a);
                let edits = vec![EditOp::replace(&existing_line, &updated_line)];
                let new_content = apply_edits(&content, edits)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                save_task_content(capsa_path, &new_content)?;
            }
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

    if let Some(ref a) = agent_marker {
        eprintln!("Took {} as {}", task_id, a);
    } else {
        eprintln!("Took {}", task_id);
    }
    println!("{}", task_id);
    Ok(())
}

/// Add comment to task
fn cmd_comment(
    capsa_path: &Path,
    task_id: &str,
    message: &str,
    git: Option<&str>,
    dry_run: bool,
) -> io::Result<()> {
    let content = load_task_content(capsa_path)?;
    let reader = TaskFileReader {
        content: content.clone(),
        prefix: "task-".to_string(),
    };

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
        eprintln!("Added comment to {}", task_id);
    }

    Ok(())
}

/// Release task(s)
fn cmd_release(
    capsa_path: &Path,
    task_ids: &[String],
    done: bool,
    force: bool,
    dry_run: bool,
) -> io::Result<()> {
    // Check --force with multiple tasks
    if force && task_ids.len() > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--force only works with single task"
        ));
    }

    let content = load_task_content(capsa_path)?;
    let reader = TaskFileReader {
        content: content.clone(),
        prefix: "task-".to_string(),
    };

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

        // Check ownership unless --force
        if !force && task.owner.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Task '{}' has no owner to release", task_id)
            ));
        }

        // Find and update task entry
        let current_reader = TaskFileReader {
            content: current_content.clone(),
            prefix: "task-".to_string(),
        };

        if let Some((_, task_line)) = current_reader.find_task_entry_line(task_id) {
            // task_line is the unique source locator (contains unique task-id like [task-01])
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
            let edits = vec![EditOp::replace(task_line, &final_line)];
            current_content = apply_edits(&current_content, edits)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            released_count += 1;
        }
    }

    if released_count > 0 {
        save_task_content(capsa_path, &current_content)?;
    }

    for task_id in task_ids {
        eprintln!("Released {}", task_id);
    }
    Ok(())
}

/// List tasks
fn cmd_list(
    capsa_path: &Path,
    status: Option<&str>,
    oneline: bool,
    owner: Option<&str>,
) -> io::Result<()> {
    let path = task_file_path(capsa_path);

    if !path.exists() {
        return Ok(()); // No tasks file, empty result
    }

    let reader = TaskFileReader::load(&path)?;
    let tasks = reader.all_tasks();

    // Filter by status
    let filtered: Vec<&Task> = tasks.iter().filter(|t| {
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

/// Show task details
fn cmd_show(capsa_path: &Path, task_id: &str) -> io::Result<()> {
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

    println!("ID:       {}", task.id);
    println!("Title:    {}", task.title.as_deref().unwrap_or("-"));
    println!("Status:   {}", status_str);
    println!("Owner:    {}", task.owner.as_deref().unwrap_or("(none)"));
    println!("File:     {}", task.node_ref);
    println!("Comments: {}", task.comments.len());

    Ok(())
}

/// Show execution log
fn cmd_log(capsa_path: &Path, task_id: &str) -> io::Result<()> {
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

/// Find tasks by note reference
fn cmd_find(capsa_path: &Path, node_ref: &str) -> io::Result<()> {
    let path = task_file_path(capsa_path);

    if !path.exists() {
        eprintln!("No tasks found matching '{}'", node_ref);
        return Ok(());
    }

    let reader = TaskFileReader::load(&path)?;
    let tasks = reader.all_tasks();

    let matching: Vec<&Task> = tasks.iter()
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
