//! The `search` command and the formatting of its listing.

use std::cmp::Ordering;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::{Cell, Row, Table};

use crate::cli::SearchTaskArgs;
use crate::taiga::{Error, Taiga, TaigaProject, TaigaTask};

/// A search with every name resolved to an id.
struct Filter<'a> {
    team: Option<bool>,
    client: Option<bool>,
    block: Option<bool>,
    no_due: bool,
    due_limit: Option<NaiveDate>,
    include_members: Vec<i32>,
    exclude_members: Vec<i32>,
    include_statuses: Vec<i32>,
    exclude_statuses: Vec<i32>,
    include_sprints: Vec<i32>,
    exclude_sprints: Vec<i32>,
    query: &'a [String],
}

impl<'a> Filter<'a> {
    fn new(args: &'a SearchTaskArgs, project: &TaigaProject, me: i32) -> Result<Self> {
        let members = |names: &[String]| {
            names
                .iter()
                .map(|name| project.member_id(name, me))
                .collect::<Result<Vec<_>, _>>()
        };
        let statuses = |slugs: &[String]| {
            slugs
                .iter()
                .map(|slug| project.status_id(slug))
                .collect::<Result<Vec<_>, _>>()
        };
        let sprints = |names: &[String]| {
            names
                .iter()
                .map(|name| project.sprint_id(name))
                .collect::<Result<Vec<_>, _>>()
        };

        Ok(Self {
            team: args.team,
            client: args.client,
            block: args.block,
            no_due: args.due_date.as_deref() == Some(""),
            due_limit: args
                .due_date
                .as_deref()
                .filter(|due_date| !due_date.is_empty())
                .map(|due_date| NaiveDate::parse_from_str(due_date, "%Y-%m-%d"))
                .transpose()?,
            include_members: members(&args.include_assigned)?,
            exclude_members: members(&args.exclude_assigned)?,
            include_statuses: statuses(&args.include_statuses)?,
            exclude_statuses: statuses(&args.exclude_statuses)?,
            include_sprints: sprints(&args.include_sprints)?,
            exclude_sprints: sprints(&args.exclude_sprints)?,
            query: &args.query,
        })
    }

    fn matches(&self, task: &TaigaTask) -> bool {
        let assigned_to = |ids: &[i32]| task.assigned.iter().any(|id| ids.contains(id));
        let in_sprint = |ids: &[i32]| task.sprint_id.is_some_and(|id| ids.contains(&id));

        self.team.is_none_or(|team| task.team == team)
            && self.client.is_none_or(|client| task.client == client)
            && self.block.is_none_or(|block| task.blocked == block)
            && !(self.no_due && task.due.is_some())
            && self
                .due_limit
                .is_none_or(|limit| task.due.is_some_and(|due| due.date_naive() <= limit))
            && (self.include_members.is_empty() || assigned_to(&self.include_members))
            && !assigned_to(&self.exclude_members)
            && (self.include_statuses.is_empty() || self.include_statuses.contains(&task.status_id))
            && !self.exclude_statuses.contains(&task.status_id)
            && (self.include_sprints.is_empty() || in_sprint(&self.include_sprints))
            && !in_sprint(&self.exclude_sprints)
            && fzf_match(&task.name, self.query)
    }
}

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
    sort_tasks(&mut tasks);

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

    let filter = Filter::new(&args, &project, me)?;

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
        .filter(|task| filter.matches(task))
        .collect();

    let now = Utc::now();
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

        let due = task
            .due
            .as_ref()
            .map(|due| format_due(due, now))
            .unwrap_or_default();

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

