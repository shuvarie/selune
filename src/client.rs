use crate::types::Provider;
use reqwest::blocking::Client as HttpClient;
use thiserror::Error;

/// Placeholder base URL for the catalog service. The user hosts the configs
/// themselves; this is the default the client falls back to.
pub const DEFAULT_URL: &str = "https://catalog.example.invalid";

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
    base_url: String,
    http: HttpClient,
}

impl Client {
    /// Create a client using the `CATALOG_URL` environment variable, falling
    /// back to [`DEFAULT_URL`].
    pub fn new() -> Self {
        let url = std::env::var("CATALOG_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
        Self::new_with_url(&url)
    }

    /// Create a client with a specific base URL.
    pub fn new_with_url(url: &str) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            http: HttpClient::new(),
        }
    }

    /// Retrieve all available providers from the service.
    pub fn get_providers(&self) -> Result<Vec<Provider>, ClientError> {
        let url = format!("{}/v2/providers", self.base_url);
        let response = self.http.get(&url).send()?;
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
