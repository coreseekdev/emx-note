//! Utility functions for secure path handling and common operations

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use crate::constants as C;

/// Validate and resolve a note path, ensuring it stays within the capsa directory
/// Returns an error if the path attempts to escape the base directory
pub fn secure_path(base: &Path, relative: &str) -> io::Result<PathBuf> {
    let mut result = base.to_path_buf();

    for component in relative.split(|c| c == '/' || c == '\\') {
        match component {
            "" => continue,           // Skip empty components
            "." => continue,          // Current directory - no-op
            ".." => {
                // Check if going up would escape the base
                if !result.starts_with(base) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Path traversal detected: cannot escape capsa directory"
                    ));
                }
                result.pop();
            }
            _ => {
                // Check for absolute paths (Unix and Windows)
                if component.starts_with('/') ||
                   (component.len() >= 2 && component.as_bytes()[1] == b':') {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Absolute paths are not allowed"
                    ));
                }
                result.push(component);
            }
        }
    }

    // Final check: ensure the resolved path is within the base
    // Use dunce::canonicalize if base exists to avoid UNC prefix on Windows
    if base.exists() {
        let canonical_base = dunce::canonicalize(&base).unwrap_or_else(|_| base.to_path_buf());
        // For result, we can only canonicalize if it exists, otherwise just check prefix
        if result.exists() {
            let canonical_result = dunce::canonicalize(&result).unwrap_or_else(|_| result.clone());
            if !canonical_result.starts_with(&canonical_base) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path traversal detected: resolved path escapes capsa directory"
                ));
            }
        } else {
            // Path doesn't exist yet - check that it's under the base by comparing non-canonical paths
            // This is safe because we've already rejected ".." and absolute paths above
            if !result.starts_with(base) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path traversal detected: resolved path escapes capsa directory"
                ));
            }
        }
    } else {
        // Base doesn't exist - just check prefix match
        if !result.starts_with(base) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path traversal detected: resolved path escapes capsa directory"
            ));
        }
    }

    Ok(result)
}

/// Validate a link target path (from link files)
/// Returns an error if the target is suspicious or dangerous
pub fn validate_link_target(target: &Path, home: &Path) -> io::Result<PathBuf> {
    // Resolve to absolute path
    let resolved = if target.is_absolute() {
        // Additional check: block system directories
        validate_not_system_directory(target)?;
        target.to_path_buf()
    } else {
        home.join(target)
    };

    // Canonicalize if exists (use dunce to avoid UNC prefix on Windows)
    let canonical = dunce::canonicalize(&resolved).unwrap_or(resolved.clone());

    // Check if canonical path is a system directory
    validate_not_system_directory(&canonical)?;

    // Check if it's a directory (valid link target)
    if !canonical.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Link target must be a directory"
        ));
    }

    Ok(canonical)
}

/// Validate that path is not a system directory
fn validate_not_system_directory(path: &Path) -> io::Result<()> {
    let path_str = path.to_string_lossy().to_lowercase();

    // Unix system directories
    #[cfg(unix)]
    {
        // Block root system directories
        if path_str == "/" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot link to system root directory"
            ));
        }

        // Block specific system directories
        let system_dirs = [
            "/etc", "/boot", "/sys", "/proc", "/dev", "/bin", "/sbin",
            "/lib", "/lib32", "/lib64", "/usr/bin", "/usr/sbin", "/usr/lib",
            "/var", "/root", "/run", "/opt"
        ];

        for sys_dir in &system_dirs {
            if path_str == *sys_dir || path_str.starts_with(&format!("{}/", sys_dir)) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Cannot link to system directory: {}", sys_dir)
                ));
            }
        }
    }

    // Windows system directories
    #[cfg(windows)]
    {
        // Block Windows directories
        let system_dirs = [
            "\\windows", "\\windows\\system32", "\\windows\\syswow64",
            "\\program files", "\\program files (x86)",
            "\\programdata", "\\system volume information"
        ];

        for sys_dir in &system_dirs {
            let sys_dir_lower = sys_dir.to_lowercase().replace('\\', "/");
            let path_normalized = path_str.replace('\\', "/");
            if path_normalized.contains(&sys_dir_lower) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Cannot link to Windows system directory")
                ));
            }
        }
    }

    Ok(())
}

/// Extract title from note content (first H1 heading or filename)
pub fn extract_note_title(note_path: &Path, content: &str) -> String {
    // Look for first # heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed.trim_start_matches("# ").to_string();
        }
    }

    // Fallback to filename
    note_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| C::UNTITLED_NOTE_TITLE.to_string())
}

/// Convert title to URL-safe slug
pub fn slugify(title: &str) -> String {
    let mut result = String::with_capacity(title.len());
    let mut prev_is_dash = false;

    for c in title.to_lowercase().chars() {
        if c.is_alphanumeric() {
            result.push(c);
            prev_is_dash = false;
        } else if !prev_is_dash && !result.is_empty() {
            result.push('-');
            prev_is_dash = true;
        }
    }

    result.trim_matches('-').to_string()
}

