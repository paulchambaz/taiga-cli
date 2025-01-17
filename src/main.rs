mod cli;
mod taiga;
mod utils;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::{row, Cell, Row, Table};
use std::collections::HashSet;
use std::process::exit;
use taiga::{Taiga, TaigaProject, TaigaTask, TaigaTasks};

use cli::{parse_args, TaigaCmd};
use cli::{
    AssignTaskArgs, BlockTaskArgs, ClientTaskArgs, DeleteTaskArgs, DoneTaskArgs, DueTaskArgs,
    ModifyTaskArgs, MoveTaskArgs, NewTaskArgs, ProjectUserArgs, RenameTaskArgs, SearchTaskArgs,
    TeamTaskArgs,
};

fn main() -> Result<()> {
    let mut taiga = match Taiga::from_cache() {
        Some(taiga) => taiga,
        _ => Taiga::auth(None)?,
    };
    let cmd = parse_args(&Some(taiga.clone()));

    match cmd {
        TaigaCmd::Default => taiga_default(&mut taiga),
        TaigaCmd::Login(args) => {
            Taiga::auth(args.address);
        }
        TaigaCmd::Projects => taiga_projects(&mut taiga),
        TaigaCmd::NewTask(args) => taiga_new(&mut taiga, args),
        TaigaCmd::MoveTask(args) => taiga_move(&mut taiga, args),
        TaigaCmd::DoneTask(args) => taiga_done(&mut taiga, args),
        TaigaCmd::RenameTask(args) => taiga_rename(&mut taiga, args),
        TaigaCmd::AssignTask(args) => taiga_assign(&mut taiga, args),
        TaigaCmd::DueTask(args) => taiga_due(&mut taiga, args),
        TaigaCmd::TeamTask(args) => taiga_team(&mut taiga, args),
        TaigaCmd::ClientTask(args) => taiga_client(&mut taiga, args),
        TaigaCmd::BlockTask(args) => taiga_block(&mut taiga, args),
        TaigaCmd::ModifyTask(args) => taiga_modify(&mut taiga, args),
        TaigaCmd::DeleteTask(args) => taiga_delete(&mut taiga, args),
        TaigaCmd::SearchTask(args) => taiga_search(&mut taiga, args),
        TaigaCmd::ProjectUsers(args) => taiga_users(&mut taiga, args),
        other => println!("TODO: {:?}", other),
    }
    Ok(())
}

pub fn taiga_default(taiga: &mut Taiga) {
    for project in &taiga.projects {
        println!("{}", project.name);
    }
}

pub fn taiga_projects(taiga: &mut Taiga) {
    let projects = taiga.get_projects().unwrap_or_else(|err| {
        eprintln!("Error, could not get project: {}", err);
        exit(1);
    });

    for project in &projects {
        println!("{}", project.name);
    }

    taiga.projects = projects;
    taiga.save_cache();
}

