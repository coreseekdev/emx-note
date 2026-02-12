use clap::Parser;
use emx_note::{Cli, Command, ResolveContext, notes_path};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let home_path = notes_path(cli.home.as_deref());
    let ctx = ResolveContext::new(home_path, cli.global);

    match cli.command {
        Command::Open { note_name } => cmd::open::run(&ctx, cli.caps.as_deref(), note_name),
        Command::Daily { title } => cmd::daily::run(&ctx, cli.caps.as_deref(), title),
        Command::Create { note_name, content, append, overwrite, open } => {
            cmd::create::run(&ctx, cli.caps.as_deref(), note_name, content, append, overwrite, open)
        }
        Command::Move { current, new, open } => {
            cmd::r#move::run(&ctx, cli.caps.as_deref(), current, new, open)
        }
        Command::Delete { note_path } => cmd::delete::run(&ctx, cli.caps.as_deref(), note_path),
        Command::List { path } => cmd::list::run(&ctx, cli.caps.as_deref(), path),
        Command::Print { note_name, mentions } => {
            cmd::print::run(&ctx, cli.caps.as_deref(), note_name, mentions)
        }
        Command::Search => cmd::search::run(&ctx, cli.caps.as_deref()),
        Command::SearchContent { search_term } => {
            cmd::search_content::run(&ctx, cli.caps.as_deref(), search_term)
        }
        Command::FrontMatter { note_name, action } => {
            cmd::frontmatter::run(&ctx, cli.caps.as_deref(), note_name, action)
        }
        Command::Capssa(cmd) => cmd::capsa::run(&ctx, cmd),
        Command::SetDefault { caps } => cmd::set_default::run(&ctx, caps),
        Command::PrintDefault { path_only } => cmd::print_default::run(&ctx, path_only),
    }
}

mod cmd {
    pub mod open;
    pub mod daily;
    pub mod create;
    pub mod r#move;
    pub mod delete;
    pub mod list;
    pub mod print;
    pub mod search;
    pub mod search_content;
    pub mod frontmatter;
    pub mod capsa;
    pub mod set_default;
    pub mod print_default;
}
