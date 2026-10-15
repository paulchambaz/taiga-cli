//! The hidden `complete` command: cached names for shell completion.

use std::borrow::Cow;

use crate::cli::{Candidates, CompleteArgs};
use crate::taiga::{Taiga, TaigaProject};

/// Prints cached names, one per line, without any request or error.
///
/// Task lines look like `N:name` where N is the position in the last listing.
pub fn complete(taiga: Option<&Taiga>, args: &CompleteArgs) {
    if let Some(taiga) = taiga {
        for name in names(&taiga.projects, args) {
            println!("{name}");
        }
    }
}

fn names<'a>(projects: &'a [TaigaProject], args: &CompleteArgs) -> Vec<Cow<'a, str>> {
    if args.candidates == Candidates::Projects {
        return projects
            .iter()
            .map(|p| Cow::Borrowed(p.name.as_str()))
            .collect();
    }

    let Some(project) = projects
        .iter()
        .find(|project| Some(&project.name) == args.project.as_ref())
    else {
        return Vec::new();
    };

    match args.candidates {
        Candidates::Statuses => project
            .statuses
            .iter()
            .map(|s| Cow::Borrowed(s.slug.as_str()))
            .collect(),
        Candidates::Members => project
            .members
            .iter()
            .map(|m| Cow::Borrowed(m.username.as_str()))
            .collect(),
        Candidates::Sprints => project
            .sprints
            .iter()
            .map(|s| Cow::Borrowed(s.slug.as_str()))
            .collect(),
        Candidates::Tasks => project
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| Cow::Owned(format!("{}:{}", i + 1, task.name)))
            .collect(),
        Candidates::Projects => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taiga::{TaigaStatus, TaigaUser};

    fn projects() -> Vec<TaigaProject> {
        vec![
            TaigaProject {
                name: "demo".to_string(),
                members: vec![TaigaUser {
                    id: 1,
                    username: "paul".to_string(),
                }],
                statuses: vec![TaigaStatus {
                    id: 10,
                    slug: "new".to_string(),
                    is_closed: false,
                }],
                ..Default::default()
            },
            TaigaProject {
                name: "other".to_string(),
                ..Default::default()
            },
        ]
    }

    fn args(candidates: Candidates, project: Option<&str>) -> CompleteArgs {
        CompleteArgs {
            candidates,
            project: project.map(str::to_string),
        }
    }

    #[test]
    fn lists_projects_and_the_names_of_one_project() {
        let projects = projects();
        assert_eq!(
            names(&projects, &args(Candidates::Projects, None)),
            ["demo", "other"]
        );
        assert_eq!(
            names(&projects, &args(Candidates::Members, Some("demo"))),
            ["paul"]
        );
        assert_eq!(
            names(&projects, &args(Candidates::Statuses, Some("demo"))),
            ["new"]
        );
    }

    #[test]
    fn an_unknown_project_has_no_candidates() {
        let projects = projects();
        assert!(names(&projects, &args(Candidates::Tasks, Some("nope"))).is_empty());
        assert!(names(&projects, &args(Candidates::Sprints, Some("other"))).is_empty());
    }
}
