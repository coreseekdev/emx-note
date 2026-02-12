# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build the project
cargo build

# Run all tests (unit tests + E2E tests)
cargo test

# Run a specific E2E test file
cargo test -- test_e2e --nocapture
# E2E tests are defined in tests/*.txtar files using the emx-testspec framework
```

## Architecture Overview

emx-note is a Zettelkasten-style note management CLI tool. The codebase is split into a library (`src/lib.rs`) and binary (`src/main.rs`).

### Core Concepts

- **Capsa** (Latin for "box"): A note collection/vault. Similar to Obsidian's vault concept.
- **Default location**: `~/.emx-notes/` (configurable via `--home` flag or `$EMX_NOTE_HOME`)
- **Agent prefixing**: When `$EMX_AGENT_NAME` is set, capsa operations are automatically prefixed (e.g., `my-notes` becomes `agent1-my-notes`)

### Key Modules

- **`src/resolve.rs`**: Core resolution logic for capsa paths with multi-level priority:
  1. `$EMX_NOTE_DEFAULT` environment variable
  2. `$EMX_AGENT_NAME` (agent's default capsa is the agent name itself)
  3. Hardcoded `.default` directory

  Handles agent prefixing, link files (INI-style external directories), and global vs agent-scoped operations.

- **`src/cli.rs`**: CLI definition using clap derive macros. All commands and subcommands are defined here.

- **`src/cmd/`**: Individual command implementations. Each command module has a `run()` function that takes a `ResolveContext`.

### Resolution Flow

1. `ResolveContext::new(home, global)` reads environment variables
2. `resolve_capsa(name)` applies agent prefix and resolves to `CapssaRef`
3. `CapssaRef` contains the actual path (may be external if it's a link file)

### Special Behaviors

- **Agent prefix bypass**: Use `-g/--global` flag to skip agent prefixing
- **Link files**: Capsa can be a file (not directory) containing `[link]\ntarget = /path`
- **Tag files**: Stored as `#xxxx.md` in capsa root, with notes grouped by date
- **Daily notes**: Created in `daily/YYYYMMDD/HHMMSS[-title].md` with a link file `daily/YYYYMMDD.md`

### Testing

E2E tests use `emx-testspec` (in sibling directory `../emx-testspec`), inspired by Go's testscript. Test files are `.txtar` format in `tests/`. The test runner executes commands and asserts on stdout/stderr/file existence.
