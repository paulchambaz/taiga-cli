mod cli;
mod taiga;
mod utils;

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, NaiveDate, Utc};
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::{row, Cell, Row, Table};
use taiga::{Error, Taiga, TaigaSubtask};

use cli::{parse_args, TaigaCmd};
use cli::{
    AssignTaskArgs, BlockTaskArgs, ClientTaskArgs, DeleteTaskArgs, DoneTaskArgs, DueTaskArgs,
    ModifyTaskArgs, MoveTaskArgs, NewTaskArgs, ProjectUserArgs, RenameTaskArgs, SearchTaskArgs,
    SprintTaskArgs, SubtaskAction, SubtaskArgs, TeamTaskArgs,
};

fn main() -> Result<()> {
    let cached = Taiga::from_cache()?;

    match parse_args(cached.as_ref()) {
        TaigaCmd::Login(args) => {
            Taiga::auth(args.address)?;
            Ok(())
        }
        cmd => run(cmd, cached.ok_or(Error::NotLoggedIn)?),
    }
}

fn run(cmd: TaigaCmd, mut taiga: Taiga) -> Result<()> {
    match cmd {
        TaigaCmd::Default => taiga_default(&mut taiga),
        TaigaCmd::Projects => taiga_projects(&mut taiga)?,
        TaigaCmd::NewTask(args) => taiga_new(&mut taiga, args)?,
        TaigaCmd::MoveTask(args) => taiga_move(&mut taiga, args)?,
        TaigaCmd::DoneTask(args) => taiga_done(&mut taiga, args)?,
        TaigaCmd::RenameTask(args) => taiga_rename(&mut taiga, args)?,
        TaigaCmd::AssignTask(args) => taiga_assign(&mut taiga, args)?,
        TaigaCmd::DueTask(args) => taiga_due(&mut taiga, args)?,
        TaigaCmd::SprintTask(args) => taiga_sprint(&mut taiga, args)?,
        TaigaCmd::Subtask(args) => taiga_subtask(&mut taiga, args)?,
        TaigaCmd::TeamTask(args) => taiga_team(&mut taiga, args)?,
        TaigaCmd::ClientTask(args) => taiga_client(&mut taiga, args)?,
        TaigaCmd::BlockTask(args) => taiga_block(&mut taiga, args)?,
        TaigaCmd::ModifyTask(args) => taiga_modify(&mut taiga, args)?,
        TaigaCmd::DeleteTask(args) => taiga_delete(&mut taiga, args)?,
        TaigaCmd::SearchTask(args) => taiga_search(&mut taiga, args)?,
        TaigaCmd::ProjectUsers(args) => taiga_users(&mut taiga, args)?,
        other => println!("TODO: {:?}", other),
    }
    Ok(())
}

pub fn taiga_default(taiga: &mut Taiga) {
    for project in &taiga.projects {
        println!("{}", project.name);
    }
}

pub fn taiga_projects(taiga: &mut Taiga) -> Result<()> {
    taiga.update_projects()?;

    for project in &taiga.projects {
        println!("{}", project.name);
    }
    Ok(())
}

