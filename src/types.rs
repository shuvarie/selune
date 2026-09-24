use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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
    Chatgpt,
    Copilot,
    Cohere,
    Deepseek,
    Doubleword,
    Groq,
    Huggingface,
    Hyperbolic,
    Llamafile,
    Minimax,
    Mira,
    Mistral,
    Moonshot,
    Perplexity,
    Together,
    Venice,
    Voyageai,
    Xai,
    Xiaomimimo,
    Zai,
}

/// The inference provider identifier, mirroring Catwalk's `InferenceProvider`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InferenceProvider(pub String);

/// How a provider authenticates: a pasted API key or an OAuth2 device flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMethod {
    ApiKey,
    Oauth2Device,
}

/// An AI provider configuration, mirroring Catwalk's `Provider`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub id: InferenceProvider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthMethod>,
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

/// Context/output token limits for a model, mirroring OpenCode's `limit`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelLimit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<i64>,
}

/// Per-million-token costs for a model, mirroring OpenCode's `cost`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelCost {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<f64>,
}

/// A reasoning-effort/level option for a model, mirroring OpenCode's
/// `reasoning_options` entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningOption {
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
}

/// A model code identifying a model across providers: `org/model` or
/// `org/model:variant`. The org is the training organization's id (Hugging
/// Face or GitHub style, lowercase kebab-case); the model id is kebab-case,
/// case sensitive, dots allowed; the optional variant is lowercase kebab-case.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModelCode {
    org: String,
    model: String,
    variant: Option<String>,
}

impl ModelCode {
    /// Builds a model code from its parts, validating each one.
    pub fn new(org: &str, model: &str, variant: Option<&str>) -> Result<Self, ModelCodeError> {
        if !valid_kebab_id(org) {
            return Err(ModelCodeError::Org(org.to_string()));
        }
        if !valid_model_id(model) {
            return Err(ModelCodeError::Model(model.to_string()));
        }
        if let Some(variant) = variant
            && !valid_kebab_id(variant)
        {
            return Err(ModelCodeError::Variant(variant.to_string()));
        }
        Ok(Self {
            org: org.to_string(),
            model: model.to_string(),
            variant: variant.map(str::to_string),
        })
    }

    /// The training organization id (lowercase kebab-case).
    pub fn org(&self) -> &str {
        &self.org
    }

    /// The model id (case sensitive kebab-case).
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The optional serving variant (lowercase kebab-case).
    pub fn variant(&self) -> Option<&str> {
        self.variant.as_deref()
    }

    fn parse(code: &str) -> Result<Self, ModelCodeError> {
        if code.matches(':').count() > 1 {
            return Err(ModelCodeError::Format(code.to_string()));
        }
        let (body, variant) = match code.split_once(':') {
            Some((body, variant)) => (body, Some(variant)),
            None => (code, None),
        };
        let (org, model) = body
            .split_once('/')
            .ok_or_else(|| ModelCodeError::Format(code.to_string()))?;
        if !valid_kebab_id(org) {
            return Err(ModelCodeError::Org(org.to_string()));
        }
        if !valid_model_id(model) {
            return Err(ModelCodeError::Model(model.to_string()));
        }
        if let Some(variant) = variant
            && !valid_kebab_id(variant)
        {
            return Err(ModelCodeError::Variant(variant.to_string()));
        }
        Ok(Self {
            org: org.to_string(),
            model: model.to_string(),
            variant: variant.map(str::to_string),
        })
    }
}

impl fmt::Display for ModelCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant {
            Some(variant) => write!(f, "{}/{}:{}", self.org, self.model, variant),
            None => write!(f, "{}/{}", self.org, self.model),
        }
    }
}

impl FromStr for ModelCode {
    type Err = ModelCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ModelCode::parse(s)
    }
}

impl TryFrom<String> for ModelCode {
    type Error = ModelCodeError;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        ModelCode::parse(&code)
    }
}

impl From<ModelCode> for String {
    fn from(code: ModelCode) -> String {
        code.to_string()
    }
}

/// Why a model code string failed to parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelCodeError {
    #[error("invalid model code {0:?}: expected `org/model` or `org/model:variant`")]
    Format(String),
    #[error("invalid org id {0:?} in model code: must be lowercase kebab-case")]
    Org(String),
    #[error(
        "invalid model id {0:?} in model code: must be kebab-case, case sensitive, dots allowed"
    )]
    Model(String),
    #[error("invalid variant {0:?} in model code: must be lowercase kebab-case")]
    Variant(String),
}

fn valid_kebab_id(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn valid_model_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .split(['-', '.'])
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// An AI model configuration, mirroring OpenCode's model schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(rename = "modelCode")]
    pub model_code: ModelCode,
    pub name: String,
    pub reasoning: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasoning_options: Vec<ReasoningOption>,
    pub attachment: bool,
    pub limit: ModelLimit,
    pub cost: ModelCost,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
}

impl Provider {
    /// Whether sign-in for this provider runs the OAuth2 device flow instead
    /// of asking for an API key.
    pub fn oauth_device_login(&self) -> bool {
        self.auth
            .as_ref()
            .is_some_and(|a| matches!(a, AuthMethod::Oauth2Device))
    }

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

    /// Look up a model by model code.
    pub fn model_by_code(&self, code: &ModelCode) -> Option<&Model> {
        self.models.iter().find(|m| &m.model_code == code)
    }
}
