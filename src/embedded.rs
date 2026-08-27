use crate::types::Provider;

const ANTHROPIC: &str = include_str!("../configs/anthropic.json");
const DEEPSEEK: &str = include_str!("../configs/deepseek.json");
const GEMINI: &str = include_str!("../configs/gemini.json");
const OPENAI: &str = include_str!("../configs/openai.json");
const OPENROUTER: &str = include_str!("../configs/openrouter.json");

/// All embedded provider configs, in a stable order.
pub fn all() -> Vec<Provider> {
    [ANTHROPIC, DEEPSEEK, GEMINI, OPENAI, OPENROUTER]
        .into_iter()
        .map(|json| serde_json::from_str(json).expect("embedded provider config must parse"))
        .collect()
}
