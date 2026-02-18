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
use crate::constants as C;

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
        let timestamp = now.format(C::DAILY_TIMESTAMP_FORMAT).to_string();

        // Generate filename
        let filename = if let Some(t) = title {
            format!("{}{}", util::slugify(t), C::MARKDOWN_EXTENSION)
        } else {
            format!("{}{}", timestamp, C::MARKDOWN_EXTENSION)
        };

        // Determine the directory path
        let note_dir = if let Some(src) = source {
            // With source: note/{hash}/
            let hash = util::abbreviate_hash(&util::hash_source(src));
            self.inner.path.join(C::NOTE_SUBDIR).join(&hash)
        } else {
            // Without source: note/
            self.inner.path.join(C::NOTE_SUBDIR)
        };

        // Create directory and note file
        fs::create_dir_all(&note_dir)?;
        let note_path = note_dir.join(&filename);

        // Write the file
        fs::write(&note_path, content)?;

        // If source is provided, create a .source file with the original source string
        if let Some(src) = source {
            let source_file = note_dir.join(C::SOURCE_FILENAME);
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
        let date_str = now.format(C::DAILY_DATE_FORMAT).to_string();
        let time_str = now.format(C::DAILY_TIME_FORMAT).to_string();
        let date_display = now.format(C::DAILY_DATE_DISPLAY_FORMAT).to_string();

        // Use provided title or default
        let title = title.unwrap_or(C::DEFAULT_DAILY_TITLE);

        // Create slug (empty if default title)
        let slug = if title == C::DEFAULT_DAILY_TITLE {
            String::new()
        } else {
            format!("-{}", util::slugify(title))
        };

        // Generate filename: HHmmSS[-title].md
        let filename = format!("{}{}{}", time_str, slug, C::MARKDOWN_EXTENSION);

        // Create daily subdirectory: #daily/YYYYMMDD/
        let daily_dir = self.inner.path.join(C::DAILY_SUBDIR).join(&date_str);
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
        let note_dir = self.inner.path.join(C::NOTE_SUBDIR);
        let daily_link_path = note_dir.join(C::DAILY_LINK_FILENAME);

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
                if name_str.starts_with(C::TAG_PREFIX) && name_str.ends_with(C::MARKDOWN_EXTENSION) {
                    // Extract tag name without # and .md
                    let tag_name = &name_str[1..name_str.len() - C::MARKDOWN_EXTENSION.len()];
                    tags.push(tag_name.to_string());
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
        self.capsa.path.join(format!("{}{}{}", C::TAG_PREFIX, self.name, C::MARKDOWN_EXTENSION))
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
        let date_display = now.format(C::DAILY_DATE_DISPLAY_FORMAT).to_string();

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
            .unwrap_or_else(|| C::UNTITLED_NOTE_TITLE.to_string()))
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
            self.capsa.path.join(C::TASK_FILENAME)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Clean all environment variables used by tests
    fn clean_env() {
        std::env::remove_var("EMX_TASK_TIMESTAMP");
        std::env::remove_var("EMX_TASKFILE");
        std::env::remove_var("EMX_AGENT_NAME");
    }

    // === CapsaEngine Tests ===

    #[test]
    fn test_create_permanent_note_with_title() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let result = engine.create_permanent_note(
            Some("Test Note"),
            None,
            "# Test Content\n\nThis is a test note."
        );

        assert!(result.is_ok());
        let note_path = result.unwrap();
        assert!(note_path.exists());
        assert!(note_path.to_string_lossy().contains("test-note"));
    }

    #[test]
    fn test_create_permanent_note_with_source() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let source = "https://example.com/article";
        let result = engine.create_permanent_note(
            Some("Article"),
            Some(source),
            "# Article Content"
        );

        assert!(result.is_ok());
        let note_path = result.unwrap();

        // Check note is in hash directory
        assert!(note_path.to_string_lossy().contains("note"));
        assert!(note_path.exists());

        // Check .source file exists
        let note_dir = note_path.parent().unwrap();
        let source_file = note_dir.join(".source");
        assert!(source_file.exists());
        let source_content = fs::read_to_string(&source_file).unwrap();
        assert_eq!(source_content, source);
    }

    #[test]
    fn test_create_permanent_note_without_title() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let result = engine.create_permanent_note(
            None,
            None,
            "# Content"
        );

        assert!(result.is_ok());
        let note_path = result.unwrap();
        // Should use timestamp as filename
        assert!(note_path.exists());
    }

    #[test]
    fn test_create_daily_note_with_title() {
        clean_env();

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        // Set fixed timestamp for testing
        std::env::set_var("EMX_TASK_TIMESTAMP", "2024-01-15 10:30");

        let result = engine.create_daily_note(
            Some("Meeting Notes"),
            "# Meeting\n\nAttendees: Alice, Bob"
        );

        std::env::remove_var("EMX_TASK_TIMESTAMP");

        assert!(result.is_ok());
        let note_path = result.unwrap();

        // Check path: #daily/20240115/103000-meeting-notes.md
        assert!(note_path.to_string_lossy().contains("#daily"));
        assert!(note_path.to_string_lossy().contains("20240115"));
        assert!(note_path.to_string_lossy().contains("meeting-notes"));
        assert!(note_path.exists());

        // Check daily link file was created
        let daily_link = temp_dir.path().join("note").join("#daily.md");
        assert!(daily_link.exists());
        let link_content = fs::read_to_string(&daily_link).unwrap();
        assert!(link_content.contains("Meeting Notes"));
        assert!(link_content.contains("#daily/"));
    }

    #[test]
    fn test_create_daily_note_without_title() {
        clean_env();

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        std::env::set_var("EMX_TASK_TIMESTAMP", "2024-01-15 10:30");

        let result = engine.create_daily_note(
            None,
            "# Daily Note Content"
        );

        std::env::remove_var("EMX_TASK_TIMESTAMP");

        assert!(result.is_ok());
        let note_path = result.unwrap();
        // Should not have title suffix
        assert!(!note_path.to_string_lossy().contains("-"));
        assert!(note_path.exists());
    }

    // === Tags Tests ===

    #[test]
    fn test_tags_get() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tags = engine.tags();
        let tag = tags.get("research");

        assert_eq!(tag.name, "research");
        assert_eq!(tag.file(), temp_dir.path().join("#research.md"));
    }

    #[test]
    fn test_tags_get_with_hash_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tags = engine.tags();
        let tag = tags.get("#research");

        // Should strip the # prefix
        assert_eq!(tag.name, "research");
        assert_eq!(tag.file(), temp_dir.path().join("#research.md"));
    }

    #[test]
    fn test_tags_list_empty() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tags = engine.tags().list().unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_tags_list_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create tag files
        fs::write(temp_dir.path().join("#research.md"), "# research\n").unwrap();
        fs::write(temp_dir.path().join("#ideas.md"), "# ideas\n").unwrap();
        fs::write(temp_dir.path().join("regular.md"), "# regular\n").unwrap();

        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tags = engine.tags().list().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"ideas".to_string()));
        assert!(tags.contains(&"research".to_string()));
    }

    // === Tag Tests ===

    #[test]
    fn test_tag_add_note() {
        let temp_dir = TempDir::new().unwrap();

        // Create a note in the capsa
        let note_path = temp_dir.path().join("note.md");
        fs::write(&note_path, "# Test Note\n\nContent").unwrap();

        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tag = engine.tags().get("test-tag");
        let result = tag.add_note(&note_path);

        assert!(result.is_ok());

        // Check tag file was created
        let tag_file = temp_dir.path().join("#test-tag.md");
        assert!(tag_file.exists());

        let content = fs::read_to_string(&tag_file).unwrap();
        assert!(content.contains("# test-tag"));
        assert!(content.contains("Test Note"));
        assert!(content.contains("note.md"));
    }

    #[test]
    fn test_tag_add_duplicate_note() {
        let temp_dir = TempDir::new().unwrap();

        let note_path = temp_dir.path().join("note.md");
        fs::write(&note_path, "# Test Note").unwrap();

        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tag = engine.tags().get("test");

        // Add note twice
        tag.add_note(&note_path).unwrap();
        tag.add_note(&note_path).unwrap();

        // Should only appear once
        let tag_file = temp_dir.path().join("#test.md");
        let content = fs::read_to_string(&tag_file).unwrap();
        let count = content.matches("Test Note").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_tag_remove_note() {
        let temp_dir = TempDir::new().unwrap();

        let note_path = temp_dir.path().join("note.md");
        fs::write(&note_path, "# Test Note").unwrap();

        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tag = engine.tags().get("test");
        tag.add_note(&note_path).unwrap();

        // Verify note was added
        let tag_file = temp_dir.path().join("#test.md");
        assert!(tag_file.exists());

        // Remove the note
        tag.remove_note("note.md").unwrap();

        // Verify note was removed from tag file
        if tag_file.exists() {
            let content = fs::read_to_string(&tag_file).unwrap();
            // Should not contain the note link anymore
            assert!(!content.contains("Test Note"));
            assert!(!content.contains("note.md"));
        }
    }

    #[test]
    fn test_tag_list_notes() {
        let temp_dir = TempDir::new().unwrap();

        // Create notes
        let note1 = temp_dir.path().join("note1.md");
        let note2 = temp_dir.path().join("note2.md");
        fs::write(&note1, "# Note 1").unwrap();
        fs::write(&note2, "# Note 2").unwrap();

        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let tag = engine.tags().get("test");
        tag.add_note(&note1).unwrap();
        tag.add_note(&note2).unwrap();

        let notes = tag.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }

    // === TaskFile Tests ===

    #[test]
    fn test_task_file_path() {
        clean_env();

        // Verify environment is clean
        assert!(std::env::var("EMX_TASKFILE").is_err());

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();
        assert_eq!(task_file.file(), temp_dir.path().join("TASK.md"));
    }

    #[test]
    fn test_task_file_path_with_env_override() {
        clean_env();
        std::env::set_var("EMX_TASKFILE", "CUSTOM.md");

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();
        assert_eq!(task_file.file(), temp_dir.path().join("CUSTOM.md"));

        std::env::remove_var("EMX_TASKFILE");
    }

    #[test]
    fn test_task_file_load_default() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();
        let content = task_file.load().unwrap();

        assert!(content.contains("---"));
        assert!(content.contains("PREFIX: TASK-"));
    }

    #[test]
    fn test_task_file_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();

        let content = r#"---
