use reqwest::blocking::{Client, Response};
use reqwest::{Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use super::{Error, Result, Taiga};

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

    pub fn get<R>(&mut self, endpoint: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        Ok(self.send::<()>(Method::GET, endpoint, None)?.json()?)
    }

    pub fn post<T, R>(&mut self, endpoint: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        Ok(self.send(Method::POST, endpoint, Some(body))?.json()?)
    }

    pub fn patch<T, R>(&mut self, endpoint: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        Ok(self.send(Method::PATCH, endpoint, Some(body))?.json()?)
    }

    pub fn delete(&mut self, endpoint: &str) -> Result<()> {
        self.send::<()>(Method::DELETE, endpoint, None).map(|_| ())
    }
}
