use directories::ProjectDirs;
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use super::{Error, Result, Taiga};

/// Layout version of the cache file, to bump whenever `Taiga` changes.
const CACHE_VERSION: u32 = 1;

fn cache_path() -> Result<PathBuf> {
    ProjectDirs::from("", "", "taiga")
        .map(|dirs| dirs.cache_dir().join("config"))
        .ok_or(Error::NoCacheDir)
}

impl Taiga {
    /// Loads the cache, or `None` when it is missing, unreadable or from another version.
    ///
    /// # Errors
    /// Returns an error when the cache file exists but cannot be read.
    pub fn from_cache() -> Result<Option<Self>> {
        let path = cache_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path)?;
        if bincode::deserialize::<u32>(&bytes).ok() != Some(CACHE_VERSION) {
            return Ok(None);
        }
        Ok(bincode::deserialize::<(u32, Self)>(&bytes)
            .ok()
            .map(|(_, taiga)| taiga))
    }

    /// Writes the cache atomically, readable by the current user only.
    ///
    /// # Errors
    /// Returns an error when the cache directory or file cannot be written.
    pub fn save_cache(&self) -> Result<()> {
        let path = cache_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = path.with_extension("tmp");
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        options.mode(0o600);

        options
            .open(&tmp)?
            .write_all(&bincode::serialize(&(CACHE_VERSION, self))?)?;
        fs::rename(tmp, path)?;
        Ok(())
    }
}
