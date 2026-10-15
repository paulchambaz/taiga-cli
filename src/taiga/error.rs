//! Errors returned by the Taiga client.

use reqwest::StatusCode;
use thiserror::Error;

/// Result alias using the client [`enum@Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong while talking to Taiga or reading the local cache.
#[derive(Debug, Error)]
pub enum Error {
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{url} answered {status}: {body}")]
    Api {
        url: String,
        status: StatusCode,
        body: String,
    },
    #[error("not logged in, run `taiga login`")]
    NotLoggedIn,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("could not serialize the cache: {0}")]
    Cache(#[from] bincode::Error),
    #[error("could not find a cache directory")]
    NoCacheDir,
    #[error("no cached tasks for this project, run search first")]
    NoTaskCache,
    #[error("project '{0}' not found")]
    ProjectNotFound(String),
    #[error("status '{0}' not found")]
    StatusNotFound(String),
    #[error("sprint '{0}' not found")]
    SprintNotFound(String),
    #[error("user '{0}' is not a member of the project")]
    UserNotFound(String),
    #[error("task {0} is not in the last search")]
    TaskNotFound(usize),
    #[error("the project has no statuses")]
    NoStatuses,
}
