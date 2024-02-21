use std::{process::exit, time::SystemTime};

use serde::{Deserialize, Serialize};

use super::project::TaigaProject;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Taiga {
    pub auth_token: String,
    pub refresh: String,
    pub refresh_time: SystemTime,
    pub url: String,
    pub id: i32,
    pub projects: Vec<TaigaProject>,
}

impl Taiga {
    /// loads taiga from cache, updating it when needed
    /// returns if it exists an up-to-date taiga struct
    pub fn load() -> Option<Self> {
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
}
