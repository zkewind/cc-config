use std::sync::Arc;

use cc_config_lib::{resolve_current_provider_for_project_test_hook, AppState, JsonStore};

#[path = "support.rs"]
mod support;
use support::{ensure_test_home, reset_test_fs, test_mutex};

fn write_project_settings(dir: &std::path::Path, content: &str) {
    let claude_dir = dir.join(".claude");
    std::fs::create_dir_all(&claude_dir).expect("create .claude dir");
    std::fs::write(claude_dir.join("settings.json"), content).expect("write settings.json");
}

#[test]
fn resolve_returns_empty_when_no_settings_file() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let state = AppState::new(db);

    let project_path = project_dir.path().to_string_lossy().to_string();
    let id = resolve_current_provider_for_project_test_hook(&state, "claude", &project_path)
        .expect("resolve should succeed");
    assert_eq!(id, "");
}

#[test]
fn resolve_returns_empty_when_no_api_config() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    write_project_settings(project_dir.path(), r#"{"env":{"OTHER":"x"}}"#);

    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let state = AppState::new(db);
    let project_path = project_dir.path().to_string_lossy().to_string();
    let id = resolve_current_provider_for_project_test_hook(&state, "claude", &project_path)
        .expect("resolve ok");
    assert_eq!(id, "");
}

#[test]
fn resolve_reuses_matching_provider_without_writing_field() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    write_project_settings(
        project_dir.path(),
        r#"{"env":{"ANTHROPIC_AUTH_TOKEN":"sk-x","ANTHROPIC_BASE_URL":"https://a.example.com"}}"#,
    );

    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let existing = cc_config_lib::Provider::with_id(
        "p1".to_string(),
        "P1".to_string(),
        serde_json::json!({"env":{"ANTHROPIC_AUTH_TOKEN":"sk-x","ANTHROPIC_BASE_URL":"https://a.example.com"}}),
        None,
    );
    db.save_provider("claude", &existing).expect("save");

    let state = AppState::new(db.clone());
    let project_path = project_dir.path().to_string_lossy().to_string();
    let id = resolve_current_provider_for_project_test_hook(&state, "claude", &project_path)
        .expect("resolve ok");
    assert_eq!(id, "p1");
    assert_eq!(
        db.get_all_providers("claude").expect("get").len(),
        1,
        "不应创建重复供应商"
    );
    let config_key = format!("project_provider_claude_{project_path}");
    assert_eq!(db.get_setting(&config_key).expect("get"), None);
}

#[test]
fn resolve_imports_new_provider_when_no_match() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    write_project_settings(
        project_dir.path(),
        r#"{"env":{"ANTHROPIC_AUTH_TOKEN":"sk-new","ANTHROPIC_BASE_URL":"https://new.example.com"}}"#,
    );

    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let state = AppState::new(db.clone());
    let project_path = project_dir.path().to_string_lossy().to_string();
    let id = resolve_current_provider_for_project_test_hook(&state, "claude", &project_path)
        .expect("resolve ok");
    assert!(!id.is_empty());
    let providers = db.get_all_providers("claude").expect("get");
    assert_eq!(providers.len(), 1);
    assert!(providers.contains_key(&id));
}

#[test]
fn resolve_returns_empty_on_invalid_json() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let project_dir = tempfile::tempdir().expect("create temp project dir");
    write_project_settings(project_dir.path(), "{ not valid json");

    let db = Arc::new(JsonStore::memory().expect("create memory db"));
    let state = AppState::new(db);
    let project_path = project_dir.path().to_string_lossy().to_string();
    let id = resolve_current_provider_for_project_test_hook(&state, "claude", &project_path)
        .expect("resolve should tolerate invalid json");
    assert_eq!(id, "");
}
