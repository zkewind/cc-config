use std::sync::Arc;

use cc_config_lib::{AppState, AppType, JsonStore, Provider, ProviderService};
use serde_json::json;

mod support;

#[test]
fn regenerate_name_uses_domain_and_key_tail() {
    let _guard = support::test_mutex().lock().unwrap();
    support::reset_test_fs();

    let db = Arc::new(JsonStore::init().unwrap());
    let state = AppState::new(db);

    let provider = Provider::with_id(
        "p1".to_string(),
        "old-name".to_string(),
        json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com/v1",
                "ANTHROPIC_AUTH_TOKEN": "sk-abcdef"
            }
        }),
        None,
    );
    state
        .db
        .save_provider(AppType::Claude.as_str(), &provider)
        .unwrap();

    let new_name = ProviderService::regenerate_name(&state, AppType::Claude, "p1").unwrap();
    assert_eq!(new_name, "api.anthropic.com_*ef");

    let providers = state
        .db
        .get_all_providers(AppType::Claude.as_str())
        .unwrap();
    assert_eq!(providers.get("p1").unwrap().name, "api.anthropic.com_*ef");
}

#[test]
fn regenerate_name_errors_when_config_lacks_url_or_key() {
    let _guard = support::test_mutex().lock().unwrap();
    support::reset_test_fs();

    let db = Arc::new(JsonStore::init().unwrap());
    let state = AppState::new(db);

    let provider = Provider::with_id(
        "p2".to_string(),
        "official-ish".to_string(),
        json!({ "env": {} }),
        None,
    );
    state
        .db
        .save_provider(AppType::Claude.as_str(), &provider)
        .unwrap();

    let result = ProviderService::regenerate_name(&state, AppType::Claude, "p2");
    assert!(result.is_err(), "应因缺少 base_url/key 而失败");
}
