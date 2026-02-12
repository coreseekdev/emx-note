use clap::{Parser, Subcommand};

use crate::resolve::DEFAULT_CAPSA_NAME;

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

    /// Create a new note
    Create {
        /// Note name (can include path)
        note_name: String,

        /// Initial content for the note ("-" to read from stdin, or reads from stdin by default)
        #[arg(short = 'C', long)]
        content: Option<String>,

        /// Overwrite existing note
        #[arg(short = 'W', long)]
        overwrite: bool,
    },

    /// Move/rename a note (updates all links)
    Move {
        /// Current note path
        current: String,

        /// New note path
        new: String,
    },

    /// Copy a note to a new location
    Copy {
        /// Source note path
        source: String,

        /// Destination note path
        dest: String,

        /// Overwrite destination if exists
        #[arg(short = 'W', long)]
        overwrite: bool,
    },

    /// Delete a note
    Delete {
        /// Note relative path
        note_path: String,
    },

    /// List notes in the capsa
    #[command(alias = "ls")]
    List {
        /// Relative path (default: root directory)
        path: Option<String>,
    },

    /// Print note content to stdout
    #[command(alias = "p")]
    Print {
        /// Note name or relative path
        note_name: String,

        /// Include backlink list
        #[arg(short, long)]
        mentions: bool,
    },

    /// Interactive fuzzy search for notes
    #[command(alias = "s")]
    Search,

    /// Search note content
    #[command(alias = "sc")]
    SearchContent {
        /// Search keyword
        search_term: String,
    },

    /// Manage note frontmatter
    #[command(alias = "fm")]
    FrontMatter {
        /// Note name
        note_name: String,

        #[command(subcommand)]
        action: FrontMatterAction,
    },

    /// Manage capsae (note collections)
    #[command(subcommand)]
    Capsa(CapsaCommand),

    /// Set the default capsa
    SetDefault {
        /// Capsa name
        caps: String,
    },

    /// Print default capsa info
    PrintDefault {
        /// Output path only
        #[arg(long)]
        path_only: bool,
    },

    /// Find and manage orphaned notes (no incoming links)
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

    /// Manage tags/labels (#xxxx.md files)
    #[command(subcommand)]
    Tag(TagCommand),

    /// Alias for tag command
    #[command(hide = true, subcommand)]
    Label(TagCommand),
}

#[derive(Subcommand, Debug)]
pub enum TagCommand {
    /// Add a note to a tag
    Add {
        /// Tag name (without # prefix)
        tag: String,

        /// Note path to tag
        note: String,
    },

    /// Remove a note from a tag
    Remove {
        /// Tag name (without # prefix)
        tag: String,

        /// Note path to untag
        note: String,
    },

    /// List all tags or notes in a tag
    List {
        /// Tag name to list notes (optional, lists all tags if not specified)
        tag: Option<String>,
    },

    /// Delete a tag file
    Delete {
        /// Tag name to delete
        tag: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum FrontMatterAction {
    /// Print frontmatter
    Print,

    /// Edit a key-value pair
    Edit {
        /// Key to edit (supports nested keys with dots)
        #[arg(short, long)]
        key: String,

        /// Value to set
        value: String,
    },

    /// Delete a key
    Delete {
        /// Key to delete
        #[arg(short, long)]
        key: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum CapsaCommand {
    /// List all capsae
    List,

    /// Create a new capsa
    Create {
        /// Name for the new capsa
        name: String,
    },

    /// Show info about a capsa
    Info {
        /// Name of the capsa (default: default)
        #[arg(short, long, default_value = DEFAULT_CAPSA_NAME)]
        name: String,
    },

    /// Delete a capsa
    Delete {
        /// Name of the capsa to delete
        name: String,
    },
}
