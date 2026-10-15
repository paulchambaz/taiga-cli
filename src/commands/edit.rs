//! The commands that change one field of one task.

use anyhow::{bail, Result};

use super::edit;
use crate::cli::{
    AssignTaskArgs, DueTaskArgs, MoveTaskArgs, RenameTaskArgs, SprintTaskArgs, TaskArgs,
    ToggleTaskArgs,
};
use crate::taiga::{Error, Taiga, TaskPatch};

/// Moves a task to a status.
///
/// # Errors
/// Returns an error when the task or status is unknown or the request fails.
pub fn move_task(taiga: &mut Taiga, args: MoveTaskArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |project| {
        project.status_id(&args.status).is_err()
    })?;
    let status = project.status_id(&args.status)?;

    edit(taiga, project, args.id, |_| {
        Ok(TaskPatch {
            status: Some(status),
            ..Default::default()
        })
    })
}

/// Moves a task to the closed status of its project.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn done(taiga: &mut Taiga, args: TaskArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |_| false)?;
    let status = project
        .statuses
        .iter()
        .find(|status| status.is_closed)
        .or_else(|| project.statuses.iter().find(|status| status.slug == "done"))
        .or_else(|| project.statuses.last())
        .ok_or(Error::NoStatuses)?
        .id;

    edit(taiga, project, args.id, |_| {
        Ok(TaskPatch {
            status: Some(status),
            ..Default::default()
        })
    })
}

/// Deletes a task.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn delete(taiga: &mut Taiga, args: TaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task_id = project.get_task(args.id)?.id;

    taiga.delete_task(task_id)?;

    project.tasks.retain(|task| task.id != task_id);
    Ok(taiga.save_project(project)?)
}

/// Renames a task.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn rename(taiga: &mut Taiga, args: RenameTaskArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |_| false)?;

    edit(taiga, project, args.id, |_| {
        Ok(TaskPatch {
            subject: Some(args.name),
            ..Default::default()
        })
    })
}

/// Adds a user to a task, or removes them with `--remove`.
///
/// # Errors
/// Returns an error when the task or user is unknown, the user is already
/// (or not) assigned, or the request fails.
pub fn assign(taiga: &mut Taiga, args: AssignTaskArgs) -> Result<()> {
    let me = taiga.id;
    let project = taiga.load_project(&args.project, |project| {
        project.member_id(&args.username, me).is_err()
    })?;
    let member_id = project.member_id(&args.username, me)?;

    edit(taiga, project, args.id, |task| {
        let is_assigned = task.assigned.contains(&member_id);
        let assigned: Vec<i32> = match (args.remove, is_assigned) {
            (true, false) => bail!("the user is not assigned to the task, cannot remove"),
            (false, true) => bail!("the user is already assigned to the task, cannot add"),
            (true, true) => task
                .assigned
                .iter()
                .copied()
                .filter(|user| *user != member_id)
                .collect(),
            (false, false) => task.assigned.iter().copied().chain([member_id]).collect(),
        };
        Ok(TaskPatch {
            assigned_users: Some(assigned),
            ..Default::default()
        })
    })
}

/// Sets the due date of a task, or clears it.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn due(taiga: &mut Taiga, args: DueTaskArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |_| false)?;

    edit(taiga, project, args.id, |_| {
        Ok(TaskPatch {
            due_date: Some(args.due_date),
            ..Default::default()
        })
    })
}

/// Puts a task in a sprint, or takes it out of its sprint.
///
/// # Errors
/// Returns an error when the task or sprint is unknown or the request fails.
pub fn sprint(taiga: &mut Taiga, args: SprintTaskArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |project| {
        args.sprint
            .as_deref()
            .is_some_and(|sprint| project.find_sprint(sprint).is_none())
    })?;
    let milestone = args
        .sprint
        .as_deref()
        .map(|sprint| project.sprint_id(sprint))
        .transpose()?;

    edit(taiga, project, args.id, |_| {
        Ok(TaskPatch {
            milestone: Some(milestone),
            ..Default::default()
        })
    })
}

/// Sets the team requirement of a task, or removes it.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn team(taiga: &mut Taiga, args: ToggleTaskArgs) -> Result<()> {
    toggle(taiga, args, |on| TaskPatch {
        team_requirement: Some(on),
        ..Default::default()
    })
}

/// Sets the client requirement of a task, or removes it.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn client(taiga: &mut Taiga, args: ToggleTaskArgs) -> Result<()> {
    toggle(taiga, args, |on| TaskPatch {
        client_requirement: Some(on),
        ..Default::default()
    })
}

/// Blocks a task, or unblocks it.
///
/// # Errors
/// Returns an error when the task is unknown or the request fails.
pub fn block(taiga: &mut Taiga, args: ToggleTaskArgs) -> Result<()> {
    toggle(taiga, args, |on| TaskPatch {
        is_blocked: Some(on),
        ..Default::default()
    })
}

fn toggle(taiga: &mut Taiga, args: ToggleTaskArgs, patch: fn(bool) -> TaskPatch) -> Result<()> {
    let project = taiga.load_project(&args.project, |_| false)?;
    edit(taiga, project, args.id, |_| Ok(patch(!args.remove)))
}
