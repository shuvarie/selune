pub mod client;
pub mod embedded;
pub mod types;

#[cfg(test)]
mod tests;

pub use chrono_tz::Tz;
pub use client::{Client, ClientError, DEFAULT_URL};
pub use types::{
    AuthMethod, Dialect, InferenceProvider, Model, ModelCode, ModelCodeError, ModelCost,
    ModelLimit, ModelOptions, PeakExemption, PeakPeriod, PeakPeriodError, Peaks, Provider,
    ProviderType, ReasoningOption, TimeOfDay, TimeOfDayError,
};
