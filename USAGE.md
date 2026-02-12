# emx-note Usage Guide

emx-note is a Zettelkasten-style note management CLI tool for organizing notes in collections called "capsae" (Latin for "boxes").

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Create a new note collection (capsa)
emx-note capsa create my-notes

# Set it as default
emx-note default my-notes

# Create a permanent note
echo "# Hello World" | emx-note note "hello-world"

# Create today's daily note
echo "Meeting notes" | emx-note daily "standup"

# Print a note
emx-note print hello-world

# List all notes
emx-note list
```

## Global Options

These options can be used with any command:

| Option | Short | Description |
|--------|-------|-------------|
| `--home <PATH>` | | Set home directory (default: `~/.emx-notes` or `$EMX_NOTE_HOME`) |
| `-g, --global` | | Bypass agent prefixing for global operations |
| `-c, --caps <CAPSA>` | | Specify the capsa (note collection) to use |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `EMX_NOTE_HOME` | Default home directory for notes |
| `EMX_NOTE_DEFAULT` | Default capsa name (highest priority) |
| `EMX_AGENT_NAME` | Agent name for prefixing (becomes default capsa when set) |

## Commands

### `daily` - Create Daily Note

Create a temporary/daily note. Daily notes are organized in `#daily/YYYYMMDD/` directories with timestamp-based filenames.

```bash
emx-note daily [title]
```

**Options:**
- `title` - Optional title for the daily note

**Content:** Read from stdin (empty file if no input).

**Output:** Full path to the created note file.

**Examples:**
```bash
# Create empty daily note
emx-note daily

# Create with content from stdin
echo "Meeting notes" | emx-note daily "standup"
# Output: /home/user/.emx-notes/default/#daily/20260212/143022-standup.md
```

**Directory Structure:**
```
capsa/
├── #daily/
│   └── 20260212/
│       ├── 143022.md
│       └── 150845-standup.md
└── note/
    └── #daily.md       # Index with links to all daily notes
```

---

### `note` - Create Permanent Note

Create a permanent note in the `note/` directory. Use `-s/--source` for literature notes with source tracking.

```bash
emx-note note [title] [OPTIONS]
```

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--source <TEXT>` | `-s` | Source of the note (creates in hash subdirectory) |

**Content:** Read from stdin (empty file if no input).

**Output:** Full path to the created note file.

**Filename Rules:**
- No title: `YYYYMMDDHHmmSS.md` (full timestamp)
- With title, no source: `{slug}.md` (title converted to slug)
- With source: `note/{hash}/{slug}.md`

**Examples:**
```bash
# Create permanent note with timestamp
emx-note note

# Create with title
echo "Content here" | emx-note note "my-idea"
# → note/my-idea.md

# Create literature note with source
echo "Book summary" | emx-note note "book-xyz" --source "book:xyz"
# → note/{hash}/book-xyz.md
# → note/{hash}/.source (contains "book:xyz")
```

---

### `resolve` - Resolve Note Reference

Resolve a note reference to its full file path.

```bash
emx-note resolve <note_reference>
```

**Aliases:** `rv`

**Output:** Full path to the note file (normalized with forward slashes).

**Exit Codes:**
- `0` - Note found, path printed to stdout
- `1` - Note not found or ambiguous, error message printed to stderr

**Note Reference:** Uses the same resolution rules as `print` (see below).

**Examples:**
```bash
# Get full path of a note
emx-note resolve hello-world
# Output: C:/Users/user/.emx-notes/default/note/hello-world.md

# Resolve by time prefix
emx-note resolve 22
# Output: C:/Users/.../#daily/20260212/222714-task.md

