//! The on-disk cache: the login and everything fetched from Taiga.

use directories::ProjectDirs;
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use super::{Error, Result, Taiga};

/// Layout version of the cache file, to bump whenever `Taiga` changes.
const CACHE_VERSION: u32 = 2;

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
        Self::read_from(&cache_path()?)
    }

    /// Writes the cache atomically, readable by the current user only.
    ///
    /// # Errors
    /// Returns an error when the cache directory or file cannot be written.
    pub fn save_cache(&self) -> Result<()> {
        self.write_to(&cache_path()?)
    }

    fn read_from(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path)?;
        let config = bincode::config::standard();
        let version = bincode::serde::decode_from_slice::<u32, _>(&bytes, config);
        if version.ok().map(|(version, _)| version) != Some(CACHE_VERSION) {
            return Ok(None);
        }
        Ok(
            bincode::serde::decode_from_slice::<(u32, Self), _>(&bytes, config)
                .ok()
                .map(|((_, taiga), _)| taiga),
        )
    }

    fn write_to(&self, path: &Path) -> Result<()> {
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
            .write_all(&bincode::serde::encode_to_vec(
                (CACHE_VERSION, self),
                bincode::config::standard(),
            )?)?;
        fs::rename(tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taiga::TaigaProject;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("taiga-test-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn cache(&self) -> PathBuf {
            self.0.join("config")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn session() -> Taiga {
        Taiga {
            auth_token: "token".to_string(),
            refresh: "refresh".to_string(),
            url: "http://localhost/api/v1".to_string(),
            id: 1,
            username: "paul".to_string(),
            password: "secret".to_string(),
            projects: vec![TaigaProject {
                id: 7,
                name: "demo".to_string(),
                ..Default::default()
            }],
        }
    }

    #[test]
    fn a_saved_session_is_read_back() {
        let dir = TempDir::new("round-trip");
        session().write_to(&dir.cache()).unwrap();

        let loaded = Taiga::read_from(&dir.cache()).unwrap().unwrap();
        assert_eq!(loaded.auth_token, "token");
        assert_eq!(loaded.password, "secret");
        assert_eq!(loaded.projects[0].name, "demo");
    }

    #[test]
    fn a_missing_cache_is_not_an_error() {
        let dir = TempDir::new("missing");
        assert!(Taiga::read_from(&dir.cache()).unwrap().is_none());
    }

    #[test]
    fn an_unusable_cache_counts_as_no_cache() {
        let dir = TempDir::new("unusable");
        session().write_to(&dir.cache()).unwrap();
        let valid = fs::read(dir.cache()).unwrap();

        let mut other_version = fs::read(dir.cache()).unwrap();
        other_version[0] = other_version[0].wrapping_add(1);

        let cases = [
            ("empty", Vec::new()),
            ("garbage", b"definitely not a cache".to_vec()),
            ("truncated", valid[..valid.len() / 2].to_vec()),
            ("other version", other_version),
        ];
        for (name, bytes) in cases {
            fs::write(dir.cache(), bytes).unwrap();
            assert!(Taiga::read_from(&dir.cache()).unwrap().is_none(), "{name}");
        }
    }

    #[test]
    fn saving_replaces_the_previous_cache_without_leftovers() {
        let dir = TempDir::new("replace");
        session().write_to(&dir.cache()).unwrap();

        let mut newer = session();
        newer.auth_token = "newer".to_string();
        newer.write_to(&dir.cache()).unwrap();

        let loaded = Taiga::read_from(&dir.cache()).unwrap().unwrap();
        assert_eq!(loaded.auth_token, "newer");
        assert!(!dir.cache().with_extension("tmp").exists());
    }

    #[cfg(unix)]
    #[test]
    fn the_cache_is_private_even_over_a_world_readable_file() {
        let dir = TempDir::new("private");
        session().write_to(&dir.cache()).unwrap();
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&dir.cache()), 0o600);

        fs::set_permissions(dir.cache(), fs::Permissions::from_mode(0o644)).unwrap();
        session().write_to(&dir.cache()).unwrap();
        assert_eq!(mode(&dir.cache()), 0o600);
    }
}
