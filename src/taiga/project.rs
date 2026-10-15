use serde::{Deserialize, Serialize};

use super::Error;
use super::Result;
use super::Taiga;
use super::TaigaSprint;
use super::TaigaStatus;
use super::TaigaTask;
use super::TaigaUser;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TaigaProject {
    pub id: i32,
    pub name: String,
    pub members: Vec<TaigaUser>,
    pub statuses: Vec<TaigaStatus>,
    pub task_statuses: Vec<TaigaStatus>,
    pub sprints: Vec<TaigaSprint>,
    pub tasks: Vec<TaigaTask>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectsResponse {
    pub id: i32,
    pub name: String,
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

    pub fn find_sprint(&self, name: &str) -> Option<&TaigaSprint> {
        self.sprints
            .iter()
            .find(|s| s.slug == name || s.name == name)
    }

    pub fn sprint_id(&self, name: &str) -> Result<i32> {
        self.find_sprint(name)
            .map(|s| s.id)
            .ok_or_else(|| Error::SprintNotFound(name.to_string()))
    }

    pub fn status_id(&self, slug: &str) -> Result<i32> {
        self.statuses
            .iter()
            .find(|s| s.slug == slug)
            .map(|s| s.id)
            .ok_or_else(|| Error::StatusNotFound(slug.to_string()))
    }

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
        self.projects
            .retain(|p| fetched.iter().any(|f| f.id == p.id));
        for project in fetched {
            match self.projects.iter_mut().find(|p| p.id == project.id) {
                Some(known) => known.name = project.name,
                None => self.projects.push(project),
            }
        }
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
