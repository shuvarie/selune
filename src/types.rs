use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use holidays2::Country;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The protocol `kind` of an AI provider, mirroring Catwalk's `Type`.
/// Vendor flavors that are dialects of `openai-compat` are not separate
/// kinds here; they are carried on `Provider::dialect` instead.
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
    Llamafile,
    Voyageai,
}

/// The vendor dialect of a provider's wire protocol. These kinds are all
/// variants of the OpenAI-compatible API, so the provider's `type` stays
/// `openai-compat` and `dialect` selects the vendor flavor; a missing
/// dialect is the base OpenAI dialect. (Selune extension, not Catwalk.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dialect {
    Deepseek,
    Doubleword,
    Groq,
    Huggingface,
    Hyperbolic,
    Minimax,
    Mira,
    Mistral,
    Moonshot,
    Perplexity,
    Together,
    Venice,
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
    pub dialect: Option<Dialect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peaks: Option<Peaks>,
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

/// A time of day with minute resolution, written `HH:MM`. Times are
/// interpreted in the timezone of the schedule that contains them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TimeOfDay {
    minutes: u16,
}

impl TimeOfDay {
    /// Builds a time of day from an hour and a minute.
    pub fn new(hour: u8, minute: u8) -> Result<Self, TimeOfDayError> {
        if hour >= 24 || minute >= 60 {
            return Err(TimeOfDayError(format!("{hour:02}:{minute:02}")));
        }
        Ok(Self {
            minutes: u16::from(hour) * 60 + u16::from(minute),
        })
    }

    /// The hour of the day (0..=23).
    pub fn hour(&self) -> u8 {
        (self.minutes / 60) as u8
    }

    /// The minute of the hour (0..=59).
    pub fn minute(&self) -> u8 {
        (self.minutes % 60) as u8
    }

    /// The number of minutes elapsed since midnight (0..1440).
    pub fn minutes_since_midnight(&self) -> u16 {
        self.minutes
    }

    fn from_naive_time(time: NaiveTime) -> Self {
        Self {
            minutes: (time.hour() * 60 + time.minute()) as u16,
        }
    }

    fn parse(value: &str) -> Result<Self, TimeOfDayError> {
        let invalid = || TimeOfDayError(value.to_string());
        let bytes = value.as_bytes();
        if bytes.len() != 5 || bytes[2] != b':' {
            return Err(invalid());
        }
        if !bytes[..2]
            .iter()
            .chain(&bytes[3..])
            .all(|b| b.is_ascii_digit())
        {
            return Err(invalid());
        }
        let hour = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
        let minute = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
        Self::new(hour, minute).map_err(|_| invalid())
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour(), self.minute())
    }
}

impl FromStr for TimeOfDay {
    type Err = TimeOfDayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TimeOfDay::parse(s)
    }
}

impl TryFrom<String> for TimeOfDay {
    type Error = TimeOfDayError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TimeOfDay::parse(&value)
    }
}

impl From<TimeOfDay> for String {
    fn from(time: TimeOfDay) -> String {
        time.to_string()
    }
}

/// Why a time-of-day string failed to parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid time {0:?}: expected `HH:MM` between 00:00 and 23:59")]
pub struct TimeOfDayError(String);

/// A peak window within a day of the schedule's timezone. `start` is inclusive
/// and `end` is exclusive; when `end` is earlier than `start` the window wraps
/// past midnight (for example `23:00`–`01:00`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "PeakPeriodDef", into = "PeakPeriodDef")]
pub struct PeakPeriod {
    start: TimeOfDay,
    end: TimeOfDay,
}

impl PeakPeriod {
    /// Builds a peak window. The window must span a non-zero amount of time.
    pub fn new(start: TimeOfDay, end: TimeOfDay) -> Result<Self, PeakPeriodError> {
        if start == end {
            return Err(PeakPeriodError);
        }
        Ok(Self { start, end })
    }

    /// The inclusive start of the window.
    pub fn start(&self) -> TimeOfDay {
        self.start
    }

    /// The exclusive end of the window.
    pub fn end(&self) -> TimeOfDay {
        self.end
    }

    /// Whether the window ends on the following day.
    pub fn crosses_midnight(&self) -> bool {
        self.end < self.start
    }

