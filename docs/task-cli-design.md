# task Command Design (Simplified v2.0)

## CLI Overview

```
emx-note task <operation> [arguments]

Operations:
  add      - Add new task (auto-increment ID)
  take     - Take ownership of a task
  comment  - Add comment to task
  release  - Release task ownership
  list     - List tasks by status
  show     - Show task details
  log      - Show execution log for a task
  find     - Find tasks by note reference
```

## TASK.md File Structure

**File location**: `{capsa_root}/TASK.md` (filename configurable via `EMX_TASKFILE` environment variable)

### Sections

| Section | Description |
|---------|-------------|
| **meta** | Frontmatter with PREFIX configuration |
| **description** | ≤5 lines summary (identifies task file purpose when multiple exist, e.g., sub-projects) |
| **body** | Task entries under any headers (state by `[ ]`/`[x]`) |
| **reference** | Task definitions `[task-NN]: node_ref` |

**Task status is determined by location and checkbox**:
- In **reference** only → **backlog**
- In **body** with `[ ]` → **doing**
- In **body** with `[x]` → **done**

Headers in body are for agent categorization only, not status. Tool does not preset any header names.

### Section Boundaries

Using markdown horizontal rules (`---` or `***`). Per markdown spec, frontmatter must start at file beginning with `---`.

```markdown
---
PREFIX: task-
---

This is the description section (≤5 lines).

---

## Tasks (body section)

- [ ] [Task title][task-01] @agent
  - Comment

---

[task-01]: ./path/to/note.md
[task-02]: ./path/to/another.md
```

### Initial File Creation

When TASK.md doesn't exist, first `task add` creates:
```markdown
---
PREFIX: task-
---

---

[task-01]: 143022    # node_ref from: emx-note task add 143022
```

Note: Two horizontal rules create empty body section between meta and reference.

### Task Entry Format

```markdown
- [ ] [Implement OAuth flow][task-01] @agent-1
  - Researched JWT libraries
  - Implemented token refresh
  - Fixed edge case with expired tokens

- [x] [Database optimization][task-02]
  - Added indexes
  - Query performance improved 50%
```

### Parser Rules

1. **Line-based**: Each task starts with `- [STATE] [title][task-id]` optionally followed by `@agent-name`
2. **Checkbox**: `[ ]` = open, `[x]` = done
3. **Agent**: `@agent-name` suffix is optional (present when `EMX_AGENT_NAME` is set and task is taken)
4. **Sub-items**: Indented lines are comment entries

**Task entry syntax**:
```
- [x?] [title][task-id] (@agent-name)?
```

Where:
- `[x?]` = `[ ]` or `[x]`
- `[title][task-id]` = markdown link (title is display text, task-id references definition)
- `(@agent-name)?` = optional agent marker

## Subcommands

### add - Add New Task

```bash
emx-note task add node_ref
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `node_ref` | Note reference (smart resolution, see below) |

**node_ref Resolution Rules** (same as `print` command):
1. `YYYYMMDDHHmmSS` → Full timestamp → `#daily/YYYYMMDD/HHmmSS*.md`
2. `HHmmSS` | `HHmm` | `HH` → Time prefix → today's daily directory
3. `YYYYMMDD/prefix` → Date + prefix match in that date's directory
4. Title slug → Prefix match in `#daily/{today}/`, then `note/`, then index files

**Behavior**:
- Adds task definition to **reference** section: `[task-NN]: node_ref`
- Task ID is previous task number + 1 (auto-increment)
- Task ID prefix can be configured via frontmatter `PREFIX` key
- Manual deletion: edit TASK.md directly

**Output**:
- On success: outputs task-id to stdout (e.g., `task-01`)
- If node_ref already exists: outputs existing task-id (no duplicate created)

**Examples**:
```bash
# Add task for daily note (today at 14:30:22)
emx-note task add 143022
# Output: task-01

# Add task for specific date + prefix
emx-note task add 20260212/api
# Output: task-02

# Add same node_ref again - returns existing task-id
emx-note task add 143022
# Output: task-01 (already exists)
```

---

### take - Take Ownership of Task

