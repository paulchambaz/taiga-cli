//! Projects: their cached metadata and the lookups on it.

use serde::{Deserialize, Serialize};

use super::Error;
use super::Result;
use super::Taiga;
use super::TaigaSprint;
use super::TaigaStatus;
use super::TaigaTask;
use super::TaigaUser;

/// A project with its cached metadata and the listing that task numbers refer to.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TaigaProject {
    pub id: i32,
    pub name: String,
    pub members: Vec<TaigaUser>,
    pub statuses: Vec<TaigaStatus>,
    pub task_statuses: Vec<TaigaStatus>,
    pub sprints: Vec<TaigaSprint>,
    /// The tasks of the last search, in the order they were printed.
    pub tasks: Vec<TaigaTask>,
}

#[derive(Deserialize, Debug)]
struct ProjectsResponse {
    id: i32,
    name: String,
}

#[derive(Deserialize, Debug)]
struct ProjectResponse {
    id: i32,
    name: String,
    members: Vec<MemberResponse>,
    us_statuses: Vec<Status>,
    task_statuses: Vec<Status>,
}

#[derive(Deserialize, Debug)]
struct MemberResponse {
    id: i32,
    username: String,
}

#[derive(Deserialize, Debug)]
struct Status {
    id: i32,
    slug: String,
    is_closed: bool,
}

#[derive(Deserialize, Debug)]
struct MilestoneResponse {
    id: i32,
    name: String,
    slug: String,
}

fn merge_projects(known: &mut Vec<TaigaProject>, fetched: Vec<TaigaProject>) {
    known.retain(|project| fetched.iter().any(|f| f.id == project.id));
    for project in fetched {
        match known.iter_mut().find(|p| p.id == project.id) {
            Some(cached) => cached.name = project.name,
            None => known.push(project),
        }
    }
}

impl TaigaProject {
    /// Returns the task at a 1-based position in the last listing.
    ///
    /// # Errors
    /// Returns an error when the listing has no such position.
    pub fn get_task(&mut self, id: usize) -> Result<&mut TaigaTask> {
        id.checked_sub(1)
            .and_then(|index| self.tasks.get_mut(index))
            .ok_or(Error::TaskNotFound(id))
    }

    /// Finds a sprint by name or slug.
    pub fn find_sprint(&self, name: &str) -> Option<&TaigaSprint> {
        self.sprints
            .iter()
            .find(|s| s.slug == name || s.name == name)
    }

    /// Finds the id of a sprint by name or slug.
    ///
    /// # Errors
    /// Returns an error when the project has no such sprint.
    pub fn sprint_id(&self, name: &str) -> Result<i32> {
        self.find_sprint(name)
            .map(|s| s.id)
            .ok_or_else(|| Error::SprintNotFound(name.to_string()))
    }

    /// Finds the id of a story status by slug.
    ///
    /// # Errors
    /// Returns an error when the project has no such status.
    pub fn status_id(&self, slug: &str) -> Result<i32> {
        self.statuses
            .iter()
            .find(|s| s.slug == slug)
            .map(|s| s.id)
            .ok_or_else(|| Error::StatusNotFound(slug.to_string()))
    }

    /// Finds the id of a member by username, where `"me"` means the logged-in user.
    ///
    /// # Errors
    /// Returns an error when the project has no such member.
    pub fn member_id(&self, username: &str, me: i32) -> Result<i32> {
        self.members
            .iter()
            .find(|m| {
                if username == "me" {
                    m.id == me
                } else {
                    m.username == username
                }
            })
            .map(|m| m.id)
            .ok_or_else(|| Error::UserNotFound(username.to_string()))
    }
}

