use crate::client::Client;
use crate::embedded;
use crate::types::{
    AuthMethod, InferenceProvider, Model, ModelCode, ModelCodeError, PeakExemption, PeakPeriod,
    Peaks, Provider, ProviderType, TimeOfDay,
};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;

fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .expect("valid UTC datetime")
}

fn time(value: &str) -> TimeOfDay {
    value.parse().expect("valid time of day")
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
fn provider_lookup_and_defaults() {
    let providers = embedded::all();
    let anthropic = providers
        .iter()
        .find(|p| p.id == InferenceProvider("anthropic".into()))
        .unwrap();
    assert_eq!(anthropic.default_large_model(), Some("claude-sonnet-5-5"));
    assert_eq!(
        anthropic.default_small_model(),
        Some("claude-haiku-4-5-20251001")
    );
    let model = anthropic.model("claude-sonnet-4-6").unwrap();
    assert_eq!(model.limit.context, Some(1_000_000));
    assert_eq!(model.limit.output, Some(128_000));
    assert_eq!(model.cost.input, Some(3.0));
    assert_eq!(model.cost.cache_read, Some(0.3));
    assert_eq!(model.cost.cache_write, Some(3.75));
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
fn model_code_parses_display_and_round_trips() {
    let code: ModelCode = "zai-org/GLM-5.3-Flash:high".parse().unwrap();
    assert_eq!(code.org(), "zai-org");
    assert_eq!(code.model(), "GLM-5.3-Flash");
    assert_eq!(code.variant(), Some("high"));
    assert_eq!(code.to_string(), "zai-org/GLM-5.3-Flash:high");

    let bare: ModelCode = "deepseek-ai/DeepSeek-V4-Flash".parse().unwrap();
    assert_eq!(bare.variant(), None);
    assert_eq!(bare.to_string(), "deepseek-ai/DeepSeek-V4-Flash");

    assert_eq!(
        "anthropic/claude-opus-4-8:fast"
            .parse::<ModelCode>()
            .unwrap()
            .to_string(),
        "anthropic/claude-opus-4-8:fast"
    );
}

#[test]
fn model_code_rejects_invalid_input() {
    for input in [
        "zai-org",               // missing model
        "ZAI-org/GLM-5.3",       // org must be lowercase
        "zai-org/GLM 5.3",       // space in model id
        "zai-org/GLM-5.3:High",  // variant must be lowercase
        "zai-org/GLM-5.3:hi:gh", // two variants
        "/GLM-5.3",              // empty org
        "zai-org/",              // empty model
        "zai-org/GLM-5.3:",      // empty variant
    ] {
        assert!(
            input.parse::<ModelCode>().is_err(),
            "{input:?} should not parse"
        );
    }
}

#[test]
fn model_code_parts_must_be_valid() {
    assert!(matches!(
        ModelCode::new("Zai-Org", "GLM-5.3", None),
        Err(ModelCodeError::Org(_))
    ));
    assert!(matches!(
        ModelCode::new("zai-org", "GLM 5.3", None),
        Err(ModelCodeError::Model(_))
    ));
    assert!(matches!(
        ModelCode::new("zai-org", "GLM-5.3", Some("High")),
        Err(ModelCodeError::Variant(_))
    ));
    assert!(ModelCode::new("zai-org", "GLM-5.3", Some("high")).is_ok());
}

#[test]
fn model_code_serde_round_trip() {
    let code: ModelCode = serde_json::from_str("\"zai-org/GLM-5.3-Flash:high\"").unwrap();
    assert_eq!(code.to_string(), "zai-org/GLM-5.3-Flash:high");
    assert_eq!(
        serde_json::to_string(&code).unwrap(),
        "\"zai-org/GLM-5.3-Flash:high\""
    );
    assert!(serde_json::from_str::<ModelCode>("\"zai-org\"").is_err());
}

#[test]
fn embedded_models_have_valid_model_codes() {
    for provider in embedded::all() {
        for model in &provider.models {
            assert!(
                model
                    .model_code
                    .org()
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "org {:?} for {}",
                model.model_code.org(),
                provider.name
            );
            assert!(!model.model_code.model().is_empty(), "empty model id");
            if let Some(variant) = model.model_code.variant() {
                assert!(
                    variant
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                    "variant {variant:?} for {}",
                    provider.name
                );
            }
        }
    }
}

#[test]
fn provider_lookup_by_model_code() {
    let providers = embedded::all();
    let anthropic = providers
        .iter()
        .find(|p| p.id == InferenceProvider("anthropic".into()))
        .unwrap();
    let code: ModelCode = "anthropic/claude-sonnet-4-6".parse().unwrap();
    let model = anthropic.model_by_code(&code).unwrap();
    assert_eq!(model.id, "claude-sonnet-4-6");
    assert_eq!(model.model_code, code);
}

#[test]
fn model_codes_identify_models_across_providers() {
    let providers = embedded::all();
    let code: ModelCode = "zai-org/GLM-5.3-Flash".parse().unwrap();
    let found_in: Vec<&str> = providers
        .iter()
        .filter(|p| p.model_by_code(&code).is_some())
        .map(|p| p.id.0.as_str())
        .collect();
    assert!(
        found_in.len() >= 4,
        "GLM-5.3-Flash should be recognizable across providers, found in {found_in:?}"
    );
    for name in ["huggingface", "openrouter", "vercel", "zai"] {
        assert!(
            found_in.contains(&name),
            "expected {name} to carry {code}, found in {found_in:?}"
        );
    }
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

#[test]
fn time_of_day_parses_and_round_trips() {
    let t = time("01:30");
    assert_eq!(t.hour(), 1);
    assert_eq!(t.minute(), 30);
    assert_eq!(t.minutes_since_midnight(), 90);
    assert_eq!(t.to_string(), "01:30");
    assert_eq!(TimeOfDay::new(0, 0).unwrap().to_string(), "00:00");
    assert_eq!(TimeOfDay::new(23, 59).unwrap().to_string(), "23:59");

    let back: TimeOfDay = serde_json::from_str("\"07:05\"").unwrap();
    assert_eq!(back, TimeOfDay::new(7, 5).unwrap());
    assert_eq!(serde_json::to_string(&back).unwrap(), "\"07:05\"");
}

#[test]
fn time_of_day_rejects_malformed_input() {
    for input in [
        "24:00", "1:00", "01:60", "01:00:00", "0100", "", "aa:bb", "23:5", " 1:00",
    ] {
        assert!(
            input.parse::<TimeOfDay>().is_err(),
            "{input:?} should not parse"
        );
    }
    assert!(TimeOfDay::new(24, 0).is_err());
    assert!(TimeOfDay::new(0, 60).is_err());
}

#[test]
fn peak_period_window_boundaries() {
    let period = PeakPeriod::new(time("01:00"), time("04:00")).unwrap();
    assert!(!period.crosses_midnight());
    assert!(period.contains(time("01:00")), "start is inclusive");
    assert!(period.contains(time("03:59")));
    assert!(!period.contains(time("00:59")));
    assert!(!period.contains(time("04:00")), "end is exclusive");

    let overnight = PeakPeriod::new(time("23:00"), time("01:00")).unwrap();
    assert!(overnight.crosses_midnight());
    assert!(overnight.contains(time("23:00")));
    assert!(overnight.contains(time("00:59")));
    assert!(!overnight.contains(time("01:00")));
    assert!(!overnight.contains(time("12:00")));

    assert!(PeakPeriod::new(time("01:00"), time("01:00")).is_err());
}

#[test]
fn peak_period_and_peaks_serde_round_trip() {
    let period: PeakPeriod = serde_json::from_str(r#"{"start": "23:00", "end": "01:00"}"#).unwrap();
    assert_eq!(period.start(), time("23:00"));
    assert_eq!(period.end(), time("01:00"));
    assert_eq!(
        serde_json::to_string(&period).unwrap(),
        r#"{"start":"23:00","end":"01:00"}"#
    );

    let peaks: Peaks = serde_json::from_str(
        r#"{"timezone": "Asia/Shanghai", "periods": [{"start": "01:00", "end": "04:00"}], "exempt": ["weekends", "holidays-us"]}"#,
    )
    .unwrap();
    assert_eq!(peaks.timezone, Tz::Asia__Shanghai);
    assert_eq!(
        peaks.exempt,
        [PeakExemption::Weekends, PeakExemption::HolidaysUs]
    );
    assert_eq!(
        serde_json::to_string(&peaks).unwrap(),
        r#"{"timezone":"Asia/Shanghai","periods":[{"start":"01:00","end":"04:00"}],"exempt":["weekends","holidays-us"]}"#
    );

    let utc_peaks: Peaks =
        serde_json::from_str(r#"{"periods": [{"start": "01:00", "end": "04:00"}]}"#).unwrap();
    assert_eq!(utc_peaks.timezone, Tz::UTC, "UTC is the default timezone");
    assert!(
        !serde_json::to_string(&utc_peaks)
            .unwrap()
            .contains("timezone"),
        "the default UTC timezone is omitted"
    );
}

#[test]
fn invalid_peak_periods_are_rejected() {
    assert!(
        serde_json::from_str::<PeakPeriod>(r#"{"start": "01:00", "end": "01:00"}"#).is_err(),
        "zero-length windows are invalid"
    );
    assert!(
        serde_json::from_str::<PeakPeriod>(r#"{"start": "25:00", "end": "01:00"}"#).is_err(),
        "out-of-range times are invalid"
    );
    assert!(
        serde_json::from_str::<Peaks>(
            r#"{"periods": [{"start": "01:00", "end": "04:00"}], "exempt": ["weekdays"]}"#
        )
        .is_err(),
        "unknown exemptions are invalid"
    );
    assert!(
        serde_json::from_str::<Peaks>(r#"{"timezone": "Mars/Olympus"}"#).is_err(),
        "unknown timezones are invalid"
    );
}

#[test]
fn peaks_respect_weekend_and_holiday_exemptions() {
    let peaks = Peaks {
        timezone: Tz::UTC,
        periods: vec![
            PeakPeriod::new(time("01:00"), time("04:00")).unwrap(),
            PeakPeriod::new(time("06:00"), time("10:00")).unwrap(),
        ],
        exempt: vec![PeakExemption::Weekends, PeakExemption::HolidaysCn],
    };

    let monday = utc(2025, 6, 9, 2, 0);
    assert!(!peaks.is_exempt(monday));
    assert!(peaks.is_peak(monday));
    assert!(peaks.is_peak(utc(2025, 6, 9, 6, 0)), "start is inclusive");
    assert!(!peaks.is_peak(utc(2025, 6, 9, 5, 0)), "between windows");
    assert!(!peaks.is_peak(utc(2025, 6, 9, 10, 0)), "end is exclusive");

    let saturday = utc(2025, 6, 7, 2, 0);
    assert!(peaks.is_exempt(saturday));
    assert!(!peaks.is_peak(saturday));

    let national_day = utc(2025, 10, 1, 2, 0);
    assert!(PeakExemption::HolidaysCn.applies(national_day.date_naive()));
    assert!(peaks.is_exempt(national_day));
    assert!(!peaks.is_peak(national_day));

    assert!(PeakExemption::HolidaysUs.applies(utc(2025, 7, 4, 2, 0).date_naive()));
    assert!(!PeakExemption::HolidaysUs.applies(national_day.date_naive()));
}

#[test]
fn overnight_peak_exemption_is_evaluated_per_instant() {
    let peaks = Peaks {
        timezone: Tz::UTC,
        periods: vec![PeakPeriod::new(time("23:00"), time("01:00")).unwrap()],
        exempt: vec![PeakExemption::Weekends],
    };

    assert!(peaks.is_peak(utc(2025, 6, 6, 23, 30)), "Friday night");
    assert!(peaks.is_peak(utc(2025, 6, 10, 0, 30)), "Tuesday night");
    assert!(!peaks.is_peak(utc(2025, 6, 7, 0, 30)), "Saturday morning");
    assert!(!peaks.is_peak(utc(2025, 6, 7, 23, 30)), "Saturday night");
}

#[test]
fn peaks_evaluate_windows_in_the_schedule_timezone() {
    let peaks = Peaks {
        timezone: Tz::Asia__Shanghai,
        periods: vec![PeakPeriod::new(time("09:00"), time("12:00")).unwrap()],
        exempt: vec![PeakExemption::Weekends],
    };

    assert_eq!(
        peaks.local(utc(2025, 6, 9, 1, 0)).time().to_string(),
        "09:00:00",
        "01:00 UTC is 09:00 in Shanghai"
    );
    assert!(peaks.is_peak(utc(2025, 6, 9, 1, 0)), "start is inclusive");
    assert!(!peaks.is_peak(utc(2025, 6, 9, 0, 59)));
    assert!(peaks.is_peak(utc(2025, 6, 9, 3, 59)));
    assert!(!peaks.is_peak(utc(2025, 6, 9, 4, 0)), "end is exclusive");

    assert!(
        peaks.is_exempt(utc(2025, 6, 6, 23, 30)),
        "Friday 23:30 UTC is Saturday in Shanghai"
    );
}

#[test]
fn timezone_aligns_holiday_exemptions_with_the_local_date() {
    let shanghai = Peaks {
        timezone: Tz::Asia__Shanghai,
        periods: vec![PeakPeriod::new(time("00:00"), time("01:00")).unwrap()],
        exempt: vec![PeakExemption::HolidaysCn],
    };
    let utc_schedule = Peaks {
        timezone: Tz::UTC,
        periods: vec![PeakPeriod::new(time("16:00"), time("17:00")).unwrap()],
        exempt: vec![PeakExemption::HolidaysCn],
    };

    let holiday_midnight = utc(2025, 9, 30, 16, 30);
    assert!(
        shanghai.is_exempt(holiday_midnight),
        "National Day starts at 00:00 in Shanghai"
    );
    assert!(!shanghai.is_peak(holiday_midnight));
    assert!(
        utc_schedule.is_peak(holiday_midnight),
        "the UTC date is still September 30"
    );

    let holiday_noon = utc(2025, 10, 1, 4, 0);
    assert!(shanghai.is_exempt(holiday_noon));
    assert!(utc_schedule.is_exempt(holiday_noon));
}

#[test]
fn peaks_follow_daylight_saving_in_the_schedule_timezone() {
    let peaks = Peaks {
        timezone: Tz::America__New_York,
        periods: vec![PeakPeriod::new(time("09:00"), time("10:00")).unwrap()],
        exempt: vec![],
    };

    assert!(peaks.is_peak(utc(2025, 1, 15, 14, 0)), "09:00 EST");
    assert!(!peaks.is_peak(utc(2025, 1, 15, 13, 0)), "08:00 EST");
    assert!(peaks.is_peak(utc(2025, 7, 15, 13, 0)), "09:00 EDT");
    assert!(!peaks.is_peak(utc(2025, 7, 15, 14, 0)), "10:00 EDT");
}

#[test]
fn model_peaks_override_provider_peaks() {
    let provider: Provider = serde_json::from_str(
        r#"{
            "name": "Acme",
            "id": "acme",
            "peaks": {
                "periods": [{"start": "01:00", "end": "04:00"}],
                "exempt": ["weekends"]
            },
            "models": [
                {
                    "id": "inherit",
                    "modelCode": "acme/inherit-model",
                    "name": "Inherit",
                    "reasoning": false,
                    "attachment": false,
                    "limit": {},
                    "cost": {}
                },
                {
                    "id": "custom",
                    "modelCode": "acme/custom-model",
                    "name": "Custom",
                    "reasoning": false,
                    "attachment": false,
                    "limit": {},
                    "cost": {},
                    "peaks": {
                        "periods": [{"start": "12:00", "end": "13:00"}]
                    }
                }
            ]
        }"#,
    )
    .unwrap();

    let inherit = provider.model("inherit").unwrap();
    let custom = provider.model("custom").unwrap();
    assert_eq!(
        provider.peaks_for(inherit).unwrap().periods,
        [PeakPeriod::new(time("01:00"), time("04:00")).unwrap()]
    );
    assert_eq!(
        provider.peaks_for(custom).unwrap().periods,
        [PeakPeriod::new(time("12:00"), time("13:00")).unwrap()]
    );

    assert!(provider.is_peak(inherit, utc(2025, 6, 9, 2, 0)));
    assert!(provider.is_peak(custom, utc(2025, 6, 9, 12, 30)));
    assert!(!provider.is_peak(custom, utc(2025, 6, 9, 2, 0)));

    assert!(!provider.is_peak(inherit, utc(2025, 6, 7, 2, 0)), "weekend");
    assert!(
        provider.is_peak(custom, utc(2025, 6, 7, 12, 30)),
        "model peaks replace the provider exemptions too"
    );
}

#[test]
fn model_peaks_override_the_provider_timezone() {
    let provider: Provider = serde_json::from_str(
        r#"{
            "name": "Acme",
            "id": "acme",
            "peaks": {
                "timezone": "Asia/Shanghai",
                "periods": [{"start": "09:00", "end": "10:00"}]
            },
            "models": [
                {
                    "id": "ny",
                    "modelCode": "acme/ny-model",
                    "name": "NY",
                    "reasoning": false,
                    "attachment": false,
                    "limit": {},
                    "cost": {},
                    "peaks": {
                        "timezone": "America/New_York",
                        "periods": [{"start": "09:00", "end": "10:00"}]
                    }
                }
            ]
        }"#,
    )
    .unwrap();

    let model = provider.model("ny").unwrap();
    assert_eq!(
        provider.peaks_for(model).unwrap().timezone,
        Tz::America__New_York
    );
    assert!(
        provider.is_peak(model, utc(2025, 1, 15, 14, 0)),
        "09:00 EST"
    );
    assert!(
        !provider.is_peak(model, utc(2025, 1, 15, 1, 0)),
        "09:00 in Shanghai must not apply to the model"
    );
}

#[test]
fn peaks_are_optional_and_skipped_when_absent() {
    let provider: Provider = serde_json::from_str(r#"{"name": "Acme", "id": "acme"}"#).unwrap();
    assert!(provider.peaks.is_none());
    assert!(!serde_json::to_string(&provider).unwrap().contains("peaks"));

    let model: Model = serde_json::from_str(
        r#"{
            "id": "m",
            "modelCode": "acme/m",
            "name": "M",
            "reasoning": false,
            "attachment": false,
            "limit": {},
            "cost": {}
        }"#,
    )
    .unwrap();
    assert!(model.peaks.is_none());
    assert!(!serde_json::to_string(&model).unwrap().contains("peaks"));
    assert_eq!(Peaks::default().timezone, Tz::UTC);
}
