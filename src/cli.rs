use std::{cmp::Ordering, env, process::exit};

use colored::Colorize;
use prettytable::{ format::FormatBuilder, Cell, Row, Table, };

use crate::{taiga::Taiga, time::TaigaTime};

#[derive(Debug)]
pub struct LoginArgs {
    pub address: Option<String>,
}

#[derive(Debug)]
pub struct ProjectArgs {
    pub project: String,
}

#[derive(Debug)]
pub struct NewTaskArgs {
    pub project: String,
    pub status: Option<String>,
    pub name: String,
    pub assign: Vec<String>,
    pub due_date: Option<String>,
    pub team: bool,
    pub client: bool,
    pub block: bool,
}

#[derive(Debug)]
pub struct MoveTaskArgs {
    pub project: String,
    pub id: usize,
    pub status: String,
}

#[derive(Debug)]
pub struct DoneTaskArgs {
    pub project: String,
    pub id: usize,
}

#[derive(Debug)]
pub struct RenameTaskArgs {
    pub project: String,
    pub id: usize,
    pub name: String,
}

#[derive(Debug)]
pub struct AssignTaskArgs {
    pub project: String,
    pub id: usize,
    pub username: String,
    pub remove: bool,
}

#[derive(Debug)]
pub struct DueTaskArgs {
    pub project: String,
    pub id: usize,
    pub due_date: Option<String>,
}

#[derive(Debug)]
pub struct TeamTaskArgs {
    pub project: String,
    pub id: usize,
    pub remove: bool,
}

#[derive(Debug)]
pub struct ClientTaskArgs {
    pub project: String,
    pub id: usize,
    pub remove: bool,
}

#[derive(Debug)]
pub struct BlockTaskArgs {
    pub project: String,
    pub id: usize,
    pub remove: bool,
}

#[derive(Debug)]
pub struct ModifyTaskArgs {
    pub project: String,
    pub id: usize,
    pub status: Option<String>,
    pub rename: Option<String>,
    pub assign: Option<Vec<String>>,
    pub due_date: Option<String>,
    pub team: Option<bool>,
    pub client: Option<bool>,
    pub block: Option<bool>,
}


#[derive(Debug)]
pub struct SearchTaskArgs {
    pub project: String,
    pub include_statuses: Vec<String>,
    pub exclude_statuses: Vec<String>,
    pub include_assigned: Vec<String>,
    pub exclude_assigned: Vec<String>,
    pub due_date: Option<String>,
    pub team: Option<bool>,
    pub client: Option<bool>,
    pub block: Option<bool>,
    pub query: Vec<String>,
}

#[derive(Debug)]
pub struct ProjectUserArgs {
    pub project: String,
}

#[derive(Debug)]
pub struct DeleteTaskArgs {
    pub project: String,
    pub id: usize,
}

#[derive(Debug)]
pub struct ProjectBurndownArgs {
    pub project: String,
}

#[derive(Debug)]
pub enum TaigaCmd {
    Default,
    Login(LoginArgs),
    Projects,
    NewTask(NewTaskArgs),
    MoveTask(MoveTaskArgs),
    DoneTask(DoneTaskArgs),
    RenameTask(RenameTaskArgs),
    AssignTask(AssignTaskArgs),
    DueTask(DueTaskArgs),
    TeamTask(TeamTaskArgs),
    ClientTask(ClientTaskArgs),
    BlockTask(BlockTaskArgs),
    ModifyTask(ModifyTaskArgs),
    SearchTask(SearchTaskArgs),
    DeleteTask(DeleteTaskArgs),
    ProjectUsers(ProjectUserArgs),
    ProjectBurndown(ProjectBurndownArgs),
}