pub fn taiga_search(taiga: &mut Taiga, args: SearchTaskArgs) -> Result<()> {
    let id = taiga.find_project(&args.project)?.id;

    let mut tasks = taiga.get_tasks(id)?;

    tasks.retain(|task| !task.closed);
    tasks.sort_by(|a, b| {
        b.status_id
            .cmp(&a.status_id)
            .then_with(|| match (&a.due, &b.due) {
                (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
                (Some(_), _) => std::cmp::Ordering::Less,
                (_, Some(_)) => std::cmp::Ordering::Greater,
                (_, _) => std::cmp::Ordering::Equal,
            })
    });

    let me = taiga.id;
    let mut project = taiga.load_project(&args.project, |project| {
        args.include_statuses
            .iter()
            .chain(&args.exclude_statuses)
            .any(|status| project.status_id(status).is_err())
            || args
                .include_sprints
                .iter()
                .chain(&args.exclude_sprints)
                .any(|sprint| project.find_sprint(sprint).is_none())
            || args
                .include_assigned
                .iter()
                .chain(&args.exclude_assigned)
                .any(|username| project.member_id(username, me).is_err())
            || tasks
                .iter()
                .flat_map(|task| &task.assigned)
                .any(|id| project.members.iter().all(|m| m.id != *id))
    })?;

    let include_status_ids = args
        .include_statuses
        .iter()
        .map(|status| project.status_id(status))
        .collect::<Result<Vec<_>, _>>()?;
    let exclude_status_ids = args
        .exclude_statuses
        .iter()
        .map(|status| project.status_id(status))
        .collect::<Result<Vec<_>, _>>()?;

    let include_sprint_ids = args
        .include_sprints
        .iter()
        .map(|sprint| project.sprint_id(sprint))
        .collect::<Result<Vec<_>, _>>()?;
    let exclude_sprint_ids = args
        .exclude_sprints
        .iter()
        .map(|sprint| project.sprint_id(sprint))
        .collect::<Result<Vec<_>, _>>()?;

    let include_member_ids = args
        .include_assigned
        .iter()
        .map(|username| project.member_id(username, me))
        .collect::<Result<Vec<_>, _>>()?;
    let exclude_member_ids = args
        .exclude_assigned
        .iter()
        .map(|username| project.member_id(username, me))
        .collect::<Result<Vec<_>, _>>()?;

    let no_due = args.due_date.as_deref() == Some("");
    let due_limit = args
        .due_date
        .as_deref()
        .filter(|due_date| !due_date.is_empty())
        .map(|due_date| NaiveDate::parse_from_str(due_date, "%Y-%m-%d"))
        .transpose()?;

    let mut table = Table::new();
    table.set_format(*FORMAT_CLEAN);

    let has_team = tasks.iter().any(|task| task.team);
    let has_client = tasks.iter().any(|task| task.client);
    let has_block = tasks.iter().any(|task| task.blocked);

    let has_sprint = tasks.iter().any(|task| task.sprint_id.is_some());

    let mut headers = vec!["ID", "STATUS", "DUE", "NAME", "ASSIGN"];
    if has_sprint {
        headers.push("SPRINT");
    }
    if has_team {
        headers.push("T");
    }
    if has_client {
        headers.push("C");
    }
    if has_block {
        headers.push("B");
    }

    table.add_row(Row::from_iter(headers.iter().copied()));

    project.tasks = tasks
        .into_iter()
        .filter(|task| {
            if let Some(team) = args.team {
                if task.team != team {
                    return false;
                }
            }

            if let Some(client) = args.client {
                if task.client != client {
                    return false;
                }
            }

            if let Some(block) = args.block {
                if task.blocked != block {
                    return false;
                }
            }

            if no_due && task.due.is_some() {
                return false;
            }

            if let Some(limit) = due_limit {
                match task.due {
                    Some(due) if due.date_naive() <= limit => {}
                    _ => return false,
                }
            }

            if !include_member_ids.is_empty()
                && !task
                    .assigned
                    .iter()
                    .any(|id| include_member_ids.contains(id))
            {
                return false;
            }

            if !exclude_member_ids.is_empty()
                && task
                    .assigned
                    .iter()
                    .any(|id| exclude_member_ids.contains(id))
            {
                return false;
            }

            if !include_status_ids.is_empty() && !include_status_ids.contains(&task.status_id) {
                return false;
            }

            if exclude_status_ids.contains(&task.status_id) {
                return false;
            }

            if !include_sprint_ids.is_empty()
                && !task
                    .sprint_id
                    .is_some_and(|id| include_sprint_ids.contains(&id))
            {
                return false;
            }

            if task
                .sprint_id
                .is_some_and(|id| exclude_sprint_ids.contains(&id))
            {
                return false;
            }

            fzf_match(&task.name, &args.query)
        })
        .collect();

    for (i, task) in project.tasks.iter().enumerate() {
        let assigned = task
            .assigned
            .iter()
            .map(|id| {
                project
                    .members
                    .iter()
                    .find(|m| m.id == *id)
                    .map(|m| m.username.as_str())
                    .ok_or_else(|| Error::UserNotFound(id.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");

        let due = task.due.as_ref().map(format_due).unwrap_or_default();

        let mut row_cells = vec![
            Cell::new(&format!("{}", i + 1)),
            Cell::new(&task.status),
            Cell::new(&due),
            Cell::new(&task.name),
            Cell::new(&assigned),
        ];

        if has_sprint {
            let sprint_name = task
                .sprint_id
                .and_then(|id| project.sprints.iter().find(|s| s.id == id))
                .map(|s| s.name.as_str())
                .unwrap_or("");
            row_cells.push(Cell::new(sprint_name));
        }

        if has_team {
            row_cells.push(Cell::new(if task.team { "Y" } else { "" }));
        }
        if has_client {
            row_cells.push(Cell::new(if task.client { "Y" } else { "" }));
        }
        if has_block {
            row_cells.push(Cell::new(if task.blocked { "Y" } else { "" }));
        }

        table.add_row(Row::from_iter(row_cells));
    }

    taiga.save_project(project)?;
    table.printstd();
    Ok(())
}

pub fn taiga_new(taiga: &mut Taiga, args: NewTaskArgs) -> Result<()> {
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

    let status_id = match &args.status {
        Some(status) => project.status_id(status)?,
        None => project.statuses.first().ok_or(Error::NoStatuses)?.id,
    };

    let sprint_id = args
        .sprint
        .as_deref()
        .map(|sprint| project.sprint_id(sprint))
        .transpose()?;

    let assigned_ids = args
        .assign
        .iter()
        .map(|username| project.member_id(username, me))
        .collect::<Result<Vec<_>, _>>()?;

    let new_task = taiga.new_task(
        project.id,
        status_id,
        args.name.clone(),
        assigned_ids.clone(),
        args.team,
        args.client,
        args.block,
    )?;

    let new_task = taiga.modify_task(
        new_task.id,
        status_id,
        args.name,
        assigned_ids,
        args.due_date,
        args.team,
        args.client,
        args.block,
        sprint_id,
        new_task.version,
    )?;

    project.tasks.push(new_task);
    Ok(taiga.save_project(project)?)
}

pub fn taiga_users(taiga: &mut Taiga, args: ProjectUserArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |_| true)?;

    for user in &project.members {
        println!("{}", user.username);
    }
    Ok(())
}

pub fn taiga_move(taiga: &mut Taiga, args: MoveTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |project| {
        project.status_id(&args.status).is_err()
    })?;

    let status_id = project.status_id(&args.status)?;

    let task = project.get_task(args.id)?;
    *task = taiga.move_task(task.id, status_id, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_delete(taiga: &mut Taiga, args: DeleteTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task_id = project.get_task(args.id)?.id;

    taiga.delete_task(task_id)?;

    project.tasks.retain(|t| t.id != task_id);
    Ok(taiga.save_project(project)?)
}

pub fn taiga_done(taiga: &mut Taiga, args: DoneTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;

    let status_id = project
        .statuses
        .iter()
        .find(|status| status.is_closed)
        .or_else(|| project.statuses.iter().find(|status| status.slug == "done"))
        .or_else(|| project.statuses.last())
        .ok_or(Error::NoStatuses)?
        .id;

    let task = project.get_task(args.id)?;
    *task = taiga.move_task(task.id, status_id, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_rename(taiga: &mut Taiga, args: RenameTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task = project.get_task(args.id)?;

    *task = taiga.rename_task(task.id, args.name, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_assign(taiga: &mut Taiga, args: AssignTaskArgs) -> Result<()> {
    let me = taiga.id;
    let mut project = taiga.load_project(&args.project, |project| {
        project.member_id(&args.username, me).is_err()
    })?;

    let member_id = project.member_id(&args.username, me)?;

    let task = project.get_task(args.id)?;

    let is_assigned = task.assigned.contains(&member_id);
    let assigned: Vec<i32> = if args.remove {
        if !is_assigned {
            bail!("the user is not assigned to the task, cannot remove");
        }
        task.assigned
            .iter()
            .copied()
            .filter(|m| *m != member_id)
            .collect()
    } else if is_assigned {
        bail!("the user is already assigned to the task, cannot add");
    } else {
        let mut assigned = task.assigned.clone();
        assigned.push(member_id);
        assigned
    };

    *task = taiga.assign_task(task.id, assigned, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_due(taiga: &mut Taiga, args: DueTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task = project.get_task(args.id)?;

    *task = taiga.due_task(task.id, args.due_date, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_sprint(taiga: &mut Taiga, args: SprintTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |project| {
        args.sprint
            .as_deref()
            .is_some_and(|sprint| project.find_sprint(sprint).is_none())
    })?;

    let milestone_id = args
        .sprint
        .as_deref()
        .map(|sprint| project.sprint_id(sprint))
        .transpose()?;

    let task = project.get_task(args.id)?;

    *task = taiga.sprint_task(task.id, milestone_id, task.version)?;
    Ok(taiga.save_project(project)?)
}

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

pub fn taiga_subtask(taiga: &mut Taiga, args: SubtaskArgs) -> Result<()> {
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
            let subtasks = taiga.get_subtasks(project.id, story_id)?;

            let subtasks = subtasks
                .iter()
                .map(|s| taiga.get_subtask(s.id))
                .collect::<Result<Vec<_>, _>>()?;

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

pub fn taiga_team(taiga: &mut Taiga, args: TeamTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task = project.get_task(args.id)?;

    *task = taiga.team_task(task.id, args.remove, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_client(taiga: &mut Taiga, args: ClientTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task = project.get_task(args.id)?;

    *task = taiga.client_task(task.id, args.remove, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_block(taiga: &mut Taiga, args: BlockTaskArgs) -> Result<()> {
    let mut project = taiga.load_project(&args.project, |_| false)?;
    let task = project.get_task(args.id)?;

    *task = taiga.block_task(task.id, args.remove, task.version)?;
    Ok(taiga.save_project(project)?)
}

pub fn taiga_modify(taiga: &mut Taiga, args: ModifyTaskArgs) -> Result<()> {
    let me = taiga.id;
    let mut project = taiga.load_project(&args.project, |project| {
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
                .flatten()
                .any(|username| project.member_id(username, me).is_err())
    })?;

    let status_id = args
        .status
        .as_deref()
        .map(|status| project.status_id(status))
        .transpose()?;

    let sprint_id: Option<Option<i32>> = match &args.sprint {
        None => None,
        Some(None) => Some(None),
        Some(Some(sprint)) => Some(Some(project.sprint_id(sprint)?)),
    };

    let assigned_ids = args
        .assign
        .iter()
        .flatten()
        .map(|username| project.member_id(username, me))
        .collect::<Result<Vec<_>, _>>()?;

    let task = project.get_task(args.id)?;

    let mut assign = task.assigned.clone();
    if args.assign.is_some() {
        assign.extend(assigned_ids);
        assign.sort();
        assign.dedup();
    }

    let due_date = args
        .due_date
        .or_else(|| task.due.map(|due| due.format("%Y-%m-%d").to_string()));

    *task = taiga.modify_task(
        task.id,
        status_id.unwrap_or(task.status_id),
        args.rename.unwrap_or_else(|| task.name.clone()),
        assign,
        due_date,
        args.team.unwrap_or(task.team),
        args.client.unwrap_or(task.client),
        args.block.unwrap_or(task.blocked),
        sprint_id.unwrap_or(task.sprint_id),
        task.version,
    )?;
    Ok(taiga.save_project(project)?)
}

fn format_due(due: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = due.signed_duration_since(now);

    let is_past = duration.num_seconds() < 0;
    let duration = duration.abs();

    if duration.num_days() >= 365 {
        format!(
            "{}{}y",
            if is_past { "-" } else { "" },
            duration.num_days() / 365
        )
    } else if duration.num_days() >= 30 {
        format!(
            "{}{}m",
            if is_past { "-" } else { "" },
            duration.num_days() / 30
        )
    } else if duration.num_days() >= 7 {
        format!(
            "{}{}w",
            if is_past { "-" } else { "" },
            duration.num_days() / 7
        )
    } else if duration.num_hours() >= 24 {
        format!(
            "{}{}d",
            if is_past { "-" } else { "" },
            duration.num_hours() / 24
        )
    } else {
        format!(
            "{}{}h",
            if is_past { "-" } else { "" },
            duration.num_hours()
        )
    }
}

fn fzf_match(input: &str, query: &[String]) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut index_match = 0;
    for word in input.split(' ') {
        if let Some(matching) = query.get(index_match) {
            if word.contains(matching) {
                index_match += 1;
            }
        } else {
            return true;
        }
    }

    index_match == query.len()
}
