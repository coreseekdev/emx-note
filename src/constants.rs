//! Constants for emx-note
//!
//! This module contains all magic numbers, format strings, and hardcoded values
//! used throughout the codebase to improve maintainability and avoid duplication.

// === File and Directory Names ===

/// Subdirectory for permanent notes
pub const NOTE_SUBDIR: &str = "note";

/// Subdirectory for daily notes
pub const DAILY_SUBDIR: &str = "#daily";

/// Filename for source tracking
pub const SOURCE_FILENAME: &str = ".source";

/// Filename for daily notes index
pub const DAILY_LINK_FILENAME: &str = "#daily.md";

/// Filename for task file
pub const TASK_FILENAME: &str = "TASK.md";

/// Default file extension for notes
pub const MARKDOWN_EXTENSION: &str = ".md";

/// Prefix for tag files
pub const TAG_PREFIX: &str = "#";

/// Newline character
pub const NEWLINE: &str = "\n";

// === Note Titles ===

/// Default title for daily notes
pub const DEFAULT_DAILY_TITLE: &str = "Daily Note";

/// Default title for untitled notes
pub const UNTITLED_NOTE_TITLE: &str = "Untitled";

// === Date and Time Format Strings ===

/// Full timestamp format for daily note filenames: %Y%m%d%H%M%S
pub const DAILY_TIMESTAMP_FORMAT: &str = "#Y%m%d%H%M%S";

/// Date format for daily directories: %Y%m%d
pub const DAILY_DATE_FORMAT: &str = "%Y%m%d";

/// Time format for daily note files: %H%M%S
pub const DAILY_TIME_FORMAT: &str = "%H%M%S";

/// Display format for dates in links: %Y-%m-%d
pub const DAILY_DATE_DISPLAY_FORMAT: &str = "%Y-%m-%d";

// === Markdown Format Strings ===

/// Daily note link format: - [{}](#daily/{}/{})
/// Arguments: title, date, filename
pub const DAILY_LINK_FORMAT: &str = "- [{}](#daily/{}/{})";

/// Header for daily notes index file
pub const DAILY_NOTES_HEADER: &str = "# Daily Notes\n\n";

/// Tag link format: - [{}]({})
/// Arguments: note title, relative path
pub const TAG_LINK_FORMAT: &str = "- [{}]({})";

/// Date header format in tag files: ## {}
pub const TAG_DATE_HEADER_FORMAT: &str = "## {}";

/// Tag file template: # {}\n\n{}\n{}
/// Arguments: tag name, existing content, new entry
pub const TAG_FILE_TEMPLATE: &str = "# {}\n\n{}\n{}";

// === Task File Constants ===

/// Default task prefix
pub const DEFAULT_TASK_PREFIX: &str = "TASK-";

/// Task ID format (zero-padded to 2 digits): {:02}
pub const TASK_ID_FORMAT: &str = "{:02}";

/// Default line where task body starts
pub const DEFAULT_TASK_BODY_START: usize = 3;

/// Checkbox format for completed tasks
pub const TASK_CHECKBOX_DONE: &str = "[x]";

/// Checkbox format for pending tasks
pub const TASK_CHECKBOX_PENDING: &str = "[ ]";

// === Hash and ID Constants ===

/// Length of abbreviated hash for source tracking
pub const HASH_ABBREVIATION_LENGTH: usize = 12;

// === Validation Limits ===

/// Maximum size of frontmatter to parse (prevents DoS on malformed files)
pub const MAX_FRONTMATTER_SIZE: usize = 64 * 1024; // 64KB

// === Error Messages ===

/// Error message for invalid capsa names
pub const ERROR_CAPSA_NAME_STARTS_WITH_DOT: &str = "Capsa name cannot start with '.' (reserved for system)";

/// Error message when capsa already exists
pub const ERROR_CAPSA_ALREADY_EXISTS: &str = "Capsa '{}' already exists";

/// Error message when capsa not found
pub const ERROR_CAPSA_NOT_FOUND: &str = "Capsa '{}' not found";

/// Error message for invalid target path
pub const ERROR_INVALID_TARGET_PATH: &str = "Invalid target path: {}";

/// Error message when target path does not exist
pub const ERROR_TARGET_PATH_NOT_FOUND: &str = "Target path does not exist: {}";
