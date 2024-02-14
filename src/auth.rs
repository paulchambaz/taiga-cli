use crate::project::TaigaProject;
use crate::{project::Projects, Taiga};
use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::io::{self, stdin, Write};
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Debug)]
struct AuthRequest {
    username: String,
    password: String,
    #[serde(rename = "type")]
    type_field: String,
}

#[derive(Deserialize, Debug)]
struct AuthResponse {
    auth_token: String,
    refresh: String,
    id: i32,
}

#[derive(Serialize, Debug)]
struct RefreshRequest {
    refresh: String,
}

#[derive(Deserialize, Debug)]
struct RefreshResponse {
    auth_token: String,
    refresh: String,
}

impl Taiga {
    fn request_info() -> (String, String) {
        print!("Username: ");
        io::stdout().flush().expect("Could not flush terminal");
        let mut username = String::new();
        stdin()
            .read_line(&mut username)
            .expect("Could not read username");
        username = username.trim().to_string();

        print!("Password: ");
        io::stdout().flush().expect("Could not flush terminal");
        let password = read_password().expect("Could not read password");

        (username, password)
    }

    pub fn auth(url: Option<String>) -> Result<()> {
        let (username, password) = Taiga::request_info();

        let url = url.unwrap_or("https://api.taiga.io/api/v1".to_string());

        // get auth
        let client = Client::new();

        let auth_body = AuthRequest {
            username,
            password,
            type_field: "normal".to_string(),
        };

        let request = client.post(format!("{}/auth", url)).json(&auth_body);
        let response = request.send()?;
        let text = response.text()?;

        let auth_response: AuthResponse = serde_json::from_str(&text)?;

        let auth_token = auth_response.auth_token;
        let refresh = auth_response.refresh;
        let id = auth_response.id;
        let refresh_time = SystemTime::now().add(Duration::from_secs(3000));

        // get projects
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token)).expect("Could not get token"),
        );

        let request = client
            .get(format!("{}/projects?member={}", url, id))
            .headers(headers);
        let response = request.send()?;
        let text = response.text()?;

        let projects_response: Vec<Projects> = serde_json::from_str(&text)?;

        let projects = projects_response
            .iter()
            .map(|project| TaigaProject {
                id: project.id,
                name: project.name.clone(),
                members: Vec::new(),
                statuses: Vec::new(),
            })
            .collect();

        // save to cache
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

    pub fn refresh(&mut self) -> Result<()> {
        let client = Client::new();

        let refresh_body = RefreshRequest {
            refresh: self.refresh.clone(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let request = client
            .post(format!("{}/auth/refresh", self.url))
            .json(&refresh_body);
        let response = request.send()?;
        let text = response.text()?;

        let request_response: RefreshResponse = serde_json::from_str(&text)?;

        self.auth_token = request_response.auth_token;
        self.refresh = request_response.refresh;
        self.refresh_time = SystemTime::now().add(Duration::from_secs(3000));

        self.save_cache();

        Ok(())
    }
}
