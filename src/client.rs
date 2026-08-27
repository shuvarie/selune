use crate::types::Provider;
use reqwest::blocking::Client as HttpClient;
use thiserror::Error;

/// Default URL for the catalog service. The user hosts the configs
/// themselves; this is the default the client falls back to.
pub const DEFAULT_URL: &str = "https://selune.shuvarie.org/v1/providers.json";

/// Error returned when the catalog service cannot be reached or parsed.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to build request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("unexpected status code: {0}")]
    Status(u16),
    #[error("failed to decode response: {0}")]
    Decode(#[from] serde_json::Error),
}

/// A client for the catalog service, mirroring Catwalk's `Client`.
pub struct Client {
    url: String,
    http: HttpClient,
}

impl Client {
    /// Create a client using the `CATALOG_URL` environment variable, falling
    /// back to [`DEFAULT_URL`].
    pub fn new() -> Self {
        let url = std::env::var("CATALOG_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
        Self::new_with_url(&url)
    }

    /// Create a client with a specific catalog URL.
    pub fn new_with_url(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            http: HttpClient::new(),
        }
    }

    /// Set the catalog URL.
    pub fn set_url(&mut self, url: &str) {
        self.url = url.trim_end_matches('/').to_string();
    }

    /// Retrieve all available providers from the service.
    pub fn get_providers(&self) -> Result<Vec<Provider>, ClientError> {
        let response = self.http.get(&self.url).send()?;
        if response.status() != 200 {
            return Err(ClientError::Status(response.status().as_u16()));
        }
        Ok(response.json()?)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
