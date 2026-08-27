use crate::embedded;
use crate::types::{InferenceProvider, ProviderType};

#[test]
fn embedded_providers_parse_and_are_ordered() {
    let providers = embedded::all();
    let ids: Vec<&str> = providers.iter().map(|p| p.id.0.as_str()).collect();
    assert_eq!(
        ids,
        ["anthropic", "deepseek", "gemini", "openai", "openrouter"]
    );
}

#[test]
fn embedded_providers_have_valid_default_models() {
    for provider in embedded::all() {
        let ids: Vec<&str> = provider.models.iter().map(|m| m.id.as_str()).collect();
        if let Some(large) = provider.default_large_model() {
            assert!(
                ids.contains(&large),
                "default large model {large:?} not in {}",
                provider.name
            );
        }
        if let Some(small) = provider.default_small_model() {
            assert!(
                ids.contains(&small),
                "default small model {small:?} not in {}",
                provider.name
            );
        }
    }
}

#[test]
fn provider_type_serde_round_trip() {
    let t: ProviderType = serde_json::from_str("\"openai-compat\"").unwrap();
    assert_eq!(t, ProviderType::OpenaiCompat);
    let back = serde_json::to_string(&t).unwrap();
    assert_eq!(back, "\"openai-compat\"");
}

#[test]
fn provider_lookup_and_defaults() {
    let providers = embedded::all();
    let anthropic = providers
        .iter()
        .find(|p| p.id == InferenceProvider("anthropic".into()))
        .unwrap();
    assert_eq!(anthropic.default_large_model(), Some("claude-sonnet-4-6"));
    assert_eq!(
        anthropic.default_small_model(),
        Some("claude-haiku-4-5-20251001")
    );
    let model = anthropic.model("claude-sonnet-4-6").unwrap();
    assert_eq!(model.context_window, 200_000);
    assert_eq!(model.cost_per_1m_in, 3.0);
    assert!(model.can_reason);
    assert!(model.supports_attachments);
}

#[test]
fn openrouter_carries_default_headers() {
    let providers = embedded::all();
    let openrouter = providers
        .iter()
        .find(|p| p.id == InferenceProvider("openrouter".into()))
        .unwrap();
    let headers = openrouter.default_headers.as_ref().unwrap();
    assert_eq!(headers.get("X-Title").map(String::as_str), Some("Shuvarie"));
}
