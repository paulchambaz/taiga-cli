//! Authenticated requests to the Taiga API, with one retry after the session expires.

use reqwest::blocking::{Client, Response};
use reqwest::{Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use super::{Error, Result, Taiga};

/// Turns an unsuccessful answer into an error carrying the server's own explanation.
///
/// # Errors
/// Returns an error when the status is not a success.
pub(super) fn check(response: Response) -> Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let url = response.url().to_string();
    let body = response.text()?;
    Err(Error::Api { url, status, body })
}

impl Taiga {
    fn attempt<T>(&self, method: &Method, endpoint: &str, body: Option<&T>) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        let request = Client::new()
            .request(method.clone(), format!("{}{}", self.url, endpoint))
            .bearer_auth(&self.auth_token)
            .header("x-disable-pagination", "True");

        let request = match body {
            Some(body) => request.json(body),
            None => request,
        };

        Ok(request.send()?)
    }

    fn send<T>(&mut self, method: Method, endpoint: &str, body: Option<&T>) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        let response = self.attempt(&method, endpoint, body)?;
        if response.status() != StatusCode::UNAUTHORIZED {
            return check(response);
        }

        if self.refresh().is_err() {
            self.reauth()?;
        }
        check(self.attempt(&method, endpoint, body)?)
    }

    /// Fetches once, without refreshing the token, so it can run from several threads.
    ///
    /// # Errors
    /// Returns an error when the request fails or the token is no longer valid.
    pub fn get_once<R>(&self, endpoint: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        Ok(check(self.attempt::<()>(&Method::GET, endpoint, None)?)?.json()?)
    }

    /// Fetches and decodes a resource, renewing the session once if it expired.
    ///
    /// # Errors
    /// Returns an error when the request fails or the answer cannot be decoded.
    pub fn get<R>(&mut self, endpoint: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        Ok(self.send::<()>(Method::GET, endpoint, None)?.json()?)
    }

    /// Creates a resource and decodes the answer.
    ///
    /// # Errors
    /// Returns an error when the request fails or the answer cannot be decoded.
    pub fn post<T, R>(&mut self, endpoint: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        Ok(self.send(Method::POST, endpoint, Some(body))?.json()?)
    }

    /// Updates a resource and decodes the answer.
    ///
    /// # Errors
    /// Returns an error when the request fails or the answer cannot be decoded.
    pub fn patch<T, R>(&mut self, endpoint: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        Ok(self.send(Method::PATCH, endpoint, Some(body))?.json()?)
    }

    /// Deletes a resource.
    ///
    /// # Errors
    /// Returns an error when the request fails.
    pub fn delete(&mut self, endpoint: &str) -> Result<()> {
        self.send::<()>(Method::DELETE, endpoint, None).map(|_| ())
    }
}
