use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    process::exit,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::taiga::Taiga;

/// Load arbitrary deserializable data into a given struct
///
/// Args:
/// - `path`: the path to the file containing the cache
///
/// Returns:
/// The loaded struct
fn from_cache<T>(path: PathBuf) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let mut file = File::open(path).unwrap_or_else(|err| {
        eprintln!("Error, could not load taiga file: '{}'", err);
        exit(1);
    });
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap_or_else(|err| {
        eprintln!("Error, could not load taiga file: '{}'", err);
        exit(1);
    });
    bincode::deserialize::<T>(&buffer[..]).unwrap_or_else(|err| {
        eprintln!("Error, could not parse taiga file: '{}'", err);
        exit(1);
    })
}

/// Save arbitrary serializable data into a cache file
///
/// Args:
/// - `path`: the path to the file to save to
/// - `data`: the data to save
fn save_cache<T>(path: PathBuf, data: &T)
where
    T: Serialize,
{
    let data = bincode::serialize(data).unwrap_or_else(|err| {
        eprintln!("Error, could not create data: '{}'", err);
        exit(1);
    });
    let mut file = File::create(path).unwrap_or_else(|err| {
        eprintln!("Error, could not create file: '{}'", err);
        exit(1);
    });
    file.write_all(&data).unwrap_or_else(|err| {
        eprintln!("Error, could not write file: '{}'", err);
        exit(1);
    });
}

impl Taiga {
    /// Get file in the standard cache directory
    ///
    /// Args:
    /// - `file`: The name of the file requested
    ///
    /// Returns:
    /// The path to the file
    fn get_cache_file(file: &str) -> PathBuf {
        let project_dirs = ProjectDirs::from("", "", "taiga").unwrap_or_else(|| {
            eprintln!("Could not get standard directories");
            exit(1);
        });
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).expect("Could not create parent directories");
        cache_dir.join(file)
    }

    /// Loads taiga struct from the standard taiga cache file
    ///
    /// Returns:
    /// The taiga struct from the cache file if it exists
    pub fn from_cache() -> Option<Self> {
        // get path to the taiga cache file
        let cache_file = Self::get_cache_file("config");
        if !cache_file.exists() {
            return None;
        }
        let taiga = from_cache::<Taiga>(cache_file);
        Some(taiga)
    }

    /// Save taiga struct to the standard taiga cache file
    ///
    /// Args:
    /// - `self`: Taiga struct
    pub fn save_cache(&self) {
        let cache_file = Self::get_cache_file("config");
        save_cache::<Taiga>(cache_file, self);
    }
}