pub fn taiga_search(taiga: &mut Taiga, args: SearchTaskArgs) {
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let id = project.id;

    let mut tasks = taiga.get_tasks(id).unwrap_or_else(|err| {
        eprintln!("Error, could not get tasks: {}", err);
        exit(1);
    });

    tasks.retain(|task| !task.closed);
    tasks.sort_by(|a, b| {
        b.status_id
            .cmp(&a.status_id) // Reverse the order to ensure higher status_ids go first
            .then_with(|| {
                match (&a.due, &b.due) {
                    // If both tasks have due dates, compare them directly.
                    (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
                    // If only one task has a due date, it goes first.
                    (Some(_), _) => std::cmp::Ordering::Less,
                    (_, Some(_)) => std::cmp::Ordering::Greater,
                    // If neither task has a due date, they are considered equal.
                    (_, _) => std::cmp::Ordering::Equal,
                }
            })
    });

    let project = match TaigaProject::from_cache(id) {
        Ok(Some(project)) => {
            let members: Vec<i32> = tasks
                .iter()
                .flat_map(|task| &task.assigned)
                .cloned()
                .collect::<HashSet<i32>>()
                .into_iter()
                .collect();
            let all_present = members
                .iter()
                .all(|id| project.members.iter().any(|m| m.id == *id));
            if all_present {
                project
            } else {
                let project = taiga.get_project(project.id).unwrap_or_else(|err| {
                    eprintln!("Error, could not get project: {}", err);
                    exit(1);
                });
                project.save_cache();
                project
            }
        }
        _ => {
            let project = taiga.get_project(project.id).unwrap_or_else(|err| {
                eprintln!("Error, could not get project: {}", err);
                exit(1);
            });
            project.save_cache();
            project
        }
    };

    let taiga_tasks = TaigaTasks {
        id: project.id,
        tasks: tasks.clone(),
        members: project.members.clone(),
        statuses: project.statuses.clone(),
    };
    taiga_tasks.save_cache();

    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| {
        for status in &args.include_statuses {
            if !tasks.statuses.iter().any(|s| s.slug == *status) {
                return true;
            }
        }
        for status in &args.exclude_statuses {
            if !tasks.statuses.iter().any(|s| s.slug == *status) {
                return true;
            }
        }
        for username in &args.include_assigned {
            if username != "me" && !tasks.members.iter().any(|m| m.username == *username) {
                return true;
            }
        }
        for username in &args.exclude_assigned {
            if username != "me" && !tasks.members.iter().any(|m| m.username == *username) {
                return true;
            }
        }
        false
    });

    let mut include_status_ids = Vec::new();
    for status in args.include_statuses {
        let status_id = if let Some(status) = tasks.statuses.iter().find(|s| s.slug == *status) {
            status.id
        } else {
            eprintln!("Error, could not find given status");
            exit(1);
        };
        include_status_ids.push(status_id);
    }

    let mut exclude_status_ids = Vec::new();
    for status in args.exclude_statuses {
        let status_id = if let Some(status) = tasks.statuses.iter().find(|s| s.slug == *status) {
            status.id
        } else {
            eprintln!("Error, could not find given status");
            exit(1);
        };
        exclude_status_ids.push(status_id);
    }

    let mut include_member_ids = Vec::new();
    for username in args.include_assigned {
        let member_id = if username == "me" {
            tasks
                .members
                .iter()
                .find(|member| member.id == taiga.id)
                .map(|m| m.id)
                .unwrap_or_else(|| {
                    eprintln!("Could not find your username on the project");
                    exit(1);
                })
        } else {
            tasks
                .members
                .iter()
                .find(|member| member.username == username)
                .map(|m| m.id)
                .unwrap_or_else(|| {
                    eprintln!("Could not find username on the project");
                    exit(1);
                })
        };
        include_member_ids.push(member_id);
    }
    let mut exclude_member_ids = Vec::new();
    for username in args.exclude_assigned {
        let member_id = if username == "me" {
            tasks
                .members
                .iter()
                .find(|member| member.id == taiga.id)
                .map(|m| m.id)
                .unwrap_or_else(|| {
                    eprintln!("Could not find your username on the project");
                    exit(1);
                })
        } else {
            tasks
                .members
                .iter()
                .find(|member| member.username == username)
                .map(|m| m.id)
                .unwrap_or_else(|| {
                    eprintln!("Could not find username on the project");
                    exit(1);
                })
        };
        exclude_member_ids.push(member_id);
    }

    let mut table = Table::new();
    table.set_format(*FORMAT_CLEAN);
    table.add_row(row!["ID", "STATUS", "DUE", "NAME", "ASSIGN", "T", "C", "B"]);

    let filter_tasks: Vec<TaigaTask> = tasks
        .tasks
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

            if let Some(due_date) = &args.due_date {
                if due_date.is_empty() {
                    if task.due.is_some() {
                        return false;
                    }
                } else if let Some(task_due) = task.due {
                    let a = task_due.date_naive();
                    let b = NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                        .expect("Could not parse due date");
                    if a > b {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            if !include_member_ids.is_empty()
                && !task
                    .assigned
                    .iter()
                    .any(|id| include_member_ids.iter().any(|member_id| member_id == id))
            {
                return false;
            }

            if !exclude_member_ids.is_empty()
                && task
                    .assigned
                    .iter()
                    .any(|id| exclude_member_ids.iter().any(|member_id| member_id == id))
            {
                return false;
            }

            if !include_status_ids.is_empty()
                && !include_status_ids.iter().any(|id| *id == task.status_id)
            {
                return false;
            }

            if !exclude_status_ids.is_empty()
                && exclude_status_ids.iter().any(|id| *id == task.status_id)
            {
                return false;
            }

            if !fzf_match(&task.name, &args.query) {
                return false;
            }

            true
        })
        .collect();
    tasks.tasks = filter_tasks;
    tasks.clone().save_cache();

    for (i, task) in tasks.tasks.iter().enumerate() {
        let assigned = task
            .assigned
            .iter()
            .map(|id| {
                project
                    .members
                    .iter()
                    .find(|m| m.id == *id)
                    .expect("Could not find user")
                    .username
                    .clone()
            })
            .collect::<Vec<String>>()
            .join(", ");

        let due = if let Some(due) = task.due {
            format_due(&due)
        } else {
            "".to_string()
        };

        table.add_row(Row::new(vec![
            Cell::new(&format!("{}", i + 1)),
            Cell::new(&task.status),
            Cell::new(&due),
            Cell::new(&task.name),
            Cell::new(&assigned),
            Cell::new(if task.team { "Y" } else { "" }),
            Cell::new(if task.client { "Y" } else { "" }),
            Cell::new(if task.blocked { "Y" } else { "" }),
        ]));
    }
    table.printstd();
}

pub fn taiga_new(taiga: &mut Taiga, args: NewTaskArgs) {
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });

    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| {
        if let Some(status) = &args.status {
            if !tasks.statuses.iter().any(|s| s.slug == *status) {
                return true;
            }
        }
        for username in &args.assign {
            if username != "me" && !tasks.members.iter().any(|m| m.username == *username) {
                return true;
            }
        }
        false
    });

    let status_id;
    if let Some(status) = &args.status {
        status_id = if let Some(status) = tasks.statuses.iter().find(|s| s.slug == *status) {
            status.id
        } else {
            eprintln!("Error, could not find given status");
            exit(1);
        };
    } else {
        status_id = tasks
            .statuses
            .first()
            .expect("This project does not have any statuses")
            .id;
    }

    let mut assigned_ids = Vec::new();
    for username in args.assign {
        let member_id = if username == "me" {
            tasks
                .members
                .iter()
                .find(|member| member.id == taiga.id)
                .map(|m| m.id)
                .unwrap_or_else(|| {
                    eprintln!("Could not find your username on the project");
                    exit(1);
                })
        } else {
            tasks
                .members
                .iter()
                .find(|member| member.username == username)
                .map(|m| m.id)
                .unwrap_or_else(|| {
                    eprintln!("Could not find username on the project");
                    exit(1);
                })
        };
        assigned_ids.push(member_id);
    }

    if let Ok(new_task) = taiga
        .new_task(
            project.id,
            status_id,
            args.name.clone(),
            assigned_ids.clone(),
            args.team,
            args.client,
            args.block,
        )
        .as_mut()
    {
        let task_id = new_task.id;
        let version = new_task.version;

        if let Ok(mod_task) = taiga.modify_task(
            task_id,
            status_id,
            args.name,
            assigned_ids,
            args.due_date,
            args.team,
            args.client,
            args.block,
            version,
        ) {
            *new_task = mod_task;
            let new_task = new_task.clone();
            tasks.tasks.push(new_task);
            tasks.save_cache();
        } else {
            eprintln!("Error, could not modify new task");
            exit(1);
        }
    } else {
        eprintln!("Error, could not create new task");
        exit(1);
    }
}

