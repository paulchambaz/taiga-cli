//! User stories: fetching, creating, updating and deleting them.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use super::{Result, Taiga};
use crate::utils::slug;

/// A user story, the card of the kanban.
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
    /// Whether its status counts as finished.
    pub closed: bool,
    /// Taiga's edit counter, which updates must send back to avoid overwriting newer changes.
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
    pub assigned_to: Option<Option<i32>>,
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

impl TaskPatch {
    /// Sets the assignees, keeping the primary assignee in step so removals stick.
    pub fn assigned(users: Vec<i32>) -> Self {
        Self {
            assigned_to: Some(users.first().copied()),
            assigned_users: Some(users),
            ..Default::default()
        }
    }
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
    /// Lists the unarchived user stories of a project.
    ///
    /// # Errors
    /// Returns an error when the request fails.
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

    /// Deletes a user story.
    ///
    /// # Errors
    /// Returns an error when the request fails.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_patch_sends_only_what_is_set_and_always_the_version() {
        let patch = TaskPatch {
            status: Some(2),
            version: 3,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&patch).unwrap(),
            json!({"status": 2, "version": 3})
        );

        let patch = TaskPatch {
            subject: Some("renamed".to_string()),
            assigned_users: Some(vec![1, 2]),
            team_requirement: Some(false),
            version: 1,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&patch).unwrap(),
            json!({
                "subject": "renamed",
                "assigned_users": [1, 2],
                "team_requirement": false,
                "version": 1
            })
        );
    }

    #[test]
    fn a_patch_clears_the_due_date_and_sprint_with_null() {
        let patch = TaskPatch {
            due_date: Some(None),
            milestone: Some(None),
            version: 1,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&patch).unwrap(),
            json!({"due_date": null, "milestone": null, "version": 1})
        );

        let patch = TaskPatch {
            due_date: Some(Some("2030-01-02".to_string())),
            milestone: Some(Some(5)),
            version: 1,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&patch).unwrap(),
            json!({"due_date": "2030-01-02", "milestone": 5, "version": 1})
        );
    }

    fn story(overrides: serde_json::Value) -> UserStory {
        let mut story = json!({
            "id": 11,
            "subject": "first story",
            "status": 3,
            "status_extra_info": {"name": "In progress"},
            "team_requirement": true,
            "client_requirement": false,
            "is_blocked": false,
            "assigned_users": [1, 2],
            "due_date": "2030-01-02",
            "is_closed": false,
            "version": 4,
            "milestone": 5
        });
        story
            .as_object_mut()
            .unwrap()
            .extend(overrides.as_object().unwrap().clone());
        serde_json::from_value(story).unwrap()
    }

    #[test]
    fn a_story_maps_to_a_task() {
        let task = TaigaTask::new(&story(json!({})));
        assert_eq!(task.id, 11);
        assert_eq!(task.name, "first story");
        assert_eq!(task.status_id, 3);
        assert_eq!(task.status, "in-progress");
        assert!(task.team && !task.client && !task.blocked);
        assert_eq!(task.assigned, [1, 2]);
        assert_eq!(task.version, 4);
        assert_eq!(task.sprint_id, Some(5));
        assert!(!task.closed);
        assert_eq!(task.due.unwrap().to_rfc3339(), "2030-01-02T00:00:00+00:00");
    }

    #[test]
    fn closed_stories_are_closed_and_odd_dates_are_ignored() {
        assert!(TaigaTask::new(&story(json!({"is_closed": true}))).closed);
        assert!(TaigaTask::new(&story(json!({"due_date": null})))
            .due
            .is_none());
        assert!(TaigaTask::new(&story(json!({"due_date": "soon"})))
            .due
            .is_none());
        assert!(TaigaTask::new(&story(json!({"milestone": null})))
            .sprint_id
            .is_none());
    }
}
