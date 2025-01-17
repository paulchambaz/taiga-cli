use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::io::{self, stdin, Write};
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Debug)]
pub struct AuthRequest {
    username: String,
    password: String,
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub auth_token: String,
    pub refresh: String,
    pub id: i32,
}

use super::taiga::Taiga;

impl Taiga {
    pub fn request_credentials() -> (String, String) {
        print!("Username: ");
        io::stdout().flush().unwrap();
        let mut username = String::new();
        stdin().read_line(&mut username).unwrap();

        print!("Password: ");
        io::stdout().flush().unwrap();
        let password = read_password().unwrap();

        (username.trim().to_string(), password)
    }

    pub fn auth(url: Option<String>) -> Result<Self> {
        let (username, password) = Self::request_credentials();
        let base_url = url.unwrap_or("https://api.taiga.io/api/v1".to_string());

        let client = Client::new();
        let auth_request = AuthRequest {
            username: username.clone(),
            password: password.clone(),
            auth_type: "normal".to_string(),
        };

        let response = client
            .post(format!("{}/auth", base_url))
            .header(CONTENT_TYPE, "application/json")
            .json(&auth_request)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Authentication failed. Please check your credentials."
            ));
        }

        let auth_response: AuthResponse = response.json()?;

        let mut taiga = Taiga {
            auth_token: auth_response.auth_token,
            url: base_url,
            id: auth_response.id,
            refresh: auth_response.refresh,
            refresh_time: SystemTime::now().add(Duration::from_secs(24 * 60 * 60)),
            username,
            password,
            projects: vec![],
        };

        match taiga.get_projects() {
            Ok(fetched_projects) => {
                taiga.projects = fetched_projects;
            }
            Err(e) => {
                return Err(anyhow!(
                    "Authentication successful but failed to fetch projects: {}",
                    e
                ));
            }
        }

        taiga.save_cache()?;
        Ok(taiga)
    }

    // Token refresh
    pub fn refresh(&mut self) -> Result<()> {
        let client = Client::new();
        let refresh_request = serde_json::json!({
            "refresh": self.refresh
        });

        let response = client
            .post(format!("{}/auth/refresh", self.url))
            .header(CONTENT_TYPE, "application/json")
            .json(&refresh_request)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to refresh token"));
        }

        let refresh_response: AuthResponse = response.json()?;

        self.auth_token = refresh_response.auth_token;
        self.refresh = refresh_response.refresh;
        self.refresh_time = SystemTime::now().add(Duration::from_secs(24 * 60 * 60));

        self.save_cache()?;
        Ok(())
    }

    // Full reauthorization using stored credentials
    pub fn reauth(&mut self) -> Result<()> {
        let client = Client::new();
        let auth_request = AuthRequest {
            username: self.username.clone(),
            password: self.password.clone(),
            auth_type: "normal".to_string(),
        };

        let response = client
            .post(format!("{}/auth", self.url))
            .header(CONTENT_TYPE, "application/json")
            .json(&auth_request)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!("Reauthorization failed"));
        }

        let auth_response: AuthResponse = response.json()?;

        self.auth_token = auth_response.auth_token;
        self.refresh = auth_response.refresh;
        self.refresh_time = SystemTime::now().add(Duration::from_secs(24 * 60 * 60));

        self.save_cache()?;
        Ok(())
    }
}
