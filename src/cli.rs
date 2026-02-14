use clap::{Parser, Subcommand};

/// emx-note - A Zettelkasten-style note management tool
///
/// # LLM Agent Quick Reference
///
/// ## Basic Notes
///
/// ```bash
/// emx-note note "My Idea"           # Create permanent note
/// emx-note note                    # Create with timestamp title
/// emx-note note "Idea" -s "book" # Create in note/{hash}/
/// emx-note print "Idea"           # Print note content
/// emx-note resolve "Idea"          # Get file path
///
/// # From stdin/heredoc:
/// emx-note note "Idea" <<EOF
/// # My Idea
///
/// Detailed description here...
/// EOF
/// ```
///
/// ## Daily Notes
///
/// ```bash
/// emx-note daily                   # Today's daily note
/// emx-note daily "Meeting Notes"    # With title
/// emx-note list "#daily"           # List all dates
///
/// # From stdin/heredoc:
/// emx-note daily "Meeting" <<EOF
/// ## Attendees
/// - Alice
/// - Bob
///
/// ## Notes
/// Discussion points...
/// EOF
/// ```
///
/// ## Tags
///
/// ```bash
/// emx-note tag add "Idea" rust programming
/// emx-note tag remove "Idea" rust
/// emx-note tag add "Note" tag --force    # Force: apply to all matches
/// emx-note list "#rust"                # List notes with #rust tag
/// emx-note list "#rust" --json         # JSON output for scripting
/// ```
///
/// ## List
///
/// ```bash
/// emx-note list                           # List top-level files in note/
/// emx-note list "#tag"                    # List tag contents (grouped by date)
/// emx-note list "#daily"                  # List daily dates
/// emx-note list "a1b2c3d4e5f6"         # List files in hash directory
/// emx-note list "20250113"               # List files in daily/YYYYMMDD/
/// ```
///
/// ## Metadata
///
/// ```bash
/// emx-note meta "Idea" status "in-progress"
/// emx-note meta "Idea" status      # Get metadata value
/// emx-note meta "Idea" status --delete
/// ```
///
/// ## Capsae (Collections)
///
/// ```bash
/// emx-note capsa list                      # List all capsae
/// emx-note capsa create "work"            # Create new capsa
/// emx-note capsa create "blog" --path ~/blog  # Link to external directory
/// emx-note capsa resolve "work"           # Get capsa path
/// emx-note default "work"                 # Set default capsa
/// emx-note default                        # View current default
/// ```
///
/// ## Global Options
///
/// ```bash
/// emx-note --caps work list "#tags"    # Use specific capsa
/// emx-note --global list "#tags"        # Bypass agent prefixing
/// emx-note --home ~/notes list         # Use custom notes directory
/// ```
///
/// ## Environment Variables
///
/// - `EMX_NOTE_HOME`: Base directory for all capsae (default: ~/.emx-notes)
/// - `EMX_NOTE_DEFAULT`: Default capsa name (overrides .default)
/// - `EMX_AGENT_NAME`: Agent name for prefixing (e.g., "agent1" → "agent1-work")
///
/// ## Agent Prefixing
///
/// When `EMX_AGENT_NAME` is set, capsa names are automatically prefixed:
/// - "my-notes" becomes "agent1-my-notes"
/// - ".default" becomes "agent1" (agent's personal default)
/// Use `--global` flag to bypass prefixing for shared capsae
///
/// ## Scripting/JSON Mode
///
/// ```bash
/// emx-note --json list "#tag"      # Date-grouped JSON:
///                                  # {"2025-01-15": ["20250115/hello", "note"]}
/// emx-note --json list "#daily"    # Flat JSON array:
///                                  # ["20250101", "20250213"]
/// emx-note --json capsa list       # List capsae as JSON:
///                                  # ["default", "work", "personal"]
/// ```
///
/// ## Note Resolution
///
/// Notes can be referenced by:
/// - Exact name: "My Note"
/// - Timestamp: "20250113120000" (daily: "20250113/120000")
/// - Prefix: "My" (first match starting with "My")
/// - Hash: "a1b2c3d4e5f6" (source hash directory)
/// - Relative path: "daily/20250113/120000-meeting.md"
///
#[derive(Parser, Debug)]
#[command(name = "emx-note")]
#[command(author = "nzinfo <li.monan@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "A Zettelkasten-style note management tool")]
pub struct Cli {
    /// Home directory for all notes (default: ~/.emx-notes or $EMX_NOTE_HOME)
    #[arg(long, value_name = "PATH")]
    pub home: Option<String>,

    /// Global operation (bypasses agent prefixing)
    #[arg(short = 'g', long, global = true)]
    pub global: bool,

    /// Name of the capsa (note collection)
    #[arg(short, long, alias = "vault", global = true, value_name = "CAPSA")]
    pub caps: Option<String>,

    /// Output in JSON format (for scripting/LLM usage)
    #[arg(short = 'j', long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or open today's daily note
    Daily {
        /// Optional title for daily note
        title: Option<String>,
    },

