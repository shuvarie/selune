use serde::{Deserialize, Serialize};

/// The type of AI provider, mirroring Catwalk's `Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderType {
    Openai,
    OpenaiCompat,
    Openrouter,
    Vercel,
    Anthropic,
    Google,
    Azure,
    Bedrock,
    GoogleVertex,
    Ollama,
}

/// The inference provider identifier, mirroring Catwalk's `InferenceProvider`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InferenceProvider(pub String);

/// An AI provider configuration, mirroring Catwalk's `Provider`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub id: InferenceProvider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ProviderType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_large_model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_small_model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<Model>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_headers: Option<std::collections::HashMap<String, String>>,
}

/// Extra per-model options, mirroring Catwalk's `ModelOptions`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<serde_json::Map<String, serde_json::Value>>,
}

/// An AI model configuration, mirroring Catwalk's `Model`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub cost_per_1m_in: f64,
    pub cost_per_1m_out: f64,
    pub cost_per_1m_in_cached: f64,
    pub cost_per_1m_out_cached: f64,
    pub context_window: i64,
    pub default_max_tokens: i64,
    pub can_reason: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasoning_levels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_reasoning_effort: Option<String>,
    pub supports_attachments: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
}

impl Provider {
    /// The default large model id, or the first model's id when unset.
    pub fn default_large_model(&self) -> Option<&str> {
        self.default_large_model_id
            .as_deref()
            .or_else(|| self.models.first().map(|m| m.id.as_str()))
    }

    /// The default small model id, or the last model's id when unset.
    pub fn default_small_model(&self) -> Option<&str> {
        self.default_small_model_id
            .as_deref()
            .or_else(|| self.models.last().map(|m| m.id.as_str()))
    }

    /// Look up a model by id.
    pub fn model(&self, id: &str) -> Option<&Model> {
        self.models.iter().find(|m| m.id == id)
    }
}
