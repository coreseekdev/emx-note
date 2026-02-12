//! Note resolution module
//!
//! Resolves note reference strings to actual file paths.
//!
//! Resolution rules (in order):
//! 1. Full timestamp (YYYYMMDDHHmmSS) → #daily/YYYYMMDD/HHmmSS*.md
//! 2. Time only (HHmmSS) → #daily/{current_date}/HHmmSS*.md
//! 3. Title slug → prefix match in #daily/{current_date}/, then note/, then search index files

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use chrono::Local;

/// Resolution result
#[derive(Debug, Clone)]
pub enum ResolvedNote {
    /// Single file found
    Found(PathBuf),
    /// Multiple candidates (ambiguous)
    Ambiguous(Vec<PathBuf>),
    /// Not found
    NotFound,
}

/// Resolve a note reference to a file path
pub fn resolve_note(
    capsa_path: &Path,
    reference: &str,
    extensions: &[&str],
) -> io::Result<ResolvedNote> {
    // Normalize path separators: backslashes to forward slashes (cross-platform)
    let reference = reference.trim().replace('\\', "/");

    // Rule 0: Path format YYYYMMDD/prefix (date + prefix match in that date's directory)
    if reference.contains('/') {
        let parts: Vec<&str> = reference.splitn(2, '/').collect();
        if parts.len() == 2 {
            let date = parts[0].trim();
            let prefix = parts[1].trim();
            // Date must be 8 digits
            if date.len() == 8 && date.chars().all(|c| c.is_ascii_digit()) {
                if validate_date(date) {
                    // Slugify the prefix for matching
                    let slug = crate::slugify(prefix);
                    return resolve_in_date_dir(capsa_path, date, &slug, extensions);
                }
            }
        }
    }

    // Rule 1: Full timestamp YYYYMMDDHHmmSS (14 digits)
    if let Some((date, time)) = parse_full_timestamp(&reference) {
        return resolve_by_timestamp(capsa_path, &date, &time, extensions);
    }

    // Rule 2: Time prefix HHmmSS... (starts with 6 digits, or fewer digits)
    // Check if it starts with digits (time prefix)
    if let Some(time_prefix) = extract_time_prefix(&reference) {
        let today = Local::now().format("%Y%m%d").to_string();
        return resolve_in_date_dir(capsa_path, &today, &time_prefix, extensions);
    }

    // Rule 3: Title slug (prefix match)
    let slug = crate::slugify(&reference);
    resolve_by_title(capsa_path, &slug, extensions)
}

/// Extract time prefix from input (HH, HHmm, or HHmmSS format)
/// Returns None if input doesn't start with digits
fn extract_time_prefix(s: &str) -> Option<String> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && digits.len() <= 6 {
        Some(digits)
    } else {
        None
    }
}

/// Parse full timestamp YYYYMMDDHHmmSS
fn parse_full_timestamp(s: &str) -> Option<(String, String)> {
    if s.len() >= 14 && s.chars().all(|c| c.is_ascii_digit()) {
        let date = s[0..8].to_string();
        let time = s[8..14].to_string();
        // Validate date/time ranges
        if validate_date(&date) && validate_time(&time) {
            return Some((date, time));
        }
    }
    None
}

/// Validate date string YYYYMMDD
fn validate_date(s: &str) -> bool {
    if s.len() != 8 {
        return false;
    }
    let year: u32 = s[0..4].parse().unwrap_or(0);
    let month: u32 = s[4..6].parse().unwrap_or(0);
    let day: u32 = s[6..8].parse().unwrap_or(0);

    year >= 1900 && year <= 2100 && month >= 1 && month <= 12 && day >= 1 && day <= 31
}

/// Validate time string HHmmSS
fn validate_time(s: &str) -> bool {
    if s.len() != 6 {
        return false;
    }
    let hour: u32 = s[0..2].parse().unwrap_or(0);
    let minute: u32 = s[2..4].parse().unwrap_or(0);
    let second: u32 = s[4..6].parse().unwrap_or(0);

    hour <= 23 && minute <= 59 && second <= 59
}

/// Parse and validate a time string (HHmmSS format)
#[cfg(test)]
fn parse_time_only(s: &str) -> Option<String> {
    if validate_time(s) {
        Some(s.to_string())
    } else {
        None
    }
}

/// Resolve by timestamp (date + time)
fn resolve_by_timestamp(
    capsa_path: &Path,
    date: &str,
    time: &str,
    extensions: &[&str],
) -> io::Result<ResolvedNote> {
    resolve_in_date_dir(capsa_path, date, time, extensions)
}

