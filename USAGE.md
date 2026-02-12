# emx-note Usage Guide

emx-note is a Zettelkasten-style note management CLI tool for organizing notes in collections called "capsae" (Latin for "boxes").

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Create a new note collection (capsa)
emx-note capssa create my-notes

# Set it as default
emx-note set-default my-notes

# Create a note
emx-note create hello-world

# Create today's daily note
emx-note daily

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

Create today's daily note. Daily notes are organized in `daily/YYYYMMDD/` directories with timestamp-based filenames.

```bash
emx-note daily [title]
```

**Options:**
- `title` - Optional title for the daily note (default: "Daily Note")

**Examples:**
```bash
# Create a basic daily note
emx-note daily

# Create with custom title
emx-note daily "Team Standup"

# Creates: daily/20250212/143022-team-standup.md
```

**Template Support:**
Create a template at `.template/DAILY.md` in your capsa with placeholders:
- `{{title}}` - Note title
- `{{date}}` - Date (YYYY-MM-DD)
- `{{datetime}}` - Full datetime (YYYY-MM-DD HH:MM:SS)

---

### `create` - Create a Note

Create a new note with content from argument or stdin.

```bash
emx-note create <note_name> [OPTIONS]
```

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--content <TEXT|- >` | `-C` | Note content (use `-` to read from stdin) |

**Content Reading Behavior:**
1. With `-C "text"`: Use the provided text directly
2. With `-C -`: Read from stdin
3. Without `-C`: Read from stdin (empty string if no data)

**Examples:**
```bash
# Create empty note
emx-note create ideas

# Create with inline content
emx-note create todo --content "# Tasks\n- [ ] Task 1"

# Read from stdin
echo "# Meeting Notes" | emx-note create meeting -
emx-note create notes --content -

# Pipe content
cat draft.md | emx-note create final/draft
```

---

### `copy` - Copy Notes (TBD)

Copy a note to a new location. Useful for copying from `daily/` to permanent storage locations like `持久/` (persistent) or `文献/` (literature), following Zettelkasten principles.

```bash
emx-note copy <source> <dest> [OPTIONS]
```

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--overwrite` | `-W` | Overwrite destination if exists |

**Examples:**
```bash
# Copy daily note to permanent storage
emx-note copy daily/20250212/143022.md 持久/team-meeting.md

# Copy to literature folder
emx-note copy daily/20250212/150845-idea.md 文献/20250212-idea.md
```

**Note:** This command is under design consideration.

---

### `delete` - Delete a Note

Delete a note from the capsa.

```bash
emx-note delete <note_path>
```

**Example:**
```bash
emx-note delete old-note
emx-note delete drafts/unused-idea
```

---

### `list` - List Notes

List all notes in the capsa or a specific directory.

```bash
emx-note list [path]
```

**Aliases:** `ls`

**Examples:**
```bash
# List all notes (root directory)
emx-note list

# List notes in a subdirectory
emx-note list docs

# List daily notes
emx-note list daily
```

---

### `print` - Print Note Content

Print note content to stdout.

```bash
emx-note print <note_name> [OPTIONS]
```

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--mentions` | `-m` | Include backlink list (notes that link to this note) |

**Aliases:** `p`

**Examples:**
```bash
# Print note content
emx-note print hello-world

# Print with backlinks
emx-note print hello-world --mentions
```

---

### `search` - Fuzzy Search

Interactive fuzzy search for notes using terminal selection UI.

```bash
emx-note search
```

**Aliases:** `s`

---

### `search-content` - Search Note Content

Search for text within note contents.

```bash
emx-note search-content <search_term>
```

**Aliases:** `sc`

**Example:**
```bash
emx-note search-content "Zettelkasten"
emx-note search-content "TODO"
```

---

### `frontmatter` - Manage Frontmatter

Manage YAML frontmatter in notes.

```bash
emx-note frontmatter <note_name> <action>
```

**Aliases:** `fm`

**Actions:**

#### `print` - Print Frontmatter
```bash
emx-note frontmatter <note_name> print
```

#### `edit` - Edit Key-Value
```bash
emx-note frontmatter <note_name> edit --key <key> --value <value>
```
Supports nested keys with dots (e.g., `meta.tags`).

#### `delete` - Delete Key
```bash
emx-note frontmatter <note_name> delete --key <key>
```

**Examples:**
```bash
# Print frontmatter
emx-note fm note print

# Set a value
emx-note fm note edit --key tags --value " rust, cli"

# Edit nested key
emx-note fm note edit --key "meta.status" --value "draft"

