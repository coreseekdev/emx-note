use clap::Parser;
use emx_note::{Cli, notes_path};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let home = notes_path(cli.home.as_deref());

    match cli.command {
        emx_note::Command::Capsa(cmd) => emx_note::commands::capsa::run(&home, cmd),
        emx_note::Command::Note(cmd) => emx_note::commands::note::run(&home, cmd),
    }
}