/// Resolve by date and prefix in the date's daily directory
/// Prefix can be a time prefix (HH, HHmm, HHmmSS) or a title slug
fn resolve_in_date_dir(
    capsa_path: &Path,
    date: &str,
    prefix: &str,
    extensions: &[&str],
) -> io::Result<ResolvedNote> {
    let daily_dir = capsa_path.join("#daily").join(date);

    if !daily_dir.exists() {
        return Ok(ResolvedNote::NotFound);
    }

    // Find files matching prefix
    let mut candidates = Vec::new();

    for entry in fs::read_dir(&daily_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();

                // Check if starts with prefix and has valid extension
                for ext in extensions {
                    if !name_str.ends_with(ext) {
                        continue;
                    }

                    let stem = name_str.strip_suffix(ext).unwrap_or(&name_str);

                    // Match 1: Direct prefix match or hybrid time+title match
                    // Cases:
                    // - "22" (all digits) → time prefix match, allow more digits
                    // - "222714-s" (digits + title) → match exact timestamp prefix + title prefix
                    // - "some" (no digits) → title prefix match, require dash or end
                    if stem.starts_with(prefix) {
                        let rest = &stem[prefix.len()..];
                        // Check if prefix is all digits (time prefix)
                        let is_time_prefix = prefix.chars().all(|c| c.is_ascii_digit());

                        if is_time_prefix {
                            // Time prefix: next char can be digit or dash
                            if rest.is_empty() || rest.starts_with('-') || rest.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                                candidates.push(path.clone());
                                break;
                            }
                        } else {
                            // Check if prefix starts with digits (hybrid: time+title)
                            let digit_part: String = prefix.chars().take_while(|c| c.is_ascii_digit()).collect();
                            if !digit_part.is_empty() {
                                // Hybrid format: "222714-s" means timestamp "222714" + title "s"
                                if stem.starts_with(&digit_part) {
                                    let after_digits = &stem[digit_part.len()..];
                                    // After digits, we should have "-title-..."
                                    if let Some(after_dash) = after_digits.strip_prefix('-') {
                                        let title_prefix = &prefix[digit_part.len()..];
                                        // title_prefix starts with "-" from "222714-s", so strip it
                                        if let Some(title_without_dash) = title_prefix.strip_prefix('-') {
                                            if after_dash.starts_with(title_without_dash) {
                                                candidates.push(path.clone());
                                                break;
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Pure title prefix: require dash or end
                                if rest.is_empty() || rest.starts_with('-') {
                                    candidates.push(path.clone());
                                    break;
                                }
                            }
                        }
                    }

                    // Match 2: Title part after timestamp (HHmmSS-title format)
                    // "some" should match "222714-some-new-test"
                    // Check if stem starts with digits (timestamp) followed by dash
                    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() && stem.len() > digits.len() {
                        let title_part = &stem[digits.len()..];
                        // title_part starts with "-", so strip it
                        if let Some(title_without_dash) = title_part.strip_prefix('-') {
                            if title_without_dash.starts_with(prefix) {
                                candidates.push(path.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    match candidates.len() {
        0 => Ok(ResolvedNote::NotFound),
        1 => Ok(ResolvedNote::Found(candidates.into_iter().next().unwrap())),
        _ => Ok(ResolvedNote::Ambiguous(candidates)),
    }
}

/// Resolve by title slug
fn resolve_by_title(
    capsa_path: &Path,
    slug: &str,
    extensions: &[&str],
) -> io::Result<ResolvedNote> {
    // Rule 3a: Try #daily/{current_date}/ first (with timestamp prefix handling)
    let today = Local::now().format("%Y%m%d").to_string();
    let daily_dir = capsa_path.join("#daily").join(&today);

    if daily_dir.exists() {
        let result = find_by_prefix(&daily_dir, slug, extensions, true)?;
        if !matches!(result, ResolvedNote::NotFound) {
            return Ok(result);
        }
    }

    // Rule 3b: Try note/ directory (without timestamp prefix handling)
    let note_dir = capsa_path.join("note");
    if note_dir.exists() {
        let result = find_by_prefix(&note_dir, slug, extensions, false)?;
        if !matches!(result, ResolvedNote::NotFound) {
            return Ok(result);
        }
    }

    // Rule 3c: Search index files (#*.md in root directory)
    search_in_index_files(capsa_path, slug, extensions)
}

/// Find files by prefix in a directory
/// If allow_timestamp_prefix is true, also matches files like HHmmSS-slug.md
fn find_by_prefix(
    dir: &Path,
    prefix: &str,
    extensions: &[&str],
    allow_timestamp_prefix: bool,
) -> io::Result<ResolvedNote> {
    let mut candidates = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                let name_bytes = name_str.as_bytes();

                // Check if matches prefix with any extension
                let matches = if allow_timestamp_prefix {
                    // For daily notes: try matching after HHmmSS- prefix
                    // Format: HHmmSS[-title].md
                    if name_bytes.len() >= 7 {
                        // Check if starts with 6 digits (HHmmSS)
                        let has_time_prefix = name_bytes[0..6].iter().all(|b| b.is_ascii_digit());
                        let has_separator = name_bytes.len() > 6 && (name_bytes[6] == b'-');

                        if has_time_prefix {
                            // Try matching after the timestamp
                            let rest_start = if has_separator { 7 } else { 6 };
                            let rest = &name_str[rest_start..];

                            // Check if rest starts with prefix
                            if rest.starts_with(prefix) {
                                // Then check extension
                                for ext in extensions {
                                    if name_str.ends_with(ext) {
                                        candidates.push(path.clone());
                                        break;
                                    }
                                }
                                continue;
                            }
                        }
                    }
                    // Fall through to regular prefix check
                    name_str.starts_with(prefix)
                } else {
                    // Regular prefix check
                    name_str.starts_with(prefix)
                };

                if matches {
                    // Check if it has one of the supported extensions
                    for ext in extensions {
                        if name_str.ends_with(ext) {
                            candidates.push(path.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    match candidates.len() {
        0 => Ok(ResolvedNote::NotFound),
        1 => Ok(ResolvedNote::Found(candidates.into_iter().next().unwrap())),
        _ => Ok(ResolvedNote::Ambiguous(candidates)),
    }
}

/// Search for slug in index files (root directory #*.md)
fn search_in_index_files(
    capsa_path: &Path,
    slug: &str,
    extensions: &[&str],
) -> io::Result<ResolvedNote> {
    // Find all #*.md files in root directory
    let mut candidates = Vec::new();

    for entry in fs::read_dir(capsa_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();

                // Check if it's an index file (#*.md, #*.mx, #*.emx)
                if name_str.starts_with('#') {
                    for ext in extensions {
                        if name_str.ends_with(ext) {
                            // Search this index file for the slug
                            if let Some(found) = find_in_index_file(&path, slug)? {
                                candidates.push(found);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    match candidates.len() {
        0 => Ok(ResolvedNote::NotFound),
        1 => Ok(ResolvedNote::Found(candidates.into_iter().next().unwrap())),
        _ => Ok(ResolvedNote::Ambiguous(candidates)),
    }
}

/// Search for a link target in an index file
fn find_in_index_file(index_path: &Path, slug: &str) -> io::Result<Option<PathBuf>> {
    let content = fs::read_to_string(index_path)?;

    // Get the directory containing the index file
    let base_dir = index_path.parent().unwrap_or_else(|| Path::new("."));

    for line in content.lines() {
        // Look for markdown links: [text](path)
        if let Some(start) = line.find("](") {
            if let Some(end) = line[start..].find(')') {
                let link_target = &line[start + 2..start + end];

                // Check if the link text or target contains our slug
                let link_text_before = &line[..start];
                if link_text_before.contains(slug) || link_target.contains(slug) {
                    // Resolve the link target relative to the index file
                    let full_path = base_dir.join(link_target);
                    if full_path.exists() {
                        return Ok(Some(full_path));
                    }
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_timestamp() {
        assert_eq!(
            parse_full_timestamp("20260212143022"),
            Some(("20260212".to_string(), "143022".to_string()))
        );
        assert_eq!(parse_full_timestamp("20260212"), None);
        assert_eq!(parse_full_timestamp("invalid"), None);
    }

    #[test]
    fn test_parse_time_only() {
        assert_eq!(parse_time_only("143022"), Some("143022".to_string()));
        assert_eq!(parse_time_only("25"), None);
        assert_eq!(parse_time_only("invalid"), None);
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date("20260212"));
        assert!(validate_date("20260228")); // Valid Feb date (basic check)
        assert!(!validate_date("20261301")); // Invalid month
        assert!(!validate_date("20260001")); // Invalid month (0)
        assert!(!validate_date("20260100")); // Invalid day (0)
        assert!(!validate_date("20260132")); // Invalid day (32)
    }

    #[test]
    fn test_validate_time() {
        assert!(validate_time("143022"));
        assert!(!validate_time("250000")); // Invalid hour
        assert!(!validate_time("236000")); // Invalid minute
    }
}
