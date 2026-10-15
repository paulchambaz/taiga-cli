//! The `search` command and the formatting of its listing.

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::{Cell, Row, Table};

use crate::cli::SearchTaskArgs;
use crate::taiga::{Error, Taiga};

/// Lists the tasks that fit the filters and remembers the listing for index-based commands.
///
/// # Errors
/// Returns an error when a filter names something unknown or a request fails.
pub fn search(taiga: &mut Taiga, args: SearchTaskArgs) -> Result<()> {
    let id = taiga.find_project(&args.project)?.id;

    let mut tasks = taiga.get_tasks(id)?;

    if args.include_statuses.is_empty() {
        tasks.retain(|task| !task.closed);
    }
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
