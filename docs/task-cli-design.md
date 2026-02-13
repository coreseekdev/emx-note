# task Command Design

## CLI Overview

```
emx-note task <operation> [options] [arguments]

Operations:
  new       - Create new task
  list      - List all tasks
  show      - Show task details
  done      - Mark task as complete
  log       - Add execution log entry
  artifact   - Link intermediate result
  help      - Show this help
```

## Subcommands

### new - Create Task

```bash
emx-note task new "Implement authentication system" \
  --assigned @myself \
  --priority P0 \
  --depends #[task-db-schema] \
  --related "#jwt #security"
```

**Creates**:
- TASK.md entry: `- [ ] [task description][task-id]`
- Task detail file if `--file` specified

**Options**:
| Option | Short | Argument | Description |
|--------|-------|----------|-------------|
| `--assigned` | `-a` | agent-id | Assign task to agent |
| `--priority` | `-p` | level | Priority: P0 (highest), P1, P2 |
| `--depends` | `-d` | task-id | Dependency on another task |
| `--related` | `-r` | tags | Related concept/tag references |
| `--file` | `-f` | path | Task detail file location (default: ./tasks/{id}.md) |

### list - List Tasks

```bash
# List all tasks
emx-note task list

# List only pending
emx-note task list --status "[ ]"

# List assigned to agent
emx-note task list --assigned @myself

# Output in JSON
emx-note task list --json
```

**Output format**:
```
## [ ] Task description @agent-name
```

### show - Show Task Details

```bash
# Show task details
emx-note task show task-id

# Open task detail file
emx-note task show task-id --edit

# Resolve reference (opens ./path/to/file.md)
emx-note task show task-id --resolve
```

### done - Complete Task

```bash
# Mark as complete (checkbox: [ ] -> [x])
emx-note task done task-id

# With completion note
emx-note task done task-id \
  --duration "3h" \
  --harvest "Connection pooling reduces overhead"

# Mark multiple tasks
emx-note task done task-1 task-2 task-3
```

### log - Add Execution Log

```bash
# Add log entry to task
emx-note task log task-id "Investigated JWT vs OAuth"

# Add with file output
emx-note task log task-id "Debugged authentication flow" --file ./debug.md

# Add with link to result
emx-note task log task-id "Found optimal max_conn=50" --link ./conn-pool-report.md
```

**Log entry format**: `- [log-entry-text] [timestamp]`

### artifact - Link Intermediate Result

```bash
# Link intermediate result
emx-note task artifact task-id "connection-pool-report.md"

# Link multiple results
emx-note task artifact task-id \
  ./conn-pool-report.md \
  ./auth-design-notes.md \
  ./bench-results.md
```

### help - Show Help

```bash
emx-note task help
```

**Topics**:
- Task lifecycle: new -> list -> show -> done
- Execution tracking: log, artifact
- Output formats: text, json
- Agent coordination: @assigned flag
```

## Integration with TASK.md

### Parser Rules

1. **Line-based**: Each task entry starts with `- [STATE] [task-id][task-ref]`
2. **Checkbox parsing**: `[ ]` = UNDONE, `[x]` = DONE
3. **Agent detection**: `@agent-name` in same line
4. **Sub-item detection**: Indented lines under task entry

### Parser Implementation

```rust
// Pseudo-code for reference
enum TaskState {
    Undone = "[ ]",
    Done = "[x]",
}

fn parse_task_line(line: &str) -> Option<TaskEntry> {
    // Extract state, task-id, agent, refs, sub-items
}

fn parse_sub_item(line: &str) -> SubItem {
    // Parse log entries, artifacts
}
```

### Error Handling

| Error | Condition | Handling |
|------|----------|-------------|
| Task not found | Search TASK.md, fallback to resolve by task-id |
| Invalid task-id | Suggest similar IDs from TASK.md |
| Invalid agent | Default to current agent or @myself |
| Invalid state | Only `[ ]` or `[x]` allowed |

### Examples

```bash
# Create and start working
emx-note task new "Fix database bug" --assigned @myself
emx-note task start task-1

# Add execution log
emx-note task log task-1 "Reproduced bug in local environment"
emx-note task log task-1 "Root cause: connection pool exhaustion"
emx-note task log task-1 "Solution: Implement pooling layer"

# Mark as done
emx-note task done task-1 --duration "2h" --harvest "Added pooling, fixed exhaustion issue"

# Output
## Tasks

- [x] [Fix database bug][task-1] @myself
  - [task-1-log-entry-1](./tasks/task-1/log.md)
  - [task-1-solution](./tasks/task-1/solution.md)
```

---

**Status**: Design draft for review
