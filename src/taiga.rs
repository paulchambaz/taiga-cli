use directories::ProjectDirs;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::exit;
use std::time::SystemTime;

use crate::project::TaigaProject;
use crate::task::TaigaTasks;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Taiga {
    pub auth_token: String,
    pub refresh: String,
    pub refresh_time: SystemTime,
    pub url: String,
    pub id: i32,
    pub projects: Vec<TaigaProject>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaUser {
    pub id: i32,
    pub username: String,
}

impl Taiga {
    pub fn new() -> Option<Self> {
        let mut taiga = Self::from_cache();

        if let Some(taiga) = taiga.as_mut() {
            if taiga.refresh_time < SystemTime::now() {
                taiga
                    .refresh()
                    .map_err(|err| {
                        eprintln!("Could not refresh connection: {}", err);
                        exit(1);
                    })
                    .ok();
            }
        }

        taiga
    }

    pub fn from_cache() -> Option<Self> {
        let project_dirs = ProjectDirs::from("", "", "taiga").unwrap_or_else(|| {
            eprintln!("Could not get standard directories");
            exit(1);
        });

        let cache_dir = project_dirs.cache_dir();
        let path = cache_dir.join("config");

        if !path.exists() {
            return None;
        }

        // TODO: return warnings here, that the cache file is corrupt
        // we should make sure that attempting to login results in a clean try
        let mut file = File::open(&path).unwrap_or_else(|_| {
            eprintln!("Could not load taiga config. Please log in.");
            exit(1);
        });

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap_or_else(|_| {
            eprintln!("Could not read file from {:?}", path);
            exit(1);
        });

        let taiga = bincode::deserialize::<Self>(&buffer[..]).unwrap_or_else(|_| {
            eprintln!("Deserialization failed for file {:?}", path);
            exit(1);
        });

        Some(taiga)
    }

    pub fn save_cache(&self) {
        let project_dirs =
            ProjectDirs::from("", "", "taiga").expect("Could not get standard directories");
        let serialized_data = bincode::serialize(&self).expect("Serialization failed");
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).expect("Could not create parent directories");
        let taiga_path = cache_dir.join("config");

        let mut file = File::create(taiga_path).expect("Could not save create cache file");
        file.write_all(&serialized_data)
            .expect("Could not save cache");
    }

    pub fn get_request(&self, path: &str) -> RequestBuilder {
        let client = Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.auth_token))
                .expect("Could not get token"),
        );

        client.get(format!("{}{}", self.url, path)).headers(headers)
    }

    pub fn delete_request(&self, path: &str) -> RequestBuilder {
        let client = Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.auth_token))
                .expect("Could not get token"),
        );


        client.delete(format!("{}{}", self.url, path)).headers(headers)
    }

    pub fn post_request<T>(&self, path: &str, json: &T) -> RequestBuilder
    where
        T: Serialize + ?Sized,
    {
        let client = Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.auth_token))
                .expect("Could not get token"),
        );

        client
            .post(format!("{}{}", self.url, path))
            .headers(headers)
            .json(json)
    }

    pub fn patch_request<T>(&self, path: &str, json: &T) -> RequestBuilder
    where
        T: Serialize + ?Sized,
    {
        let client = Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.auth_token))
                .expect("Could not get token"),
        );

        client
            .patch(format!("{}{}", self.url, path))
            .headers(headers)
            .json(json)
    }

    pub fn find_project(&self, name: String) -> TaigaProject {
        self.projects.iter().find(|p| p.name == name).unwrap_or_else(|| {
            eprintln!("Error, could not find project");
            exit(1);
        }).clone()
    }

    pub fn update_tasks(&self, id: i32, tasks: TaigaTasks) -> TaigaTasks {
        let project = self.get_project(id).unwrap_or_else(|err| {
            eprintln!("Error, could not get project: {}", err);
            exit(1);
        });

        let tasks = TaigaTasks {
            id: tasks.id,
            tasks: tasks.tasks.clone(),
            members: project.members,
            statuses: project.statuses,
        };

        tasks.clone().save_cache();

        tasks
    }

    pub fn tasks_from_cache<F>(&self, id: i32, update: F) -> TaigaTasks
where
        F: FnOnce(&TaigaTasks) -> bool,
    {
        match TaigaTasks::from_cache(id) {
            Some(tasks) => {
                if update(&tasks) {
                    self.update_tasks(id, tasks)
                } else {
                    tasks
                }
            },
            None => {
                eprintln!("Invalid task id for this project");
                exit(1);
            },
        }
    }
}

