use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{Taiga, TaigaStatus};
use crate::utils::slug;

// A Taiga "Task" nested under a User Story - named TaigaSubtask here to
// avoid colliding with TaigaTask, which this CLI already uses to mean a
// User Story (see README: every top-level "task" command in this tool is a
// Taiga user story, not a Taiga task).
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

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TaskStatusResponse {
    id: i32,
    slug: String,
    is_closed: bool,
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
    pub fn get_task_statuses(&mut self, project_id: i32) -> Result<Vec<TaigaStatus>> {
        self.get::<Vec<TaskStatusResponse>>(&format!("/task-statuses?project={}", project_id))
            .map(|ss| {
                ss.iter()
                    .map(|s| TaigaStatus {
                        id: s.id,
                        slug: s.slug.clone(),
                        is_closed: s.is_closed,
                    })
                    .collect()
            })
    }

    pub fn get_subtasks(
        &mut self,
        project_id: i32,
        user_story_id: i32,
    ) -> Result<Vec<TaigaSubtask>> {
        self.get::<Vec<TaskResponse>>(&format!(
            "/tasks?project={}&user_story={}",
            project_id, user_story_id
        ))
        .map(|ts| ts.iter().map(TaigaSubtask::new).collect())
    }

    // The list endpoint above uses Taiga's lean list serializer, which
    // omits `description` - only the single-object detail endpoint
    // includes it.
    pub fn get_subtask(&mut self, id: i32) -> Result<TaigaSubtask> {
        self.get::<TaskResponse>(&format!("/tasks/{}", id))
            .map(|t| TaigaSubtask::new(&t))
    }

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
        .map(|t| TaigaSubtask::new(&t))
    }

    pub fn describe_subtask(
        &mut self,
        id: i32,
        description: String,
        version: i32,
    ) -> Result<TaigaSubtask> {
        self.patch::<SubtaskDescribeRequest, TaskResponse>(
            &format!("/tasks/{}", id),
            &SubtaskDescribeRequest { description, version },
        )
        .map(|t| TaigaSubtask::new(&t))
    }

    pub fn delete_subtask(&mut self, id: i32) -> Result<()> {
        self.delete(&format!("/tasks/{}", id)).map(|_| ())
    }
}

impl TaigaSubtask {
    fn new(t: &TaskResponse) -> TaigaSubtask {
        TaigaSubtask {
            id: t.id,
            name: t.subject.clone(),
            description: t.description.clone(),
            status_id: t.status,
            status: slug(t.status_extra_info.name.clone()),
            assigned: t.assigned_to,
            closed: t.is_closed,
            version: t.version,
        }
    }
}