pub fn cli(taiga: &Option<Taiga>) -> TaigaCmd {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return TaigaCmd::Default;
    }

    if args.contains(&"--version".to_string()) {
        cli_version();
        exit(0);
    }

    let mut allowed_projects = Vec::new();
    if let Some(taiga) = taiga {
        for project in &taiga.projects {
            allowed_projects.push(project.name.clone());
        }
    }

    let verb = args.first().expect("Could not get verb");

    match verb.as_str() {
        "login" => cli_login(&args[1..]),
        "projects" => cli_projects(&args[1..]),
        "--help" => {
            cli_help(allowed_projects);
            exit(0);
        }
        "--version" => {
            cli_version();
            exit(0);
        }
        project => {
            let project = project.to_string();
            if allowed_projects.contains(&project) {
                cli_project(project, &args[1..])
            } else {
                eprintln!("Error, not a valid command");
                exit(1);
            }
        }
    }
}

fn cli_login(args: &[String]) -> TaigaCmd {
    if args.is_empty() {
        return TaigaCmd::Login(LoginArgs { address: None });
    }

    if args.contains(&"--help".to_string()) {
        cli_login_help();
        exit(0);
    }

    let verb = args.first().expect("Could not get verb");

    match verb.as_str() {
        "--address" => {
            if let Some(address) = args.get(1) {
                TaigaCmd::Login(LoginArgs { address: Some(address.to_string()) })
            } else {
                cli_login_help();
                exit(1);
            }
        },
        _ => {
            cli_login_help();
            exit(1);
        }
    }
}

fn cli_login_help() {
    let mut help_message = HelpMessage::new("Login to a taiga instance", "taiga login", "<OPTIONS>");
    help_message.add_section("Options");
    help_message.add_command("--address <ADDRESS>", "Address to login to");
    help_message.add_command("--help", "Print help message and exit");
    help_message.display();
}

fn cli_projects(args: &[String]) -> TaigaCmd {
    if !args.is_empty() {
        cli_login_projects();
        exit(1);
    }

    TaigaCmd::Projects
}

fn cli_login_projects() {
    let mut help_message = HelpMessage::new("Refresh and print the project list", "taiga projects", "");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print help message and exit");
    help_message.display();
}

