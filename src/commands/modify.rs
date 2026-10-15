//! The `modify` command.

use anyhow::Result;

use super::edit;
use crate::cli::ModifyTaskArgs;
use crate::taiga::{Taiga, TaskPatch};

/// Changes several fields of a task at once, sending only what was asked for.
///
/// # Errors
/// Returns an error when the task, status, sprint or a user is unknown or the request fails.
pub fn modify(taiga: &mut Taiga, args: ModifyTaskArgs) -> Result<()> {
    let me = taiga.id;
    let project = taiga.load_project(&args.project, |project| {
        args.status
            .as_deref()
            .is_some_and(|status| project.status_id(status).is_err())
            || args
                .sprint
                .as_ref()
                .and_then(|sprint| sprint.as_deref())
                .is_some_and(|sprint| project.find_sprint(sprint).is_none())
            || args
                .assign
                .iter()
                .chain(&args.unassign)
                .any(|username| project.member_id(username, me).is_err())
    })?;

    let status = args
        .status
        .as_deref()
        .map(|status| project.status_id(status))
        .transpose()?;
    let milestone = match &args.sprint {
        None => None,
        Some(None) => Some(None),
        Some(Some(sprint)) => Some(Some(project.sprint_id(sprint)?)),
    };
    let ids = |usernames: &[String]| {
        usernames
            .iter()
            .map(|username| project.member_id(username, me))
            .collect::<Result<Vec<_>, _>>()
    };
    let added = ids(&args.assign)?;
    let removed = ids(&args.unassign)?;

    edit(taiga, project, args.id, |task| {
        let assigned = (!added.is_empty() || !removed.is_empty()).then(|| {
            let mut users = task.assigned.clone();
            users.extend(&added);
            users.retain(|user| !removed.contains(user));
            users.sort();
            users.dedup();
            users
        });

        Ok(TaskPatch {
            assigned_to: assigned.as_ref().map(|users| users.first().copied()),
            assigned_users: assigned,
            status,
            subject: args.rename,
            due_date: args.due_date,
            team_requirement: args.team,
            client_requirement: args.client,
            is_blocked: args.block,
            milestone,
            ..Default::default()
        })
    })
}
