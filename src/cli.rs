use clap::{Parser, Subcommand};

/// emx-note - A Zettelkasten-style note management tool
#[derive(Parser, Debug)]
#[command(name = "emx-note")]
#[command(author = "nzinfo <li.monan@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "A Zettelkasten-style note management tool", long_about = None)]
pub struct Cli {
    /// Name of the caps (note collection)
    #[arg(short, long, default_value = "default")]
    pub caps: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all capsae (note collections)
    List,

    /// Create a new capsa
    Create {
        /// Name for the new capsa
        name: String,
    },

    /// Show info about the current capsa
    Info,
}