fn cli_project(project: String, args: &[String]) -> TaigaCmd {
    if args.is_empty() {
        return cli_project_search(project, args);
    }

    let verb = args.first().expect("Could not get verb");

    match verb.as_str() {
        "add" | "new" => cli_project_new(project, &args[1..]),
        "move" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_move(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "done" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_done(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "rename" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_rename(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "assign" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_assign(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "due" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_due(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "team" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_team(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "client" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_client(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "block" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_block(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "mod" | "modify" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_modify(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "del" | "delete" => {
            if let Some(id) = args.get(1).and_then(|id| id.parse::<usize>().ok()) {
                cli_project_delete(project, id, &args[2..])
            } else {
                cli_help_project(project);
                exit(1);
            }
        },
        "search" => cli_project_search(project, &args[1..]),
        "burndown" => cli_project_burndown(project, &args[1..]),
        "users" => cli_project_users(project, &args[1..]),
        "--help" => {
            cli_help_project(project);
            exit(0);
        },
        id => {
            if let Ok(id) = id.parse::<usize>() {
                if let Some(new_verb) = args.get(1) {
                    match new_verb.as_str() {
                        "move" => cli_project_move(project, id, &args[2..]),
                        "done" => cli_project_done(project, id, &args[2..]),
                        "rename" => cli_project_rename(project, id, &args[2..]),
                        "assign" => cli_project_assign(project, id, &args[2..]),
                        "due" => cli_project_due(project, id, &args[2..]),
                        "team" => cli_project_team(project, id, &args[2..]),
                        "client" => cli_project_client(project, id, &args[2..]),
                        "block" => cli_project_block(project, id, &args[2..]),
                        "mod" | "modify" => cli_project_modify(project, id, &args[2..]),
                        "del" | "delete" => cli_project_delete(project, id, &args[2..]),
                        _ => {
                            cli_help_project(project);
                            exit(1);
                        }
                    }
                } else {
                    cli_help_project(project);
                    exit(1);
                }
            } else {
                cli_project_search(project, args)
            }
        }
    }
}

fn cli_project_new(project: String, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_new_help(project);
        exit(0);
    }

    let mut options = Vec::new();
    let mut statuses = Vec::new();
    let mut assigned = Vec::new();
    let mut dues = Vec::new();
    let mut teams = Vec::new();
    let mut clients = Vec::new();
    let mut blocks = Vec::new();
    let mut others = Vec::new();
    let mut can_continuous = true;

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("status:") || arg.starts_with("stat:") {
            statuses.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with('@') {
            assigned.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("due:") {
            dues.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with('+') || arg.starts_with('-') {
            match arg.as_str() {
                "+team" => teams.push(true),
                "-team" => teams.push(false),
                "+client" => clients.push(true),
                "-client" => clients.push(false),
                "+block" => blocks.push(true),
                "-block" => blocks.push(false),
                _ => {
                    cli_project_new_help(project);
                    exit(1);
                }
            };
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.contains(':') {
            cli_project_new_help(project);
            exit(1);
        } else {
            if !can_continuous {
                cli_project_new_help(project);
                exit(0);
            }
            others.push(arg.clone());
        }
    }

    if !options.is_empty() {
        cli_project_new_help(project);
        exit(1);
    }


    let team = match teams.len().cmp(&1) {
        Ordering::Equal => *teams.first().expect("Could not get team"),
        Ordering::Greater => {
            cli_project_new_help(project);
            exit(1);
        },
        Ordering::Less => false,
    };

    let client = match clients.len().cmp(&1) {
        Ordering::Equal => *clients.first().expect("Could not get client"),
        Ordering::Greater => {
            cli_project_new_help(project);
            exit(1);
        },
        Ordering::Less => false,
    };

    let block = match blocks.len().cmp(&1) {
        Ordering::Equal => *blocks.first().expect("Could not get block"),
        Ordering::Greater => {
            cli_project_new_help(project);
            exit(1);
        },
        Ordering::Less => false,
    };

    let due_date = match dues.len().cmp(&1) {
        Ordering::Equal => {
            let due_str = *dues.first().expect("Could not get due");
            let date_rest = &due_str["due:".len()..];
            if date_rest.is_empty() {
                cli_project_new_help(project);
                exit(1);
            }
            let due_date = TaigaTime::new(date_rest.to_string());
            Some(due_date.format())
        },
        Ordering::Greater => {
            cli_project_new_help(project);
            exit(1);
        },
        Ordering::Less => None,
    };

    let status = match statuses.len().cmp(&1) {
        Ordering::Equal => {
            let status_str = *statuses.first().expect("Could not get due");

            let status_rest = if let Some(rest) = status_str.strip_prefix("status:") {
                rest
            } else if let Some(rest) = status_str.strip_prefix("stat:") {
                rest
            } else {
                unreachable!()
            };

            if status_rest.is_empty() {
                cli_project_new_help(project);
                exit(1);
            }
            Some(status_rest.to_string())
        },
        Ordering::Greater => {
            cli_project_new_help(project);
            exit(1);
        },
        Ordering::Less => None,
    };

    let assign = if assigned.is_empty() {
        Vec::new()
    } else {
        let assigned = assigned.iter().map(|string| {
            let sub_str = &string[1..];
            if sub_str.is_empty() {
                cli_project_new_help(project.clone());
                exit(1);
            }
            sub_str.to_string()
        }).collect::<Vec<String>>();

        assigned
    };

    let name = if others.is_empty() {
        cli_project_new_help(project);
        exit(1);
    } else {
        others.join(" ")
    };

    TaigaCmd::NewTask(NewTaskArgs { project, status, name, assign, due_date, team, client, block })
}

fn cli_project_new_help(project: String) {
    let mut help_message = HelpMessage::new("Create a new task", &format!("taiga {} new", project), "<MODIFIERS> <OPTIONS> ...");
    help_message.add_section("Modifiers");
    help_message.add_command("status:<STATUS>", "The status to set the task to [default: new]");
    help_message.add_command("@<USERNAME>", "The user to assign the task to");
    help_message.add_command("due:<DATE>", "The due date to give to the task");
    help_message.add_command("+/-team", "Set or unset the team requirement");
    help_message.add_command("+/-client", "Set or unset the client requirement");
    help_message.add_command("+/-block", "Set or unset the block");
    help_message.add_command("...", "The name for the task");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_move(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_move_help(project, id);
        exit(0);
    }

    if args.len() != 1 {
        cli_project_move_help(project, id);
        exit(1);
    }

    let status = args.first().expect("Could not get status").to_string();
    TaigaCmd::MoveTask(MoveTaskArgs { project, id, status })
}

fn cli_project_move_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Move a task to a status", &format!("taiga {} move {}", project, id), "<ARGS> <OPTIONS>");
    help_message.add_section("Arguments");
    help_message.add_command("<STATUS>", "The status to move the task to");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_done(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_done_help(project, id);
        exit(0);
    }

    if !args.is_empty() {
        cli_project_done_help(project, id);
        exit(1);
    }

    TaigaCmd::DoneTask(DoneTaskArgs { project, id })
}

fn cli_project_done_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Declare a task as done", &format!("taiga {} done {}", project, id), "<OPTIONS>");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_delete(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_delete_help(project, id);
        exit(0);
    }

    if !args.is_empty() {
        cli_project_delete_help(project, id);
        exit(1);
    }

    TaigaCmd::DeleteTask(DeleteTaskArgs { project, id })
}

fn cli_project_delete_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Delete a task", &format!("taiga {} delete {}", project, id), "<OPTIONS>");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_rename(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_rename_help(project, id);
        exit(0);
    }

    let contains_options = args.iter().any(|s| s.starts_with("--"));

    if args.is_empty() || contains_options {
        cli_project_rename_help(project, id);
        exit(1);
    }

    let name = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ");

    TaigaCmd::RenameTask(RenameTaskArgs { project, id, name })
}

fn cli_project_rename_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Rename a task", &format!("taiga {} rename {}", project, id), "<ARGS> <OPTIONS>");
    help_message.add_section("Arguments");
    help_message.add_command("<NAME>", "The new name for the task");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_assign(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_assign_help(project, id);
        exit(0);
    }

    let mut options = Vec::new();
    let mut others = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
        } else {
            others.push(arg);
        }
    }

    let mut remove = false;

    if options.len() == 1 && options.first().expect("Could not get first option").as_str() == "--remove" {
        remove = true;
    } else if !options.is_empty() {
        cli_project_assign_help(project, id);
        exit(1);
    }

    if others.len() != 1 {
        cli_project_assign_help(project, id);
        exit(1);
    }

    let username = others.first().expect("Could not get status").to_string();
    TaigaCmd::AssignTask(AssignTaskArgs { project, id, username, remove })
}

fn cli_project_assign_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Assign a task", &format!("taiga {} assign {}", project, id), "<ARGS> <OPTIONS>");
    help_message.add_section("Arguments");
    help_message.add_command("<USERNAME>", "The username to assign the task to");
    help_message.add_section("Options");
    help_message.add_command("--remove", "Remove the user instead of adding it");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_due(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_due_help(project, id);
        exit(0);
    }

    let mut options = Vec::new();
    let mut others = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
        } else {
            others.push(arg);
        }
    }

    if options.len() == 1 && options.first().expect("Could not get first option").as_str() == "--remove" {
        if others.is_empty() {
            return TaigaCmd::DueTask(DueTaskArgs { project, id, due_date: None });
        } else {
            cli_project_due_help(project, id);
            exit(1);
        }
    } else if !options.is_empty() {
        cli_project_due_help(project, id);
        exit(1);
    }

    if others.len() != 1 {
        cli_project_assign_help(project, id);
        exit(1);
    }

    let due_date = others.first().expect("Could not get status").to_string();
    let date = TaigaTime::new(due_date).format();

    TaigaCmd::DueTask(DueTaskArgs { project, id, due_date: Some(date) })
}

