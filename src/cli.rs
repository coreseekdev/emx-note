use clap::{Parser, Subcommand};

/// emx-note - A Zettelkasten-style note management tool
#[derive(Parser, Debug)]
#[command(name = "emx-note")]
#[command(author = "nzinfo <li.monan@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "A Zettelkasten-style note management tool", long_about = None)]
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

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or open today's daily note
    Daily {
        /// Optional title for the daily note
        title: Option<String>,
    },

    /// Create a permanent note (in note/ directory)
    Note {
        /// Optional title for the note (defaults to timestamp if not provided)
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
