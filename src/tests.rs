use crate::client::Client;
use crate::embedded;
use crate::types::{InferenceProvider, ProviderType};

#[test]
fn embedded_providers_parse_and_are_ordered() {
    let providers = embedded::all();
    let ids: Vec<&str> = providers.iter().map(|p| p.id.0.as_str()).collect();
    assert_eq!(
        ids,
        [
            "aihubmix",
            "alibaba-singapore",
            "alibaba-us",
            "anthropic",
            "atlascloud",
            "avian",
            "azure",
            "baseten",
            "bedrock-europe",
            "bedrock",
            "cerebras",
            "chutes",
            "copilot",
            "cortecs",
            "deepseek",
            "fireworks",
            "gemini",
            "groq",
            "huggingface",
            "hyper",
            "ionet",
            "kimi-code",
            "minimax-china",
            "minimax",
            "moonshot",
            "nebius",
            "neuralwatt",
            "ollama-cloud",
            "openai",
            "opencode-go",
            "opencode-zen",
            "openrouter",
            "qiniucloud",
            "scaleway",
            "synthetic",
            "togetherai",
            "venice",
            "vercel",
            "vertexai",
            "xai",
            "zai",
            "zhipu-coding",
            "zhipu",
        ]
    );
}

#[test]
fn embedded_providers_have_doc() {
    for provider in embedded::all() {
        assert!(
            provider.doc.as_deref().is_some_and(|d| !d.is_empty()),
            "provider {} missing doc",
            provider.name
        );
    }
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
    assert_eq!(model.limit.context, Some(200_000));
    assert_eq!(model.cost.input, Some(3.0));
    assert!(model.reasoning);
    assert!(model.attachment);
}

#[test]
fn openrouter_carries_default_headers() {
    let providers = embedded::all();
    let openrouter = providers
        .iter()
        .find(|p| p.id == InferenceProvider("openrouter".into()))
        .unwrap();
    let headers = openrouter.default_headers.as_ref().unwrap();
    assert_eq!(headers.get("X-Title").map(String::as_str), Some("Crush"));
    assert_eq!(
        headers.get("HTTP-Referer").map(String::as_str),
        Some("https://charm.land")
    );
}

#[test]
fn kimi_code_is_renamed() {
    let providers = embedded::all();
    let kimi = providers
        .iter()
        .find(|p| p.id == InferenceProvider("kimi-code".into()))
        .unwrap();
    assert_eq!(kimi.name, "Kimi Code");
    assert_eq!(kimi.default_large_model(), Some("k3"));
}
#[test]
fn save_and_load_from_local_round_trip() {
    let client = Client::new();
    let providers = embedded::all();
    let path = std::env::temp_dir().join("selune_test_catalog.json");
    let path = path.to_str().unwrap();

    client.save_to_local(path, &providers).unwrap();
    let loaded = client.load_from_local(path).unwrap();
    assert_eq!(loaded, providers);

    std::fs::remove_file(path).unwrap();
}