/// Hash source string using SHA256
pub fn hash_source(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

/// Abbreviate hash to git-style length (12 characters for SHA256)
/// Git uses 7 characters for SHA1 (160 bits), for SHA256 we use 12
pub fn abbreviate_hash(full_hash: &str) -> String {
    full_hash.chars().take(C::HASH_ABBREVIATION_LENGTH).collect()
}

/// Display a path with forward slashes (cross-platform standard)
/// Converts Windows backslashes to forward slashes for consistent output
pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Read content from stdin, returns empty string if no data
pub fn read_stdin_content() -> io::Result<String> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(0) => Ok(String::new()),
        Ok(_) => Ok(buffer),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(String::new()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_path_normal() {
        let base = PathBuf::from("/home/user/notes");
        let result = secure_path(&base, "folder/note.md").unwrap();
        assert_eq!(result, PathBuf::from("/home/user/notes/folder/note.md"));
    }

    #[test]
    fn test_secure_path_traversal_blocked() {
        let base = PathBuf::from("/home/user/notes");
        let result = secure_path(&base, "../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_path_absolute_blocked() {
        let base = PathBuf::from("/home/user/notes");
        // On Unix, /etc/passwd is absolute and should be blocked
        // On Windows, it would be treated as relative, so we test with the appropriate format
        #[cfg(unix)]
        {
            let result = secure_path(&base, "/etc/passwd");
            assert!(result.is_err());
        }
        #[cfg(windows)]
        {
            let result = secure_path(&base, "C:\\Windows\\System32");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_secure_path_windows_absolute_blocked() {
        // This test is Windows-specific
        #[cfg(windows)]
        {
            let base = PathBuf::from("C:\\Users\\notes");
            let result = secure_path(&base, "C:\\Windows\\System32");
            assert!(result.is_err());
        }
        #[cfg(not(windows))]
        {
            // On non-Windows, just pass the test
            assert!(true);
        }
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test@Note#123"), "test-note-123");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }

    // Security tests for validate_link_target

    #[test]
    fn test_validate_link_target_valid_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let valid_dir = temp_dir.path().join("valid_dir");
        std::fs::create_dir(&valid_dir).unwrap();

        let result = validate_link_target(&valid_dir, temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_dir);
    }

    #[test]
    fn test_validate_link_target_file_rejected() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();

        let result = validate_link_target(&file_path, temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a directory"));
    }

    #[test]
    fn test_validate_link_target_system_directory_unix() {
        #[cfg(unix)]
        {
            let result = validate_link_target(&PathBuf::from("/etc"), &PathBuf::from("/home"));
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("system"));
        }

        #[cfg(not(unix))]
        {
            assert!(true); // Skip on non-Unix
        }
    }

    #[test]
    fn test_validate_link_target_system_directory_windows() {
        #[cfg(windows)]
        {
            let result = validate_link_target(
                &PathBuf::from("C:\\Windows"),
                &PathBuf::from("C:\\Users")
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("system"));
        }

        #[cfg(not(windows))]
        {
            assert!(true); // Skip on non-Windows
        }
    }

    #[test]
    fn test_validate_link_target_subdirectory_of_system_unix() {
        #[cfg(unix)]
        {
            let result = validate_link_target(&PathBuf::from("/etc/subdir"), &PathBuf::from("/home"));
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("system"));
        }

        #[cfg(not(unix))]
        {
            assert!(true); // Skip on non-Unix
        }
    }

    // Additional edge case tests for secure_path

    #[test]
    fn test_secure_path_with_mixed_separators() {
        let base = PathBuf::from("/home/user/notes");
        // Mixed separators should be normalized
        let result = secure_path(&base, "folder\\subfolder/note.md");
        assert!(result.is_ok());
        let result_path = result.unwrap();
        assert!(result_path.starts_with(&base));
    }

    #[test]
    fn test_secure_path_with_dots_at_end() {
        let base = PathBuf::from("/home/user/notes");
        let result = secure_path(&base, "folder/.");
        assert!(result.is_ok());
        // "." should be normalized away
        assert_eq!(result.unwrap(), base.join("folder"));
    }

    #[test]
    fn test_secure_path_with_double_dots_at_end() {
        let base = PathBuf::from("/home/user/notes");
        let result = secure_path(&base, "folder/subfolder/..");
        assert!(result.is_ok());
        // ".." should navigate up one level
        assert_eq!(result.unwrap(), base.join("folder"));
    }

    #[test]
    fn test_secure_path_empty_components() {
        let base = PathBuf::from("/home/user/notes");
        let result = secure_path(&base, "folder///note.md");
        assert!(result.is_ok());
        // Multiple slashes should be handled
        assert!(result.unwrap().starts_with(&base));
    }
}
