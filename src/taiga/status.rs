//! Statuses of user stories and tasks.

use serde::{Deserialize, Serialize};

/// A status a user story or task can be in.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaStatus {
    pub id: i32,
    pub slug: String,
    pub is_closed: bool,
}
