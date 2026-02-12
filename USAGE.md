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
emx-note set-default my-notes

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

**Source Hashing:**
- Source string is hashed using SHA256
- First 12 characters of hash used as directory name (git-style abbreviation)
- Original source stored in `.source` file

---

### `delete` - Delete a Note

Delete a note from the capsa.

```bash
emx-note delete <note_reference>
```

**Note Reference:** Uses the same resolution rules as `print` (see above).

**Examples:**
```bash
emx-note delete old-note
emx-note delete 20260212/22     # Delete by date + time prefix
emx-note delete some-task       # Delete by title prefix
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
| `#tag` | Show contents of tag file `note/#tag.md` |
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

**Path Separator:** Both `/` and `\` are supported and normalized to `/`.

**Examples:**
```bash
# Print by exact name
emx-note print hello-world

# Print by time prefix (today's daily notes)
emx-note print 22              # Matches 222714-*.md
emx-note print 2227            # Matches 222714-*.md

# Print by date and title prefix
emx-note print 20260212/some   # Matches *some*.md in #daily/20260212/
emx-note print 20260212\22     # Same as above (Windows-style)

# Print by hybrid timestamp + title prefix
emx-note print 20260212/222714-s  # Matches 222714-s*.md
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
emx-note frontmatter <note_reference> <action>
```

**Note Reference:** Uses the same resolution rules as `print` (see above).

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

### `capsa` - Manage Note Collections

Manage capsae (note collections/vaults).

```bash
emx-note capsa <command>
```

#### `capsa list` - List All Collections
```bash
emx-note capsa list
```

#### `capsa create` - Create New Collection
```bash
emx-note capsa create <name>
```

**Restrictions:** Names cannot start with `.` (reserved for system use).

#### `capsa info` - Show Collection Info
```bash
emx-note capsa info [--name <name>]
```

Default name is `.default` (or agent name when using agent mode).

#### `capsa delete` - Delete Collection
```bash
emx-note capsa delete <name>
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
- `#daily/` - Daily notes (temporary)
- `note/` - Permanent notes (already curated)

---

### `tag` / `label` - Manage Tags

Manage tags (stored as `#tagname.md` files in the capsa root directory).

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

# Alternative using list command
emx-note list "#rust"
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

### `move` - Move/Rename Notes

Move or rename notes within the capsa, updating any links that reference them.

```bash
emx-note move <source> <dest>
```

**Example:**
```bash
emx-note move old-note.md new-note.md
emx-note move draft.md ideas/refined-draft.md
```

---

## Advanced Concepts

### Agent Prefixing

When `$EMX_AGENT_NAME` is set, capsa operations are automatically prefixed:

```bash
export EMX_AGENT_NAME="agent1"

# Creates capsa "agent1-shared-notes"
emx-note capsa create shared-notes

# Access agent1's personal default (resolves to "agent1")
emx-note daily
```

Use `-g/--global` to bypass agent prefixing:

```bash
# Access global "shared-notes" instead of "agent1-shared-notes"
emx-note --global capsa create shared-notes
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

### Zettelkasten Workflow

The recommended workflow for Zettelkasten-style note management:

1. **Capture** - Use `emx-note daily` for quick daily notes
2. **Process** - Review daily notes and identify valuable content
3. **Curate** - Move valuable notes to permanent locations using `emx-note note` with appropriate titles
4. **Link** - Create connections between notes using markdown links
5. **Tag** - Use tags for cross-cutting organization

---

## See Also

- `emx-note --help` - CLI help
- `emx-note <command> --help` - Command-specific help
- Project README for development information
