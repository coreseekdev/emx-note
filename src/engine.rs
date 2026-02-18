//! CapsaEngine - High-level capsa operations
//!
//! Provides an object-oriented interface for capsa operations including:
//! - Note creation (permanent and daily)
//! - Tag management
//! - Task file operations

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::ops::Deref;
use chrono::{Local, DateTime, NaiveDateTime, TimeZone};

use crate::{CapsaRef, EditOp, apply_edits, DEFAULT_EXTENSIONS};
use crate::util;
use crate::note_resolver;

// === CapsaEngine ===

/// Core engine for capsa operations
pub struct CapsaEngine {
    inner: CapsaRef,
}

impl CapsaEngine {
    /// Create a new CapsaEngine
    pub fn new(ref_: CapsaRef) -> Self {
        Self { inner: ref_ }
    }

    // === Note Operations ===

    /// Create a permanent note
    pub fn create_permanent_note(
        &self,
        title: Option<&str>,
        source: Option<&str>,
        content: &str,
    ) -> io::Result<PathBuf> {
        let now = Local::now();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();

        // Generate filename
        let filename = if let Some(t) = title {
            format!("{}.md", util::slugify(t))
        } else {
            format!("{}.md", timestamp)
        };

        // Determine the directory path
        let note_dir = if let Some(src) = source {
            // With source: note/{hash}/
            let hash = util::abbreviate_hash(&util::hash_source(src));
            self.inner.path.join("note").join(&hash)
        } else {
            // Without source: note/
            self.inner.path.join("note")
        };

        // Create directory and note file
        fs::create_dir_all(&note_dir)?;
        let note_path = note_dir.join(&filename);

        // Write the file
        fs::write(&note_path, content)?;

        // If source is provided, create a .source file with the original source string
        if let Some(src) = source {
            let source_file = note_dir.join(".source");
            fs::write(&source_file, src)?;
        }

        Ok(note_path)
    }

    /// Create a daily note
    pub fn create_daily_note(
        &self,
        title: Option<&str>,
        content: &str,
    ) -> io::Result<PathBuf> {
        let now = Self::get_timestamp();
        let date_str = now.format("%Y%m%d").to_string();
        let time_str = now.format("%H%M%S").to_string();
        let date_display = now.format("%Y-%m-%d").to_string();

        // Use provided title or default
        let title = title.unwrap_or("Daily Note");

        // Create slug (empty if default title)
        let slug = if title == "Daily Note" {
            String::new()
        } else {
            format!("-{}", util::slugify(title))
        };

        // Generate filename: HHmmSS[-title].md
        let filename = format!("{}{}.md", time_str, slug);

        // Create daily subdirectory: #daily/YYYYMMDD/
        let daily_dir = self.inner.path.join("#daily").join(&date_str);
        fs::create_dir_all(&daily_dir)?;

        // Create note file
        let note_path = daily_dir.join(&filename);

        // Write the file
        fs::write(&note_path, content)?;

        // Update daily link file (note/#daily.md)
        self.update_daily_link(&date_str, &date_display, &filename, title)?;

        Ok(note_path)
    }

    /// Get current timestamp, allowing override via EMX_TASK_TIMESTAMP for testing
    fn get_timestamp() -> DateTime<Local> {
        if let Ok(ts) = std::env::var("EMX_TASK_TIMESTAMP") {
            // Parse "YYYY-MM-DD HH:MM" format
            if let Ok(naive) = NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M") {
                // Use from_local_datetime to treat the input as local time, not UTC
                return Local.from_local_datetime(&naive).single().unwrap_or_else(|| Local::now());
            }
        }
        Local::now()
    }

    /// Update the daily link file (note/#daily.md) with new note link
    fn update_daily_link(
        &self,
        date_str: &str,
        _date_display: &str,
        filename: &str,
        title: &str,
    ) -> io::Result<()> {
        let note_dir = self.inner.path.join("note");
        let daily_link_path = note_dir.join("#daily.md");

        // Ensure note/ directory exists
        fs::create_dir_all(&note_dir)?;

        // The link line (unique content to append)
        let link_line = format!("- [{}](#daily/{}/{})", title, date_str, filename);

        if daily_link_path.exists() {
            // Append to existing file using EditOp
            let content = fs::read_to_string(&daily_link_path)?;
            let edits = vec![EditOp::append(&link_line)];
            let new_content = apply_edits(&content, edits)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            fs::write(&daily_link_path, new_content)?;
        } else {
            // Create new file with title and link
            let content = format!("# Daily Notes\n\n{}", link_line);
            fs::write(&daily_link_path, content)?;
        }

        Ok(())
    }

    /// Resolve a note reference
    pub fn resolve_note(&self, note_ref: &str, force: bool) -> io::Result<Vec<PathBuf>> {
        note_resolver::resolve_note_with_force(
            &self.inner.path,
            note_ref,
            DEFAULT_EXTENSIONS,
            force,
            "resolve note"
        )
    }

    // === Tags Collection ===

    /// Get the Tags collection for this capsa
    pub fn tags(&self) -> Tags<'_> {
        Tags { capsa: &self.inner }
    }

    // === Task File ===

    /// Get the TaskFile for this capsa
    pub fn task_file(&self) -> TaskFile<'_> {
        TaskFile { capsa: &self.inner }
    }
}

