use std::{fs::{self, File}, process::exit};

use super::{
    taiga::TaigaUser,
    Taiga,
};
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use sha1::{Digest, Sha1};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaStatus {
    pub id: i32,
    pub slug: String,
    pub is_closed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaProject {
    pub id: i32,
    pub name: String,
    pub members: Vec<TaigaUser>,
    pub statuses: Vec<TaigaStatus>,
}

#[derive(Deserialize, Debug)]
pub struct Projects {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct Member {
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
struct Project {
    members: Vec<Member>,
    us_statuses: Vec<Status>,
}

impl Taiga {
    pub fn get_projects(&self) -> Result<Vec<TaigaProject>> {
        let url = format!("/projects?member={}", self.id);
        let request = self.get_request(&url);
        let response = request.send()?;
        let text = response.text()?;

        let projects_response: Vec<Projects> = serde_json::from_str(&text)?;

        let projects = projects_response
            .iter()
            .map(|project| TaigaProject {
                id: project.id,
                name: project.name.clone(),
                members: Vec::new(),
                statuses: Vec::new(),
            })
            .collect();

        Ok(projects)
    }

    pub fn get_project(&self, id: i32) -> Result<TaigaProject> {
        let url = format!("/projects/{}", id);

        let request = self.get_request(&url);
        let response = request.send()?;
        let text = response.text()?;

        let project_response: Project = serde_json::from_str(&text)?;

        let project = self
            .projects
            .iter()
            .find(|p| p.id == id)
            .expect("Could not find project");

        Ok(TaigaProject {
            id: project.id,
            name: project.name.clone(),
            members: project_response
                .members
                .iter()
                .map(|member| TaigaUser {
                    id: member.id,
                    username: member.username.clone(),
                })
                .collect(),
            statuses: project_response
                .us_statuses
                .iter()
                .map(|status| TaigaStatus {
                    id: status.id,
                    slug: status.slug.clone(),
                    is_closed: status.is_closed,
                })
                .collect(),
        })
    }
}

impl TaigaProject {
    pub fn from_cache(id: i32) -> Option<Self> {
        let project_dirs = ProjectDirs::from("", "", "taiga").unwrap_or_else(|| {
            eprintln!("Could not get standard directories");
            exit(1);
        });
        let cache_dir = project_dirs.cache_dir();

        let mut hasher = Sha1::new();
        hasher.update(id.to_string().as_bytes());
        let hash = hasher.finalize();
        let filename = format!("{:x}", hash);

        let path = cache_dir.join(filename);

        if !path.exists() {
            return None;
        }

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
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).expect("Could not create parent directories");

        let mut hasher = Sha1::new();
        hasher.update(self.id.to_string().as_bytes());
        let hash = hasher.finalize();
        let filename = format!("{:x}", hash);

        let path = cache_dir.join(filename);

        let serialized_data = bincode::serialize(&self).expect("Serialization failed");
        let mut file = File::create(path).expect("Could not save create cache file");
        file.write_all(&serialized_data)
            .expect("Could not save cache");
    }
}
