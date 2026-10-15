//! Tasks nested under user stories.

use serde::{Deserialize, Serialize};
use std::panic::resume_unwind;
use std::thread;

use super::{Result, Taiga};
use crate::utils::slug;

/// A task nested under a user story.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaSubtask {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub status_id: i32,
    pub status: String,
    pub assigned: Option<i32>,
    pub closed: bool,
    pub version: i32,
}

#[derive(Deserialize, Debug)]
struct TaskResponse {
    id: i32,
    subject: String,
    #[serde(default)]
    description: String,
    status: i32,
    status_extra_info: TaskStatusInfo,
    assigned_to: Option<i32>,
    is_closed: bool,
    version: i32,
}

#[derive(Deserialize, Debug)]
struct TaskStatusInfo {
    name: String,
}

#[derive(Debug, Serialize)]
struct SubtaskNewRequest {
    project: i32,
    user_story: i32,
    subject: String,
    description: String,
    status: i32,
    assigned_to: Option<i32>,
}

#[derive(Debug, Serialize)]
struct SubtaskDescribeRequest {
    description: String,
    version: i32,
}

impl Taiga {
    /// Lists the subtasks of a story, without their descriptions.
    ///
    /// # Errors
    /// Returns an error when the request fails.
    pub fn get_subtasks(
        &mut self,
        project_id: i32,
        user_story_id: i32,
    ) -> Result<Vec<TaigaSubtask>> {
        self.get::<Vec<TaskResponse>>(&format!(
            "/tasks?project={}&user_story={}",
            project_id, user_story_id
        ))
        .map(|ts| ts.into_iter().map(TaigaSubtask::new).collect())
    }

    /// Lists the subtasks of a story with their descriptions, which only the detail view has.
    ///
    /// The detail requests run in parallel, right after the list request has checked the token.
    ///
    /// # Errors
    /// Returns an error when any of the requests fails.
    pub fn get_described_subtasks(
        &mut self,
        project_id: i32,
        user_story_id: i32,
    ) -> Result<Vec<TaigaSubtask>> {
        let subtasks = self.get_subtasks(project_id, user_story_id)?;
        let taiga = &*self;

        thread::scope(|scope| {
            let requests: Vec<_> = subtasks
                .iter()
                .map(|subtask| {
                    scope.spawn(move || {
                        taiga.get_once::<TaskResponse>(&format!("/tasks/{}", subtask.id))
                    })
                })
                .collect();

            requests
                .into_iter()
                .map(|request| {
                    request
                        .join()
                        .unwrap_or_else(|panic| resume_unwind(panic))
                        .map(TaigaSubtask::new)
                })
                .collect()
        })
    }

    /// Creates a subtask under a story.
    ///
    /// # Errors
    /// Returns an error when the request fails.
    pub fn new_subtask(
        &mut self,
        project_id: i32,
        user_story_id: i32,
        status: i32,
        name: String,
        description: String,
        assign: Option<i32>,
    ) -> Result<TaigaSubtask> {
        self.post::<SubtaskNewRequest, TaskResponse>(
            "/tasks",
            &SubtaskNewRequest {
                project: project_id,
                user_story: user_story_id,
                subject: name,
                description,
                status,
                assigned_to: assign,
            },
        )
        .map(TaigaSubtask::new)
    }

    /// Replaces the description of a subtask.
    ///
    /// # Errors
    /// Returns an error when the request fails, for example on a version conflict.
    pub fn describe_subtask(
        &mut self,
        id: i32,
        description: String,
        version: i32,
    ) -> Result<TaigaSubtask> {
        self.patch::<SubtaskDescribeRequest, TaskResponse>(
            &format!("/tasks/{}", id),
            &SubtaskDescribeRequest {
                description,
                version,
            },
        )
        .map(TaigaSubtask::new)
    }

    /// Deletes a subtask.
    ///
    /// # Errors
    /// Returns an error when the request fails.
    pub fn delete_subtask(&mut self, id: i32) -> Result<()> {
        self.delete(&format!("/tasks/{}", id)).map(|_| ())
    }
}

impl TaigaSubtask {
    fn new(t: TaskResponse) -> TaigaSubtask {
        TaigaSubtask {
            id: t.id,
            name: t.subject,
            description: t.description,
            status_id: t.status,
            status: slug(t.status_extra_info.name),
            assigned: t.assigned_to,
            closed: t.is_closed,
            version: t.version,
        }
    }
}
