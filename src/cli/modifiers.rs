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

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    fn new(words: &[&str]) -> Option<NewTaskArgs> {
        match parse_new("demo", &strings(words))? {
            TaigaCmd::NewTask(args) => Some(args),
            other => panic!("unexpected {other:?}"),
        }
    }

    fn modify(words: &[&str]) -> Option<ModifyTaskArgs> {
        match parse_modify("demo", 3, &strings(words))? {
            TaigaCmd::ModifyTask(args) => Some(args),
            other => panic!("unexpected {other:?}"),
        }
    }

    fn search(words: &[&str]) -> Option<SearchTaskArgs> {
        match parse_search("demo", &strings(words))? {
            TaigaCmd::SearchTask(args) => Some(args),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn new_joins_the_words_into_the_name() {
        let args = new(&["fix", "the", "bug"]).unwrap();
        assert_eq!(args.name, "fix the bug");
        assert_eq!(args.status, None);
        assert!(args.assign.is_empty());
        assert_eq!(args.due_date, None);
        assert_eq!(args.sprint, None);
        assert!(!args.team && !args.client && !args.block);
    }

    #[test]
    fn new_takes_modifiers_before_and_after_the_name() {
        let args = new(&[
            "@paul",
            "fix",
            "bug",
            "due:2030-01-02",
            "+team",
            "stat:done",
        ])
        .unwrap();
        assert_eq!(args.name, "fix bug");
        assert_eq!(args.assign, ["paul"]);
        assert_eq!(args.due_date.as_deref(), Some("2030-01-02"));
        assert_eq!(args.status.as_deref(), Some("done"));
        assert!(args.team && !args.client);
    }

    #[test]
    fn new_rejects_a_name_split_by_a_modifier() {
        assert!(new(&["fix", "@paul", "bug"]).is_none());
        assert!(new(&["fix", "+team", "bug"]).is_none());
    }

    #[test]
    fn new_needs_a_name() {
        assert!(new(&[]).is_none());
        assert!(new(&["@paul", "+team"]).is_none());
    }

    #[test]
    fn new_allows_many_assignees_but_one_of_everything_else() {
        assert_eq!(
            new(&["x", "@paul", "@arjun"]).unwrap().assign,
            ["paul", "arjun"]
        );
        for words in [
            ["x", "status:a", "status:b"],
            ["x", "sprint:a", "sprint:b"],
            ["x", "due:2030-01-01", "due:2030-01-02"],
            ["x", "+team", "-team"],
            ["x", "+client", "+client"],
            ["x", "+block", "-block"],
        ] {
            assert!(new(&words).is_none(), "{words:?}");
        }
    }

    #[test]
    fn new_rejects_empty_unknown_and_misplaced_modifiers() {
        for modifier in [
            "status:",
            "@",
            "sprint:",
            "due:",
            "due:garbage",
            "+foo",
            "-foo",
            "-sprint",
            "-due",
            "-@paul",
            "-status:a",
            "-sprint:a",
            "a:b",
            "--x",
        ] {
            assert!(new(&["x", modifier]).is_none(), "{modifier}");
        }
    }

    #[test]
    fn modify_with_nothing_to_change_changes_nothing() {
        let args = modify(&[]).unwrap();
        assert_eq!(args.id, 3);
        assert_eq!(args.status, None);
        assert_eq!(args.rename, None);
        assert!(args.assign.is_empty() && args.unassign.is_empty());
        assert_eq!(args.due_date, None);
        assert_eq!((args.team, args.client, args.block), (None, None, None));
        assert_eq!(args.sprint, None);
    }

    #[test]
    fn modify_sets_flags_and_renames() {
        let args = modify(&["new", "name", "+team", "-block"]).unwrap();
        assert_eq!(args.rename.as_deref(), Some("new name"));
        assert_eq!(
            (args.team, args.client, args.block),
            (Some(true), None, Some(false))
        );
    }

    #[test]
    fn modify_clears_the_sprint_or_the_due_date() {
        assert_eq!(modify(&["-sprint"]).unwrap().sprint, Some(None));
        assert_eq!(
            modify(&["sprint:s1"]).unwrap().sprint,
            Some(Some("s1".to_string()))
        );
        assert!(modify(&["-sprint", "sprint:s1"]).is_none());

        assert_eq!(modify(&["-due"]).unwrap().due_date, Some(None));
        assert_eq!(
            modify(&["due:2030-01-02"]).unwrap().due_date,
            Some(Some("2030-01-02".to_string()))
        );
        assert!(modify(&["-due", "due:2030-01-02"]).is_none());
    }

    #[test]
    fn modify_adds_and_removes_assignees() {
        let args = modify(&["@paul", "-@arjun"]).unwrap();
        assert_eq!(args.assign, ["paul"]);
        assert_eq!(args.unassign, ["arjun"]);
        assert!(modify(&["-@"]).is_none());
    }

    #[test]
    fn search_collects_includes_excludes_and_the_query() {
        let args = search(&["status:new", "-@paul", "sprint:s1", "@me", "fix", "bug"]).unwrap();
        assert_eq!(args.include_statuses, ["new"]);
        assert_eq!(args.exclude_assigned, ["paul"]);
        assert_eq!(args.include_assigned, ["me"]);
        assert_eq!(args.include_sprints, ["s1"]);
        assert_eq!(args.query, ["fix", "bug"]);
    }

    #[test]
    fn search_cannot_include_and_exclude_the_same_kind() {
        assert!(search(&["status:a", "-status:b"]).is_none());
        assert!(search(&["stat:a", "-stat:b"]).is_none());
        assert!(search(&["sprint:a", "-sprint:b"]).is_none());
        assert!(search(&["@a", "-@b"]).is_some());
    }

    #[test]
    fn search_keeps_unknown_signs_as_query_words() {
        assert_eq!(search(&["-foo", "+bar"]).unwrap().query, ["-foo", "+bar"]);
        assert_eq!(search(&["-sprint"]).unwrap().query, ["-sprint"]);
        assert_eq!(search(&["-5"]).unwrap().query, ["-5"]);
        assert!(search(&["a:b"]).is_none());
        assert!(search(&["--x"]).is_none());
    }

    #[test]
    fn search_treats_an_empty_due_date_as_no_due_date() {
        assert_eq!(search(&["due:"]).unwrap().due_date, Some(String::new()));
        assert_eq!(
            search(&["due:2030-01-02"]).unwrap().due_date.as_deref(),
            Some("2030-01-02")
        );
        assert_eq!(search(&[]).unwrap().due_date, None);
        assert!(search(&["due:garbage"]).is_none());
        assert!(search(&["due:2030-01-01", "due:2030-01-02"]).is_none());
    }

    #[test]
    fn search_filters_on_flags() {
        let args = search(&["+team", "-client"]).unwrap();
        assert_eq!(
            (args.team, args.client, args.block),
            (Some(true), Some(false), None)
        );
        assert!(search(&["+team", "-team"]).is_none());
    }

    #[test]
    fn search_words_must_be_contiguous_but_may_sit_beside_modifiers() {
        assert_eq!(
            search(&["status:a", "fix", "bug", "@p"]).unwrap().query,
            ["fix", "bug"]
        );
        assert!(search(&["fix", "status:a", "bug"]).is_none());
    }

    #[test]
    fn at_most_one_value_is_a_single_value() {
        assert_eq!(single(Vec::<i32>::new()), Some(None));
        assert_eq!(single(vec![1]), Some(Some(1)));
        assert_eq!(single(vec![1, 2]), None);
    }

    #[test]
    fn dates_are_normalised_or_refused() {
        assert_eq!(parse_date("2030-01-02").as_deref(), Some("2030-01-02"));
        assert_eq!(parse_date("not a date"), None);
    }
}