# Delete a key
emx-note fm note delete --key "meta.status"
```

---

### `capssa` - Manage Note Collections

Manage capsae (note collections/vaults).

```bash
emx-note capssa <command>
```

#### `capssa list` - List All Collections
```bash
emx-note capssa list
```

#### `capssa create` - Create New Collection
```bash
emx-note capssa create <name>
```

**Restrictions:** Names cannot start with `.` (reserved for system use).

#### `capssa info` - Show Collection Info
```bash
emx-note capssa info [--name <name>]
```

Default name is `.default` (or agent name when using agent mode).

#### `capssa delete` - Delete Collection
```bash
emx-note capssa delete <name>
```

**Restrictions:**
- Cannot delete `.default` (system default)
- Cannot delete linked capsae (delete the link file instead)

---

### `set-default` - Set Default Capsa

Set the default capsa for future operations.

```bash
emx-note set-default <caps>
```

**Example:**
```bash
emx-note set-default my-notes
```

**Note:** Default capsa resolution priority:
1. `$EMX_NOTE_DEFAULT` environment variable
2. `$EMX_AGENT_NAME` (when set, agent's personal default)
3. Hardcoded `.default` directory

---

### `print-default` - Show Default Capsa Info

Print information about the current default capsa.

```bash
emx-note print-default [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--path-only` | Output only the path |

**Examples:**
```bash
# Show full info
emx-note print-default

# Show path only (useful for scripts)
emx-note print-default --path-only
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

**Orphaned Notes Criteria:**
1. Older than N days (default: 7)
2. No incoming links from other notes in the capsa

**Examples:**
```bash
# Dry-run: list orphaned notes older than 7 days
emx-note gc

# List orphaned notes older than 30 days
emx-note gc --days 30

# Actually delete orphaned notes (with confirmation)
emx-note gc --execute

# Delete without confirmation
emx-note gc --execute --force

# Verbose mode
emx-note gc --verbose
```

**Skipped Directories:**
- `.template/` - Template files
- `daily/` - Daily notes

---

### `tag` / `label` - Manage Tags

Manage tags (stored as `#tagname.md` files in the capsa root).

```bash
emx-note tag <command>
```

**Aliases:** `label` (hidden command, same functionality)

#### `tag add` - Add Note to Tag
```bash
emx-note tag add <tag> <note>
```

Adds a note to a tag. Tags are stored with date-grouped links.

**Example:**
```bash
emx-note tag add rust README.md
emx-note tag add todo docs/task-list
```

#### `tag remove` - Remove Note from Tag
```bash
emx-note tag remove <tag> <note>
```

**Example:**
```bash
emx-note tag remove rust README.md
```

#### `tag list` - List Tags or Notes in Tag
```bash
emx-note tag list [tag]
```

**Examples:**
```bash
# List all tags
emx-note tag list

# List notes in a specific tag
emx-note tag list rust
```

#### `tag delete` - Delete Tag
```bash
emx-note tag delete <tag>
```

**Example:**
```bash
emx-note tag delete old-tag
```

**Tag File Format:**
```markdown
# tagname

## 2025-02-12
- [Note Title](path/to/note.md)
- [Another Note](docs/another.md)

## 2025-02-10
- [Old Note](old.md)
```

---

## Advanced Concepts

### Agent Prefixing

When `$EMX_AGENT_NAME` is set, capsa operations are automatically prefixed:

```bash
export EMX_AGENT_NAME="agent1"

# Creates capsa "agent1-shared-notes"
emx-note capssa create shared-notes

# Access agent1's personal default (resolves to "agent1")
emx-note daily
```

Use `-g/--global` to bypass agent prefixing:

```bash
# Access global "shared-notes" instead of "agent1-shared-notes"
emx-note --global capssa create shared-notes
```

### Link Files

Capsae can link to external directories using INI-style link files:

```bash
# Create a link file
cat > my-project << 'EOF'
[link]
target = /path/to/project/docs
EOF

# Now my-project points to external directory
emx-note --caps my-project list
```

### Default Capsa Resolution

The default capsa is resolved in this priority order:

1. **`$EMX_NOTE_DEFAULT`** - Explicit environment variable
2. **`$EMX_AGENT_NAME`** - Agent's personal default (when set and not global)
3. **`.default`** - System default directory

### Daily Notes Structure

```
capsa/
├── daily/
│   ├── 20250212.md          # Link file (today's daily index)
│   └── 20250212/
│       ├── 143022.md        # First note of the day
│       ├── 150845-team.md   # Note with title
│       └── 161030.md        # Another note
```

The `daily/YYYYMMDD.md` file contains links to all notes created that day.

### Zettelkasten Workflow

The recommended workflow for Zettelkasten-style note management:

1. **Capture** - Use `emx-note daily` for quick daily notes
2. **Process** - Review daily notes and identify valuable content
3. **Copy** - Copy valuable notes to permanent locations:
   - `persistent/` - Persistent notes
   - `literature/` - Literature/reference notes
4. **Link** - Create connections between notes using markdown links
5. **Tag** - Use tags for cross-cutting organization

---

## Implementation Notes

**Pending Changes:**

1. **`create` command modifications:**
   - Remove `--append` option (to be implemented separately)
   - Remove `--open` option (not suitable for CLI environment)
   - Add stdin reading support (`-C -` or default to stdin)

2. **`copy` command (TBD):**
   - Design under consideration for copying notes from `daily/` to permanent storage
   - Would replace the need for `move` command

---

## See Also

- `emx-note --help` - CLI help
- `emx-note <command> --help` - Command-specific help
- Project README for development information
