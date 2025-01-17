use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use super::Taiga;
use crate::utils::slug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaTask {
    pub id: i32,
    pub name: String,
    pub status_id: i32,
    pub status: String,
    pub team: bool,
    pub client: bool,
    pub blocked: bool,
    pub assigned: Vec<i32>,
    pub due: Option<DateTime<Utc>>,
    pub closed: bool,
    pub version: i32,
}

#[derive(Deserialize, Debug)]
struct UserStory {
    id: i32,
    subject: String,
    status: i32,
    status_extra_info: UserStoryStatus,
    team_requirement: bool,
    client_requirement: bool,
    is_blocked: bool,
    assigned_users: Vec<i32>,
    due_date: Option<String>,
    is_closed: bool,
    version: i32,
}

#[derive(Deserialize, Debug)]
struct UserStoryStatus {
    name: String,
}

#[derive(Debug, Serialize)]
struct TaskStatusRequest {
    status: i32,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskRenameRequest {
    subject: String,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskAssignRequest {
    assigned_users: Vec<i32>,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskDueRequest {
    due_date: Option<String>,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskTeamRequest {
    team_requirement: bool,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskClientRequest {
    client_requirement: bool,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskBlockRequest {
    is_blocked: bool,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskModifyRequest {
    status: i32,
    subject: String,
    assigned_users: Vec<i32>,
    due_date: Option<String>,
    team_requirement: bool,
    client_requirement: bool,
    is_blocked: bool,
    version: i32,
}

#[derive(Debug, Serialize)]
struct TaskNewRequest {
    assigned_to: Option<i32>,
    client_requirement: bool,
    is_blocked: bool,
    project: i32,
    status: i32,
    subject: String,
    team_requirement: bool,
}

impl Taiga {
    pub fn get_tasks(&mut self, id: i32) -> Result<Vec<TaigaTask>> {
        self.get::<Vec<UserStory>>(&format!(
            "/userstories?project={}&status__is_archived=false",
            id
        ))
        .map(|ts| ts.iter().map(TaigaTask::new).collect())
    }

    pub fn move_task(&mut self, task_id: i32, status_id: i32, version: i32) -> Result<TaigaTask> {
        self.patch::<TaskStatusRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskStatusRequest {
                status: status_id,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn delete_task(&mut self, task_id: i32) -> Result<()> {
        self.delete(&format!("/userstories/{}", task_id))
            .map(|_| ())
    }

    pub fn rename_task(&mut self, task_id: i32, name: String, version: i32) -> Result<TaigaTask> {
        self.patch::<TaskRenameRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskRenameRequest {
                subject: name,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn assign_task(
        &mut self,
        task_id: i32,
        assign: Vec<i32>,
        version: i32,
    ) -> Result<TaigaTask> {
        self.patch::<TaskAssignRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskAssignRequest {
                assigned_users: assign,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn due_task(
        &mut self,
        task_id: i32,
        due: Option<String>,
        version: i32,
    ) -> Result<TaigaTask> {
        self.patch::<TaskDueRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskDueRequest {
                due_date: due,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn team_task(&mut self, task_id: i32, remove: bool, version: i32) -> Result<TaigaTask> {
        self.patch::<TaskTeamRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskTeamRequest {
                team_requirement: !remove,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn client_task(&mut self, task_id: i32, remove: bool, version: i32) -> Result<TaigaTask> {
        self.patch::<TaskClientRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskClientRequest {
                client_requirement: !remove,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn block_task(&mut self, task_id: i32, remove: bool, version: i32) -> Result<TaigaTask> {
        self.patch::<TaskBlockRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskBlockRequest {
                is_blocked: !remove,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn modify_task(
        &mut self,
        task_id: i32,
        status: i32,
        rename: String,
        assign: Vec<i32>,
        due_date: Option<String>,
        team: bool,
        client: bool,
        block: bool,
        version: i32,
    ) -> Result<TaigaTask> {
        self.patch::<TaskModifyRequest, UserStory>(
            &format!("/userstories/{}", task_id),
            &TaskModifyRequest {
                status,
                subject: rename,
                assigned_users: assign,
                due_date,
                team_requirement: team,
                client_requirement: client,
                is_blocked: block,
                version,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }

    pub fn new_task(
        &mut self,
        project_id: i32,
        status: i32,
        name: String,
        assign: Vec<i32>,
        team: bool,
        client: bool,
        block: bool,
    ) -> Result<TaigaTask> {
        self.post::<TaskNewRequest, UserStory>(
            "/userstories",
            &TaskNewRequest {
                project: project_id,
                status,
                subject: name,
                assigned_to: assign.first().copied(),
                team_requirement: team,
                client_requirement: client,
                is_blocked: block,
            },
        )
        .map(|t| TaigaTask::new(&t))
    }
}

impl TaigaTask {
    fn new(t: &UserStory) -> TaigaTask {
        TaigaTask {
            id: t.id,
            name: t.subject.clone(),
            status_id: t.status,
            status: slug(t.status_extra_info.name.clone()),
            team: t.team_requirement,
            client: t.client_requirement,
            blocked: t.is_blocked,
            assigned: t.assigned_users.clone(),
            due: t.due_date.as_ref().and_then(|date| {
                format!("{date} 00:00:00")
                    .parse::<NaiveDateTime>()
                    .ok()
                    .map(|dt| Utc.from_utc_datetime(&dt))
            }),
            closed: false,
            version: t.version,
        }
    }
}
