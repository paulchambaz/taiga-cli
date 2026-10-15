//! The commands that list projects and their members.

use anyhow::Result;

use crate::cli::ProjectUserArgs;
use crate::taiga::Taiga;

/// Prints the cached project names.
pub fn list(taiga: &Taiga) {
    for project in &taiga.projects {
        println!("{}", project.name);
    }
}

/// Refreshes the project list from Taiga and prints it.
///
/// # Errors
/// Returns an error when the list cannot be fetched or cached.
pub fn refresh(taiga: &mut Taiga) -> Result<()> {
    taiga.update_projects()?;
    list(taiga);
    Ok(())
}

/// Prints the members of a project, refreshed from Taiga.
///
/// # Errors
/// Returns an error when the project is unknown or the refresh fails.
pub fn users(taiga: &mut Taiga, args: ProjectUserArgs) -> Result<()> {
    let project = taiga.load_project(&args.project, |_| true)?;

    for user in &project.members {
        println!("{}", user.username);
    }
    Ok(())
}
