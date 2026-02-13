# TASK.md Format Specification

Version: 1.0
Status: Draft

## Overview

This document defines the format for `TASK.md` files used for task tracking within a capsa (note collection). The format leverages standard Markdown reference-style links to provide task management with minimal syntax extension.

## Design Principles

1. **Markdown-native**: Uses only standard Markdown syntax, no custom markup
2. **Reference-oriented**: Task details are stored in separate note files, linked via reference definitions
3. **Unified definitions**: All reference definitions grouped at end of document for easy maintenance
4. **Checkbox-driven**: Status indicated with Markdown-compatible checkbox symbols
5. **Human-readable**: Both humans and AI agents can parse and understand the format
6. **Version-control friendly**: All changes tracked through git

## File Structure

A `TASK.md` file consists of three main sections:

1. **Header section**: File metadata and configuration (optional)
2. **Tasks section**: Single list with checkbox status ([ ] or [x])
3. **Reference definitions section**: ALL link definitions after last horizontal rule

### Section Format

```markdown
# TASK.md

[Metadata]

---

## Tasks

- [ ] [Pending task][task-id]
  metadata-key: value

- [.] [In progress task][task-id]
  metadata-key: value

- [x] [Completed task][task-id]
  metadata-key: value
---

## Reference Definitions

[task-id]: ./path/to/task-detail.md
[concept-id]: ./path/to/concept-note.md
```

**Separator rule**: Horizontal rule (`---`, `___`, etc.) must separate metadata from tasks section.
**Purpose**: Clear visual boundary between file configuration and actionable content.

## Task States

### Two-State System

| Symbol | Meaning | Description |
|--------|----------|-------------|
| `[ ]` | UNDONE | Task not yet started |
| `[x]` | DONE | Task has been completed |

### Rationale

- **Minimal**: Only two states needed for task tracking
- **Clear transition**: UNDONE -> DONE is one-way
- **Markdown-native**: Standard checkbox syntax
- **No sub-status**: Task changes become standard list items


### Task Display Syntax

```markdown
## IN_PROGRESS

- [.] [Implement user authentication][task-auth]
  assigned: @agent-1
  started: 2025-01-15
  priority: P0
  depends: #[db-schema-update]
  related: #[jwt-concept] #session-mgmt
  estimate: 4h

## DONE

- [x] [Fix database connection pool][task-db-pool]
  assigned: @agent-2
  completed: 2025-01-13
  duration: 3h
  reviewed-by: @agent-1
```

## Metadata Fields

### Required Fields

| Field | Format | Description |
|--------|----------|-------------|
| `task-id` | `[task-xxx]` | Unique identifier for the task |

### Optional Fields

