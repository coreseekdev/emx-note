//! Task file reader module
//!
//! Provides read-only access to TASK.md files.

use std::fs;
use std::io;
use std::path::Path;
use emx_note::{extract_references, extract_frontmatter_prefix, get_reference_dest};

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
            .unwrap_or_else(|| "TASK-".to_string());
        Ok(TaskFileReader { content, prefix })
    }

    /// Create reader with given content and default prefix
    pub fn new(content: String) -> Self {
        TaskFileReader {
            content,
            prefix: "TASK-".to_string(),
        }
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