impl Deref for CapsaEngine {
    type Target = CapsaRef;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// === Tags ===

/// Tag collection for a capsa
pub struct Tags<'a> {
    capsa: &'a CapsaRef,
}

impl<'a> Tags<'a> {
    /// Get a specific tag
    pub fn get(&self, tag: &str) -> Tag<'a> {
        Tag {
            capsa: self.capsa,
            name: tag.trim_start_matches('#').to_string(),
        }
    }

    /// List all tags (scan for #*.md files)
    pub fn list(&self) -> io::Result<Vec<String>> {
        let mut tags = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.capsa.path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Check for #*.md pattern
                if name_str.starts_with('#') && name_str.ends_with(".md") {
                    // Extract tag name without # and .md
                    let tag_name = name_str[1..name_str.len() - 3].to_string();
                    tags.push(tag_name);
                }
            }
        }

        tags.sort();
        Ok(tags)
    }
}

// === Tag ===

/// Single tag operations
pub struct Tag<'a> {
    capsa: &'a CapsaRef,
    name: String,
}

impl<'a> Tag<'a> {
    /// Get the tag file path
    pub fn file(&self) -> PathBuf {
        self.capsa.path.join(format!("#{}.md", self.name))
    }

    /// Add a note to this tag
    pub fn add_note(&self, note_path: &PathBuf) -> io::Result<()> {
        let relative = note_path.strip_prefix(&self.capsa.path)
            .unwrap_or(note_path)
            .to_string_lossy()
            .replace('\\', "/");

        self.add_note_internal(&relative, note_path)
    }

    /// Internal add note logic
    fn add_note_internal(&self, note_relative: &str, note_path: &PathBuf) -> io::Result<()> {
        let tag_file = self.file();
        let now = Local::now();
        let date_display = now.format("%Y-%m-%d").to_string();

        // Get note title
        let note_title = Self::extract_note_title(note_path)?;

        // The link line is our unique source locator for checking duplicates
        let link_line = format!("- [{}]({})", note_title, note_relative);

        if tag_file.exists() {
            // Check if note already tagged
            let content = fs::read_to_string(&tag_file)?;
            if content.lines().any(|line| line.contains(&format!("]({})", note_relative))) {
                return Ok(()); // Already tagged, silently skip
            }

            // Build edits to add date header (if needed) and link
            let date_header = format!("## {}", date_display);
            let has_date_header = content.lines().any(|line| line == date_header);

            let mut edits = Vec::new();

            if !has_date_header {
                // Need to add date header first
                edits.push(EditOp::append(format!("\n{}", date_header)));
            }

            // Add the link line (unique source locator is the link content)
            edits.push(EditOp::append(link_line));

            let new_content = apply_edits(&content, edits)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            fs::write(&tag_file, new_content)?;
        } else {
            // Create new tag file with heading, date header, and link
            let content = format!("# {}\n\n{}\n{}", self.name, date_display, link_line);
            fs::write(&tag_file, content)?;
        }

        // Output tag file path
        println!("{}", util::display_path(&tag_file));

        Ok(())
    }

