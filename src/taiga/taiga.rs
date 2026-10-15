use serde::{Deserialize, Serialize};

use super::TaigaProject;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Taiga {
    pub auth_token: String,
    pub refresh: String,
    pub url: String,
    pub id: i32,
    pub username: String,
    pub password: String,
    pub projects: Vec<TaigaProject>,
}
