use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaUser {
    pub id: i32,
    pub username: String,
}

impl TaigaUser {}
