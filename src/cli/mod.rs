//! Command line parsing: turns the process arguments into a [`TaigaCmd`].

mod args;
mod help;
mod modifiers;

use std::env;
use std::process::exit;

pub use self::args::*;
use self::help::{help_message, project_help, top_help};
use self::modifiers::{parse_date, parse_modify, parse_new, parse_search};
use crate::taiga::Taiga;

/// Parses the process arguments, printing help or the version and exiting when asked to.
///
/// Project names come from the cached login, which is `None` before the first `taiga login`.
pub fn parse_args(taiga: Option<&Taiga>) -> TaigaCmd {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some((verb, rest)) = args.split_first() else {
        return TaigaCmd::Default;
    };

    if args.iter().any(|arg| arg == "--version") {
        println!("taiga {}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    let projects: Vec<&str> = taiga
        .map(|taiga| taiga.projects.iter().map(|p| p.name.as_str()).collect())
        .unwrap_or_default();

    match verb.as_str() {
        "login" => finish("", "login", None, rest, || parse_login(rest)),
        "projects" => finish("", "projects", None, rest, || {
            rest.is_empty().then_some(TaigaCmd::Projects)
        }),
        "--help" => {
            top_help(&projects).display();
            exit(0);
        }
        project if projects.contains(&project) => parse_project(project, rest),
        _ if taiga.is_none() => {
            eprintln!("Error, not logged in, run `taiga login`");
            exit(1);
        }
        _ => {
            eprintln!("Error, not a valid command");
            exit(1);
        }
    }
}

/// Runs a parser, showing the command help on `--help` (exit 0) or on bad usage (exit 1).
fn finish(
    project: &str,
    verb: &str,
    id: Option<usize>,
    args: &[String],
    parse: impl FnOnce() -> Option<TaigaCmd>,
) -> TaigaCmd {
    if args.iter().any(|arg| arg == "--help") {
        help_message(project, verb, id).display();
        exit(0);
    }
    parse().unwrap_or_else(|| {
        help_message(project, verb, id).display();
        exit(1);
    })
}

fn parse_project(project: &str, args: &[String]) -> TaigaCmd {
    let Some((first, rest)) = args.split_first() else {
        return finish(project, "search", None, args, || {
            parse_search(project, args)
        });
    };

    match first.as_str() {
        "new" | "add" => finish(project, "new", None, rest, || parse_new(project, rest)),
        "search" => finish(project, "search", None, rest, || {
            parse_search(project, rest)
        }),
        "burndown" => finish(project, "burndown", None, rest, || {
            parse_burndown(project, rest)
        }),
        "users" => TaigaCmd::ProjectUsers(ProjectUserArgs {
            project: project.to_string(),
        }),
        "--help" => {
            project_help(project).display();
            exit(0);
        }
        _ => match split_task_command(args) {
            Some((verb, id, rest)) => finish(project, verb, Some(id), rest, || {
                parse_task(project, verb, id, rest)
            }),
            None if task_verb(first).is_some() || first.parse::<usize>().is_ok() => {
                project_help(project).display();
                exit(1);
            }
            None => finish(project, "search", None, args, || {
                parse_search(project, args)
            }),
        },
    }
}

fn task_verb(word: &str) -> Option<&'static str> {
    Some(match word {
        "move" => "move",
        "done" => "done",
        "rename" => "rename",
        "assign" => "assign",
        "due" => "due",
        "sprint" => "sprint",
        "subtask" => "subtask",
        "team" => "team",
        "client" => "client",
        "block" => "block",
        "mod" | "modify" => "modify",
        "del" | "delete" => "delete",
        _ => return None,
    })
}

/// Splits `done 3 ...` and `3 done ...` alike into the verb, the task id and the rest.
fn split_task_command(args: &[String]) -> Option<(&'static str, usize, &[String])> {
    let [first, second, rest @ ..] = args else {
        return None;
    };

    match task_verb(first) {
        Some(verb) => Some((verb, second.parse().ok()?, rest)),
        None => Some((task_verb(second)?, first.parse().ok()?, rest)),
    }
}

fn parse_login(args: &[String]) -> Option<TaigaCmd> {
    let address = match args {
        [] => None,
        [flag, address, ..] if flag == "--address" => Some(address.clone()),
        _ => return None,
    };
    Some(TaigaCmd::Login(LoginArgs { address }))
}

fn parse_burndown(project: &str, args: &[String]) -> Option<TaigaCmd> {
    let sprint = match args {
        [] => None,
        [sprint] if !sprint.starts_with("--") => Some(sprint.clone()),
        _ => return None,
    };
    Some(TaigaCmd::ProjectBurndown(ProjectBurndownArgs {
        project: project.to_string(),
        sprint,
    }))
}

/// Separates `--remove` from the other arguments; any other option is a usage error.
fn remove_flag(args: &[String]) -> Option<(bool, Vec<&String>)> {
    let (options, others): (Vec<&String>, Vec<&String>) =
        args.iter().partition(|arg| arg.starts_with("--"));

    match options.as_slice() {
        [] => Some((false, others)),
        [option] if option.as_str() == "--remove" => Some((true, others)),
        _ => None,
    }
}

fn parse_task(project: &str, verb: &str, id: usize, args: &[String]) -> Option<TaigaCmd> {
    let project = project.to_string();
    match verb {
        "move" => match args {
            [status] => Some(TaigaCmd::MoveTask(MoveTaskArgs {
                project,
                id,
                status: status.clone(),
            })),
            _ => None,
        },
        "done" => args
            .is_empty()
            .then_some(TaigaCmd::DoneTask(TaskArgs { project, id })),
        "delete" => args
            .is_empty()
            .then_some(TaigaCmd::DeleteTask(TaskArgs { project, id })),
        "rename" => {
            (!args.is_empty() && !args.iter().any(|arg| arg.starts_with("--"))).then(|| {
                TaigaCmd::RenameTask(RenameTaskArgs {
                    project,
                    id,
                    name: args.join(" "),
                })
            })
        }
        "assign" => {
            let (remove, others) = remove_flag(args)?;
            let [username] = others.as_slice() else {
                return None;
            };
            Some(TaigaCmd::AssignTask(AssignTaskArgs {
                project,
                id,
                username: username.to_string(),
                remove,
            }))
        }
        "due" => {
            let (remove, others) = remove_flag(args)?;
            let due_date = match (remove, others.as_slice()) {
                (true, []) => None,
                (false, [date]) => Some(parse_date(date)?),
                _ => return None,
            };
            Some(TaigaCmd::DueTask(DueTaskArgs {
                project,
                id,
                due_date,
            }))
        }
        "sprint" => {
            let (remove, others) = remove_flag(args)?;
            let sprint = match (remove, others.as_slice()) {
                (true, []) => None,
                (false, [sprint]) => Some(sprint.to_string()),
                _ => return None,
            };
            Some(TaigaCmd::SprintTask(SprintTaskArgs {
                project,
                id,
                sprint,
            }))
        }
        "team" | "client" | "block" => {
            let (remove, others) = remove_flag(args)?;
            if !others.is_empty() {
                return None;
            }
            let toggle = ToggleTaskArgs {
                project,
                id,
                remove,
            };
            Some(match verb {
                "team" => TaigaCmd::TeamTask(toggle),
                "client" => TaigaCmd::ClientTask(toggle),
                _ => TaigaCmd::BlockTask(toggle),
            })
        }
        "modify" => parse_modify(&project, id, args),
        "subtask" => parse_subtask(project, id, args),
        _ => None,
    }
}

fn parse_subtask(project: String, story_id: usize, args: &[String]) -> Option<TaigaCmd> {
    let (verb, rest) = args.split_first()?;
    let action = match (verb.as_str(), rest) {
        ("new", _) => parse_subtask_new(rest)?,
        ("list", []) => SubtaskAction::List,
        ("desc" | "describe", [index, description @ ..]) if !description.is_empty() => {
            SubtaskAction::Describe {
                index: index.parse().ok()?,
                description: description.join(" "),
            }
        }
        ("del" | "delete", [index]) => SubtaskAction::Delete {
            index: index.parse().ok()?,
        },
        _ => return None,
    };
    Some(TaigaCmd::Subtask(SubtaskArgs {
        project,
        story_id,
        action,
    }))
}

fn parse_subtask_new(args: &[String]) -> Option<SubtaskAction> {
    let mut assign = None;
    let mut description = None;
    let mut name = Vec::new();

    for arg in args {
        if let Some(username) = arg.strip_prefix('@') {
            if username.is_empty() || assign.replace(username.to_string()).is_some() {
                return None;
            }
        } else if let Some(text) = arg.strip_prefix("desc:") {
            if text.is_empty() || description.replace(text.to_string()).is_some() {
                return None;
            }
        } else {
            name.push(arg.as_str());
        }
    }

    (!name.is_empty()).then(|| SubtaskAction::New {
        name: name.join(" "),
        description: description.unwrap_or_default(),
        assign,
    })
}