    /// Create a permanent note (in note/ directory)
    Note {
        /// Optional title for note (defaults to timestamp if not provided)
        title: Option<String>,

        /// Source of the note (creates in note/{hash}/ subdirectory if provided)
        #[arg(short = 's', long)]
        source: Option<String>,
    },

    /// Resolve note reference to file path
    #[command(alias = "rv")]
    Resolve {
        /// Note reference (name, timestamp, or date/prefix)
        note_name: String,
    },

    /// List notes
    #[command(alias = "ls")]
    List {
        /// Filter: #tag, date hash, or leave empty for note/ root
        filter: Option<String>,
    },

    /// Print note content to stdout
    #[command(alias = "p")]
    Print {
        /// Note name or relative path
        note_name: String,
    },

    /// Manage note metadata (YAML frontmatter)
    #[command(alias = "m")]
    Meta {
        /// Note reference
        note_ref: String,

        /// Key to get/set/delete (supports nested keys like "meta.status")
        key: Option<String>,

        /// Values to set (single value = string, multiple values = array)
        value: Vec<String>,

        /// Delete the specified key
        #[arg(long)]
        delete: bool,
    },

    /// Manage capsae (note collections)
    #[command(subcommand)]
    Capsa(CapsaCommand),

    /// Get or set the default capsa
    Default {
        /// Capsa name to set as default (optional, prints current if not provided)
        caps: Option<String>,
    },

    /// Find and manage orphaned notes (no incoming links)
    /// NOTE: This command is not yet fully implemented
    #[command(hide = true)]
    Gc {
        /// Minimum age in days for notes to be considered for GC
        #[arg(short, long, default_value = "7")]
        days: u32,

        /// Actually delete the orphaned notes (default is dry-run)
        #[arg(short, long)]
        execute: bool,

        /// Skip confirmation prompt when deleting
        #[arg(short = 'f', long)]
        force: bool,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Manage tags (#xxxx.md files)
    #[command(subcommand)]
    Tag(TagCommand),

    /// Check and manage links between notes
    #[command(subcommand)]
    Link(LinkCommand),

    /// Manage tasks in TASK.md
    #[command(subcommand)]
    Task(TaskCommand),
}

#[derive(Subcommand, Debug)]
pub enum LinkCommand {
    /// Check for broken local links
    Check {
        /// Path to check (default: current capsa root)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// List all local links
    List {
        /// Path to scan (default: current capsa root)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Find orphaned files (not linked by any other file)
    Orphans {
        /// Path to scan (default: current capsa root)
        #[arg(short, long)]
        path: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagCommand {
    /// Add tags to a note
    Add {
        /// Note reference (supports resolve/print format)
        note_ref: String,

        /// Tags to add (without # prefix)
        #[arg(required = true)]
        tags: Vec<String>,

        /// Force: apply to all matching notes if ambiguous
        #[arg(short, long)]
        force: bool,
    },

    /// Remove tags from a note
    Remove {
        /// Note reference (supports resolve/print format)
        note_ref: String,

        /// Tags to remove (without # prefix)
        #[arg(required = true)]
        tags: Vec<String>,

        /// Force: apply to all matching notes if ambiguous
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum CapsaCommand {
    /// List all capsae
    List,

    /// Create a new capsa (or link to external directory)
    Create {
        /// Name for the new capsa
        name: String,

        /// Optional path to link to (creates a link capsa if provided)
        path: Option<String>,
    },

    /// Resolve a capsa to its actual path (useful for links)
    Resolve {
        /// Name of the capsa to resolve
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    /// Add a new task (auto-increment ID)
    Add {
        /// Note reference (timestamp, date/prefix, or slug)
        node_ref: String,
    },

    /// Take ownership of a task
    Take {
        /// Task ID (e.g., task-01)
        task_id: String,

        /// Task description (optional, defaults to note's title)
        #[arg(long)]
        title: Option<String>,

        /// Target header in body section (optional)
        #[arg(long)]
        header: Option<String>,

        /// Preview result without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Add comment to task
    Comment {
        /// Task ID (e.g., task-01)
        task_id: String,

        /// Comment message
        message: String,

        /// Attach git commit hash
        #[arg(long)]
        git: Option<String>,

        /// Preview result without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Release task ownership
    Release {
        /// Task IDs to release
        #[arg(required = true)]
        task_ids: Vec<String>,

        /// Mark task(s) as done before release
        #[arg(long)]
        done: bool,

        /// Force release even if not owner (single task only)
        #[arg(long)]
        force: bool,

        /// Preview result without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// List tasks by status
    List {
        /// Status filter: backlog, doing, done, all (default: all)
        status: Option<String>,

        /// Output only task IDs, one per line
        #[arg(long)]
        oneline: bool,

        /// Filter by owner (@agent-name or "(none)")
        #[arg(long)]
        owner: Option<String>,
    },

    /// Show task details
    Show {
        /// Task ID (e.g., task-01)
        task_id: String,
    },

    /// Show execution log for a task
    Log {
        /// Task ID (e.g., task-01)
        task_id: String,
    },

    /// Find tasks by note reference
    Find {
        /// Note reference to search for
        node_ref: String,
    },
}