    /// Whether `time` falls inside the window.
    pub fn contains(&self, time: TimeOfDay) -> bool {
        let time = time.minutes;
        let start = self.start.minutes;
        let end = self.end.minutes;
        if start < end {
            (start..end).contains(&time)
        } else {
            time >= start || time < end
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PeakPeriodDef {
    start: TimeOfDay,
    end: TimeOfDay,
}

impl TryFrom<PeakPeriodDef> for PeakPeriod {
    type Error = PeakPeriodError;

    fn try_from(def: PeakPeriodDef) -> Result<Self, Self::Error> {
        PeakPeriod::new(def.start, def.end)
    }
}

impl From<PeakPeriod> for PeakPeriodDef {
    fn from(period: PeakPeriod) -> Self {
        Self {
            start: period.start,
            end: period.end,
        }
    }
}

/// Why a peak window is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid peak period: `start` and `end` must differ")]
pub struct PeakPeriodError;

/// A day class on which a provider runs no peak/off-peak schedule at all.
/// Exemptions are evaluated against the weekday and date in the schedule's
/// timezone. Holiday data covers 2001 through 2030; dates outside that range
/// are treated as non-holidays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PeakExemption {
    /// Saturdays and Sundays.
    Weekends,
    /// Chinese public holidays.
    HolidaysCn,
    /// US public holidays.
    HolidaysUs,
}

impl PeakExemption {
    /// Whether this exemption applies on `date`, a civil date in the
    /// schedule's timezone.
    pub fn applies(&self, date: NaiveDate) -> bool {
        match self {
            Self::Weekends => matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
            Self::HolidaysCn => is_holiday(Country::CN, date),
            Self::HolidaysUs => is_holiday(Country::US, date),
        }
    }
}

fn is_holiday(country: Country, date: NaiveDate) -> bool {
    holidays2::contains(country, date).unwrap_or(false)
}

/// A peak/off-peak schedule for a provider or a single model.
///
/// `timezone` is the IANA timezone, such as `Asia/Shanghai`, in which
/// `periods` and `exempt` are evaluated; it defaults to UTC. Peak windows are
/// times of day in that timezone, and on an exempt date no time is billed as
/// peak. A model may carry its own `peaks` to override the provider-wide
/// schedule, and an explicit empty `periods` list opts a model out of peak
/// pricing entirely.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Peaks {
    #[serde(default, skip_serializing_if = "is_utc")]
    pub timezone: Tz,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub periods: Vec<PeakPeriod>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exempt: Vec<PeakExemption>,
}

fn is_utc(timezone: &Tz) -> bool {
    *timezone == Tz::UTC
}

impl Peaks {
    /// `at` expressed in the schedule's timezone.
    pub fn local(&self, at: DateTime<Utc>) -> DateTime<Tz> {
        at.with_timezone(&self.timezone)
    }

    /// Whether `at` falls on an exempt day in the schedule's timezone.
    pub fn is_exempt(&self, at: DateTime<Utc>) -> bool {
        self.is_exempt_on(self.local(at).date_naive())
    }

    /// Whether `at` falls inside a peak window on a non-exempt day.
    pub fn is_peak(&self, at: DateTime<Utc>) -> bool {
        let local = self.local(at);
        if self.is_exempt_on(local.date_naive()) {
            return false;
        }
        let time = TimeOfDay::from_naive_time(local.time());
        self.periods.iter().any(|period| period.contains(time))
    }

    fn is_exempt_on(&self, date: NaiveDate) -> bool {
        self.exempt.iter().any(|exempt| exempt.applies(date))
    }
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
    pub peaks: Option<Peaks>,
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

    /// The peak schedule that applies to `model`: the model's own `peaks`
    /// override the provider-wide schedule.
    pub fn peaks_for<'a>(&'a self, model: &'a Model) -> Option<&'a Peaks> {
        model.peaks.as_ref().or(self.peaks.as_ref())
    }

    /// Whether `model` is in a peak window at `at`.
    pub fn is_peak(&self, model: &Model, at: DateTime<Utc>) -> bool {
        self.peaks_for(model).is_some_and(|peaks| peaks.is_peak(at))
    }
}
