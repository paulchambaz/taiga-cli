use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::{Error, Result, Taiga};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaigaSprint {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

/// Burndown figures of one sprint.
#[derive(Deserialize, Debug)]
pub struct SprintStats {
    pub name: String,
    pub estimated_start: String,
    pub estimated_finish: String,
    pub total_userstories: u32,
    pub completed_userstories: u32,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub days: Vec<SprintDay>,
}

/// Points left open on one day of a sprint, next to the ideal burndown.
#[derive(Deserialize, Debug)]
pub struct SprintDay {
    pub day: String,
    pub open_points: Option<f64>,
    pub optimal_points: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct SprintDates {
    id: i32,
    estimated_start: NaiveDate,
    estimated_finish: NaiveDate,
    closed: bool,
}

impl Taiga {
    /// Finds the open sprint running today, or else the open one ending last.
    ///
    /// # Errors
    /// Returns an error when the sprints cannot be fetched or none is open.
    pub fn current_sprint(&mut self, project_id: i32) -> Result<i32> {
        let today = Utc::now().date_naive();
        let sprints: Vec<SprintDates> = self.get(&format!("/milestones?project={project_id}"))?;
        let open = || sprints.iter().filter(|sprint| !sprint.closed);

        open()
            .find(|sprint| sprint.estimated_start <= today && today <= sprint.estimated_finish)
            .or_else(|| open().max_by_key(|sprint| sprint.estimated_finish))
            .map(|sprint| sprint.id)
            .ok_or(Error::NoOpenSprint)
    }

    /// Fetches the burndown figures of a sprint.
    ///
    /// # Errors
    /// Returns an error when the request fails or the answer is not understood.
    pub fn sprint_stats(&mut self, sprint_id: i32) -> Result<SprintStats> {
        self.get(&format!("/milestones/{sprint_id}/stats"))
    }
}
