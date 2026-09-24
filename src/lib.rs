pub mod client;
pub mod embedded;
pub mod types;

#[cfg(test)]
mod tests;

pub use client::{Client, ClientError, DEFAULT_URL};
pub use types::{
    AuthMethod, InferenceProvider, Model, ModelCode, ModelCodeError, ModelCost, ModelLimit,
    ModelOptions, Provider, ProviderType, ReasoningOption,
};
