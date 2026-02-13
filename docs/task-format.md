# TASK.md Format Specification

Version: 1.0
Status: Stable

## Overview

Minimalist task tracking format using standard Markdown syntax.

## Design Principles

1. **Markdown-native**: Uses only standard Markdown syntax, no custom markup
2. **Checkbox-driven**: Status indicated with `[ ]` or `[x]`
3. **Reference-oriented**: Task details in separate note files, linked via definitions
4. **Flat structure**: Single task list, flat reference list, minimal metadata

## File Structure

```markdown
# TASK.md

[Metadata]

---

## Tasks

- [ ] [Task description][task-id]
- [x] [Completed task][task-id]

---

## Reference Definitions

[task-id]: ./path/to/task-detail.md
[concept-id]: ./path/to/concept-note.md
```

**Flat list**: All definitions in one section for easy maintenance.
