use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use super::TaigaProject;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Taiga {
    pub auth_token: String,
    pub refresh: String,
    pub refresh_time: SystemTime,
    pub url: String,
    pub id: i32,
    pub username: String,
    pub password: String,
    pub projects: Vec<TaigaProject>,
}