fn cli_project_due_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Set due date for a task", &format!("taiga {} due {}", project, id), "<ARGS> <OPTIONS>");
    help_message.add_section("Arguments");
    help_message.add_command("<DATE>", "The due date to give to the task");
    help_message.add_section("Options");
    help_message.add_command("--remove", "Remove the user instead of adding it");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_team(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_team_help(project, id);
        exit(0);
    }

    let mut options = Vec::new();
    let mut others = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
        } else {
            others.push(arg);
        }
    }

    if !others.is_empty() {
        cli_project_team_help(project, id);
        exit(1);
    }

    let remove = if options.len() == 1 && options.first().expect("Could not get first option").as_str() == "--remove" {
        true
    } else if !options.is_empty() {
        cli_project_team_help(project, id);
        exit(1);
    } else {
        false
    };

    TaigaCmd::TeamTask(TeamTaskArgs { project, id, remove })
}

fn cli_project_team_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Toggle team requirement for a task", &format!("taiga {} team {}", project, id), "<OPTIONS>");
    help_message.add_section("Options");
    help_message.add_command("--remove", "Remove the team requirement");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_client(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_client_help(project, id);
        exit(0);
    }

    let mut options = Vec::new();
    let mut others = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
        } else {
            others.push(arg);
        }
    }

    if !others.is_empty() {
        cli_project_client_help(project, id);
        exit(1);
    }

    let remove = if options.len() == 1 && options.first().expect("Could not get first option").as_str() == "--remove" {
        true
    } else if !options.is_empty() {
        cli_project_client_help(project, id);
        exit(1);
    } else {
        false
    };

    TaigaCmd::ClientTask(ClientTaskArgs { project, id, remove })
}

