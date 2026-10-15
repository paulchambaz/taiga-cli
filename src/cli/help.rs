//! Help messages for every command.

use colored::Colorize;
use prettytable::{format::FormatBuilder, Cell, Row, Table};

const HELP: (&str, &str) = ("--help", "Print the help message and exit");

/// A help page: a description, a usage line and titled lists of entries.
pub struct HelpMessage {
    description: String,
    command: String,
    usage: String,
    sections: Vec<(String, Vec<(String, String)>)>,
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

    fn section<N: AsRef<str>, D: AsRef<str>>(
        mut self,
        title: &str,
        rows: impl IntoIterator<Item = (N, D)>,
    ) -> Self {
        let rows = rows
            .into_iter()
            .map(|(name, description)| {
                (name.as_ref().to_string(), description.as_ref().to_string())
            })
            .collect();
        self.sections.push((title.to_string(), rows));
        self
    }

    /// Prints the page to the terminal.
    pub fn display(&self) {
        println!("{}", self.description);
        println!();
        println!(
            "{} {} {}",
            "Usage:".bold().underline(),
            self.command.bold(),
            self.usage
        );

        let mut table = Table::new();
        table.set_format(FormatBuilder::new().padding(0, 0).build());

        for (title, rows) in &self.sections {
            table.add_row(Row::new(vec![]));
            table.add_row(Row::new(vec![Cell::new(
                &format!("{}:", title).bold().underline().to_string(),
            )]));
            for (name, description) in rows {
                table.add_row(Row::new(vec![
                    Cell::new(&format!("  {}  ", name.bold())),
                    Cell::new(description),
                ]));
            }
        }
        table.printstd();
    }
}

/// Builds the help of one command from its canonical name; an unknown name gives `burndown`.
pub fn help_message(project: &str, verb: &str, id: Option<usize>) -> HelpMessage {
    let command = match id {
        Some(id) => format!("taiga {project} {verb} {id}"),
        None => format!("taiga {project} {verb}"),
    };
    let args = "<ARGS> <OPTIONS>";
    let options = "<OPTIONS>";
    let modifiers = "<MODIFIERS> <OPTIONS> ...";

    match verb {
        "login" => HelpMessage::new("Login to a taiga instance", "taiga login", options).section(
            "Options",
            [("--address <ADDRESS>", "Address to login to"), HELP],
        ),
        "projects" => HelpMessage::new("Refresh and print the project list", "taiga projects", "")
            .section("Options", [HELP]),
        "new" => HelpMessage::new("Create a new task", &command, modifiers)
            .section(
                "Modifiers",
                [
                    (
                        "status:<STATUS>",
                        "The status to set the task to [default: new]",
                    ),
                    ("@<USERNAME>", "The user to assign the task to"),
                    ("due:<DATE>", "The due date to give to the task"),
                    ("sprint:<SPRINT>", "The sprint to put the task in"),
                    ("+/-team", "Set or unset the team requirement"),
                    ("+/-client", "Set or unset the client requirement"),
                    ("+/-block", "Set or unset the block"),
                    ("...", "The name for the task"),
                ],
            )
            .section("Options", [HELP]),
        "move" => HelpMessage::new("Move a task to a status", &command, args)
            .section(
                "Arguments",
                [("<STATUS>", "The status to move the task to")],
            )
            .section("Options", [HELP]),
        "done" => {
            HelpMessage::new("Declare a task as done", &command, options).section("Options", [HELP])
        }
        "delete" => HelpMessage::new("Delete a task", &command, options).section("Options", [HELP]),
        "rename" => HelpMessage::new("Rename a task", &command, args)
            .section("Arguments", [("<NAME>", "The new name for the task")])
            .section("Options", [HELP]),
        "assign" => HelpMessage::new("Assign a task", &command, args)
            .section(
                "Arguments",
                [("<USERNAME>", "The username to assign the task to")],
            )
            .section(
                "Options",
                [("--remove", "Remove the user instead of adding it"), HELP],
            ),
        "due" => HelpMessage::new("Set due date for a task", &command, args)
            .section(
                "Arguments",
                [("<DATE>", "The due date to give to the task")],
            )
            .section(
                "Options",
                [("--remove", "Remove the due date of the task"), HELP],
            ),
        "sprint" => HelpMessage::new("Set the sprint for a task", &command, args)
            .section("Arguments", [("<SPRINT>", "The sprint to put the task in")])
            .section(
                "Options",
                [("--remove", "Take the task out of its sprint"), HELP],
            ),
        "subtask" => HelpMessage::new(
            "Manage the tasks nested under a user story",
            &command,
            "[new|list|describe|delete] <ARGS>",
        )
        .section(
            "Commands",
            [
                (
                    "new <NAME> <@USERNAME> <desc:TEXT>",
                    "Create a task under the story",
                ),
                ("list", "List the tasks under the story"),
                (
                    "describe <INDEX> <TEXT>",
                    "Set the description for a task by its list position",
                ),
                ("delete <INDEX>", "Delete a task by its list position"),
            ],
        )
        .section("Options", [HELP]),
        "team" => HelpMessage::new("Toggle team requirement for a task", &command, options)
            .section(
                "Options",
                [("--remove", "Remove the team requirement"), HELP],
            ),
        "client" => HelpMessage::new("Toggle client requirement for a task", &command, options)
            .section(
                "Options",
                [("--remove", "Remove the client requirement"), HELP],
            ),
        "block" => HelpMessage::new("Toggle block for a task", &command, options)
            .section("Options", [("--remove", "Remove the block"), HELP]),
        "modify" => HelpMessage::new("Modify a task", &command, modifiers)
            .section(
                "Modifiers",
                [
                    ("status:<STATUS>", "The status to move the task to"),
                    ("@<USERNAME>", "The user to assign the task to"),
                    ("-@<USERNAME>", "The user to take off the task"),
                    ("due:<DATE>", "The due date to give to the task"),
                    ("-due", "Remove the due date of the task"),
                    ("sprint:<SPRINT>", "The sprint to put the task in"),
                    ("-sprint", "Take the task out of its sprint"),
                    ("+/-team", "Add or remove the team requirement"),
                    ("+/-client", "Add or remove the client requirement"),
                    ("+/-block", "Add or remove the block"),
                    ("...", "The new name for the task"),
                ],
            )
            .section("Options", [HELP]),
        "search" => HelpMessage::new(
            "Search for tasks that fit requirements",
            &command,
            modifiers,
        )
        .section(
            "Modifiers",
            [
                ("status:<STATUS>", "A status the task is in"),
                ("-status:<STATUS>", "A status the task is not in"),
                ("@<USERNAME>", "A username that the task is assigned to"),
                (
                    "-@<USERNAME>",
                    "A username that the task is not assigned to",
                ),
                (
                    "due:<DATE>",
                    "The due date to give to the task - empty for no dues",
                ),
                ("sprint:<SPRINT>", "A sprint the task is in"),
                ("-sprint:<SPRINT>", "A sprint the task is not in"),
                ("+/-team", "Filter team requirement"),
                ("+/-client", "Filter client requirement"),
                ("+/-block", "Filter blocked tasks"),
                ("...", "A query for the tasks"),
            ],
        )
        .section("Options", [HELP]),
        _ => HelpMessage::new("Show the burndown of a sprint", &command, args)
            .section(
                "Arguments",
                [("<SPRINT>", "The sprint to show [default: the current one]")],
            )
            .section("Options", [HELP]),
    }
}