    /// Remove a note from this tag
    pub fn remove_note(&self, note_relative: &str) -> io::Result<()> {
        let tag_file = self.file();

        if !tag_file.exists() {
            // Tag doesn't exist, silently succeed (idempotent)
            return Ok(());
        }

        let content = fs::read_to_string(&tag_file)?;

        // Step 1: Find the exact line to delete (unique source locator)
        // The line contains ](note_relative) which should be unique
        let line_to_delete = content.lines()
            .find(|line| line.contains(&format!("]({})", note_relative)));

        let link_line = match line_to_delete {
            Some(line) => line.to_string(),
            None => return Ok(()), // Note wasn't in this tag, silently succeed
        };

        // Step 2: Find the date header for this link (to check if section becomes empty)
        let mut date_to_delete: Option<String> = None;
        let mut current_date_header: Option<String> = None;
        let mut links_count = 0;

        for line in content.lines() {
            if line.starts_with("## ") {
                // Check previous section
                if links_count == 0 && current_date_header.is_some() {
                    // Previous section was already empty (shouldn't happen but handle it)
                }
                current_date_header = Some(line.to_string());
                links_count = 0;
            } else if line.starts_with("- [") {
                if line == link_line {
                    // This is the line we're deleting, don't count it
                } else {
                    links_count += 1;
                }
            }
        }

        // After the loop, check if current section becomes empty
        if links_count == 0 {
            date_to_delete = current_date_header;
        }

        // Step 3: Build edits - collect all lines to delete
        let mut edits = Vec::new();

        // Delete the link line (unique source locator)
        edits.push(EditOp::delete_line(&link_line));

        // Add date header deletion if needed
        if let Some(ref date_line) = date_to_delete {
            edits.push(EditOp::delete_line(date_line));
        }

        // Step 4: Apply edits
        let new_content = apply_edits(&content, edits)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Step 5: Check if file should be deleted (only main heading left)
        let non_empty: Vec<&str> = new_content.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        if non_empty.is_empty() || (non_empty.len() == 1 && non_empty[0].starts_with('#')) {
            fs::remove_file(&tag_file)?;
        } else {
            fs::write(&tag_file, &new_content)?;
        }

        // Output tag file path
        println!("{}", util::display_path(&tag_file));

        Ok(())
    }

    /// List notes in this tag
    pub fn list_notes(&self) -> io::Result<Vec<PathBuf>> {
        let tag_file = self.file();

        if !tag_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&tag_file)?;
        let mut notes = Vec::new();

        for line in content.lines() {
            if line.starts_with("- [") {
                // Extract link target: - [title](path)
                if let Some(start) = line.find("](") {
                    if let Some(end) = line[start + 2..].find(')') {
                        let relative = &line[start + 2..start + 2 + end];
                        let full_path = self.capsa.path.join(relative);
                        notes.push(full_path);
                    }
                }
            }
        }

        Ok(notes)
    }

    /// Extract title from note (first H1 heading or filename)
    fn extract_note_title(note_path: &Path) -> io::Result<String> {
        let content = fs::read_to_string(note_path)?;

        // Look for first # heading
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return Ok(trimmed.trim_start_matches("# ").to_string());
            }
        }

        // Fallback to filename
        Ok(note_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string()))
    }
}

// === TaskFile ===

/// Task file operations
pub struct TaskFile<'a> {
    capsa: &'a CapsaRef,
}

impl<'a> TaskFile<'a> {
    /// Get the task file path
    pub fn file(&self) -> PathBuf {
        // Allow override via environment variable
        if let Ok(filename) = std::env::var("EMX_TASKFILE") {
            self.capsa.path.join(filename)
        } else {
            self.capsa.path.join("TASK.md")
        }
    }

    /// Get current timestamp for comments
    pub fn get_timestamp(&self) -> String {
        // Allow override via environment variable for reproducible tests
        if let Ok(ts) = std::env::var("EMX_TASK_TIMESTAMP") {
            ts
        } else {
            Local::now().format("%Y-%m-%d %H:%M").to_string()
        }
    }

    /// Get agent name from environment
    pub fn get_agent_name(&self) -> Option<String> {
        std::env::var("EMX_AGENT_NAME").ok()
            .filter(|s| !s.is_empty())
            .map(|s| format!("@{}", s))
    }

    /// Load or create TASK.md content
    pub fn load(&self) -> io::Result<String> {
        let path = self.file();

        if path.exists() {
            fs::read_to_string(&path)
        } else {
            // Return default empty TASK.md content
            // Format: frontmatter, blank line, body separator, blank line for references
            Ok(
r#"---
PREFIX: TASK-
---

---

"#.to_string())
        }
    }

    /// Save content to TASK.md
    pub fn save(&self, content: &str) -> io::Result<()> {
        let path = self.file();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, content)
    }
}
