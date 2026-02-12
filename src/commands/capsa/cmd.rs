use std::path::Path;

use crate::cli::CapsaCommand;

pub fn run(home: &Path, cmd: CapsaCommand) -> std::io::Result<()> {
    match cmd {
        CapsaCommand::List => super::list::run(home),
        CapsaCommand::Create { name } => super::create::run(home, &name),
        CapsaCommand::Info { name } => {
            super::info::run(home, &name);
            Ok(())
        }
        CapsaCommand::Delete { name } => super::delete::run(home, &name),
    }
}
