use crate::client::Client;
use crate::embedded;
use crate::types::{AuthMethod, InferenceProvider, Provider, ProviderType};

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
fn auth_backed_provider_types_round_trip() {
    for (kind, expected) in [
        ("chatgpt", ProviderType::Chatgpt),
        ("copilot", ProviderType::Copilot),
    ] {
        let t: ProviderType = serde_json::from_str(&format!("\"{kind}\"")).unwrap();
        assert_eq!(t, expected);
        assert_eq!(serde_json::to_string(&t).unwrap(), format!("\"{kind}\""));
    }
}

#[test]
fn auth_method_serde_round_trip() {
    let m: AuthMethod = serde_json::from_str("\"oauth2-device\"").unwrap();
    assert_eq!(m, AuthMethod::Oauth2Device);
    assert_eq!(serde_json::to_string(&m).unwrap(), "\"oauth2-device\"");
    let m: AuthMethod = serde_json::from_str("\"api-key\"").unwrap();
    assert_eq!(m, AuthMethod::ApiKey);
    assert_eq!(serde_json::to_string(&m).unwrap(), "\"api-key\"");
}

#[test]
fn auth_backed_providers_marked_for_device_login() {
    let providers = embedded::all();
    for id in ["chatgpt", "copilot"] {
        let p = providers
            .iter()
            .find(|p| p.id == InferenceProvider(id.into()))
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(p.auth, Some(AuthMethod::Oauth2Device), "auth for {id}");
        assert!(p.oauth_device_login(), "device login for {id}");
    }
    for id in ["openai", "anthropic"] {
        let p = providers
            .iter()
            .find(|p| p.id == InferenceProvider(id.into()))
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(p.auth, None, "auth for {id}");
        assert!(!p.oauth_device_login(), "device login for {id}");
    }
}

#[test]
fn auth_field_is_optional_and_skipped_when_absent() {
    let p: Provider = serde_json::from_str(r#"{"name": "Acme", "id": "acme"}"#).unwrap();
    assert_eq!(p.auth, None);
    assert!(!p.oauth_device_login());
    let back = serde_json::to_string(&p).unwrap();
    assert!(
        !back.contains("auth"),
        "absent auth must not serialize: {back}"
    );
}

#[test]
fn rig_transport_provider_types_round_trip() {
    for (kind, expected) in [
        ("cohere", ProviderType::Cohere),
        ("deepseek", ProviderType::Deepseek),
        ("doubleword", ProviderType::Doubleword),
        ("groq", ProviderType::Groq),
        ("huggingface", ProviderType::Huggingface),
        ("hyperbolic", ProviderType::Hyperbolic),
        ("llamafile", ProviderType::Llamafile),
        ("minimax", ProviderType::Minimax),
        ("mira", ProviderType::Mira),
        ("mistral", ProviderType::Mistral),
        ("moonshot", ProviderType::Moonshot),
        ("perplexity", ProviderType::Perplexity),
        ("together", ProviderType::Together),
        ("venice", ProviderType::Venice),
        ("voyageai", ProviderType::Voyageai),
        ("xai", ProviderType::Xai),
        ("xiaomimimo", ProviderType::Xiaomimimo),
        ("zai", ProviderType::Zai),
    ] {
        let t: ProviderType = serde_json::from_str(&format!("\"{kind}\"")).unwrap();
        assert_eq!(t, expected);
        assert_eq!(serde_json::to_string(&t).unwrap(), format!("\"{kind}\""));
    }
}

#[test]
fn new_transport_catalog_configs_retyped() {
    let providers = embedded::all();
    for (id, expected) in [
        ("deepseek", ProviderType::Deepseek),
        ("groq", ProviderType::Groq),
        ("togetherai", ProviderType::Together),
        ("venice", ProviderType::Venice),
        ("xai", ProviderType::Xai),
        ("zai", ProviderType::Zai),
        ("zai-coding", ProviderType::Zai),
        ("moonshotai", ProviderType::Moonshot),
        ("moonshotai-cn", ProviderType::Moonshot),
        ("huggingface", ProviderType::Huggingface),
        ("azure", ProviderType::Azure),
        ("cohere", ProviderType::Cohere),
        ("mistral", ProviderType::Mistral),
        ("perplexity", ProviderType::Perplexity),
        ("xiaomimimo", ProviderType::Xiaomimimo),
        ("hyperbolic", ProviderType::Hyperbolic),
        ("llamafile", ProviderType::Llamafile),
        ("mira", ProviderType::Mira),
        ("doubleword", ProviderType::Doubleword),
        ("voyageai", ProviderType::Voyageai),
        ("chatgpt", ProviderType::Chatgpt),
    ] {
        let p = providers
            .iter()
            .find(|p| p.id == InferenceProvider(id.into()))
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(p.r#type, Some(expected), "type mismatch for {id}");
    }
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
fn openai_gpt_6_astra_is_default_large() {
    let providers = embedded::all();
    let openai = providers
        .iter()
        .find(|p| p.id == InferenceProvider("openai".into()))
        .unwrap();
    assert_eq!(openai.default_large_model(), Some("gpt-6-astra"));
    let astra = openai.model("gpt-6-astra").unwrap();
    assert_eq!(astra.name, "GPT-6 Astra");
    assert_eq!(astra.limit.context, Some(1_050_000));
    assert_eq!(astra.limit.output, Some(128_000));
    assert_eq!(astra.cost.input, Some(10.0));
    assert_eq!(astra.cost.output, Some(50.0));
    assert_eq!(astra.cost.cache_read, Some(1.0));
    assert_eq!(astra.cost.cache_write, Some(12.5));
    assert!(astra.reasoning);
    assert!(astra.attachment);
    let efforts = &astra.reasoning_options[0];
    assert_eq!(efforts.r#type, "effort");
    assert_eq!(efforts.values, ["low", "medium", "high", "xhigh", "max"]);
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