pub fn taiga_users(taiga: &mut Taiga, args: ProjectUserArgs) {
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let tasks = taiga.tasks_from_cache(project.id, |_| true);
    tasks.clone().save_cache();

    for user in tasks.members {
        println!("{}", user.username);
    }
}

pub fn taiga_move(taiga: &mut Taiga, args: MoveTaskArgs) {
    // getting the necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| {
        !tasks
            .statuses
            .iter()
            .any(|status| status.slug == args.status)
    });

    // checking status is present on the project
    let status_id = if let Some(status) = tasks
        .statuses
        .iter()
        .find(|status| status.slug == args.status)
    {
        status.id
    } else {
        eprintln!("Error, could not find given status");
        exit(1);
    };

    let task = tasks.get_task(args.id);

    // pushing the changes
    if let Ok(new_task) = taiga.move_task(task.id, status_id, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not move task");
        exit(1);
    }
}

pub fn taiga_delete(taiga: &mut Taiga, args: DeleteTaskArgs) {
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| tasks.statuses.is_empty());
    let task = tasks.get_task(args.id);

    if taiga.delete_task(task.id).is_err() {
        eprintln!("Error, could not delete task");
        exit(1);
    }
}

pub fn taiga_done(taiga: &mut Taiga, args: DoneTaskArgs) {
    // getting the necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| tasks.statuses.is_empty());

    // getting the appropriate status for done
    let status_id = if let Some(status) = tasks.statuses.iter().find(|status| status.is_closed) {
        status.id
    } else if let Some(status) = tasks.statuses.iter().find(|status| status.slug == "done") {
        status.id
    } else {
        tasks
            .statuses
            .last()
            .expect("No statuses in the project")
            .id
    };

    let task = tasks.get_task(args.id);

    // pushing the change
    if let Ok(new_task) = taiga.move_task(task.id, status_id, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not done task");
        exit(1);
    }
}