```bash
emx-note task take <task_id> [--title "title"] [--header "## Header"] [--dry-run]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `task_id` | Task ID (e.g., `task-01`), must exist in reference section or body |

**Options**:
| Option | Description |
|--------|-------------|
| `--title "title"` | Task description (optional, defaults to note's frontmatter/title or filename) |
| `--header "## X"` | Target header in body section (optional, for categorization) |
| `--dry-run` | Preview result without making changes |

**Special behavior**:
- If `EMX_AGENT_NAME` env var is set: uses agent name as `@` marker
- If `EMX_AGENT_NAME` not set: no `@` mechanism
- **ERROR**: If task already has `@agent` marker (already taken)

**Header placement**:
- With `--header`: move task to end of specified header section
- Without `--header`:
  - If task already in body: keep current position
  - If task not yet in body: insert before first header (or before reference block if no header)
- Principle: task item belonging to no header is the norm, belonging to a header is special

**Agent coordination rules**:
- Task can only be taken if it has NO `@agent` marker
- To take a task owned by another agent, that agent must `release` it first
- Taking a done task (`[x]`) reopens it (changes to `[ ]`)
- Reference definition `[task-NN]: node_ref` is preserved (markdown link reference)

**Title behavior**:
- If task has no existing title: use provided title or default from note
- If task already has title: preserve original title, new title (if provided) is added as comment

**Dry-run output**:
```
--- TASK.md (lines 15-20) ---
- [ ] [Implement OAuth flow][task-01] @agent-1
---
Would insert at: line 15 (before ## Tasks)
```

**Examples**:
```bash
# Take task with default title
emx-note task take task-01

# Take task with custom title
emx-note task take task-01 --title "Implement OAuth flow"

# Take task and place under header
emx-note task take task-01 --header "## API"

# Preview before taking
emx-note task take task-01 --dry-run
```

---

### comment - Add Comment to Task

```bash
emx-note task comment <task_id> "message" [--git COMMIT_HASH] [--dry-run]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `task_id` | Task ID (e.g., `task-01`) |
| `message` | Comment message to append (any valid markdown list item) |

**Options**:
| Option | Description |
|--------|-------------|
| `--git COMMIT_HASH` | Attach git commit short hash (optional) |
| `--dry-run` | Preview result without making changes |

**Behavior**:
- Appends sub-item to task entry in body section
- Sub-item can be any valid markdown list item (text, link, etc.)
- **Timestamp**: Automatically added to all comments (format: `YYYY-MM-DD HH:MM`)
- With `--git`: appends git hash as markdown link

**Dry-run output**:
```
--- TASK.md (lines 20-23) ---
  - Previous comment
  - 2026-02-14 10:30 Fixed token refresh bug [a1b2c3d]
---
Would append to: task-01
```

**Examples**:
```bash
# Add progress comment (auto timestamp)
emx-note task comment task-01 "Researched JWT libraries"

# Add comment with git commit reference
emx-note task comment task-01 "Fixed token refresh bug" --git a1b2c3d

# Preview before commenting
emx-note task comment task-01 "Important fix" --dry-run
```

**Result format**:
```markdown
- [ ] [Implement OAuth flow][task-01] @agent-1
  - 2026-02-14 09:15 Researched JWT libraries
  - 2026-02-14 10:30 Fixed token refresh bug [a1b2c3d]
```

---

### release - Release Task

```bash
emx-note task release <task_id...> [--done] [--force] [--dry-run]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `task_id...` | One or more task IDs (e.g., `task-01 task-02 task-03`) |

**Options**:
| Option | Description |
|--------|-------------|
| `--done` | Mark task(s) as done (change `[ ]` to `[x]`) before release |
| `--force` | Force release even if not owner (single task only) |
| `--dry-run` | Preview result without making changes |

**Behavior**:
- Removes `@agent` marker from task entry
- With `--done`: marks task as complete, then removes agent marker
- Without `--done`: releases task incomplete (keeps `[ ]` state)
- After release, task can be taken by another agent
- Batch release: releases multiple tasks (all must have `@agent` marker)
- `--force`: bypasses owner check, but only one task allowed

**Dry-run output**:
```
task-01: [ ] → [x], @agent-1 → (none)
task-02: [ ] → [ ], @agent-1 → (none)
---
Would release 2 task(s)
```

**Examples**:
```bash
# Release single task as incomplete
emx-note task release task-01

# Release single task as complete
emx-note task release task-01 --done

# Release multiple tasks
emx-note task release task-01 task-02 task-03

# Preview batch release
emx-note task release task-01 task-02 --done --dry-run

# Force release (single task only)
emx-note task release task-01 --force
```

**Result format**:
```markdown
# Before release:
- [ ] [Implement OAuth flow][task-01] @agent-1
  - Researched JWT libraries

# After release (incomplete):
- [ ] [Implement OAuth flow][task-01]
  - Researched JWT libraries

# After release --done:
- [x] [Implement OAuth flow][task-01]
  - Researched JWT libraries
```

---

### list - List Tasks

```bash
emx-note task list [backlog|doing|done|all] [--oneline] [--owner @agent]
```

**Arguments**:
| Argument | Default | Description |
|----------|---------|-------------|
| (none) | `all` | List all tasks |
| `backlog` | | Tasks in reference section only |
| `doing` | | Tasks in body with `[ ]` |
| `done` | | Tasks in body with `[x]` |
| `all` | | List all tasks |

**Options**:
| Option | Description |
|--------|-------------|
| `--oneline` | Output only task IDs, one per line |
| `--owner @agent` | Filter by owner (`@agent-name` or `(none)`) |

**Output format (default)**:
```
ID        TITLE                    FILE                    STATUS    OWNER
task-01   Implement OAuth flow     20260212/api-design     doing     @agent-1
task-02   Database optimization    143022                  done      (none)
task-03   -                        20260213/login-fix      backlog   (none)
```

**Output format (--oneline)**:
```
task-01
task-02
task-03
```

Columns:
- **ID**: Task ID (e.g., `task-01`)
- **TITLE**: Task description (`-` if backlog task has no title)
- **FILE**: Note reference (from reference definition)
- **STATUS**: `backlog` | `doing` | `done`
- **OWNER**: `@agent-name` or `(none)`

Note: Both TITLE and FILE are always displayed to help identify tasks.

**Examples**:
```bash
# List all tasks
emx-note task list

# List tasks owned by specific agent
emx-note task list --owner @agent-1

# List doing tasks with no owner
emx-note task list doing --owner "(none)"

# List doing tasks (one per line)
emx-note task list doing --oneline
```

---

### log - Show Execution Log

```bash
emx-note task log [task_id]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `task_id` | Task ID (e.g., `task-01`) |

**Behavior**:
- Displays task status and comment history
- Shows all log entries in chronological order
- For backlog tasks (not taken yet): shows empty history

**Example**:
```bash
# Task in body (taken)
emx-note task log task-01

# Output:
# task-01: Implement OAuth flow
# Status: doing | Owner: @agent-1
# ---
# - 2026-02-14 09:15 Researched JWT libraries
# - 2026-02-14 10:30 Implemented token refresh

# Task in backlog (not taken)
emx-note task log task-03

# Output:
# task-03: 20260213/login-fix
# Status: backlog | Owner: (none)
# ---
# (no comments)
```

---

### show - Show Task Details

```bash
emx-note task show <task_id>
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `task_id` | Task ID (e.g., `task-01`) |

**Output format**:
```
ID:       task-01
Title:    Implement OAuth flow
Status:   doing
Owner:    @agent-1
File:     20260212/api-design
Comments: 3
```

**Examples**:
```bash
emx-note task show task-01
```

---

### find - Find Tasks by Note Reference

```bash
emx-note task find <node_ref>
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `node_ref` | Note reference to search for (supports partial match) |

**Output format**:
```
task-01  Implement OAuth flow  doing    @agent-1
task-05  Fix OAuth bug         backlog  (none)
```

**Examples**:
```bash
# Find tasks related to a note
emx-note task find 20260212/api

# Find tasks by partial reference
emx-note task find oauth
```

---

## Task Lifecycle

```bash
# 1. Add task (defined in reference section)
emx-note task add 20260212/api-design
# Result in reference section:
# [task-01]: 20260212/api-design
# Status: backlog (no owner)

# 2. Take task (moved to body section, agent takes ownership)
emx-note task take task-01 --title "Implement REST API" --header "## API"
# Result in body section under ## API:
# - [ ] [Implement REST API][task-01] @agent-1
# Status: doing (owned by @agent-1)

# 3. Comment progress (add sub-items with auto timestamp)
emx-note task comment task-01 "Drafted endpoints"
emx-note task comment task-01 "Implemented GET /users"
# Result:
# - [ ] [Implement REST API][task-01] @agent-1
#   - 2026-02-14 09:15 Drafted endpoints
#   - 2026-02-14 10:30 Implemented GET /users
# Status: doing (still owned by @agent-1)

# 4. Release task (incomplete - handoff to another agent)
emx-note task release task-01
# Result:
# - [ ] [Implement REST API][task-01]
# Status: doing (no owner, can be taken by another agent)

# 5. Another agent takes over
export EMX_AGENT_NAME=agent-2
emx-note task take task-01
# Result:
# - [ ] [Implement REST API][task-01] @agent-2
# Status: doing (owned by @agent-2)

# 6. Complete and release
emx-note task comment task-01 "All endpoints done"
emx-note task release task-01 --done
# Result:
# - [x] [Implement REST API][task-01]
# Status: done (no owner)
```

**Status transitions**:
- `backlog` → `doing` : via `task take`
- `doing` → `doing` (ownership change): via `release` + `take`
- `doing` → `done` : via `task release --done`
- `done` → `doing` : via `task take` (reopen)

**Ownership rules**:
- Only tasks without `@agent` marker can be taken
- `release` removes `@agent` marker, making task available
- `release --done` also marks task as complete

---

## Workflow Examples

### Basic Task Lifecycle

```bash
# 1. Add new task
emx-note task add 20260212/db-optimization

# 2. Take the task
emx-note task take task-01 --title "Optimize query performance" --header "## Query Engine"

# 3. Comment progress
emx-note task comment task-01 "Added indexes to user table"
emx-note task comment task-01 "Reduced query time by 50%"

# 4. Complete and release
emx-note task release task-01 --done

# 5. View log
emx-note task log task-01
```

### Agent Handoff

```bash
# Agent 1 works on task
export EMX_AGENT_NAME=agent-1
emx-note task add 20260212/api-design
emx-note task take task-01 --title "Design REST API" --header "## API"
emx-note task comment task-01 "Drafted endpoints"

# Agent 1 releases (incomplete, handoff)
emx-note task release task-01

# Agent 2 takes over
export EMX_AGENT_NAME=agent-2
emx-note task take task-01
emx-note task comment task-01 "Implemented GET /users"
emx-note task release task-01 --done
```

---

## Implementation Notes

1. **Auto-increment ID**: Task IDs are generated sequentially (task-01, task-02, ...)
2. **PREFIX**: Configured in frontmatter, defaults to `task-`
3. **Manual deletion**: Tasks are removed by editing TASK.md directly
4. **Header validation**: `target_header` must exist, command fails if not found
5. **Agent awareness**: All commands respect `EMX_AGENT_NAME` for `@` tagging
6. **Reference preservation**: `[task-NN]: node_ref` definitions are never auto-removed

---

## Output Behavior

Following emx-note conventions:

**stdout** (for programmatic use):
| Command | Success Output |
|---------|----------------|
| `add` | task-id (e.g., `task-01`) |
| `take` | task-id |
| `comment` | (silent - no output) |
| `release` | (silent - no output) |
| `list` | Table or oneline format |
| `show` | Key-value format |
| `log` | Structured log output |
| `find` | Matching tasks list |

**stderr** (for informational messages):
| Command | Message Type |
|---------|--------------|
| `take` | Confirmation: "Took task-01" |
| `comment` | Confirmation: "Added comment to task-01" |
| `release` | Confirmation: "Released task-01" |
| All | Errors with hints |

**Exit codes**:
- `0`: Success
- `1`: Error (see stderr for details)

**Dry-run behavior**:
- All `--dry-run` outputs go to stdout
- No modifications made to files
- Exit code always `0`

---

## Error Handling

| Command | Error Condition | Message + Hint |
|---------|-----------------|----------------|
| `take` | Task already has `@agent` marker | `Error: task-01 already taken by @agent-1`<br>`Hint: Use 'task release task-01' if you are @agent-1, or wait for release` |
| `take` | Task ID not found | `Error: task-99 not found`<br>`Hint: Use 'task list' to see available tasks` |
| `take` | Header not found | `Error: header '## Unknown' not found`<br>`Hint: Use existing headers or omit --header` |
| `comment` | Task ID not found | `Error: task-99 not found` |
| `comment` | Task not in body | `Error: task-01 not in body section`<br>`Hint: Use 'task take task-01' first` |
| `release` | Task ID not found | `Error: task-99 not found` |
| `release` | Task has no `@agent` marker | `Error: task-01 has no owner to release` |
| `release` | `--force` with multiple tasks | `Error: --force only works with single task` |
| `add` | node_ref not found | `Error: note 'xxx' not found`<br>`Hint: Check note reference format (timestamp, slug, etc.)` |
| `show` | Task ID not found | `Error: task-99 not found` |
| `log` | Task ID not found | `Error: task-99 not found` |
| `find` | No matches | `No tasks found matching 'xxx'` |

---

**Status**: Simplified design v2.0 - ready for implementation
