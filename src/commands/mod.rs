//! The commands, one module per command or group of closely related commands.

mod burndown;
mod edit;
mod modify;
mod new;
mod projects;
mod search;
mod subtask;

use anyhow::Result;

use crate::cli::TaigaCmd;
use crate::taiga::{Taiga, TaigaProject, TaigaTask, TaskPatch};

/// Runs a parsed command against a logged-in session.
///
/// # Errors
/// Returns the error of the step that failed.
pub fn run(cmd: TaigaCmd, mut taiga: Taiga) -> Result<()> {
    match cmd {
        TaigaCmd::Default => projects::list(&taiga),
        TaigaCmd::Projects => projects::refresh(&mut taiga)?,
        TaigaCmd::ProjectUsers(args) => projects::users(&mut taiga, args)?,
        TaigaCmd::NewTask(args) => new::new_task(&mut taiga, args)?,
        TaigaCmd::MoveTask(args) => edit::move_task(&mut taiga, args)?,
        TaigaCmd::DoneTask(args) => edit::done(&mut taiga, args)?,
        TaigaCmd::DeleteTask(args) => edit::delete(&mut taiga, args)?,
        TaigaCmd::RenameTask(args) => edit::rename(&mut taiga, args)?,
        TaigaCmd::AssignTask(args) => edit::assign(&mut taiga, args)?,
        TaigaCmd::DueTask(args) => edit::due(&mut taiga, args)?,
        TaigaCmd::SprintTask(args) => edit::sprint(&mut taiga, args)?,
        TaigaCmd::TeamTask(args) => edit::team(&mut taiga, args)?,
        TaigaCmd::ClientTask(args) => edit::client(&mut taiga, args)?,
        TaigaCmd::BlockTask(args) => edit::block(&mut taiga, args)?,
        TaigaCmd::ModifyTask(args) => modify::modify(&mut taiga, args)?,
        TaigaCmd::SearchTask(args) => search::search(&mut taiga, args)?,
        TaigaCmd::Subtask(args) => subtask::subtask(&mut taiga, args)?,
        TaigaCmd::ProjectBurndown(args) => burndown::burndown(&mut taiga, args)?,
        TaigaCmd::Login(_) => {}
    }
    Ok(())
}

/// Patches the task at a position of the last listing and stores the answer.
///
/// `patch` sees the current task; the version of the task is filled in afterwards.
fn edit(
    taiga: &mut Taiga,
    mut project: TaigaProject,
    index: usize,
    patch: impl FnOnce(&TaigaTask) -> Result<TaskPatch>,
) -> Result<()> {
    let task = project.get_task(index)?;
    let patch = TaskPatch {
        version: task.version,
        ..patch(task)?
    };
    *task = taiga.patch_task(task.id, &patch)?;
    Ok(taiga.save_project(project)?)
}
