//! The `new` command.

use anyhow::Result;

use crate::cli::NewTaskArgs;
use crate::taiga::{Error, Taiga, TaskNew};

/// Creates a task in a single request and adds it to the cached listing.
///
/// # Errors
/// Returns an error when the status, sprint or a user is unknown or the request fails.
pub fn new_task(taiga: &mut Taiga, args: NewTaskArgs) -> Result<()> {
    let me = taiga.id;
    let mut project = taiga.load_project(&args.project, |project| {
        args.status
            .as_deref()
            .is_some_and(|status| project.status_id(status).is_err())
            || args
                .sprint
                .as_deref()
                .is_some_and(|sprint| project.find_sprint(sprint).is_none())
            || args
                .assign
                .iter()
                .any(|username| project.member_id(username, me).is_err())
    })?;

    let status = match &args.status {
        Some(status) => project.status_id(status)?,
        None => project.statuses.first().ok_or(Error::NoStatuses)?.id,
    };
    let milestone = args
        .sprint
        .as_deref()
        .map(|sprint| project.sprint_id(sprint))
        .transpose()?;
    let assigned_users = args
        .assign
        .iter()
        .map(|username| project.member_id(username, me))
        .collect::<Result<Vec<_>, _>>()?;

    let task = taiga.new_task(&TaskNew {
        project: project.id,
        status,
        subject: args.name,
        assigned_to: assigned_users.first().copied(),
        assigned_users,
        due_date: args.due_date,
        milestone,
        team_requirement: args.team,
        client_requirement: args.client,
        is_blocked: args.block,
    })?;

    project.tasks.push(task);
    Ok(taiga.save_project(project)?)
}
