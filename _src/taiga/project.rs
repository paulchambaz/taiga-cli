use serde::{Deserialize, Serialize};

use super::{status::TaigaStatus, user::TaigaUser};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaProject {
    pub id: i32,
    pub name: String,
    pub members: Vec<TaigaUser>,
    pub statuses: Vec<TaigaStatus>,
}
