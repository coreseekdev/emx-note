//! Task management command module
//!
//! Implements task tracking in TASK.md files with agent coordination.
//! Uses EditOp pattern for file modifications.

mod reader;
mod add;
mod take;
mod comment;
mod release;
mod list;
mod show;
mod log;
mod find;

pub use reader::{TaskFileReader, TaskStatus};

/// Main entry point
pub fn run(ctx: &emx_note::ResolveContext, caps: Option<&str>, cmd: emx_note::TaskCommand) -> std::io::Result<()> {
    let capsa = emx_note::CapsaEngine::new(super::resolve::resolve_capsa(ctx, caps)?);

    match cmd {
        emx_note::TaskCommand::Add { node_ref } => add::run(&capsa, &node_ref),
        emx_note::TaskCommand::Take { task_id, title, header, dry_run } => {
            take::run(&capsa, &task_id, title.as_deref(), header.as_deref(), dry_run)
        }
        emx_note::TaskCommand::Comment { task_id, message, git, dry_run } => {
            comment::run(&capsa, &task_id, &message, git.as_deref(), dry_run)
        }
        emx_note::TaskCommand::Release { task_ids, done, force, dry_run } => {
            release::run(&capsa, &task_ids, done, force, dry_run)
        }
        emx_note::TaskCommand::List { status, oneline, owner } => {
            list::run(&capsa, status.as_deref(), oneline, owner.as_deref())
        }
        emx_note::TaskCommand::Show { task_id } => show::run(&capsa, &task_id),
        emx_note::TaskCommand::Log { task_id } => log::run(&capsa, &task_id),
        emx_note::TaskCommand::Find { node_ref } => find::run(&capsa, &node_ref),
    }
}
