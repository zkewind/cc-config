use std::sync::Arc;

use cc_config_lib::{import_project_provider_test_hook, AppState, JsonStore};

#[path = "support.rs"]
mod support;
use support::{ensure_test_home, reset_test_fs, test_mutex};

fn write_project_settings(dir: &std::path::Path, content: &str) {
    let claude_dir = dir.join(".claude");
    std::fs::create_dir_all(&claude_dir).expect("create .claude dir");
    std::fs::write(claude_dir.join("settings.json"), content).expect("write settings.json");
}

#[test]
fn import_creates_new_provider_from_real_world_settings() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    let settings_json = r#"{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "fe_oa_c45d5ff7290a977ef0a6122aff54d14f7560728f342aed37",
    "ANTHROPIC_BASE_URL": "https://cc.freemodel.dev",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  }
}"#;
    write_project_settings(project_dir.path(), settings_json);

    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let state = AppState::new(db.clone());

    let project_path = project_dir.path().to_string_lossy().to_string();
    let provider_id = import_project_provider_test_hook(&state, "claude", &project_path)
        .expect("import should succeed");

    assert!(!provider_id.is_empty(), "should create/find a provider");

    let providers = db.get_all_providers("claude").expect("get providers");
    let provider = providers
        .get(&provider_id)
        .expect("imported provider should be persisted");

    assert_eq!(provider.name, "cc.freemodel.dev_*37");
    assert_eq!(
        provider
            .settings_config
            .pointer("/env/ANTHROPIC_AUTH_TOKEN")
            .and_then(|v| v.as_str()),
        Some("fe_oa_c45d5ff7290a977ef0a6122aff54d14f7560728f342aed37")
    );
    assert_eq!(
        provider
            .settings_config
            .pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|v| v.as_str()),
        Some("https://cc.freemodel.dev")
    );

    // 不再持久化 project_provider_* 字段（真相源改为 live 文件）
    let config_key = format!("project_provider_claude_{project_path}");
    assert_eq!(db.get_setting(&config_key).expect("get setting"), None);
}

#[test]
fn import_returns_empty_when_no_settings_file() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let state = AppState::new(db);

    let project_path = project_dir.path().to_string_lossy().to_string();
    let provider_id = import_project_provider_test_hook(&state, "claude", &project_path)
        .expect("import should succeed even without settings file");

    assert_eq!(provider_id, "");
}

#[test]
fn import_reuses_existing_matching_provider() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    let settings_json = r#"{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "sk-shared-key",
    "ANTHROPIC_BASE_URL": "https://shared.example.com"
  }
}"#;
    write_project_settings(project_dir.path(), settings_json);

    let db = Arc::new(JsonStore::memory().expect("create memory db"));

    // 预先创建一个具有相同 key+url 的供应商
    let existing = cc_config_lib::Provider::with_id(
        "existing-id".to_string(),
        "Existing Provider".to_string(),
        serde_json::json!({
            "env": {
                "ANTHROPIC_AUTH_TOKEN": "sk-shared-key",
                "ANTHROPIC_BASE_URL": "https://shared.example.com"
            }
        }),
        None,
    );
    db.save_provider("claude", &existing)
        .expect("save existing provider");

    let state = AppState::new(db.clone());
    let project_path = project_dir.path().to_string_lossy().to_string();
    let provider_id = import_project_provider_test_hook(&state, "claude", &project_path)
        .expect("import should succeed");

    assert_eq!(provider_id, "existing-id");

    let providers = db.get_all_providers("claude").expect("get providers");
    assert_eq!(providers.len(), 1, "should not create a duplicate provider");
}
