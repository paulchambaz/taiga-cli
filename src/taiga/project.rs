use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Taiga;
use super::TaigaSprint;
use super::TaigaStatus;
use super::TaigaUser;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaProject {
    pub id: i32,
    pub name: String,
    pub members: Vec<TaigaUser>,
    pub statuses: Vec<TaigaStatus>,
    pub task_statuses: Vec<TaigaStatus>,
    pub sprints: Vec<TaigaSprint>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectsResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ProjectResponse {
    id: i32,
    name: String,
    members: Vec<MemberResponse>,
    us_statuses: Vec<Status>,
    task_statuses: Vec<Status>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MemberResponse {
    id: i32,
    username: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Status {
    id: i32,
    slug: String,
    is_closed: bool,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MilestoneResponse {
    id: i32,
    name: String,
    slug: String,
}

impl Taiga {
    pub fn find_project(&mut self, name: String) -> Result<TaigaProject> {
        self.projects
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow!("Project '{}' not found", name))
            .map(|project| project.clone())
    }

    pub fn get_projects(&mut self) -> Result<Vec<TaigaProject>> {
        self.get::<Vec<ProjectsResponse>>(&format!("/projects?member={}", self.id))
            .map(|ps| {
                ps.iter()
                    .map(|p| TaigaProject {
                        id: p.id,
                        name: p.name.clone(),
                        members: Vec::new(),
                        statuses: Vec::new(),
                        task_statuses: Vec::new(),
                        sprints: Vec::new(),
                    })
                    .collect()
            })
    }

    pub fn get_project(&mut self, id: i32) -> Result<TaigaProject> {
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
        })
    }

    pub fn get_milestones(&mut self, project_id: i32) -> Result<Vec<TaigaSprint>> {
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
