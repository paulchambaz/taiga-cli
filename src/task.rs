use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::exit;

use crate::taiga::TaigaUser;
use crate::project::TaigaStatus;
use crate::taiga::Taiga;

pub struct ReqNewArgs {
    pub status_id: i32,
    pub name: String,
    pub assign: Vec<i32>,
    pub team: bool,
    pub client: bool,
    pub block: bool,
}

pub struct ReqModArgs {
    pub status: i32,
    pub rename: String,
    pub assign: Vec<i32>,
    pub due_date: Option<String>,
    pub team: bool,
    pub client: bool,
    pub block: bool,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaTasks {
    pub id: i32,
    pub tasks: Vec<TaigaTask>,
    pub members: Vec<TaigaUser>,
    pub statuses: Vec<TaigaStatus>,
}

#[derive(Deserialize, Debug)]
struct UserStoryStatus {
    name: String,
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

// TODO: implement lazy refresh - that will happen every once in a while and im not sure its
// implemented yet
impl Taiga {
    pub fn get_tasks(&self, id: i32) -> Result<Vec<TaigaTask>> {
        let mut tasks = Vec::new();
        let mut url = format!("/userstories?project={}&status__is_archived=false", id);

        while !url.is_empty() {
            let request = self.get_request(&url);
            let response = request.send()?;

            if response.status().is_success() {

                let headers = response.headers();

                match headers.get("x-pagination-next") {
                    Some(header_value) => {
                        let mut new_url = header_value.to_str()?.to_string();
                        if let Some(pos) = new_url.rfind('/') {
                            new_url = new_url[pos..].to_string();
                        } else {
                            eprintln!("Error fetching tasks");
                            exit(1);
                        }
                        url = new_url;
                    },
                    None => url.clear(), // Clear the URL to exit the loop
                };

                let text = response.text()?;
                let user_stories_response: Vec<UserStory> = serde_json::from_str(&text)?;

                for user_story in user_stories_response.iter() {
                    tasks.push(TaigaTask::new(user_story));
                }
            } else {
                eprintln!("Error fetching tasks: {}", response.status());
                exit(1);
            }
        }

        Ok(tasks)
    }

    pub fn move_task(&self, task_id: i32, status_id: i32, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            status: i32,
            version: i32,
        }

        let status_request = StatusRequest {
            status: status_id,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn rename_task(&self, task_id: i32, name: String, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            subject: String,
            version: i32,
        }

        let status_request = StatusRequest {
            subject: name,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn assign_task(&self, task_id: i32, assign: Vec<i32>, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            assigned_users: Vec<i32>,
            version: i32,
        }

        let status_request = StatusRequest {
            assigned_users: assign,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn due_task(&self, task_id: i32, due: Option<String>, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            due_date: Option<String>,
            version: i32,
        }

        let status_request = StatusRequest {
            due_date: due,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn team_task(&self, task_id: i32, remove: bool, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            team_requirement: bool,
            version: i32,
        }

        let status_request = StatusRequest {
            team_requirement: !remove,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn client_task(&self, task_id: i32, remove: bool, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            client_requirement: bool,
            version: i32,
        }

        let status_request = StatusRequest {
            client_requirement: !remove,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn block_task(&self, task_id: i32, remove: bool, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            is_blocked: bool,
            version: i32,
        }

        let status_request = StatusRequest {
            is_blocked: !remove,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    pub fn modify_task(&self, task_id: i32, args: ReqModArgs, version: i32) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct StatusRequest {
            status: i32,
            subject: String,
            assigned_users: Vec<i32>,
            due_date: Option<String>,
            team_requirement: bool,
            client_requirement: bool,
            is_blocked: bool,
            version: i32,
        }

        let status_request = StatusRequest {
            status: args.status,
            subject: args.rename,
            assigned_users: args.assign,
            due_date: args.due_date,
            team_requirement: args.team,
            client_requirement: args.client,
            is_blocked: args.block,
            version,
        };

        let request = self.patch_request(&format!("/userstories/{}", task_id), &status_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }

    // pub fn new_task(&self, project_id: i32, status_id: i32, name: String, assign: Vec<i32>, team: bool, client: bool, block: bool) -> Result<TaigaTask> {
    pub fn new_task(&self, project_id: i32, args: ReqNewArgs) -> Result<TaigaTask> {
        #[derive(Debug, Serialize)]
        struct NewRequest {
            assigned_to: Option<i32>,
            client_requirement: bool,
            is_blocked: bool,
            project: i32,
            status: i32,
            subject: String,
            team_requirement: bool,
        }

        let member = args.assign.first().copied();

        let new_request = NewRequest {
            assigned_to: member,
            client_requirement: args.client,
            is_blocked:args. block,
            project: project_id,
            status: args.status_id,
            subject: args.name,
            team_requirement: args.team,
        };

        let request = self.post_request("/userstories", &new_request);
        let response = request.send()?;
        let text = response.text()?;

        let user_story_response: UserStory = serde_json::from_str(&text)?;

        let task = TaigaTask::new(&user_story_response);

        Ok(task)
    }
}

impl TaigaTask {
    
    fn slug(input: String) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .replace(' ', "-")
            .to_lowercase()
    }

    fn new(user_story: &UserStory) -> Self {
        TaigaTask {
            id: user_story.id,
            name: user_story.subject.clone(),
            status_id: user_story.status,
            status: Self::slug(user_story.status_extra_info.name.clone()),
            closed: user_story.is_closed,
            blocked: user_story.is_blocked,
            client: user_story.client_requirement,
            team: user_story.team_requirement,
            assigned: user_story.assigned_users.clone(),
            due: user_story.due_date.as_ref().and_then(|s| {
                NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|naive| Utc.from_utc_datetime(&naive))
            }),
            version: user_story.version,
        }
    }
}

impl TaigaTasks {
    pub fn get_task(&mut self, id: usize) -> &mut TaigaTask {
        self.tasks.get_mut(id - 1).unwrap_or_else(|| {
            eprintln!("Invalid task for this project");
            exit(1);
        })
    }

    pub fn from_cache(id: i32) -> Option<Self> {
        let project_dirs = ProjectDirs::from("", "", "taiga").unwrap_or_else(|| {
            eprintln!("Could not get standard directories");
            exit(1);
        });
        let cache_dir = project_dirs.cache_dir();

        let mut hasher = Sha1::new();
        hasher.update(format!("tasks-{}", id).as_bytes());
        let hash = hasher.finalize();
        let filename = format!("{:x}", hash);

        let path = cache_dir.join(filename);

        if !path.exists() {
            return None;
        }

        let mut file = File::open(&path).expect("Could not open cache file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Could not read cache file");
        let tasks =
            bincode::deserialize::<Self>(&buffer[..]).expect("Could not deserialize cache file");

        Some(tasks)
    }

    pub fn save_cache(self) {
        let project_dirs =
            ProjectDirs::from("", "", "taiga").expect("Could not get standard directories");
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).expect("Could not create parent directories");

        let mut hasher = Sha1::new();
        hasher.update(format!("tasks-{}", self.id).as_bytes());
        let hash = hasher.finalize();
        let filename = format!("{:x}", hash);

        let path = cache_dir.join(filename);
        let serialized_data = bincode::serialize(&self).expect("Serialization failed");
        let mut file = File::create(path).expect("Could not create cache file");
        file.write_all(&serialized_data)
            .expect("Could not save cache");
    }
}