pub fn taiga_rename(taiga: &mut Taiga, args: RenameTaskArgs) {
    // getting necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |_| false);
    let task = tasks.get_task(args.id);

    // pushing the change
    if let Ok(new_task) = taiga.rename_task(task.id, args.name, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not rename task");
        exit(1);
    }
}

pub fn taiga_assign(taiga: &mut Taiga, args: AssignTaskArgs) {
    // getting necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| {
        args.username != "me"
            && !tasks
                .members
                .iter()
                .any(|member| member.username == args.username)
    });

    let member_id = if args.username == "me" {
        tasks
            .members
            .iter()
            .find(|member| member.id == taiga.id)
            .map(|m| m.id)
            .unwrap_or_else(|| {
                eprintln!("Could not find your username on the project");
                exit(1);
            })
    } else {
        tasks
            .members
            .iter()
            .find(|member| member.username == args.username)
            .map(|m| m.id)
            .unwrap_or_else(|| {
                eprintln!("Could not find username on the project");
                exit(1);
            })
    };

    let task = tasks.get_task(args.id);

    // making sure the user can be (de)added from the task
    let mut assigned = task.assigned.clone();
    if args.remove {
        if assigned.iter().any(|m| *m == member_id) {
            assigned = assigned
                .iter()
                .filter(|&&m| m != member_id)
                .copied()
                .collect();
        } else {
            eprintln!("Error, the user is not assigned to the task, cannot remove");
            exit(1);
        }
    } else if assigned.iter().any(|m| *m == member_id) {
        eprintln!("Error, the user is already assigned to the task, cannot add");
        exit(1);
    } else {
        assigned.push(member_id);
    }

    // pushing the change
    if let Ok(new_task) = taiga.assign_task(task.id, assigned, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not assign task");
        exit(1);
    }
}

pub fn taiga_due(taiga: &mut Taiga, args: DueTaskArgs) {
    // getting necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |_| false);
    let task = tasks.get_task(args.id);

    // pushing the change
    if let Ok(new_task) = taiga.due_task(task.id, args.due_date, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not assign task");
        exit(1);
    }
}

