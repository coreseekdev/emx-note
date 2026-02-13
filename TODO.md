# emx-note TODO List

## Priority: High (Security/Bugs)

### 1. [SECURITY] Link file path injection risk
**File:** `src/resolve.rs:184-208`
**Issue:** `parse_link_content` doesn't validate the target path. A malicious link file could point to sensitive directories.
**Fix:** Add validation to ensure link targets are:
- Not pointing to system directories (/, /etc, C:\Windows, etc.)
- Not escaping the intended boundary
- Actually exist and are directories (already done in `validate_link_target`)

### 2. [SECURITY] YAML parsing in meta command
**File:** `src/cmd/meta.rs`
**Issue:** `serde_yaml::from_str` can panic on malformed YAML. While serde handles most errors, extremely large or deeply nested YAML could cause issues.
**Fix:** Add size limits on frontmatter content before parsing.

### 3. [BUG] Tag file link format uses relative paths
**File:** `src/cmd/tag.rs:126`
**Issue:** Tag files store links as `](path/to/note.md)` but if note moves, links break.
**Status:** Acceptable for now - this is by design (simple relative links).

## Priority: Medium (Code Quality)

### 4. [CLEANUP] Remove unused files in src/cmd/
**Files:**
- `src/cmd/copy.rs` - Not used
- `src/cmd/create.rs` - Not used
- `src/cmd/mod.rs` - Old module file, not used
**Action:** Delete these files.

### 5. [CLEANUP] Remove unused gc.rs test file
**File:** `tests/gc_basic.txtar`
**Issue:** gc command is not implemented but test exists.
**Action:** Delete or mark as pending.

### 6. [REFACTOR] Duplicated resolution logic
**Files:** `src/cmd/print.rs`, `src/cmd/meta.rs`, `src/cmd/tag.rs`
**Issue:** All three have similar note resolution code with error handling.
**Suggestion:** Create a helper function `resolve_note_or_error()` that handles the Ambiguous/NotFound cases with proper error messages.

### 7. [REFACTOR] Extension list hardcoded in multiple places
**Files:** `src/cmd/print.rs:16`, `src/cmd/meta.rs:17`, `src/cmd/tag.rs:41`, etc.
**Issue:** `[".md", ".mx", ".emx"]` is duplicated.
**Suggestion:** Define a constant `DEFAULT_EXTENSIONS` in lib.rs or util.rs.

## Priority: Low (Improvements)

### 8. [UX] Command output consistency
**Issue:** Some commands output paths, others output messages.
**Current state:**
- `capsa create` → path only (good)
- `default` → path only (good)
- `tag add` → path for each tag (good)
- `meta set` → message "Set 'key'" (inconsistent)

**Suggestion:** Consider making all commands output paths consistently, or use stderr for messages.

### 9. [UX] Add `--json` output format
**Issue:** For scripting, JSON output would be useful.
**Commands to support:** `list`, `capsa list`, `resolve`

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

### 12. [DOCS] Add CLI examples to --help
**Issue:** Current help text is minimal.
**Suggestion:** Add examples section to each command's help.

## Completed

- [x] Remove search/search-content commands
- [x] Simplify default command
- [x] Refactor tag commands for multiple tags
- [x] Remove move command
- [x] Remove capsa info command
- [x] Fix Windows UNC path handling with dunce crate
- [x] Fix nested key handling in meta command

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
