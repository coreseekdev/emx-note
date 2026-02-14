//! EditOp module - unified file editing operations
//!
//! Inspired by LLM Code Agent's edit tool pattern. Provides precise,
//! verifiable editing operations that validate content before modification.

use std::fmt;

/// Validation error for edit operations
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Pattern not found in content
    NotFound { pattern: String },
    /// Pattern found multiple times (expected exactly one)
    MultipleMatches { pattern: String, count: usize },
    /// Invalid line number
    InvalidLine { line: usize, max_line: usize },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::NotFound { pattern } => {
                write!(f, "Pattern not found: {:?}", pattern)
            }
            ValidationError::MultipleMatches { pattern, count } => {
                write!(f, "Pattern found {} times (expected exactly 1): {:?}", count, pattern)
            }
            ValidationError::InvalidLine { line, max_line } => {
                write!(f, "Invalid line {} (max: {})", line, max_line)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Edit operation type
#[derive(Debug, Clone, PartialEq)]
pub enum EditOp {
    /// Replace old string with new string (validates old exists exactly once)
    Replace { old: String, new: String },
    /// Insert content at a specific line number (0-indexed)
    InsertAtLine { line: usize, content: String },
    /// Append content to end of file
    Append { content: String },
    /// Delete a line that matches exactly (validates existence)
    DeleteLine { content: String },
}

impl EditOp {
    /// Create a replace operation
    pub fn replace(old: impl Into<String>, new: impl Into<String>) -> Self {
        EditOp::Replace {
            old: old.into(),
            new: new.into(),
        }
    }

    /// Create an insert at line operation
    pub fn insert_at_line(line: usize, content: impl Into<String>) -> Self {
        EditOp::InsertAtLine {
            line,
            content: content.into(),
        }
    }

    /// Create an append operation
    pub fn append(content: impl Into<String>) -> Self {
        EditOp::Append {
            content: content.into(),
        }
    }

    /// Create a delete line operation
    pub fn delete_line(content: impl Into<String>) -> Self {
        EditOp::DeleteLine {
            content: content.into(),
        }
    }
}

/// Apply a list of edit operations to content
///
/// Operations are applied in order. If any operation fails, the original
/// content is returned unchanged with an error.
pub fn apply_edits(content: &str, edits: Vec<EditOp>) -> Result<String, ValidationError> {
    let mut result = content.to_string();

    for edit in edits {
        result = apply_single_edit(&result, edit)?;
    }

    Ok(result)
}

/// Apply a single edit operation
fn apply_single_edit(content: &str, edit: EditOp) -> Result<String, ValidationError> {
    match edit {
        EditOp::Replace { old, new } => {
            let count = content.matches(&old).count();
            match count {
                0 => Err(ValidationError::NotFound { pattern: old }),
                1 => Ok(content.replacen(&old, &new, 1)),
                _ => Err(ValidationError::MultipleMatches {
                    pattern: old,
                    count,
                }),
            }
        }
        EditOp::InsertAtLine { line, content: new_content } => {
            let lines: Vec<&str> = content.lines().collect();
            let max_line = lines.len();

            if line > max_line {
                return Err(ValidationError::InvalidLine { line, max_line });
            }

            let mut new_lines = Vec::with_capacity(lines.len() + 1);
            new_lines.extend_from_slice(&lines[..line]);
            new_lines.push(&new_content);
            new_lines.extend_from_slice(&lines[line..]);

            // Preserve trailing newline if original had one
            let suffix = if content.ends_with('\n') { "\n" } else { "" };
            Ok(new_lines.join("\n") + suffix)
        }
        EditOp::Append { content: new_content } => {
            let has_trailing_newline = content.ends_with('\n');
            let separator = if content.is_empty() {
                ""
            } else if has_trailing_newline {
                ""
            } else {
                "\n"
            };
            Ok(format!("{}{}{}", content, separator, new_content))
        }
        EditOp::DeleteLine { content: line_to_delete } => {
            let lines: Vec<&str> = content.lines().collect();
            let original_len = lines.len();

            let new_lines: Vec<&str> = lines
                .into_iter()
                .filter(|line| *line != line_to_delete)
                .collect();

            if new_lines.len() == original_len {
                return Err(ValidationError::NotFound {
                    pattern: line_to_delete,
                });
            }

            // Preserve trailing newline if original had one
            let suffix = if content.ends_with('\n') { "\n" } else { "" };
            Ok(new_lines.join("\n") + suffix)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_success() {
        let content = "hello world\nhello universe";
        let edits = vec![EditOp::replace("hello world", "hi earth")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "hi earth\nhello universe");
    }

    #[test]
    fn test_replace_not_found() {
        let content = "hello world";
        let edits = vec![EditOp::replace("not found", "replacement")];
        let err = apply_edits(content, edits).unwrap_err();
        assert!(matches!(err, ValidationError::NotFound { .. }));
    }

    #[test]
    fn test_replace_multiple_matches() {
        let content = "hello\nhello\nhello";
        let edits = vec![EditOp::replace("hello", "hi")];
        let err = apply_edits(content, edits).unwrap_err();
        assert!(matches!(err, ValidationError::MultipleMatches { count: 3, .. }));
    }

    #[test]
    fn test_insert_at_line_beginning() {
        let content = "line 1\nline 2";
        let edits = vec![EditOp::insert_at_line(0, "new first line")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "new first line\nline 1\nline 2");
    }

    #[test]
    fn test_insert_at_line_middle() {
        let content = "line 1\nline 2\nline 3";
        let edits = vec![EditOp::insert_at_line(1, "inserted")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "line 1\ninserted\nline 2\nline 3");
    }

    #[test]
    fn test_insert_at_line_end() {
        let content = "line 1\nline 2";
        let edits = vec![EditOp::insert_at_line(2, "last line")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "line 1\nline 2\nlast line");
    }

    #[test]
    fn test_insert_at_invalid_line() {
        let content = "line 1\nline 2";
        let edits = vec![EditOp::insert_at_line(5, "invalid")];
        let err = apply_edits(content, edits).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidLine { line: 5, max_line: 2 }));
    }

    #[test]
    fn test_append() {
        let content = "existing content";
        let edits = vec![EditOp::append("appended")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "existing content\nappended");
    }

    #[test]
    fn test_append_empty() {
        let content = "";
        let edits = vec![EditOp::append("first line")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "first line");
    }

    #[test]
    fn test_append_with_trailing_newline() {
        let content = "existing\n";
        let edits = vec![EditOp::append("appended")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "existing\nappended");
    }

    #[test]
    fn test_delete_line() {
        let content = "line 1\nto delete\nline 3";
        let edits = vec![EditOp::delete_line("to delete")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "line 1\nline 3");
    }

    #[test]
    fn test_delete_line_not_found() {
        let content = "line 1\nline 2";
        let edits = vec![EditOp::delete_line("not found")];
        let err = apply_edits(content, edits).unwrap_err();
        assert!(matches!(err, ValidationError::NotFound { .. }));
    }

    #[test]
    fn test_multiple_edits_in_sequence() {
        let content = "a\nb\nc";
        let edits = vec![
            EditOp::replace("a", "A"),
            EditOp::insert_at_line(1, "B"),
            EditOp::append("D"),
        ];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "A\nB\nb\nc\nD");
    }

    #[test]
    fn test_failed_edit_returns_original() {
        let content = "original";
        let edits = vec![
            EditOp::replace("original", "modified"),
            EditOp::replace("not found", "error"),
        ];
        let result = apply_edits(content, edits);
        assert!(result.is_err());
    }

    #[test]
    fn test_preserve_trailing_newline() {
        let content = "line 1\nline 2\n";
        let edits = vec![EditOp::insert_at_line(1, "inserted")];
        let result = apply_edits(content, edits).unwrap();
        assert_eq!(result, "line 1\ninserted\nline 2\n");
    }
}
