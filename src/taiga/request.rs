use anyhow::{anyhow, Result};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Serialize};

use super::Taiga;

impl Taiga {
    fn send<T>(&mut self, mut builder: RequestBuilder, body: Option<&T>) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        let response = self.execute_request(builder.try_clone().unwrap(), body)?;

        if response.status().is_success() {
            return Ok(response);
        }

        if let Ok(()) = self.refresh() {
            builder = self.add_auth_header(builder.try_clone().unwrap())?;
            let response = self.execute_request(builder.try_clone().unwrap(), body)?;

            if response.status().is_success() {
                return Ok(response);
            }
        }

        self.reauth()?;
        builder = self.add_auth_header(builder.try_clone().unwrap())?;
        let response = self.execute_request(builder, body)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(anyhow!("Request failed after all retry attempts"))
        }
    }

    fn request<T, R>(&mut self, builder: RequestBuilder, body: Option<&T>) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        self.send(builder, body)?.json().map_err(|e| anyhow!(e))
    }

    fn request_no_content(&mut self, builder: RequestBuilder) -> Result<()> {
        self.send::<()>(builder, None).map(|_| ())
    }

    fn execute_request<T>(&self, builder: RequestBuilder, body: Option<&T>) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        let request = if let Some(body) = body {
            builder.json(body)
        } else {
            builder
        };

        request.send().map_err(|e| anyhow!(e))
    }

    fn add_auth_header(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        Ok(builder.header(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.auth_token))?,
        ))
    }

    pub fn get<R>(&mut self, endpoint: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let client = Client::new();
        let builder = client
            .get(format!("{}{}", self.url, endpoint))
            .header(CONTENT_TYPE, "application/json")
            .header("x-disable-pagination", "True");
        let builder = self.add_auth_header(builder)?;
        self.request::<(), R>(builder, None)
    }

    pub fn post<T, R>(&mut self, endpoint: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let client = Client::new();
        let builder = client
            .post(format!("{}{}", self.url, endpoint))
            .header(CONTENT_TYPE, "application/json");
        let builder = self.add_auth_header(builder)?;

        self.request::<T, R>(builder, Some(body))
    }

    pub fn patch<T, R>(&mut self, endpoint: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let client = Client::new();
        let builder = client
            .patch(format!("{}{}", self.url, endpoint))
            .header(CONTENT_TYPE, "application/json");
        let builder = self.add_auth_header(builder)?;

        self.request::<T, R>(builder, Some(body))
    }

    pub fn delete(&mut self, endpoint: &str) -> Result<()> {
        let client = Client::new();
        let builder = client
            .delete(format!("{}{}", self.url, endpoint))
            .header(CONTENT_TYPE, "application/json");
        let builder = self.add_auth_header(builder)?;

        self.request_no_content(builder)
    }
}
