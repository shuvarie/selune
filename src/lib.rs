pub mod client;
pub mod embedded;
pub mod types;

#[cfg(test)]
mod tests;

pub use client::{Client, ClientError, DEFAULT_URL};
pub use types::{InferenceProvider, Model, ModelOptions, Provider, ProviderType};
