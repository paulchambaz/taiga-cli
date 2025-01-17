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
    pub fn auth(url: Option<String>, app_id: String) -> Result<()> {
        let (username, password) = Taiga::request_info();
        let url = url.unwrap_or("https://api.taiga.io/api/v1".to_string());
        
        // First get a regular auth token
        let client = Client::new();
        let auth_body = AuthRequest {
            username,
            password,
            type_field: "normal".to_string(),
        };
        let response = client.post(format!("{}/auth", url))
            .json(&auth_body)
            .send()?;
        let auth_response: AuthResponse = response.json()?;
        let temp_auth_token = auth_response.auth_token;
        let id = auth_response.id;

        // Generate a random state string
        let state = uuid::Uuid::new_v4().to_string();

        // Request an application token
        let app_token_request = ApplicationTokenRequest {
            application: app_id,
            state: state.clone(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", temp_auth_token))?
        );

        let response = client.post(format!("{}/application-tokens/authorize", url))
            .headers(headers.clone())
            .json(&app_token_request)
            .send()?;
        
        let token_response: ApplicationTokenResponse = response.json()?;
        
        // Verify state matches
        if token_response.state != state {
            return Err(anyhow!("State mismatch in token response"));
        }

        // Validate the auth code
        let validate_request = serde_json::json!({
            "application": app_id,
            "auth_code": token_response.auth_code,
            "state": state
        });

        let response = client.post(format!("{}/application-tokens/validate", url))
            .headers(headers)
            .json(&validate_request)
            .send()?;

        let validate_response: ValidateTokenResponse = response.json()?;
        
        // In a real implementation, you would decrypt the cyphered_token here
        // using the application's secret key. For now, we'll store it directly
        let app_token = validate_response.cyphered_token;

        // Save to cache with application token
        let taiga = Taiga {
            auth_token: app_token,
            url,
            id,
            projects: vec![],
            token_type: "Application".to_string(),
        };
        taiga.save_cache();
        
        Ok(())
    }

    pub fn get_request(&self, endpoint: &str) -> reqwest::RequestBuilder {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("{} {}", 
                self.token_type,  // "Application" or "Bearer"
                self.auth_token
            )).expect("Invalid token format")
        );

        client.get(format!("{}{}", self.url, endpoint))
            .headers(headers)
    }
}