fn cli_project_client_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Toggle client requirement for a task", &format!("taiga {} client {}", project, id), "<OPTIONS>");
    help_message.add_section("Options");
    help_message.add_command("--remove", "Remove the client requirement");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_block(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_block_help(project, id);
        exit(0);
    }

    let mut options = Vec::new();
    let mut others = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
        } else {
            others.push(arg);
        }
    }

    if !others.is_empty() {
        cli_project_block_help(project, id);
        exit(1);
    }

    let remove = if options.len() == 1 && options.first().expect("Could not get first option").as_str() == "--remove" {
        true
    } else if !options.is_empty() {
        cli_project_block_help(project, id);
        exit(1);
    } else {
        false
    };

    TaigaCmd::BlockTask(BlockTaskArgs { project, id, remove })
}

fn cli_project_block_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Toggle block for a task", &format!("taiga {} block {}", project, id), "<OPTIONS>");
    help_message.add_section("Options");
    help_message.add_command("--remove", "Remove the block");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_modify(project: String, id: usize, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_modify_help(project, id);
        exit(0);
    }

    let mut options = Vec::new();

    let mut statuses = Vec::new();
    let mut assigned = Vec::new();

    let mut dues = Vec::new();

    let mut teams = Vec::new();
    let mut clients = Vec::new();
    let mut blocks = Vec::new();

    let mut others = Vec::new();
    let mut can_continuous = true;

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("status:") || arg.starts_with("stat:") {
            statuses.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with('@') {
            assigned.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("due:") {
            dues.push(arg);
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with('+') || arg.starts_with('-') {
            match arg.as_str() {
                "+team" => teams.push(true),
                "-team" => teams.push(false),
                "+client" => clients.push(true),
                "-client" => clients.push(false),
                "+block" => blocks.push(true),
                "-block" => blocks.push(false),
                _ => {
                    cli_project_new_help(project);
                    exit(1);
                }
            };
            if !others.is_empty() {
                can_continuous = false;
            }
        } else if arg.contains(':') {
            cli_project_new_help(project);
            exit(1);
        } else {
            if !can_continuous {
                cli_project_new_help(project);
                exit(1);
            }
            others.push(arg.clone());
        }
    }

    if !options.is_empty() {
        cli_project_modify_help(project, id);
        exit(1);
    }


    let team = match teams.len().cmp(&1) {
        Ordering::Equal => Some(*teams.first().expect("Could not get team")),
        Ordering::Greater => {
            cli_project_modify_help(project, id);
            exit(1);
        },
        Ordering::Less => None,
    };

    let client = match clients.len().cmp(&1) {
        Ordering::Equal => Some(*clients.first().expect("Could not get client")),
        Ordering::Greater => {
            cli_project_modify_help(project, id);
            exit(1);
        },
        Ordering::Less => None,
    };

    let block = match blocks.len().cmp(&1) {
        Ordering::Equal => Some(*blocks.first().expect("Could not get block")),
        Ordering::Greater => {
            cli_project_modify_help(project, id);
            exit(1);
        },
        Ordering::Less => None,
    };

    let due_date = match dues.len().cmp(&1) {
        Ordering::Equal => {
            let due_str = *dues.first().expect("Could not get due");
            let date_rest = &due_str["due:".len()..];
            if date_rest.is_empty() {
                cli_project_modify_help(project, id);
                exit(1);
            }
            let due_date = TaigaTime::new(date_rest.to_string());
            Some(due_date.format())
        },
        Ordering::Greater => {
            cli_project_modify_help(project, id);
            exit(1);
        },
        Ordering::Less => None,
    };

    let status = match statuses.len().cmp(&1) {
        Ordering::Equal => {
            let status_str = *statuses.first().expect("Could not get due");

            let status_rest = if let Some(rest) = status_str.strip_prefix("status:") {
                rest
            } else if let Some(rest) = status_str.strip_prefix("stat:") {
                rest
            } else {
                unreachable!()
            };

            if status_rest.is_empty() {
                cli_project_new_help(project);
                exit(1);
            }
            Some(status_rest.to_string())
        },
        Ordering::Greater => {
            cli_project_modify_help(project, id);
            exit(1);
        },
        Ordering::Less => None,
    };

    let assign = if assigned.is_empty() {
        None
    } else {
        let assigned = assigned.iter().map(|string| {
            let sub_str = &string[1..];
            if sub_str.is_empty() {
                cli_project_modify_help(project.clone(), id);
                exit(1);
            }
            sub_str.to_string()
        }).collect::<Vec<String>>();

        Some(assigned)
    };

    let rename = if others.is_empty() {
        None
    } else {
        Some(others.join(" "))
    };

    TaigaCmd::ModifyTask(ModifyTaskArgs { project, id, status, rename, assign, due_date, team, client, block })
}

fn cli_project_modify_help(project: String, id: usize) {
    let mut help_message = HelpMessage::new("Toggle block for a task", &format!("taiga {} modify {}", project, id), "<MODIFIERS> <OPTIONS> ...");
    help_message.add_section("Modifiers");
    help_message.add_command("status:<STATUS>", "The status to move the task to");
    help_message.add_command("@<USERNAME>", "The user to assign the task to");
    help_message.add_command("due:<DATE>", "The due date to give to the task");
    help_message.add_command("+/-team", "Add or remove the team requirement");
    help_message.add_command("+/-client", "Add or remove the client requirement");
    help_message.add_command("+/-block", "Add or remove the block");
    help_message.add_command("...", "The new name for the task");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_search(project: String, args: &[String]) -> TaigaCmd {
    if args.contains(&"--help".to_string()) {
        cli_project_search_help(project);
        exit(0);
    }

    let mut options = Vec::new();

    let mut include_statuses = Vec::new();
    let mut exclude_statuses = Vec::new();

    let mut include_assigned = Vec::new();
    let mut exclude_assigned = Vec::new();

    let mut dues = Vec::new();

    let mut teams = Vec::new();
    let mut clients = Vec::new();
    let mut blocks = Vec::new();

    let mut query = Vec::new();
    let mut can_continuous = true;

    for arg in args {
        if arg.starts_with("--") {
            options.push(arg);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("status:") || arg.starts_with("stat:") {
            include_statuses.push(arg.clone());
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("-status:") || arg.starts_with("-stat:") {
            exclude_statuses.push(arg.clone());
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with('@') {
            include_assigned.push(arg.clone());
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("-@") {
            exclude_assigned.push(arg.clone());
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg.starts_with("due:") {
            dues.push(arg);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg == "+team" {
            teams.push(true);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg == "-team" {
            teams.push(false);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg == "+client" {
            clients.push(true);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg == "-client" {
            clients.push(false);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg == "+block" {
            blocks.push(true);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg == "-block" {
            blocks.push(false);
            if !query.is_empty() {
                can_continuous = false;
            }
        } else if arg.contains(':') {
            cli_project_new_help(project);
            exit(1);
        } else {
            if !can_continuous {
                cli_project_new_help(project);
                exit(1);
            }
            query.push(arg.clone());
        }
    }

    if !options.is_empty() {
        cli_project_search_help(project);
        exit(1);
    }

    if !include_statuses.is_empty() && !exclude_statuses.is_empty() {
        cli_project_search_help(project);
        exit(1);
    }

    let include_statuses = include_statuses.iter().map(|status_str| {
        let status_rest = if let Some(rest) = status_str.strip_prefix("status:") {
            rest
        } else if let Some(rest) = status_str.strip_prefix("stat:") {
            rest
        } else {
            unreachable!()
        };
        if status_rest.is_empty() {
            cli_project_search_help(project.clone());
            exit(1);
        }
        status_rest.to_string()
    }).collect::<Vec<String>>();

    let exclude_statuses = exclude_statuses.iter().map(|status_str| {
        let status_rest = if let Some(rest) = status_str.strip_prefix("-status:") {
            rest
        } else if let Some(rest) = status_str.strip_prefix("-stat:") {
            rest
        } else {
            unreachable!()
        };
        if status_rest.is_empty() {
            cli_project_search_help(project.clone());
            exit(1);
        }
        status_rest.to_string()
    }).collect::<Vec<String>>();

    let include_assigned = include_assigned.iter().map(|assign_str| {
        let assign_rest = if let Some(rest) = assign_str.strip_prefix('@') {
            rest
        } else {
            unreachable!()
        };
        if assign_rest.is_empty() {
            cli_project_search_help(project.clone());
            exit(1);
        }
        assign_rest.to_string()
    }).collect::<Vec<String>>();

    let exclude_assigned = exclude_assigned.iter().map(|assign_str| {
        let assign_rest = if let Some(rest) = assign_str.strip_prefix("-@") {
            rest
        } else {
            unreachable!()
        };
        if assign_rest.is_empty() {
            cli_project_search_help(project.clone());
            exit(1);
        }
        assign_rest.to_string()
    }).collect::<Vec<String>>();

    let due_date = match dues.len().cmp(&1) {
        Ordering::Equal => {
            let due_str = *dues.first().expect("Could not get due");
            let date_rest = &due_str["due:".len()..];
            if date_rest.is_empty() {
                Some(String::new())
            } else {
                let due_date = TaigaTime::new(date_rest.to_string());
                Some(due_date.format())
            }
        },
        Ordering::Greater => {
            cli_project_search_help(project);
            exit(1);
        },
        Ordering::Less => None,
    };


    let team = match teams.len().cmp(&1) {
        Ordering::Equal => Some(*teams.first().expect("Could not get team")),
        Ordering::Greater => {
            cli_project_search_help(project);
            exit(1);
        },
        Ordering::Less => None,
    };

    let client = match clients.len().cmp(&1) {
        Ordering::Equal => Some(*clients.first().expect("Could not get client")),
        Ordering::Greater => {
            cli_project_search_help(project);
            exit(1);
        },
        Ordering::Less => None,
    };

    let block = match blocks.len().cmp(&1) {
        Ordering::Equal => Some(*blocks.first().expect("Could not get block")),
        Ordering::Greater => {
            cli_project_search_help(project);
            exit(1);
        },
        Ordering::Less => None,
    };

    TaigaCmd::SearchTask(SearchTaskArgs { project, include_statuses, exclude_statuses, include_assigned, exclude_assigned, due_date, team, client, block, query })
}

fn cli_project_search_help(project: String) {
    let mut help_message = HelpMessage::new("Search for tasks that fit requirements", &format!("taiga {} search", project), "<MODIFIERS> <OPTIONS> ...");
    help_message.add_section("Modifiers");
    help_message.add_command("status:<STATUS>", "A status the task is in");
    help_message.add_command("-status:<STATUS>", "A status the task is not in");
    help_message.add_command("@<USERNAME>", "A username that the task is assigned to");
    help_message.add_command("-@<USERNAME>", "A username that the task is not assigned to");
    help_message.add_command("due:<DATE>", "The due date to give to the task - empty for no dues");
    help_message.add_command("+/-team", "Filter team requirement");
    help_message.add_command("+/-client", "Filter client requirement");
    help_message.add_command("+/-block", "Filter blocked tasks");
    help_message.add_command("...", "A query for the tasks");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_project_users(project: String, _args: &[String]) -> TaigaCmd {
    // TODO: implement help
    TaigaCmd::ProjectUsers(ProjectUserArgs { project })
}

fn cli_project_burndown(project: String, _args: &[String]) -> TaigaCmd {
    // TODO: implement help
    TaigaCmd::ProjectBurndown(ProjectBurndownArgs { project })
}

fn cli_help_project(project: String) {
    let mut help_message = HelpMessage::new(&format!("Run command on {}", project), &format!("taiga {}", project), "[COMMAND] <OPTIONS>");
    help_message.add_section("Commands");
    help_message.add_command("new", "Create a new task");
    help_message.add_command("move <CARD-ID>", "Move a task to a status");
    help_message.add_command("done <CARD-ID>", "Declare a task as done");
    help_message.add_command("rename <CARD-ID>", "Rename a task");
    help_message.add_command("assign <CARD-ID>", "Assign a task");
    help_message.add_command("due <CARD-ID>", "Set due date for a task");
    help_message.add_command("team <CARD-ID>", "Toggle team requirement for a task");
    help_message.add_command("client <CARD-ID>", "Toggle client requirement for a task");
    help_message.add_command("block <CARD-ID>", "Toggle block for a task");
    help_message.add_command("modify <CARD-ID>", "Modify a task");
    help_message.add_command("delete <CARD-ID>", "Delete a task");
    help_message.add_command("search", "Search for tasks that fit requirements");
    help_message.add_command("users", "List users for the project");
    help_message.add_command("burndown", "List statistics for the project");
    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.display();
}

fn cli_help(allowed_projects: Vec<String>) {
    let mut help_message = HelpMessage::new("Cli tool for taiga.io", "taiga", "[PROJECT] [COMMAND] <OPTIONS>");

    help_message.add_section("Commands");
    help_message.add_command("login", "Login to a taiga instance");
    help_message.add_command("projects", "Refresh and print the project list");

    help_message.add_section("Projects");
    for project in &allowed_projects {
        help_message.add_command(project, &format!("Run command on {}", project));
    }

    help_message.add_section("Options");
    help_message.add_command("--help", "Print the help message and exit");
    help_message.add_command("--version", "Print the version and exit");

    help_message.display();
}

fn cli_version() {
    println!("taiga 1.0.0");
}

struct HelpMessage {
    description: String,
    command: String,
    usage: String,
    sections: Vec<HelpSection>,
}

struct HelpSection {
    title: String,
    commands: Vec<HelpCommand>,
}

struct HelpCommand {
    name: String,
    description: String,
}

impl HelpMessage {
    fn new(description: &str, command: &str, usage: &str) -> Self {
        Self {
            description: description.to_string(),
            command: command.to_string(),
            usage: usage.to_string(),
            sections: Vec::new(),
        }
    }

    fn add_section(&mut self, title: &str) {
        self.sections.push(HelpSection {
            title: title.to_string(),
            commands: Vec::new(),
        });
    }

    fn add_command(&mut self, name: &str, description: &str) {
        let section = self.sections.last_mut().expect("Error in help builder, added command without any section");

        section.commands.push(HelpCommand {
            name: name.to_string(),
            description: description.to_string(),
        });
    }

    fn display(&self) {
        println!("{}", self.description);
        println!();
        println!(
            "{} {} {}",
            "Usage:".bold().underline(),
            self.command.bold(),
            self.usage
        );

        let mut table = Table::new();
        let format = FormatBuilder::new().padding(0, 0).build();
        table.set_format(format);

        for section in &self.sections {
            table.add_row(Row::new(vec![]));
            table.add_row(Row::new(vec![Cell::new(&format!(
                "{}",
                format!("{}:", section.title).bold().underline()
            ))]));
            for command in &section.commands {
                table.add_row(Row::new(vec![
                    Cell::new(&format!("  {}  ", command.name.bold())),
                    Cell::new(&command.description),
                ]));
            }
        }
        table.printstd();
    }
}