impl Taiga {
    /// Looks a cached project up by name.
    ///
    /// # Errors
    /// Returns an error when no cached project has that name.
    pub fn find_project(&self, name: &str) -> Result<&TaigaProject> {
        self.projects
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| Error::ProjectNotFound(name.to_string()))
    }

    /// Returns a working copy of a project, refetching its metadata when missing or `stale`.
    ///
    /// Hand the copy back with [`Taiga::save_project`] to keep any change to its tasks.
    ///
    /// # Errors
    /// Returns an error when the project is unknown or the refetch fails.
    pub fn load_project(
        &mut self,
        name: &str,
        stale: impl FnOnce(&TaigaProject) -> bool,
    ) -> Result<TaigaProject> {
        let project = self.find_project(name)?.clone();
        if !project.statuses.is_empty() && !stale(&project) {
            return Ok(project);
        }

        let project = TaigaProject {
            tasks: project.tasks,
            ..self.get_project(project.id)?
        };
        self.save_project(project.clone())?;
        Ok(project)
    }

    /// Stores a working copy in the cache.
    ///
    /// # Errors
    /// Returns an error when the cache cannot be written.
    pub fn save_project(&mut self, project: TaigaProject) -> Result<()> {
        if let Some(known) = self.projects.iter_mut().find(|p| p.id == project.id) {
            *known = project;
        }
        self.save_cache()
    }

    /// Syncs the cached project list with Taiga, keeping what is already cached.
    ///
    /// # Errors
    /// Returns an error when the list cannot be fetched or the cache cannot be written.
    pub fn update_projects(&mut self) -> Result<()> {
        let fetched = self.get_projects()?;
        merge_projects(&mut self.projects, fetched);
        self.save_cache()
    }

    fn get_projects(&mut self) -> Result<Vec<TaigaProject>> {
        self.get::<Vec<ProjectsResponse>>(&format!("/projects?member={}", self.id))
            .map(|ps| {
                ps.into_iter()
                    .map(|p| TaigaProject {
                        id: p.id,
                        name: p.name,
                        ..Default::default()
                    })
                    .collect()
            })
    }

    fn get_project(&mut self, id: i32) -> Result<TaigaProject> {
        let p = self.get::<ProjectResponse>(&format!("/projects/{}", id))?;
        let sprints = self.get_milestones(id)?;
        let to_statuses = |statuses: &[Status]| -> Vec<TaigaStatus> {
            statuses
                .iter()
                .map(|status| TaigaStatus {
                    id: status.id,
                    slug: status.slug.clone(),
                    is_closed: status.is_closed,
                })
                .collect()
        };

        Ok(TaigaProject {
            id: p.id,
            name: p.name.clone(),
            members: p
                .members
                .iter()
                .map(|member| TaigaUser {
                    id: member.id,
                    username: member.username.clone(),
                })
                .collect(),
            statuses: to_statuses(&p.us_statuses),
            task_statuses: to_statuses(&p.task_statuses),
            sprints,
            tasks: Vec::new(),
        })
    }

    fn get_milestones(&mut self, project_id: i32) -> Result<Vec<TaigaSprint>> {
        self.get::<Vec<MilestoneResponse>>(&format!("/milestones?project={}", project_id))
            .map(|ms| {
                ms.iter()
                    .map(|m| TaigaSprint {
                        id: m.id,
                        name: m.name.clone(),
                        slug: m.slug.clone(),
                    })
                    .collect()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn story(id: i32) -> TaigaTask {
        TaigaTask {
            id,
            name: format!("story {id}"),
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

    fn project() -> TaigaProject {
        TaigaProject {
            id: 7,
            name: "demo".to_string(),
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
            tasks: vec![story(100), story(200), story(300)],
            ..Default::default()
        }
    }

    #[test]
    fn tasks_are_numbered_from_one_in_the_last_listing() {
        let mut project = project();
        assert_eq!(project.get_task(1).unwrap().id, 100);
        assert_eq!(project.get_task(3).unwrap().id, 300);
        assert!(matches!(project.get_task(0), Err(Error::TaskNotFound(0))));
        assert!(matches!(project.get_task(4), Err(Error::TaskNotFound(4))));
    }

    #[test]
    fn members_resolve_by_name_and_me_means_the_logged_in_user() {
        let project = project();
        assert_eq!(project.member_id("arjun", 1).unwrap(), 2);
        assert_eq!(project.member_id("me", 1).unwrap(), 1);
        assert_eq!(project.member_id("me", 2).unwrap(), 2);
        assert!(matches!(
            project.member_id("nobody", 1),
            Err(Error::UserNotFound(name)) if name == "nobody"
        ));
        assert!(project.member_id("me", 99).is_err());
    }

    #[test]
    fn statuses_and_sprints_resolve_by_their_names() {
        let project = project();
        assert_eq!(project.status_id("new").unwrap(), 10);
        assert!(matches!(
            project.status_id("missing"),
            Err(Error::StatusNotFound(_))
        ));
        assert_eq!(project.sprint_id("Sprint 1").unwrap(), 5);
        assert_eq!(project.sprint_id("sprint-1").unwrap(), 5);
        assert!(matches!(
            project.sprint_id("sprint 1"),
            Err(Error::SprintNotFound(_))
        ));
    }

    #[test]
    fn merging_keeps_cached_data_renames_adds_and_drops() {
        let named = |id, name: &str| TaigaProject {
            id,
            name: name.to_string(),
            ..Default::default()
        };
        let mut known = vec![project(), named(8, "gone")];
        merge_projects(&mut known, vec![named(7, "renamed"), named(9, "added")]);

        let names: Vec<&str> = known.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["renamed", "added"]);
        assert_eq!(known[0].tasks.len(), 3);
        assert_eq!(known[0].members.len(), 2);
        assert!(known[1].members.is_empty());
    }
}