fn sort_tasks(tasks: &mut [TaigaTask]) {
    tasks.sort_by(|a, b| {
        b.status_id
            .cmp(&a.status_id)
            .then_with(|| match (&a.due, &b.due) {
                (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
                (Some(_), _) => Ordering::Less,
                (_, Some(_)) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            })
    });
}

fn format_due(due: &DateTime<Utc>, now: DateTime<Utc>) -> String {
    let duration = due.signed_duration_since(now);
    let sign = if duration.num_seconds() < 0 { "-" } else { "" };
    let duration = duration.abs();
    let (days, hours) = (duration.num_days(), duration.num_hours());

    let (count, unit) = if days >= 365 {
        (days / 365, 'y')
    } else if days >= 30 {
        (days / 30, 'm')
    } else if days >= 7 {
        (days / 7, 'w')
    } else if hours >= 24 {
        (hours / 24, 'd')
    } else {
        (hours, 'h')
    };
    format!("{sign}{count}{unit}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn task() -> TaigaTask {
        TaigaTask {
            id: 1,
            name: "fix login bug".to_string(),
            status_id: 10,
            status: "new".to_string(),
            team: false,
            client: false,
            blocked: false,
            assigned: Vec::new(),
            due: None,
            closed: false,
            version: 1,
            sprint_id: None,
        }
    }

    fn filter() -> Filter<'static> {
        Filter {
            team: None,
            client: None,
            block: None,
            no_due: false,
            due_limit: None,
            include_members: Vec::new(),
            exclude_members: Vec::new(),
            include_statuses: Vec::new(),
            exclude_statuses: Vec::new(),
            include_sprints: Vec::new(),
            exclude_sprints: Vec::new(),
            query: &[],
        }
    }

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn an_empty_filter_matches_everything() {
        assert!(filter().matches(&task()));
    }

    #[test]
    fn flags_must_match_when_asked_for() {
        let flagged = TaigaTask {
            team: true,
            client: true,
            blocked: true,
            ..task()
        };
        for (set, task_matches, flagged_matches) in [(true, false, true), (false, true, false)] {
            let team = Filter {
                team: Some(set),
                ..filter()
            };
            let client = Filter {
                client: Some(set),
                ..filter()
            };
            let block = Filter {
                block: Some(set),
                ..filter()
            };
            for filter in [team, client, block] {
                assert_eq!(filter.matches(&task()), task_matches);
                assert_eq!(filter.matches(&flagged), flagged_matches);
            }
        }
    }

    #[test]
    fn due_filters_keep_tasks_due_by_a_date_or_without_one() {
        let undated = task();
        let due = |year, month, day| TaigaTask {
            due: Some(at(year, month, day)),
            ..task()
        };

        let no_due = Filter {
            no_due: true,
            ..filter()
        };
        assert!(no_due.matches(&undated));
        assert!(!no_due.matches(&due(2030, 1, 1)));

        let limit = NaiveDate::from_ymd_opt(2030, 1, 2);
        let by_date = Filter {
            due_limit: limit,
            ..filter()
        };
        assert!(by_date.matches(&due(2030, 1, 1)));
        assert!(by_date.matches(&due(2030, 1, 2)));
        assert!(!by_date.matches(&due(2030, 1, 3)));
        assert!(!by_date.matches(&undated));
    }

    #[test]
    fn members_statuses_and_sprints_can_be_required_or_excluded() {
        let assigned = TaigaTask {
            assigned: vec![1, 2],
            sprint_id: Some(5),
            ..task()
        };
        let other = TaigaTask {
            assigned: vec![3],
            status_id: 11,
            sprint_id: Some(6),
            ..task()
        };

        let only_members = Filter {
            include_members: vec![1],
            ..filter()
        };
        assert!(only_members.matches(&assigned) && !only_members.matches(&other));
        assert!(!only_members.matches(&task()));
        let not_members = Filter {
            exclude_members: vec![1],
            ..filter()
        };
        assert!(!not_members.matches(&assigned) && not_members.matches(&other));
        assert!(not_members.matches(&task()));

        let only_status = Filter {
            include_statuses: vec![11],
            ..filter()
        };
        assert!(only_status.matches(&other) && !only_status.matches(&assigned));
        let not_status = Filter {
            exclude_statuses: vec![11],
            ..filter()
        };
        assert!(!not_status.matches(&other) && not_status.matches(&assigned));

        let only_sprint = Filter {
            include_sprints: vec![5],
            ..filter()
        };
        assert!(only_sprint.matches(&assigned) && !only_sprint.matches(&other));
        assert!(!only_sprint.matches(&task()));
        let not_sprint = Filter {
            exclude_sprints: vec![5],
            ..filter()
        };
        assert!(!not_sprint.matches(&assigned) && not_sprint.matches(&other));
        assert!(not_sprint.matches(&task()));
    }

    #[test]
    fn the_query_filters_by_name() {
        let query = ["login".to_string()];
        let by_name = Filter {
            query: &query,
            ..filter()
        };
        assert!(by_name.matches(&task()));
        assert!(!by_name.matches(&TaigaTask {
            name: "write docs".to_string(),
            ..task()
        }));
    }

    #[test]
    fn query_words_must_appear_in_order_as_parts_of_words() {
        let query = |words: &[&str]| words.iter().map(|w| w.to_string()).collect::<Vec<_>>();
        assert!(fzf_match("fix login bug", &query(&[])));
        assert!(fzf_match("fix login bug", &query(&["fix", "bug"])));
        assert!(fzf_match("fix login bug", &query(&["lo", "ug"])));
        assert!(!fzf_match("fix login bug", &query(&["bug", "fix"])));
        assert!(!fzf_match("fix login bug", &query(&["docs"])));
        assert!(!fzf_match("fix login bug", &query(&["Fix"])));
    }

    #[test]
    fn tasks_sort_by_status_descending_then_due_date_with_undated_last() {
        let task = |id, status_id, due| TaigaTask {
            id,
            status_id,
            due,
            ..task()
        };
        let mut tasks = vec![
            task(1, 1, None),
            task(2, 2, Some(at(2030, 3, 1))),
            task(3, 2, None),
            task(4, 2, Some(at(2030, 1, 1))),
            task(5, 1, Some(at(2030, 2, 1))),
        ];
        sort_tasks(&mut tasks);
        let ids: Vec<i32> = tasks.iter().map(|t| t.id).collect();
        assert_eq!(ids, [4, 2, 3, 5, 1]);
    }

    #[test]
    fn due_dates_are_shown_in_the_largest_fitting_unit() {
        let now = at(2026, 10, 15);
        let shown = |offset: Duration| format_due(&(now + offset), now);

        assert_eq!(shown(Duration::minutes(59)), "0h");
        assert_eq!(shown(Duration::hours(2)), "2h");
        assert_eq!(shown(Duration::hours(25)), "1d");
        assert_eq!(shown(Duration::days(6)), "6d");
        assert_eq!(shown(Duration::days(7)), "1w");
        assert_eq!(shown(Duration::days(29)), "4w");
        assert_eq!(shown(Duration::days(30)), "1m");
        assert_eq!(shown(Duration::days(364)), "12m");
        assert_eq!(shown(Duration::days(365)), "1y");
        assert_eq!(shown(Duration::days(800)), "2y");
    }

    #[test]
    fn past_due_dates_get_a_minus_sign() {
        let now = at(2026, 10, 15);
        let shown = |offset: Duration| format_due(&(now - offset), now);

        assert_eq!(shown(Duration::hours(3)), "-3h");
        assert_eq!(shown(Duration::days(2)), "-2d");
        assert_eq!(shown(Duration::days(40)), "-1m");
        assert_eq!(shown(Duration::days(400)), "-1y");
    }

    fn args() -> SearchTaskArgs {
        SearchTaskArgs {
            project: "demo".to_string(),
            include_statuses: Vec::new(),
            exclude_statuses: Vec::new(),
            include_assigned: Vec::new(),
            exclude_assigned: Vec::new(),
            include_sprints: Vec::new(),
            exclude_sprints: Vec::new(),
            due_date: None,
            team: None,
            client: None,
            block: None,
            query: Vec::new(),
        }
    }

    fn project() -> TaigaProject {
        use crate::taiga::{TaigaSprint, TaigaStatus, TaigaUser};
        TaigaProject {
            members: vec![
                TaigaUser {
                    id: 1,
                    username: "paul".to_string(),
                },
                TaigaUser {
                    id: 2,
                    username: "arjun".to_string(),
                },
            ],
            statuses: vec![TaigaStatus {
                id: 10,
                slug: "new".to_string(),
                is_closed: false,
            }],
            sprints: vec![TaigaSprint {
                id: 5,
                name: "Sprint 1".to_string(),
                slug: "sprint-1".to_string(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn names_are_resolved_to_ids_when_the_filter_is_built() {
        let args = SearchTaskArgs {
            include_assigned: vec!["me".to_string()],
            exclude_assigned: vec!["arjun".to_string()],
            include_statuses: vec!["new".to_string()],
            exclude_sprints: vec!["sprint-1".to_string()],
            due_date: Some("2030-01-02".to_string()),
            ..args()
        };
        let filter = Filter::new(&args, &project(), 1).unwrap();
        assert_eq!(filter.include_members, [1]);
        assert_eq!(filter.exclude_members, [2]);
        assert_eq!(filter.include_statuses, [10]);
        assert_eq!(filter.exclude_sprints, [5]);
        assert_eq!(filter.due_limit, NaiveDate::from_ymd_opt(2030, 1, 2));
        assert!(!filter.no_due);
    }

    #[test]
    fn an_empty_due_date_means_no_due_date() {
        let args = SearchTaskArgs {
            due_date: Some(String::new()),
            ..args()
        };
        let filter = Filter::new(&args, &project(), 1).unwrap();
        assert!(filter.no_due && filter.due_limit.is_none());
    }

    #[test]
    fn unknown_names_are_errors() {
        for args in [
            SearchTaskArgs {
                include_assigned: vec!["nobody".to_string()],
                ..args()
            },
            SearchTaskArgs {
                exclude_statuses: vec!["nope".to_string()],
                ..args()
            },
            SearchTaskArgs {
                include_sprints: vec!["nope".to_string()],
                ..args()
            },
        ] {
            assert!(Filter::new(&args, &project(), 1).is_err());
        }
    }
}
