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

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage capsae (note collections)
    #[command(subcommand)]
    Capsa(CapsaCommand),

    /// Manage notes within a capsa
    #[command(subcommand)]
    Note(NoteCommand),
}

#[derive(Subcommand, Debug)]
pub enum CapsaCommand {
    /// List all capsae (note collections)
    List,

    /// Create a new capsa
    Create {
        /// Name for the new capsa
        name: String,
    },

    /// Show info about a capsa
    Info {
        /// Name of the capsa (default: default)
        #[arg(short, long, default_value = "default")]
        name: String,
    },

    /// Delete a capsa
    Delete {
        /// Name of the capsa to delete
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum NoteCommand {
    /// List all notes in the current capsa
    List {
        /// Name of the capsa
        #[arg(short, long, default_value = "default")]
        capsa: String,
    },

    /// Create a new note
    Create {
        /// Name of the capsa
        #[arg(short, long, default_value = "default")]
        capsa: String,

        /// Title for the note
        title: String,
    },

    /// Edit a note
    Edit {
        /// Note ID or title to edit
        id: String,
    },

    /// View a note
    View {
        /// Note ID or title to view
        id: String,
    },
}
