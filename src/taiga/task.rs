use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use super::{Result, Taiga};
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
    pub sprint_id: Option<i32>,
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
    milestone: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct UserStoryStatus {
    name: String,
}

/// A partial update of a user story: only the set fields are sent.
///
/// `Some(None)` on `due_date` or `milestone` clears the field on the server.
#[derive(Debug, Default, Serialize)]
pub struct TaskPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_users: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_requirement: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_requirement: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_blocked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Option<i32>>,
    pub version: i32,
}

/// A user story to create.
#[derive(Debug, Serialize)]
pub struct TaskNew {
    pub project: i32,
    pub status: i32,
    pub subject: String,
    pub assigned_to: Option<i32>,
    pub assigned_users: Vec<i32>,
    pub due_date: Option<String>,
    pub milestone: Option<i32>,
    pub team_requirement: bool,
    pub client_requirement: bool,
    pub is_blocked: bool,
}

impl Taiga {
    pub fn get_tasks(&mut self, id: i32) -> Result<Vec<TaigaTask>> {
        self.get::<Vec<UserStory>>(&format!(
            "/userstories?project={}&status__is_archived=false",
            id
        ))
        .map(|ts| ts.iter().map(TaigaTask::new).collect())
    }

    /// Applies a partial update to a user story.
    ///
    /// # Errors
    /// Returns an error when the request fails, for example on a version conflict.
    pub fn patch_task(&mut self, task_id: i32, patch: &TaskPatch) -> Result<TaigaTask> {
        self.patch::<TaskPatch, UserStory>(&format!("/userstories/{task_id}"), patch)
            .map(|t| TaigaTask::new(&t))
    }

    pub fn delete_task(&mut self, task_id: i32) -> Result<()> {
        self.delete(&format!("/userstories/{}", task_id))
    }

    /// Creates a user story in a single request.
    ///
    /// # Errors
    /// Returns an error when the request fails.
    pub fn new_task(&mut self, task: &TaskNew) -> Result<TaigaTask> {
        self.post::<TaskNew, UserStory>("/userstories", task)
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
            due: t
                .due_date
                .as_ref()
                .and_then(|date| date.parse::<NaiveDate>().ok())
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .map(|dt| Utc.from_utc_datetime(&dt)),
            closed: t.is_closed,
            version: t.version,
            sprint_id: t.milestone,
        }
    }
}
