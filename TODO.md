# emx-note TODO List

## Priority: High (Security/Bugs)

### 1. [SECURITY] Link file path injection risk ✅ FIXED
**File:** `src/resolve.rs:184-208`
**Issue:** `parse_link_content` doesn't validate the target path. A malicious link file could point to sensitive directories.
**Fix:** Added `validate_not_system_directory()` function in `src/util.rs` that blocks:
- System directories on Unix (/, /etc, /boot, /sys, /proc, /dev, /bin, /sbin, /lib, /var, /root, /run, /opt)
- Windows system directories (\Windows, \Program Files, etc.)
**Status:** Completed

### 2. [SECURITY] YAML parsing in meta command ✅ FIXED
**File:** `src/cmd/meta.rs`
**Issue:** `serde_yaml::from_str` can panic on malformed YAML. While serde handles most errors, extremely large or deeply nested YAML could cause issues.
**Fix:** Added `MAX_FRONTMATTER_SIZE` constant (64KB) and size check in `extract_frontmatter()`.
**Status:** Completed

### 3. [BUG] Tag file link format uses relative paths ✅ REVIEWED
**File:** `src/cmd/tag.rs:126`
**Issue:** Tag files store links as `](path/to/note.md)` but if note moves, links break.
**Status:** Acceptable for now - this is by design (simple relative links).

## Priority: Medium (Code Quality)

### 4. [CLEANUP] Remove unused files in src/cmd/ ✅ COMPLETED
**Files:**
- `src/cmd/copy.rs` - Deleted
- `src/cmd/create.rs` - Deleted
- `src/cmd/mod.rs` - Deleted

### 5. [CLEANUP] Remove unused gc.rs test file ✅ COMPLETED
**File:** `tests/gc_basic.txtar`
**Status:** Deleted.

### 6. [REFACTOR] Duplicated resolution logic ✅ COMPLETED
**Files:** `src/cmd/print.rs`, `src/cmd/meta.rs`, `src/cmd/tag.rs`
**Issue:** All three have similar note resolution code with error handling.
**Status:** Added helper functions `resolve_note_or_error()` and `resolve_note_with_force()` in `src/note_resolver.rs`.
- `print.rs` and `meta.rs` now use `resolve_note_or_error()`
- `tag.rs` uses `resolve_note_with_force()` to handle --force flag for multiple notes

### 7. [REFACTOR] Extension list hardcoded in multiple places ✅ COMPLETED
**Files:** `src/cmd/print.rs:16`, `src/cmd/meta.rs:17`, `src/cmd/tag.rs:41`, etc.
**Issue:** `[".md", ".mx", ".emx"]` is duplicated.
**Status:** `DEFAULT_EXTENSIONS` constant added to `src/lib.rs` and used throughout codebase.

## Priority: Low (Improvements)

### 8. [UX] Command output consistency ✅ COMPLETED
**Issue:** Some commands output paths, others output messages.
**Current state:**
- `capsa create` → path only (good)
- `default` → path only (good)
- `tag add` → path for each tag (good)
- `meta set` → message "Set 'key'" (inconsistent)
**Fix:** `meta set` now outputs the actual value that was set, confirming the action.

### 9. [UX] Add `--json` output format ✅ COMPLETED
**Issue:** For scripting, JSON output would be useful.
**Commands to support:** `list`, `capsa list`, `resolve`
**Status:** Added `--json` global flag and JSON output to `list` command:
- `emx-note --json list "#tag"` → Date-grouped JSON: `{"2025-01-15": ["link1", "link2"]}`
- `emx-note --json list "#daily"` → Flat JSON array: `["20250101", "20250213"]`
- Uses pulldown-cmark for markdown parsing (as required)

### 10. [FEATURE] Implement `gc` command
**Status:** Marked as hidden/incomplete
**Needed:**
- Scan for orphaned notes (no incoming links)
- Age-based filtering
- Dry-run by default

### 11. [FEATURE] Add `note edit` command
**Issue:** No way to open a note in an editor.
**Suggestion:**
```bash
emx-note edit <note_ref>  # Opens in $EDITOR
```

### 12. [DOCS] Add CLI examples to --help ✅ COMPLETED
**Issue:** Current help text is minimal for LLM agents.
**Status:** Added comprehensive LLM Agent Quick Reference to `--help` output including:
- Basic notes workflow
- Daily notes usage
- Tag management examples
- Metadata operations
- Capsae management
- Scripting/JSON mode
- Note resolution methods

## Completed

- [x] Remove search/search-content commands
- [x] Simplify default command
- [x] Refactor tag commands for multiple tags
- [x] Remove move command
- [x] Remove capsa info command
- [x] Fix Windows UNC path handling with dunce crate
- [x] Fix nested key handling in meta command
- [x] Implement `list #tag` functionality with pulldown-cmark parsing
- [x] Add YAML frontmatter handling in list command
- [x] [SECURITY] Link file path injection risk - added system directory validation
- [x] [SECURITY] YAML parsing size limits - added MAX_FRONTMATTER_SIZE (64KB)
- [x] [CLEANUP] Remove unused files (copy.rs, create.rs, mod.rs)
- [x] [CLEANUP] Remove gc_basic.txtar test
- [x] [REFACTOR] Add DEFAULT_EXTENSIONS constant
- [x] [REFACTOR] Duplicated resolution logic - added `resolve_note_or_error()` and `resolve_note_with_force()` helpers
- [x] [UX] Command output consistency - meta set now outputs the actual value
- [x] [UX] Add `--json` output format for list command

## Future Considerations

### Unicode handling in slugs
**Issue:** `slugify()` only handles ASCII alphanumeric.
**Impact:** Non-ASCII characters become dashes.
**Priority:** Low (acceptable for now)

### Windows symlink support
**Issue:** Link capsae are files, not actual symlinks.
**Priority:** Low (current design is simpler and cross-platform)

### Concurrent access
**Issue:** No locking mechanism for concurrent note edits.
**Priority:** Low (single-user tool assumption)

### Backup/restore
**Issue:** No built-in backup mechanism.
**Priority:** Low (use external tools like git)