PREFIX: CUSTOM-
---

---

"#;
        task_file.save(content).unwrap();

        let loaded = task_file.load().unwrap();
        assert!(loaded.contains("PREFIX: CUSTOM-"));
    }

    #[test]
    fn test_task_file_get_timestamp() {
        clean_env();

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();
        let timestamp = task_file.get_timestamp();

        // Should be in format "YYYY-MM-DD HH:MM"
        assert!(timestamp.len() == 16);
        assert!(timestamp.contains("-"));
        assert!(timestamp.contains(":"));
        assert!(timestamp.contains(" "));
    }

    #[test]
    fn test_task_file_get_timestamp_with_override() {
        clean_env();

        std::env::set_var("EMX_TASK_TIMESTAMP", "2024-01-15 10:30");

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();
        let timestamp = task_file.get_timestamp();

        assert_eq!(timestamp, "2024-01-15 10:30");

        std::env::remove_var("EMX_TASK_TIMESTAMP");
    }

    #[test]
    fn test_task_file_get_agent_name() {
        clean_env();

        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        let task_file = engine.task_file();

        // No agent set
        let agent = task_file.get_agent_name();
        assert!(agent.is_none());

        // With agent set
        std::env::set_var("EMX_AGENT_NAME", "test-agent");
        let agent = task_file.get_agent_name();
        assert_eq!(agent.unwrap(), "@test-agent");
        std::env::remove_var("EMX_AGENT_NAME");
    }

    // === Deref Tests ===

    #[test]
    fn test_capsa_engine_deref() {
        let temp_dir = TempDir::new().unwrap();
        let capsa_ref = CapsaRef {
            name: "test".to_string(),
            path: temp_dir.path().to_path_buf(),
            is_link: false,
            is_default: false,
        };
        let engine = CapsaEngine::new(capsa_ref);

        // Can access CapsaRef fields through Deref
        assert_eq!(&engine.name, &"test");
        assert_eq!(&engine.path, temp_dir.path());
    }
}
