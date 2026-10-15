use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::{Error, Result, Taiga, TaigaProject, TaigaSprint, TaigaTask, TaigaTasks};

fn cache_dir() -> Result<PathBuf> {
    ProjectDirs::from("", "", "taiga")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or(Error::NoCacheDir)
}

fn hashed_cache_path(key: &str) -> Result<PathBuf> {
    let hash = Sha1::digest(key.as_bytes());
    Ok(cache_dir()?.join(format!("{hash:x}")))
}

fn read_cache<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(bincode::deserialize(&fs::read(path)?).ok())
}

fn write_cache<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(path)?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))?;

    file.write_all(&bincode::serialize(value)?)?;
    Ok(())
}

impl Taiga {
    pub fn from_cache() -> Result<Option<Self>> {
        read_cache(&cache_dir()?.join("config"))
    }

    pub fn save_cache(&self) -> Result<()> {
        write_cache(&cache_dir()?.join("config"), self)
    }

    pub fn tasks_from_cache<F>(&mut self, id: i32, update: F) -> Result<TaigaTasks>
    where
        F: FnOnce(&TaigaTasks) -> bool,
    {
        let tasks = TaigaTasks::from_cache(id)?.ok_or(Error::NoTaskCache)?;
        if update(&tasks) {
            self.update_tasks(id, tasks)
        } else {
            Ok(tasks)
        }
    }
}

impl TaigaProject {
    pub fn from_cache(id: i32) -> Result<Option<Self>> {
        read_cache(&hashed_cache_path(&id.to_string())?)
    }

    pub fn save_cache(&self) -> Result<()> {
        write_cache(&hashed_cache_path(&self.id.to_string())?, self)
    }
}

impl TaigaTasks {
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

    pub fn from_cache(id: i32) -> Result<Option<Self>> {
        read_cache(&hashed_cache_path(&format!("tasks-{id}"))?)
    }

    pub fn save_cache(&self) -> Result<()> {
        write_cache(&hashed_cache_path(&format!("tasks-{}", self.id))?, self)
    }
}
