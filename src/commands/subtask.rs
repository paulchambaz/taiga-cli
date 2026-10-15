//! The `subtask` command.

use anyhow::{anyhow, Result};
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::{row, Cell, Row, Table};

use crate::cli::{SubtaskAction, SubtaskArgs};
use crate::taiga::{Error, Taiga, TaigaSubtask};

fn find_subtask(
    taiga: &mut Taiga,
    project_id: i32,
    story_id: i32,
    index: usize,
) -> Result<TaigaSubtask> {
    let subtasks = taiga.get_subtasks(project_id, story_id)?;
    index
        .checked_sub(1)
        .and_then(|i| subtasks.into_iter().nth(i))
        .ok_or_else(|| anyhow!("the story has no subtask {index}"))
}

/// Creates, lists, describes or deletes the tasks nested under a user story.
///
/// # Errors
/// Returns an error when the story or a subtask is unknown or a request fails.
pub fn subtask(taiga: &mut Taiga, args: SubtaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let story_id = project.get_task(args.story_id)?.id;

    match args.action {
        SubtaskAction::New {
            name,
            description,
            assign,
        } => {
            let status_id = project.task_statuses.first().ok_or(Error::NoStatuses)?.id;

            let assigned_id = assign
                .map(|username| project.member_id(&username, taiga.id))
                .transpose()?;

            taiga.new_subtask(
                project.id,
                story_id,
                status_id,
                name,
                description,
                assigned_id,
            )?;
        }
        SubtaskAction::List => {
            let subtasks = taiga.get_described_subtasks(project.id, story_id)?;

            let mut table = Table::new();
            table.set_format(*FORMAT_CLEAN);
            table.add_row(row!["ID", "STATUS", "NAME", "DESCRIPTION", "ASSIGN"]);
            for (i, subtask) in subtasks.iter().enumerate() {
                let assigned = subtask
                    .assigned
                    .and_then(|id| project.members.iter().find(|m| m.id == id))
                    .map(|m| m.username.as_str())
                    .unwrap_or("");
                table.add_row(Row::new(vec![
                    Cell::new(&format!("{}", i + 1)),
                    Cell::new(&subtask.status),
                    Cell::new(&subtask.name),
                    Cell::new(&subtask.description),
                    Cell::new(assigned),
                ]));
            }
            table.printstd();
        }
        SubtaskAction::Describe { index, description } => {
            let subtask = find_subtask(taiga, project.id, story_id, index)?;
            taiga.describe_subtask(subtask.id, description, subtask.version)?;
        }
        SubtaskAction::Delete { index } => {
            let subtask = find_subtask(taiga, project.id, story_id, index)?;
            taiga.delete_subtask(subtask.id)?;
        }
    }
    Ok(())
}
