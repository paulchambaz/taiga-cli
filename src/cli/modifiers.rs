//! The modifier syntax shared by `new`, `modify` and `search`.

use super::{ModifyTaskArgs, NewTaskArgs, SearchTaskArgs, TaigaCmd};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    New,
    Modify,
    Search,
}

#[derive(Default)]
struct Modifiers {
    statuses: Vec<String>,
    not_statuses: Vec<String>,
    assigned: Vec<String>,
    not_assigned: Vec<String>,
    sprints: Vec<String>,
    not_sprints: Vec<String>,
    dues: Vec<String>,
    teams: Vec<bool>,
    clients: Vec<bool>,
    blocks: Vec<bool>,
    no_sprint: bool,
    no_due: bool,
    words: Vec<String>,
}

/// Splits arguments into modifiers and the words of the name or query.
///
/// Words must stay contiguous, but modifiers may come before or after them.
fn parse_modifiers(args: &[String], kind: Kind) -> Option<Modifiers> {
    let mut modifiers = Modifiers::default();
    let mut closed = false;

    for arg in args {
        if arg.starts_with("--") {
            return None;
        }
        if add_modifier(&mut modifiers, arg, kind)? {
            closed |= !modifiers.words.is_empty();
        } else {
            if closed {
                return None;
            }
            modifiers.words.push(arg.clone());
        }
    }

    let values = [
        &modifiers.statuses,
        &modifiers.not_statuses,
        &modifiers.assigned,
        &modifiers.not_assigned,
        &modifiers.sprints,
        &modifiers.not_sprints,
    ];
    if values.iter().any(|list| list.iter().any(String::is_empty)) {
        return None;
    }
    Some(modifiers)
}

/// Records one modifier; `Some(false)` means the argument is a plain word.
fn add_modifier(modifiers: &mut Modifiers, arg: &str, kind: Kind) -> Option<bool> {
    let search = kind == Kind::Search;
    let removing = kind != Kind::New;
    let value = |prefixes: &[&str]| {
        prefixes
            .iter()
            .find_map(|prefix| arg.strip_prefix(prefix))
            .map(str::to_string)
    };

    if let Some(status) = value(&["status:", "stat:"]) {
        modifiers.statuses.push(status);
    } else if let Some(status) = value(&["-status:", "-stat:"]).filter(|_| search) {
        modifiers.not_statuses.push(status);
    } else if let Some(username) = value(&["@"]) {
        modifiers.assigned.push(username);
    } else if let Some(username) = value(&["-@"]).filter(|_| removing) {
        modifiers.not_assigned.push(username);
    } else if let Some(due) = value(&["due:"]) {
        modifiers.dues.push(due);
    } else if let Some(sprint) = value(&["-sprint:"]).filter(|_| search) {
        modifiers.not_sprints.push(sprint);
    } else if let Some(sprint) = value(&["sprint:"]) {
        modifiers.sprints.push(sprint);
    } else if arg == "-sprint" && kind == Kind::Modify {
        modifiers.no_sprint = true;
    } else if arg == "-due" && kind == Kind::Modify {
        modifiers.no_due = true;
    } else if matches!(
        arg,
        "+team" | "-team" | "+client" | "-client" | "+block" | "-block"
    ) {
        let on = arg.starts_with('+');
        match &arg[1..] {
            "team" => modifiers.teams.push(on),
            "client" => modifiers.clients.push(on),
            _ => modifiers.blocks.push(on),
        }
    } else if (arg.starts_with(['+', '-']) && !search) || arg.contains(':') {
        return None;
    } else {
        return Some(false);
    }
    Some(true)
}

fn single<T>(values: Vec<T>) -> Option<Option<T>> {
    let mut values = values.into_iter();
    let first = values.next();
    values.next().is_none().then_some(first)
}

/// Parses a date the way the user typed it, telling them when it is not one.
pub fn parse_date(input: &str) -> Option<String> {
    match temporis::parse_date(input) {
        Ok(date) => Some(date.format("%Y-%m-%d").to_string()),
        Err(_) => {
            eprintln!("Error, invalid date '{input}'");
            None
        }
    }
}

/// An empty due date is a valid search filter for tasks without one.
fn due_date(dues: Vec<String>, allow_empty: bool) -> Option<Option<String>> {
    match single(dues)? {
        None => Some(None),
        Some(due) if due.is_empty() => allow_empty.then_some(Some(due)),
        Some(due) => Some(Some(parse_date(&due)?)),
    }
}

/// Parses the arguments of `new`.
pub fn parse_new(project: &str, args: &[String]) -> Option<TaigaCmd> {
    let modifiers = parse_modifiers(args, Kind::New)?;
    if modifiers.words.is_empty() {
        return None;
    }

    Some(TaigaCmd::NewTask(NewTaskArgs {
        project: project.to_string(),
        status: single(modifiers.statuses)?,
        name: modifiers.words.join(" "),
        assign: modifiers.assigned,
        due_date: due_date(modifiers.dues, false)?,
        team: single(modifiers.teams)?.unwrap_or(false),
        client: single(modifiers.clients)?.unwrap_or(false),
        block: single(modifiers.blocks)?.unwrap_or(false),
        sprint: single(modifiers.sprints)?,
    }))
}

/// Parses the arguments of `modify`.
pub fn parse_modify(project: &str, id: usize, args: &[String]) -> Option<TaigaCmd> {
    let modifiers = parse_modifiers(args, Kind::Modify)?;
    let sprint = match (single(modifiers.sprints)?, modifiers.no_sprint) {
        (Some(_), true) => return None,
        (Some(sprint), false) => Some(Some(sprint)),
        (None, true) => Some(None),
        (None, false) => None,
    };

    let due_date = match (due_date(modifiers.dues, false)?, modifiers.no_due) {
        (Some(_), true) => return None,
        (Some(due), false) => Some(Some(due)),
        (None, true) => Some(None),
        (None, false) => None,
    };

    Some(TaigaCmd::ModifyTask(ModifyTaskArgs {
        project: project.to_string(),
        id,
        status: single(modifiers.statuses)?,
        rename: (!modifiers.words.is_empty()).then(|| modifiers.words.join(" ")),
        assign: modifiers.assigned,
        unassign: modifiers.not_assigned,
        due_date,
        team: single(modifiers.teams)?,
        client: single(modifiers.clients)?,
        block: single(modifiers.blocks)?,
        sprint,
    }))
}

/// Parses the arguments of `search`.
pub fn parse_search(project: &str, args: &[String]) -> Option<TaigaCmd> {
    let modifiers = parse_modifiers(args, Kind::Search)?;
    let contradicts =
        |include: &[String], exclude: &[String]| !include.is_empty() && !exclude.is_empty();
    if contradicts(&modifiers.statuses, &modifiers.not_statuses)
        || contradicts(&modifiers.sprints, &modifiers.not_sprints)
    {
        return None;
    }

    Some(TaigaCmd::SearchTask(SearchTaskArgs {
        project: project.to_string(),
        include_statuses: modifiers.statuses,
        exclude_statuses: modifiers.not_statuses,
        include_assigned: modifiers.assigned,
        exclude_assigned: modifiers.not_assigned,
        include_sprints: modifiers.sprints,
        exclude_sprints: modifiers.not_sprints,
        due_date: due_date(modifiers.dues, true)?,
        team: single(modifiers.teams)?,
        client: single(modifiers.clients)?,
        block: single(modifiers.blocks)?,
        query: modifiers.words,
    }))
}
