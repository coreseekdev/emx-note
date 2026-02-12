use clap::Parser;
use emx_note::{Cli, Command, ResolveContext, notes_path};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let home_path = notes_path(cli.home.as_deref());
    let ctx = ResolveContext::new(home_path, cli.global);

    match cli.command {
        Command::Daily { title } => cmd::daily::run(&ctx, cli.caps.as_deref(), title),
        Command::Note { title, source } => cmd::note::run(&ctx, cli.caps.as_deref(), title, source),
        Command::Create { note_name, content, overwrite } => {
            cmd::create::run(&ctx, cli.caps.as_deref(), note_name, content, overwrite)
        }
        Command::Move { current, new } => {
            cmd::r#move::run(&ctx, cli.caps.as_deref(), current, new)
        }
        Command::Copy { source, dest, overwrite } => {
            cmd::copy::run(&ctx, cli.caps.as_deref(), source, dest, overwrite)
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
        Command::Capsa(cmd) => cmd::capsa::run(&ctx, cmd),
        Command::SetDefault { caps } => cmd::set_default::run(&ctx, caps),
        Command::PrintDefault { path_only } => cmd::print_default::run(&ctx, path_only),
        Command::Gc { days, execute, force, verbose } => {
            cmd::gc::run(&ctx, cli.caps.as_deref(), days, execute, force, verbose)
        }
        Command::Tag(tag_cmd) => cmd::tag::run(&ctx, cli.caps.as_deref(), tag_cmd),
        Command::Label(tag_cmd) => cmd::tag::run(&ctx, cli.caps.as_deref(), tag_cmd),
    }
}

mod cmd {
    pub mod daily;
    pub mod note;
    pub mod create;
    pub mod r#move;
    pub mod copy;
    pub mod delete;
    pub mod list;
    pub mod print;
    pub mod search;
    pub mod search_content;
    pub mod frontmatter;
    pub mod capsa;
    pub mod set_default;
    pub mod print_default;
    pub mod tag;
    pub mod gc;
    pub mod resolve;
}
