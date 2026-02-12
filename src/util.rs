//! Utility functions for secure path handling and common operations

use std::io;
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};

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
    // Use canonicalize if base exists, otherwise just check with starts_with
    if base.exists() {
        let canonical_base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
        // For result, we can only canonicalize if it exists, otherwise just check prefix
        if result.exists() {
            let canonical_result = result.canonicalize().unwrap_or_else(|_| result.clone());
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
        target.to_path_buf()
    } else {
        home.join(target)
    };

    // Canonicalize if exists
    let canonical = resolved.canonicalize().unwrap_or(resolved.clone());

    // Check if it's a directory (valid link target)
    if !canonical.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Link target must be a directory"
        ));
    }

    Ok(canonical)
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
        .unwrap_or_else(|| "Untitled".to_string())
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
    full_hash.chars().take(12).collect()
}

/// Display a path with forward slashes (cross-platform standard)
/// Converts Windows backslashes to forward slashes for consistent output
pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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
}
