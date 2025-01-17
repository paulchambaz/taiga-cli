mod cli;
mod taiga;
mod utils;

use anyhow::Result;
use std::process::exit;
use taiga::Taiga;

use cli::{parse_args, TaigaCmd};

fn main() -> Result<()> {
    // Step 1: Initialize taiga, either from cache or fresh auth
    let mut taiga = match Taiga::from_cache() {
        Some(taiga) => {
            println!("Loaded configuration from cache");
            println!("User ID: {}", taiga.id);
            println!("Auth token: {}", taiga.auth_token);
            println!("Username: {}", taiga.username);
            taiga
        }
        _ => {
            println!("No cached configuration found, please authenticate:");
            Taiga::auth(None)?
        }
    };

    let cmd = parse_args(&Some(taiga));
    println!("{:?}", cmd);

    // // Step 2: Print projects from cache
    // println!("\nCached Projects:");
    // for project in &taiga.projects {
    //     println!("{}", project.name);
    // }
    //
    // let project = taiga.projects.get(2).expect("failed to get project");
    // println!("{:?}", project);
    //
    // if let Ok(project) = taiga.get_project(project.id) {
    //     println!("{:?}", project);
    // }

    // for project in &taiga.projects {
    //     println!("- {} (ID: {})", project.name, project.id);
    // }

    // Step 3: Print parsing message
    // println!("\nParsing CLI...");

    // Step 4: Make a sample API call - let's get tasks from the first project
    // if let Some(first_project) = taiga.projects.get(2) {
    //     println!("\nFetching tasks for project: {}", first_project.name);
    //
    //     // Example task list endpoint
    //     #[derive(serde::Deserialize, Debug)]
    //     struct TaskResponse {
    //         subject: String,
    //         status: String,
    //         #[serde(rename = "ref")]
    //         reference: i32,
    //     }
    //
    //     match taiga.get::<Vec<TaskResponse>>(&format!("/tasks?project={}", first_project.id)) {
    //         Ok(tasks) => {
    //             println!("\nTasks:");
    //             for task in tasks {
    //                 println!("#{} - [{}] {}", task.reference, task.status, task.subject);
    //             }
    //         },
    //         Err(e) => eprintln!("Failed to fetch tasks: {}", e),
    //     }
    // }

    // Step 5: Finish
    // println!("\nDone!");
    Ok(())
}
