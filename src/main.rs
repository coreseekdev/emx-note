use clap::Parser;
use emx_note::{Cli, Command, ResolveContext, notes_path};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let home_path = notes_path(cli.home.as_deref());
    let ctx = ResolveContext::new(home_path, cli.global);

    match cli.command {
        Command::Daily { title } => cmd::daily::run(&ctx, cli.caps.as_deref(), title),
        Command::Note { title, source } => cmd::note::run(&ctx, cli.caps.as_deref(), title, source),
        Command::Resolve { note_name } => cmd::note_resolve::run(&ctx, cli.caps.as_deref(), note_name),
        Command::List { filter } => cmd::list::run(&ctx, cli.caps.as_deref(), filter),
        Command::Print { note_name } => {
            cmd::print::run(&ctx, cli.caps.as_deref(), note_name)
        }
        Command::Meta { note_ref, key, value, delete } => {
            cmd::meta::run(&ctx, cli.caps.as_deref(), note_ref, key, value, delete)
        }
        Command::Capsa(cmd) => cmd::capsa::run(&ctx, cmd),
        Command::Default { caps } => cmd::default::run(&ctx, caps),
        Command::Gc { days, execute, force, verbose } => {
            cmd::gc::run(&ctx, cli.caps.as_deref(), days, execute, force, verbose)
        }
        Command::Tag(tag_cmd) => cmd::tag::run(&ctx, cli.caps.as_deref(), tag_cmd),
    }
}

mod cmd {
    pub mod daily;
    pub mod note;
    pub mod note_resolve;
    pub mod list;
    pub mod print;
    pub mod meta;
    pub mod capsa;
    pub mod default;
    pub mod tag;
    pub mod gc;
    pub mod resolve;
}
