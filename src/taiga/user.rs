//! Project members.

use serde::{Deserialize, Serialize};

/// A member of a project.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaUser {
    pub id: i32,
    pub username: String,
}