# Resolve by date + prefix
emx-note resolve 20260212/some
# Output: C:/Users/.../#daily/20260212/222714-some-task.md
```

---

### `list` - List Notes

List notes in the capsa with various filter options.

```bash
emx-note list [filter]
```

**Aliases:** `ls`

**Filter Types:**

| Filter | Description |
|--------|-------------|
| (none) | List top-level `.md` files in `note/` (excludes hash subdirectories) |
| `#tag` | Show contents of tag file `#tag.md` |
| `#daily` | List date subdirectories in `#daily/` (dates only) |
| source string | Hash the source, then list notes in `note/{hash}/` |
| date (YYYYMMDD) | List notes in `#daily/YYYYMMDD/` |

**Examples:**
```bash
# List all top-level notes in note/
emx-note list

# List notes for a specific source (hashes the source string)
emx-note list "book:xyz"

# List notes for a specific date
emx-note list "20260212"

# List all dates with daily notes
emx-note list "#daily"

# Show contents of a tag
emx-note list "#rust"
```

---

### `print` - Print Note Content

Print note content to stdout.

```bash
emx-note print <note_reference>
```

**Aliases:** `p`

**Note Reference Resolution:**

Notes are resolved using flexible prefix matching. The resolution follows these rules in order:

| Format | Description | Example |
|--------|-------------|---------|
| `YYYYMMDD/prefix` | Date-specific search in `#daily/YYYYMMDD/` | `20260212/some` → `222714-some-task.md` |
| `YYYYMMDD\prefix` | Same as above (backslash normalized) | `20260212\22` → `222714-task.md` |
| `HH...` | Today's date + time prefix (1-6 digits) | `22` → `222714-task.md` |
| `HHmmSS-prefix` | Hybrid: exact timestamp + title prefix | `222714-s` → `222714-some-task.md` |
| `title` | Title prefix search (today's daily, then note/, then index files) | `some` → `some-task.md` |
| `YYYYMMDDHHmmSS` | Full timestamp (14 digits) | `20260212222714` → `222714-task.md` |

**Examples:**
```bash
# Print by exact name
emx-note print hello-world

# Print by time prefix (today's daily notes)
emx-note print 22              # Matches 222714-*.md

# Print by date and title prefix
emx-note print 20260212/some   # Matches *some*.md in #daily/20260212/
```

---

### `meta` - Manage Metadata

Manage YAML frontmatter (metadata) in notes.

```bash
emx-note meta <note_reference> [key] [value...] [--delete]
```

**Aliases:** `m`

**Modes:**

| Mode | Command | Description |
|------|---------|-------------|
| List all | `emx-note meta note` | Print all frontmatter |
| Get value | `emx-note meta note key` | Print value of key |
| Set string | `emx-note meta note key value` | Set key to string value |
| Set array | `emx-note meta note key v1 v2 v3` | Set key to array `[v1, v2, v3]` |
| Delete key | `emx-note meta note key --delete` | Delete key |

**Nested Keys:** Supports dot notation for nested keys (e.g., `meta.status`, `tags.project`).

**Examples:**
```bash
# List all metadata
emx-note meta my-note

# Get a value
emx-note meta my-note tags

# Set a string value
emx-note meta my-note status "draft"

# Set an array value
emx-note meta my-note tags rust cli zettelkasten

# Delete a key
emx-note meta my-note status --delete
```

---

### `capsa` - Manage Note Collections

Manage capsae (note collections/vaults).

```bash
emx-note capsa <command>
```

#### `capsa list` - List All Collections
```bash
emx-note capsa list
```

#### `capsa create` - Create New Collection or Link
```bash
emx-note capsa create <name> [path]
```

**Output:** Path to the created capsa (or link file).

- Without `path`: Creates a regular capsa directory
- With `path`: Creates a link capsa pointing to an external directory

**Restrictions:** Names cannot start with `.` (reserved for system use).

**Examples:**
```bash
# Create a regular capsa
emx-note capsa create my-notes
# Output: C:/Users/user/.emx-notes/my-notes

# Create a link capsa to external directory
emx-note capsa create project-notes /path/to/project/docs
# Output: C:/Users/user/.emx-notes/project-notes
```

#### `capsa resolve` - Resolve Collection Path
```bash
emx-note capsa resolve <name>
```

Resolves a capsa to its actual file system path. Useful for link capsae to see where they point.

**Example:**
```bash
emx-note capsa resolve project-notes
# Output: /path/to/project/docs
```

---

### `default` - Get/Set Default Capsa

Get or set the default capsa for future operations.

```bash
# Get current default (prints path)
emx-note default

# Set new default
emx-note default my-notes
```

**Examples:**
```bash
# Show current default path
emx-note default
# Output: /home/user/.emx-notes/my-notes

# Set new default
emx-note default work-projects
# Output: Default capsa set to 'work-projects'
```

---

### `tag` - Manage Tags

Manage tags (stored as `#tagname.md` files in the capsa root directory).

```bash
emx-note tag <command>
```

#### `tag add` - Add Tags to Note
```bash
emx-note tag add [--force] <note_ref> <tag1> [tag2]...
```

Adds one or more tags to a note. The note reference uses the same resolution rules as `print`/`resolve`.

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Apply to all matching notes if reference is ambiguous |

**Output:** Path to each tag file that was modified.

**Examples:**
```bash
# Add single tag
emx-note tag add my-note rust

# Add multiple tags at once
emx-note tag add my-note rust cli tool

# Force add to all matching notes (if ambiguous)
emx-note tag add -f 22 important
```

#### `tag remove` - Remove Tags from Note
```bash
emx-note tag remove [--force] <note_ref> <tag1> [tag2]...
```

Removes one or more tags from a note. Silently succeeds if note wasn't in the tag.

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Apply to all matching notes if reference is ambiguous |

**Examples:**
```bash
# Remove single tag
emx-note tag remove my-note rust

# Remove multiple tags at once
emx-note tag remove my-note rust cli tool
```

**Listing Tags:** Use `emx-note list "#tagname"` to view notes in a tag.

**Tag File Format:**
Tags are stored as `#tagname.md` in the capsa root directory:
```markdown
# tagname

## 2025-02-12
- [Note Title](path/to/note.md)
- [Another Note](docs/another.md)

## 2025-02-10
- [Old Note](old.md)
```

---

### `gc` - Garbage Collection

Find and manage orphaned notes (notes with no incoming links).

```bash
emx-note gc [OPTIONS]
```

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--days <N>` | `-d` | Minimum age in days (default: 7) |
| `--execute` | `-e` | Actually delete orphaned notes (default is dry-run) |
| `--force` | `-f` | Skip confirmation prompt |
| `--verbose` | `-v` | Show verbose output |

**Examples:**
```bash
# Dry-run: list orphaned notes older than 7 days
emx-note gc

# Actually delete orphaned notes
emx-note gc --execute
```

---

## Advanced Concepts

### Agent Prefixing

When `$EMX_AGENT_NAME` is set, capsa operations are automatically prefixed:

```bash
export EMX_AGENT_NAME="agent1"

# Creates capsa "agent1-shared-notes"
emx-note capsa create shared-notes
```

Use `-g/--global` to bypass agent prefixing:

```bash
# Access global "shared-notes" instead of "agent1-shared-notes"
emx-note --global capsa create shared-notes
```

### Link Files

Capsae can link to external directories:

```bash
# Create a link capsa pointing to external directory
emx-note capsa create my-project /path/to/project/docs

# Resolve to see actual path
emx-note capsa resolve my-project
# Output: /path/to/project/docs
```

### Directory Structure

```
capsa/
├── #daily/                    # Daily notes (temporary)
│   ├── 20260212/
│   │   ├── 143022.md
│   │   └── 150845-standup.md
│   └── 20260213/
├── #rust.md                   # Tag file (root directory)
├── #todo.md                   # Another tag file
└── note/                      # Permanent notes
    ├── #daily.md              # Daily notes index
    ├── my-idea.md
    └── {hash}/                # Literature notes with sources
        ├── book-xyz.md
        └── .source            # Contains "book:xyz"
```

---

## See Also

- `emx-note --help` - CLI help
- `emx-note <command> --help` - Command-specific help
