//! Sprints and their burndown figures.

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::{Error, Result, Taiga};

/// A sprint, which Taiga calls a milestone.
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

fn pick_current(sprints: &[SprintDates], today: NaiveDate) -> Option<i32> {
    let open = || sprints.iter().filter(|sprint| !sprint.closed);

    open()
        .find(|sprint| sprint.estimated_start <= today && today <= sprint.estimated_finish)
        .or_else(|| open().max_by_key(|sprint| sprint.estimated_finish))
        .map(|sprint| sprint.id)
}

impl Taiga {
    /// Finds the open sprint running today, or else the open one ending last.
    ///
    /// # Errors
    /// Returns an error when the sprints cannot be fetched or none is open.
    pub fn current_sprint(&mut self, project_id: i32) -> Result<i32> {
        let sprints: Vec<SprintDates> = self.get(&format!("/milestones?project={project_id}"))?;
        pick_current(&sprints, Utc::now().date_naive()).ok_or(Error::NoOpenSprint)
    }

    /// Fetches the burndown figures of a sprint.
    ///
    /// # Errors
    /// Returns an error when the request fails or the answer is not understood.
    pub fn sprint_stats(&mut self, sprint_id: i32) -> Result<SprintStats> {
        self.get(&format!("/milestones/{sprint_id}/stats"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    fn sprint(id: i32, start: u32, finish: u32, closed: bool) -> SprintDates {
        SprintDates {
            id,
            estimated_start: day(start),
            estimated_finish: day(finish),
            closed,
        }
    }

    #[test]
    fn the_sprint_running_today_wins() {
        let sprints = [
            sprint(1, 1, 7, false),
            sprint(2, 8, 21, false),
            sprint(3, 22, 30, false),
        ];
        assert_eq!(pick_current(&sprints, day(10)), Some(2));
    }

    #[test]
    fn without_a_running_sprint_the_open_one_ending_last_wins() {
        let sprints = [
            sprint(1, 1, 7, false),
            sprint(2, 8, 14, false),
            sprint(3, 2, 9, true),
        ];
        assert_eq!(pick_current(&sprints, day(20)), Some(2));
    }

    #[test]
    fn closed_sprints_are_ignored_even_when_they_cover_today() {
        let sprints = [sprint(1, 1, 30, true), sprint(2, 1, 5, false)];
        assert_eq!(pick_current(&sprints, day(10)), Some(2));
        assert_eq!(pick_current(&sprints[..1], day(10)), None);
        assert_eq!(pick_current(&[], day(10)), None);
    }
}
