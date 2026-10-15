//! Client for the Taiga REST API and the cache of what it returned.

use serde::{Deserialize, Serialize};

pub mod auth;
pub mod cache;
pub mod error;
pub mod project;
pub mod request;
pub mod sprint;
pub mod status;
pub mod subtask;
pub mod task;
pub mod user;

pub use self::error::*;
pub use self::project::*;
pub use self::sprint::*;
pub use self::status::*;
pub use self::subtask::*;
pub use self::task::*;
pub use self::user::*;

/// A logged-in session together with everything cached for it.
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
