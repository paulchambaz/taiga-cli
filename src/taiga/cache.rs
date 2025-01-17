use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use super::Taiga;
use super::TaigaProject;

impl Taiga {
    pub fn from_cache() -> Option<Self> {
        let cache_path = Self::get_cache_path()?;

        if !cache_path.exists() {
            return None;
        }

        let mut file = match File::open(&cache_path) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("Could not open Taiga config. Please log in.");
                return None;
            }
        };

        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_err() {
            eprintln!("Could not read config file. Please log in.");
            return None;
        }

        match bincode::deserialize(&buffer) {
            Ok(taiga) => Some(taiga),
            Err(_) => {
                eprintln!("Config file is corrupted. Please log in.");
                None
            }
        }
    }

    pub fn save_cache(&self) -> Result<()> {
        let cache_path =
            Self::get_cache_path().ok_or_else(|| anyhow!("Could not determine cache directory"))?;

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let serialized =
            bincode::serialize(&self).map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        let mut file = File::create(&cache_path)
            .map_err(|e| anyhow!("Failed to create config file: {}", e))?;

        file.write_all(&serialized)
            .map_err(|e| anyhow!("Failed to write config: {}", e))?;

        // Set appropriate file permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&cache_path, perms)?;
        }

        Ok(())
    }

    pub fn clear_cache() -> Result<()> {
        if let Some(cache_path) = Self::get_cache_path() {
            if cache_path.exists() {
                fs::remove_file(cache_path)?;
            }
        }
        Ok(())
    }

    fn get_cache_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "taiga").map(|proj_dirs| proj_dirs.cache_dir().join("config"))
    }
}

impl TaigaProject {
    // Get cache file path for a project ID
    fn cache_path(id: i32) -> Result<PathBuf> {
        let project_dirs =
            ProjectDirs::from("", "", "taiga").context("Could not get standard directories")?;

        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).context("Could not create cache directory")?;

        let mut hasher = Sha1::new();
        hasher.update(id.to_string().as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        Ok(cache_dir.join(hash))
    }

    // Load project from cache
    pub fn from_cache(id: i32) -> Result<Option<Self>> {
        let path = Self::cache_path(id)?;

        if !path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&path).context("Could not open cache file")?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .context("Could not read cache file")?;

        bincode::deserialize(&buffer)
            .context("Could not deserialize cache data")
            .map(Some)
    }

    // Save project to cache
    pub fn save_cache(&self) -> Result<()> {
        let path = Self::cache_path(self.id)?;

        let serialized_data =
            bincode::serialize(self).context("Could not serialize project data")?;

        let mut file = File::create(&path).context("Could not create cache file")?;

        file.write_all(&serialized_data)
            .context("Could not write cache data")?;

        Ok(())
    }
}
