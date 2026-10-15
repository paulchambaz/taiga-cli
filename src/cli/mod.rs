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
        "complete" => parse_complete(rest).unwrap_or_else(|| exit(1)),
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

fn parse_complete(args: &[String]) -> Option<TaigaCmd> {
    let (candidates, project) = match args {
        [what] if what == "projects" => (Candidates::Projects, None),
        [what, project] => {
            let candidates = match what.as_str() {
                "statuses" => Candidates::Statuses,
                "members" => Candidates::Members,
                "sprints" => Candidates::Sprints,
                "tasks" => Candidates::Tasks,
                _ => return None,
            };
            (candidates, Some(project.clone()))
        }
        _ => return None,
    };
    Some(TaigaCmd::Complete(CompleteArgs {
        candidates,
        project,
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    fn task(verb: &str, words: &[&str]) -> Option<TaigaCmd> {
        parse_task("demo", verb, 3, &strings(words))
    }

    fn plain(project: &str) -> String {
        project.to_string()
    }

    #[test]
    fn verbs_have_aliases_and_canonical_names() {
        assert_eq!(task_verb("mod"), Some("modify"));
        assert_eq!(task_verb("modify"), Some("modify"));
        assert_eq!(task_verb("del"), Some("delete"));
        assert_eq!(task_verb("delete"), Some("delete"));
        assert_eq!(task_verb("done"), Some("done"));
        assert_eq!(task_verb("new"), None);
        assert_eq!(task_verb("search"), None);
    }

    #[test]
    fn the_id_comes_before_or_after_the_verb() {
        let rest = strings(&["x"]);
        let split = |words: &[&str]| {
            let words = strings(words);
            split_task_command(&words).map(|(verb, id, rest)| (verb, id, rest.to_vec()))
        };
        assert_eq!(split(&["done", "3"]), Some(("done", 3, vec![])));
        assert_eq!(split(&["3", "done"]), Some(("done", 3, vec![])));
        assert_eq!(split(&["mod", "3", "x"]), Some(("modify", 3, rest.clone())));
        assert_eq!(split(&["3", "del", "x"]), Some(("delete", 3, rest)));
    }

    #[test]
    fn a_missing_bad_or_misplaced_id_or_verb_is_no_command() {
        let split = |words: &[&str]| split_task_command(&strings(words)).is_some();
        assert!(!split(&["done"]));
        assert!(!split(&["3"]));
        assert!(!split(&["done", "x"]));
        assert!(!split(&["3", "foo"]));
        assert!(!split(&["foo", "done"]));
        assert!(!split(&["new", "3"]));
        assert!(!split(&[]));
    }

    #[test]
    fn remove_may_sit_anywhere_among_the_arguments() {
        let split = |words: &[&str]| {
            let words = strings(words);
            remove_flag(&words)
                .map(|(remove, others)| (remove, others.into_iter().cloned().collect::<Vec<_>>()))
        };
        assert_eq!(split(&["--remove", "a"]), Some((true, strings(&["a"]))));
        assert_eq!(split(&["a", "--remove"]), Some((true, strings(&["a"]))));
        assert_eq!(split(&["a"]), Some((false, strings(&["a"]))));
        assert_eq!(split(&[]), Some((false, vec![])));
        assert_eq!(split(&["--x"]), None);
        assert_eq!(split(&["--remove", "--remove"]), None);
    }

    #[test]
    fn move_takes_exactly_one_status() {
        assert_eq!(
            task("move", &["done"]),
            Some(TaigaCmd::MoveTask(MoveTaskArgs {
                project: plain("demo"),
                id: 3,
                status: plain("done"),
            }))
        );
        assert_eq!(task("move", &[]), None);
        assert_eq!(task("move", &["a", "b"]), None);
    }

    #[test]
    fn done_and_delete_take_no_arguments() {
        let args = TaskArgs {
            project: plain("demo"),
            id: 3,
        };
        assert_eq!(task("done", &[]), Some(TaigaCmd::DoneTask(args)));
        let args = TaskArgs {
            project: plain("demo"),
            id: 3,
        };
        assert_eq!(task("delete", &[]), Some(TaigaCmd::DeleteTask(args)));
        assert_eq!(task("done", &["x"]), None);
        assert_eq!(task("delete", &["--remove"]), None);
    }

    #[test]
    fn rename_joins_the_words_and_refuses_options() {
        assert_eq!(
            task("rename", &["a", "new", "name"]),
            Some(TaigaCmd::RenameTask(RenameTaskArgs {
                project: plain("demo"),
                id: 3,
                name: plain("a new name"),
            }))
        );
        assert_eq!(task("rename", &[]), None);
        assert_eq!(task("rename", &["a", "--x"]), None);
    }

    #[test]
    fn assign_takes_one_user_and_an_optional_remove() {
        let args = |remove| {
            Some(TaigaCmd::AssignTask(AssignTaskArgs {
                project: plain("demo"),
                id: 3,
                username: plain("paul"),
                remove,
            }))
        };
        assert_eq!(task("assign", &["paul"]), args(false));
        assert_eq!(task("assign", &["--remove", "paul"]), args(true));
        assert_eq!(task("assign", &["paul", "--remove"]), args(true));
        assert_eq!(task("assign", &[]), None);
        assert_eq!(task("assign", &["a", "b"]), None);
        assert_eq!(task("assign", &["--remove"]), None);
        assert_eq!(task("assign", &["paul", "--x"]), None);
    }

    #[test]
    fn due_takes_a_date_or_removes_it() {
        let args = |due_date| {
            Some(TaigaCmd::DueTask(DueTaskArgs {
                project: plain("demo"),
                id: 3,
                due_date,
            }))
        };
        assert_eq!(
            task("due", &["2030-01-02"]),
            args(Some(plain("2030-01-02")))
        );
        assert_eq!(task("due", &["--remove"]), args(None));
        assert_eq!(task("due", &[]), None);
        assert_eq!(task("due", &["--remove", "2030-01-02"]), None);
        assert_eq!(task("due", &["garbage"]), None);
        assert_eq!(task("due", &["2030-01-02", "2030-01-03"]), None);
    }

    #[test]
    fn sprint_takes_a_name_or_removes_it() {
        let args = |sprint| {
            Some(TaigaCmd::SprintTask(SprintTaskArgs {
                project: plain("demo"),
                id: 3,
                sprint,
            }))
        };
        assert_eq!(task("sprint", &["s1"]), args(Some(plain("s1"))));
        assert_eq!(task("sprint", &["--remove"]), args(None));
        assert_eq!(task("sprint", &[]), None);
        assert_eq!(task("sprint", &["--remove", "s1"]), None);
    }

    #[test]
    fn team_client_and_block_only_take_remove() {
        let args = |remove| ToggleTaskArgs {
            project: plain("demo"),
            id: 3,
            remove,
        };
        assert_eq!(task("team", &[]), Some(TaigaCmd::TeamTask(args(false))));
        assert_eq!(
            task("client", &["--remove"]),
            Some(TaigaCmd::ClientTask(args(true)))
        );
        assert_eq!(task("block", &[]), Some(TaigaCmd::BlockTask(args(false))));
        assert_eq!(task("team", &["x"]), None);
        assert_eq!(task("block", &["--x"]), None);
    }

    #[test]
    fn modify_goes_through_the_shared_modifier_parser() {
        assert!(matches!(
            task("modify", &["+team"]),
            Some(TaigaCmd::ModifyTask(_))
        ));
        assert_eq!(task("modify", &["a:b"]), None);
    }

    fn subtask(words: &[&str]) -> Option<SubtaskAction> {
        match parse_subtask(plain("demo"), 3, &strings(words))? {
            TaigaCmd::Subtask(args) => Some(args.action),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn subtask_new_takes_user_and_description_in_any_position() {
        let expected = SubtaskAction::New {
            name: plain("write tests"),
            description: plain("soon"),
            assign: Some(plain("paul")),
        };
        assert_eq!(
            subtask(&["new", "write", "tests", "@paul", "desc:soon"]),
            Some(expected)
        );
        let expected = SubtaskAction::New {
            name: plain("write tests"),
            description: plain("soon"),
            assign: Some(plain("paul")),
        };
        assert_eq!(
            subtask(&["new", "desc:soon", "@paul", "write", "tests"]),
            Some(expected)
        );
        assert_eq!(subtask(&["new"]), None);
        assert_eq!(subtask(&["new", "@paul"]), None);
        assert_eq!(subtask(&["new", "x", "@"]), None);
        assert_eq!(subtask(&["new", "x", "@a", "@b"]), None);
        assert_eq!(subtask(&["new", "x", "desc:a", "desc:b"]), None);
    }

    #[test]
    fn subtask_list_describe_and_delete() {
        assert_eq!(subtask(&["list"]), Some(SubtaskAction::List));
        assert_eq!(subtask(&["list", "x"]), None);
        assert_eq!(
            subtask(&["desc", "2", "some", "text"]),
            Some(SubtaskAction::Describe {
                index: 2,
                description: plain("some text")
            })
        );
        assert_eq!(
            subtask(&["describe", "1", "x"]),
            Some(SubtaskAction::Describe {
                index: 1,
                description: plain("x")
            })
        );
        assert_eq!(subtask(&["describe", "2"]), None);
        assert_eq!(subtask(&["describe", "x", "text"]), None);
        assert_eq!(
            subtask(&["del", "1"]),
            Some(SubtaskAction::Delete { index: 1 })
        );
        assert_eq!(
            subtask(&["delete", "1"]),
            Some(SubtaskAction::Delete { index: 1 })
        );
        assert_eq!(subtask(&["delete", "x"]), None);
        assert_eq!(subtask(&["delete"]), None);
        assert_eq!(subtask(&["foo"]), None);
        assert_eq!(subtask(&[]), None);
    }

    #[test]
    fn login_takes_an_optional_address() {
        let login = |address| Some(TaigaCmd::Login(LoginArgs { address }));
        assert_eq!(parse_login(&[]), login(None));
        assert_eq!(
            parse_login(&strings(&["--address", "http://x"])),
            login(Some(plain("http://x")))
        );
        assert_eq!(parse_login(&strings(&["--address"])), None);
        assert_eq!(parse_login(&strings(&["x"])), None);
    }

    #[test]
    fn burndown_takes_an_optional_sprint() {
        let burndown = |sprint| {
            Some(TaigaCmd::ProjectBurndown(ProjectBurndownArgs {
                project: plain("demo"),
                sprint,
            }))
        };
        assert_eq!(parse_burndown("demo", &[]), burndown(None));
        assert_eq!(
            parse_burndown("demo", &strings(&["s1"])),
            burndown(Some(plain("s1")))
        );
        assert_eq!(parse_burndown("demo", &strings(&["--x"])), None);
        assert_eq!(parse_burndown("demo", &strings(&["a", "b"])), None);
    }
}