pub fn taiga_team(taiga: &mut Taiga, args: TeamTaskArgs) {
    // getting necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |_| false);
    let task = tasks.get_task(args.id);

    // pushing the change
    if let Ok(new_task) = taiga.team_task(task.id, args.remove, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not assign task");
        exit(1);
    }
}

pub fn taiga_client(taiga: &mut Taiga, args: ClientTaskArgs) {
    // getting necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |_| false);
    let task = tasks.get_task(args.id);

    // pushing the change
    if let Ok(new_task) = taiga.client_task(task.id, args.remove, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not assign task");
        exit(1);
    }
}

pub fn taiga_block(taiga: &mut Taiga, args: BlockTaskArgs) {
    // getting necessary information
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |_| false);
    let task = tasks.get_task(args.id);

    // pushing the change
    if let Ok(new_task) = taiga.block_task(task.id, args.remove, task.version) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not assign task");
        exit(1);
    }
}

pub fn taiga_modify(taiga: &mut Taiga, args: ModifyTaskArgs) {
    let project = taiga.find_project(args.project).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    let mut tasks = taiga.tasks_from_cache(project.id, |tasks| {
        if let Some(status) = &args.status {
            if !tasks.statuses.iter().any(|s| s.slug == *status) {
                return true;
            }
        }
        if let Some(assign) = &args.assign {
            for username in assign {
                if username != "me" && !tasks.members.iter().any(|m| m.username == *username) {
                    return true;
                }
            }
        }
        false
    });

    let mut status_id = 0;
    if let Some(status) = &args.status {
        status_id = if let Some(status) = tasks.statuses.iter().find(|s| s.slug == *status) {
            status.id
        } else {
            eprintln!("Error, could not find given status");
            exit(1);
        };
    }

    let mut assigned_ids = Vec::new();
    if let Some(assign) = &args.assign {
        for username in assign {
            let member_id = if username == "me" {
                tasks
                    .members
                    .iter()
                    .find(|member| member.id == taiga.id)
                    .map(|m| m.id)
                    .unwrap_or_else(|| {
                        eprintln!("Could not find your username on the project");
                        exit(1);
                    })
            } else {
                tasks
                    .members
                    .iter()
                    .find(|member| member.username == *username)
                    .map(|m| m.id)
                    .unwrap_or_else(|| {
                        eprintln!("Could not find username on the project");
                        exit(1);
                    })
            };
            assigned_ids.push(member_id);
        }
    }

    let task = tasks.get_task(args.id);

    let status = if args.status.is_some() {
        status_id
    } else {
        task.status_id
    };

    let rename = if let Some(rename) = args.rename {
        rename
    } else {
        task.name.clone()
    };

    let assign = if args.assign.is_some() {
        let mut combined_ids = task.assigned.clone();
        combined_ids.extend(assigned_ids);
        combined_ids.sort();
        combined_ids.dedup();
        combined_ids
    } else {
        task.assigned.clone()
    };

    let due_date = args
        .due_date
        .or_else(|| task.due.map(|due| due.format("%Y-%m-%d").to_string()));

    let team = if let Some(team) = args.team {
        team
    } else {
        task.team
    };

    let client = if let Some(client) = args.client {
        client
    } else {
        task.client
    };

    let block = if let Some(block) = args.block {
        block
    } else {
        task.blocked
    };

    if let Ok(new_task) = taiga.modify_task(
        task.id,
        status,
        rename,
        assign,
        due_date,
        team,
        client,
        block,
        task.version,
    ) {
        *task = new_task;
        tasks.save_cache();
    } else {
        eprintln!("Error, could not assign task");
        exit(1);
    }
}

fn format_due(due: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = due.signed_duration_since(now);

    // Check if the duration is negative, indicating the due date is in the past
    let is_past = duration.num_seconds() < 0;
    let duration = duration.abs(); // Work with absolute value for formatting

    if duration.num_days() >= 365 {
        // format!("{}y{}", duration.num_days() / 365, if is_past { " ago" } else { "" })
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
