use std::{
    io::{self, stdin, Write},
    ops::Add,
    process::exit,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
};
use rpassword::read_password;
use serde::{Deserialize, Serialize};

use crate::taiga::project::TaigaProject;

use super::taiga::Taiga;

impl Taiga {
    /// Authenticate to taiga and get a connection
    ///
    /// Args:
    /// - `url`: the url of the taiga instance (optional)
    pub fn auth(url: Option<String>) -> Result<()> {
        // Creating the client for the client
        let client = Client::new();

        // get username, password and url
        let (username, password) = Taiga::request_info();
        let url = url.unwrap_or("https://api.taiga.io/api/v1".to_string());

        // creating body for the auth request
        #[derive(Serialize, Debug)]
        struct Request {
            username: String,
            password: String,
            #[serde(rename = "type")]
            type_field: String,
        }

        let body = Request {
            username,
            password,
            type_field: "normal".to_string(),
        };

        // creating headers for the auth request
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // sending the auth request - in which we authenticate to taiga api and get our token
        let response = client
            .post(format!("{}/auth", url))
            .headers(headers)
            .json(&body)
            .send()?;
        let text = response.text()?;

        // parsing the auth response
        #[derive(Deserialize, Debug)]
        struct Response {
            auth_token: String,
            refresh: String,
            id: i32,
        }

        let res: Response = serde_json::from_str(&text)?;
        let auth_token = res.auth_token;
        let refresh = res.refresh;
        let id = res.id;
        let refresh_time = SystemTime::now().add(Duration::from_secs(3000));

        // creating the headers for the projects request
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token)).expect("Could not get token"),
        );

        // sending the project request - in which we get the list of projects for the user
        let response = client
            .get(format!("{}/projects?member={}", url, id))
            .headers(headers)
            .send()?;
        let text = response.text()?;

        // parsing the projects response
        #[derive(Deserialize, Debug)]
        pub struct Project {
            pub id: i32,
            pub name: String,
        }

        let res: Vec<Project> = serde_json::from_str(&text)?;

        let projects: Vec<TaigaProject> = res
            .iter()
            .map(|project| TaigaProject {
                id: project.id,
                name: project.name.clone(),
                members: Vec::new(),
                statuses: Vec::new(),
            })
            .collect();

        // creating the taiga object and saving it to the cache file
        let taiga = Taiga {
            auth_token,
            refresh,
            refresh_time,
            url,
            id,
            projects,
        };
        taiga.save_cache();

        Ok(())
    }

    /// Request username and password to stdin
    ///
    /// Returns:
    /// A tuple of username and password
    fn request_info() -> (String, String) {
        print!("Username: ");
        io::stdout().flush().unwrap_or_else(|err| {
            eprintln!("Error, could not flush stdout: '{}'", err);
            exit(1)
        });
        let mut username = String::new();
        stdin().read_line(&mut username).unwrap_or_else(|err| {
            eprintln!("Error, could not read line: '{}'", err);
            exit(1);
        });
        username = username.trim().to_string();

        print!("Password: ");
        io::stdout().flush().unwrap_or_else(|err| {
            eprintln!("Error, could not flush stdout: '{}'", err);
            exit(1)
        });
        let password = read_password().unwrap_or_else(|err| {
            eprintln!("Error, could not read password: '{}'", err);
            exit(1);
        });

        (username, password)
    }

    /// Refresh the taiga connection
    ///
    /// Args:
    /// - `self`: a taiga struct
    pub fn refresh(&mut self) -> Result<()> {
        // creating the client for the request
        let client = Client::new();

        // creating the body for the refresh request
        #[derive(Serialize, Debug)]
        struct Request {
            refresh: String,
        }

        let body = Request {
            refresh: self.refresh.clone(),
        };

        // creating the headers for the refresh
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // sending the refresh request - in which we refresh the auth token with our refresh token
        let response = client
            .post(format!("{}/auth/request", self.url))
            .json(&body)
            .send()?;
        let text = response.text()?;

        // parsing the refresh response
        #[derive(Deserialize, Debug)]
        struct Response {
            auth_token: String,
            refresh: String,
        }

        let res: Response = serde_json::from_str(&text)?;

        // updating the taiga connection
        self.auth_token = res.auth_token;
        self.refresh = res.refresh;
        self.refresh_time = SystemTime::now().add(Duration::from_secs(3000));
        self.save_cache();

        Ok(())
    }
}
