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
    /// Open a note in the default application
    Open {
        /// Note name or relative path
        note_name: String,
    },

    /// Create or open today's daily note
    Daily {
        /// Optional title for the daily note
        title: Option<String>,
    },

    /// Create a new note
    Create {
        /// Note name (can include path)
        #[arg(short, long)]
        content: Option<String>,

        /// Append to existing note
        #[arg(short, long)]
        append: bool,

        /// Overwrite existing note
        #[arg(short, long)]
        overwrite: bool,

        /// Open note after creating
        #[arg(short, long)]
        open: bool,

        /// Title for the note
        note_name: String,
    },

    /// Move/rename a note (updates all links)
    Move {
        /// Current note path
        current: String,

        /// New note path
        new: String,

        /// Open after moving
        #[arg(short, long)]
        open: bool,
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
    Capssa(CapssaCommand),

    /// Set the default capsa
    SetDefault {
        /// Capssa name
        caps: String,
    },

    /// Print default capsa info
    PrintDefault {
        /// Output path only
        #[arg(long)]
        path_only: bool,
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
pub enum CapssaCommand {
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