| Field | Format | Description |
|--------|----------|-------------|
| `assigned` | `@agent-id` | Agent or person assigned to the task |
| `started` | `YYYY-MM-DD` | Task start date |
| `completed` | `YYYY-MM-DD` | Task completion date |
| `estimated` | `Xh`, `Xd` | Time estimate for completion |
| `duration` | `Xh`, `Xd` | Actual time spent |
| `priority` | `P0`, `P1`, `P2` | Priority level (P0 = highest) |
| `depends` | `#[task-id]` | Dependency on another task |
| `blocked-by` | `#[task-id]` | Task that is blocking this one |
| `related` | `#[concept-id]` or `#tag` | Related concepts or tags |
| `reviewed-by` | `@agent-id` | Agent/person who reviewed the work |
| `location` | `./path/to/file.md` | Location of implementation notes |
| `reason` | `text` | Reason for blocked status (when `!] is used) |

## Reference Definitions

Reference definitions are placed at the end of `TASK.md` following Markdown standard:

```markdown
---

## Task References

### Authentication
[task-auth]: ./tasks/auth-implementation.md
[task-login]: ./tasks/login-flow.md

### Database
[task-db-pool]: ./tasks/database-pool-fix.md
[task-db-schema]: ./tasks/database-schema-update.md

### Concepts
[jwt-concept]: ./knowledge/jwt-tokens.md
[session-mgmt]: ./knowledge/session-management.md
```

#### Reference Naming Convention

- **Task references**: `task-{short-name}` (e.g., `task-auth`, `task-db-pool`)
- **Concept references**: `{category}-{name}` (e.g., `jwt-concept`, `db-connection`)
- **External references**: Use descriptive names that clarify context

## Multi-Agent Coordination (Optional)

For scenarios where multiple agents work on shared tasks:

```markdown
## Agent Registry

| Agent ID | Specialty | Current Task | Status | Updated |
|-----------|------------|---------------|---------|----------|
| @agent-1 | authentication | task-auth | PROGRESS | 2025-01-15T10:00Z |
| @agent-2 | database | task-db-pool | IDLE | 2025-01-15T09:30Z |
| @agent-3 | frontend | IDLE | - | - |

## Claiming Rules

1. Only agents with `IDLE` status may claim new tasks
2. `P0` priority tasks take precedence over `P1` and `P2`
3. Mark task as `[PROGRESS]` with `assigned: @agent-id` to claim
4. Mark task as `[DONE]` and set agent status to `IDLE` when complete

## Conflict Resolution

When multiple agents attempt to claim the same task:

1. First agent to mark `[PROGRESS]` wins
2. Second agent must either:
   - Choose a different task, or
   - Negotiate via `CONFLICT` status entry
```

## Complete Example

```markdown
# TASK.md

version: 1.0
updated: 2025-01-15T10:00:00Z

## Tasks

- [.] [Implement user authentication system][task-auth]
  assigned: @myself
  started: 2025-01-15
  priority: P0
  depends: #[db-schema-update]
  related: #[jwt-concept] #session-mgmt #security
  estimate: 8h
  location: ./work/auth-implementation.md

- [.] [Design database schema][task-db-schema]
  assigned: @agent-db
  started: 2025-01-14
  priority: P1
  estimate: 6h

- [x] [Fix connection pool exhaustion][task-pool-fix]
  assigned: @myself
  completed: 2025-01-13
  duration: 3h
  reviewed-by: @peer-1
  location: ./work/pool-fix-notes.md
  harvest: Connection pooling reduces overhead; found optimal max_connections=50

- [x] [Research authentication options][task-auth-research]
  assigned: @myself
  completed: 2025-01-10
  duration: 4h
  related: #[oauth2-concept] #jwt-concept
  harvest: Compared OAuth2 vs JWT; chose JWT for statelessness

- [ ] [Write integration tests][task-integration-test]
  priority: P1
  tags: testing #quality-assurance
  estimate: 12h

- [ ] [Update API documentation][task-docs]
  priority: P2
  tags: documentation

- [!] [Implement caching layer][task-cache]
  blocked-by: #[cache-design-doc]
  reason: Waiting for architecture decision

## Reference Definitions

### Design Documents
[cache-design-doc]: ./designs/cache-architecture.md

### Knowledge
[jwt-concept]: ./knowledge/jwt-authentication.md
[oauth2-concept]: ./knowledge/oauth2-framework.md
[session-mgmt]: ./knowledge/session-management.md

---

### Tasks

[task-auth]: ./tasks/authentication-system.md
[task-auth-research]: ./research/authentication-options.md
[task-db-schema]: ./tasks/database-schema-design.md
[task-pool-fix]: ./tasks/connection-pool-fix.md
[task-integration-test]: ./tasks/integration-testing.md
[task-docs]: ./tasks/api-documentation.md
[task-cache]: ./tasks/caching-layer.md

```

## Parsing Rules for Implementations

1. **Line-based parsing**: Each task entry starts with `- [SYMBOL]`
2. **Checkbox recognition**: Parse `[ ]`, `[.]`, `[x]`, `[!]`, `[?]` as status
3. **Metadata extraction**: Parse key-value pairs from indented lines following task entry
4. **Reference resolution**: Extract `[task-id]` from link syntax
5. **Date format**: Support ISO 8601 (`YYYY-MM-DD`) and simple date formats
6. **Duration format**: Parse `Xh` (hours) or `Xd` (days) for time tracking
7. **Tag extraction**: Collect hashtags (e.g., `#tag`) from `related` field

## Workflow Convention (b4-style)

For tracking task progress with command operations:

### Start Work

```bash
# Mark task as in progress
emx-note task start task-1
# Equivalent checkbox: [ ] -> [.]
```

### Record Progress

```bash
# Add comment/note to task
emx-note task comment task-1 "Investigated authentication options"

# Result: Appends to task detail note
```

### Complete Work

```bash
# Mark task as complete
emx-note task finish task-1
# Equivalent checkbox: [.] -> [x]
```

### Command Reference

| Operation | Checkbox Change | Example |
|-----------|----------------|---------|
| `start` | `[ ]` -> `[.]` | `emx-note task start task-auth` |
| `finish` | `[.]` -> `[x]` | `emx-note task finish task-auth` |
| `block` | `[.]` -> `[!]` | `emx-note task block task-auth --reason "Waiting for design"` |
| `comment` | No change | `emx-note task comment task-auth "Need to add tests"` |

**Design rationale**:
- Commands map naturally to workflow transitions
- Minimal typing for common operations
- History tracking through git commits
- No need to edit TASK.md directly for status changes

---

## Command Interface Specification

### List Command

```bash
# List all tasks, grouped by status
emx-note task list

# List only specific status (by checkbox)
emx-note task list --status .
emx-note task list --status x
emx-note task list --status !

# Search tasks by pattern
emx-note task list --pattern "database"

# List tasks assigned to agent
emx-note task list --assigned @agent-1

# Output in JSON format
emx-note task list --json
```

### Start/Finish Commands

```bash
# Start working on a task ( [ ] -> [.])
emx-note task start task-id

# Finish a task ([.] -> [x])
emx-note task finish task-id

# Block a task ([.] -> [!])
emx-note task block task-id --reason "External dependency"

# Add comment to task (no checkbox change)
emx-note task comment task-id "Added error handling"
```

### Create Command

```bash
# Create new task with description
emx-note task create "Implement caching layer" \
  --priority P1 \
  --tags performance \
  --assign @myself

# Creates: task-cache.md and adds entry to TASK.md
```

### Update Command

```bash
# Mark task as in progress (checkbox .)
emx-note task update task-cache --status .

# Mark task as complete (checkbox x)
emx-note task update task-cache --status x \
  --duration "6h" \
  --harvest "Implemented Redis-based caching"

# Mark task as blocked (checkbox !)
emx-note task update task-cache --status ! \
  --reason "Waiting for architecture decision"

# Add dependency to task
emx-note task update task-cache --depends #[task-db-schema]
```

### Stats Command

```bash
# Show task statistics
emx-note task stats

# Output example:
# Tasks: 8 total (2 IN_PROGRESS, 3 DONE, 2 PENDING, 1 BLOCKED)
# Avg completion time: 5.2h
# Oldest pending: 14 days
```

## Integration with Existing Commands

The `task` command integrates with existing emx-note functionality:

- **`note` command**: Create task detail notes as standard notes
- **`link` command**: Validate task references in TASK.md
- **`list` command**: Filter notes by task-related tags
- **`resolve` command**: Quick navigation to task detail files

## Migration Path

For projects without `TASK.md`:

1. Run initialization: `emx-note task init`
2. Command scans existing notes for task patterns
3. Generates `TASK.md` with `PENDING` status for found tasks
4. User can organize and add metadata manually
