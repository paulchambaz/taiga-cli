//! Logging in and keeping the session tokens fresh.

use reqwest::blocking::Client;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::io::{self, stdin, Write};

use super::request::check;
use super::{Result, Taiga};

#[derive(Serialize, Debug)]
struct AuthRequest {
    username: String,
    password: String,
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Deserialize, Debug)]
struct AuthResponse {
    auth_token: String,
    refresh: String,
    id: i32,
}

#[derive(Deserialize, Debug)]
struct RefreshResponse {
    auth_token: String,
    refresh: String,
}

impl Taiga {
    fn request_credentials() -> Result<(String, String)> {
        print!("Username: ");
        io::stdout().flush()?;
        let mut username = String::new();
        stdin().read_line(&mut username)?;

        print!("Password: ");
        io::stdout().flush()?;
        let password = read_password()?;

        Ok((username.trim().to_string(), password))
    }

    /// Asks for credentials, logs in and caches the new session and project list.
    ///
    /// The official server is used unless `url` names another one.
    ///
    /// # Errors
    /// Returns an error when the terminal, the server or the cache fails.
    pub fn auth(url: Option<String>) -> Result<Self> {
        let (username, password) = Self::request_credentials()?;
        let base_url = url.unwrap_or("https://api.taiga.io/api/v1".to_string());

        let auth_response = Self::login(&base_url, &username, &password)?;

        let mut taiga = Taiga {
            auth_token: auth_response.auth_token,
            url: base_url,
            id: auth_response.id,
            refresh: auth_response.refresh,
            username,
            password,
            projects: vec![],
        };

        taiga.update_projects()?;
        Ok(taiga)
    }

    /// Renews the tokens with the refresh token.
    ///
    /// # Errors
    /// Returns an error when the refresh token is no longer accepted.
    pub(super) fn refresh(&mut self) -> Result<()> {
        let response = Client::new()
            .post(format!("{}/auth/refresh", self.url))
            .json(&serde_json::json!({ "refresh": self.refresh }))
            .send()?;

        let tokens: RefreshResponse = check(response)?.json()?;
        self.set_tokens(tokens.auth_token, tokens.refresh)
    }

    /// Logs in again with the stored credentials.
    ///
    /// # Errors
    /// Returns an error when the credentials are no longer accepted.
    pub(super) fn reauth(&mut self) -> Result<()> {
        let tokens = Self::login(&self.url, &self.username, &self.password)?;
        self.set_tokens(tokens.auth_token, tokens.refresh)
    }

    fn login(url: &str, username: &str, password: &str) -> Result<AuthResponse> {
        let request = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
            auth_type: "normal".to_string(),
        };

        let response = Client::new()
            .post(format!("{url}/auth"))
            .json(&request)
            .send()?;

        Ok(check(response)?.json()?)
    }

    fn set_tokens(&mut self, auth_token: String, refresh: String) -> Result<()> {
        self.auth_token = auth_token;
        self.refresh = refresh;
        self.save_cache()
    }
}
