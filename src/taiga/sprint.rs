use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaSprint {
    pub id: i32,
    pub name: String,
    pub slug: String,
}
