//! The `burndown` command.

use anyhow::Result;
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::{row, Cell, Row, Table};

use crate::cli::ProjectBurndownArgs;
use crate::taiga::Taiga;

/// Prints the burndown of a sprint, the current one by default.
///
/// # Errors
/// Returns an error when the sprint is unknown or a request fails.
pub fn burndown(taiga: &mut Taiga, args: ProjectBurndownArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |project| {
        args.sprint
            .as_deref()
            .is_some_and(|sprint| project.find_sprint(sprint).is_none())
    })?;

    let sprint_id = match &args.sprint {
        Some(sprint) => project.sprint_id(sprint)?,
        None => taiga.current_sprint(project.id)?,
    };
    let stats = taiga.sprint_stats(sprint_id)?;

    println!(
        "{}  {} to {}",
        stats.name, stats.estimated_start, stats.estimated_finish
    );
    println!(
        "stories {}/{}   tasks {}/{}",
        stats.completed_userstories,
        stats.total_userstories,
        stats.completed_tasks,
        stats.total_tasks
    );
    println!();

    let points = |points: Option<f64>| points.map(|p| format!("{p:.1}")).unwrap_or_default();
    let mut table = Table::new();
    table.set_format(*FORMAT_CLEAN);
    table.add_row(row!["DAY", "OPEN", "OPTIMAL"]);
    for day in &stats.days {
        table.add_row(Row::new(vec![
            Cell::new(&day.day),
            Cell::new(&points(day.open_points)),
            Cell::new(&points(day.optimal_points)),
        ]));
    }
    table.printstd();
    Ok(())
}
