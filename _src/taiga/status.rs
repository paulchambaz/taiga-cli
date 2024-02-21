use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaStatus {
    pub id: i32,
    pub slug: String,
    pub is_closed: bool,
}