/// Builds the help listing every command of a project.
pub fn project_help(project: &str) -> HelpMessage {
    HelpMessage::new(
        &format!("Run command on {project}"),
        &format!("taiga {project}"),
        "[COMMAND] <OPTIONS>",
    )
    .section(
        "Commands",
        [
            ("new", "Create a new task"),
            ("move <CARD-ID>", "Move a task to a status"),
            ("done <CARD-ID>", "Declare a task as done"),
            ("rename <CARD-ID>", "Rename a task"),
            ("assign <CARD-ID>", "Assign a task"),
            ("due <CARD-ID>", "Set due date for a task"),
            ("sprint <CARD-ID>", "Set the sprint for a task"),
            (
                "subtask <CARD-ID>",
                "Manage the tasks nested under a user story",
            ),
            ("team <CARD-ID>", "Toggle team requirement for a task"),
            ("client <CARD-ID>", "Toggle client requirement for a task"),
            ("block <CARD-ID>", "Toggle block for a task"),
            ("modify <CARD-ID>", "Modify a task"),
            ("delete <CARD-ID>", "Delete a task"),
            ("search", "Search for tasks that fit requirements"),
            ("users", "List users for the project"),
            ("burndown", "Show the burndown of a sprint"),
        ],
    )
    .section("Options", [HELP])
}

/// Builds the top-level help, listing the known projects.
pub fn top_help(projects: &[&str]) -> HelpMessage {
    HelpMessage::new(
        "Cli tool for taiga.io",
        "taiga",
        "[PROJECT] [COMMAND] <OPTIONS>",
    )
    .section(
        "Commands",
        [
            ("login", "Login to a taiga instance"),
            ("projects", "Refresh and print the project list"),
        ],
    )
    .section(
        "Projects",
        projects
            .iter()
            .map(|project| (*project, format!("Run command on {project}"))),
    )
    .section(
        "Options",
        [HELP, ("--version", "Print the version and exit")],
    )
}
