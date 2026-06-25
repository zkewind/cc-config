use std::fs;

use serde_json::json;

use cc_config_lib::{
    get_claude_mcp_path, get_claude_settings_path, import_default_config_test_hook, AppError,
    AppType, McpApps, McpServer, McpService, MultiAppConfig,
};

#[path = "support.rs"]
mod support;
use support::{
    create_test_state, create_test_state_with_config, ensure_test_home, reset_test_fs, test_mutex,
};

#[test]
fn import_default_config_claude_persists_provider() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let settings_path = get_claude_settings_path();
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).expect("create claude settings dir");
    }
    let settings = json!({
        "env": {
            "ANTHROPIC_AUTH_TOKEN": "test-key",
            "ANTHROPIC_BASE_URL": "https://api.test"
        }
    });
    fs::write(
        &settings_path,
        serde_json::to_string_pretty(&settings).expect("serialize settings"),
    )
    .expect("seed claude settings.json");

    let mut config = MultiAppConfig::default();
    config.ensure_app(&AppType::Claude);
    let state = create_test_state_with_config(&config).expect("create test state");

    import_default_config_test_hook(&state, AppType::Claude)
        .expect("import default config succeeds");

    let providers = state
        .db
        .get_all_providers(AppType::Claude.as_str())
        .expect("get all providers");
    let current_id = state
        .db
        .get_current_provider(AppType::Claude.as_str())
        .expect("get current provider");
    assert_eq!(current_id.as_deref(), Some("default"));
    let default_provider = providers.get("default").expect("default provider");
    assert_eq!(default_provider.settings_config, settings);
}

#[test]
fn import_default_config_without_live_file_returns_error() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let state = create_test_state().expect("create test state");

    let err = import_default_config_test_hook(&state, AppType::Claude)
        .expect_err("missing live file should error");
    match err {
        AppError::Localized { zh, .. } => assert!(zh.contains("Claude Code 配置文件不存在")),
        AppError::Message(msg) => assert!(msg.contains("Claude Code 配置文件不存在")),
        other => panic!("unexpected error variant: {other:?}"),
    }

    let providers = state
        .db
        .get_all_providers(AppType::Claude.as_str())
        .expect("get all providers");
    assert!(providers.is_empty());
}

#[test]
fn import_mcp_from_claude_creates_config_and_enables_servers() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let mcp_path = get_claude_mcp_path();
    let claude_json = json!({
        "mcpServers": {
            "echo": {
                "type": "stdio",
                "command": "echo"
            }
        }
    });
    fs::write(
        &mcp_path,
        serde_json::to_string_pretty(&claude_json).expect("serialize claude mcp"),
    )
    .expect("seed ~/.claude.json");

    let config = MultiAppConfig::default();
    let state = create_test_state_with_config(&config).expect("create test state");

    let changed = McpService::import_from_claude(&state).expect("import mcp from claude succeeds");
    assert!(changed > 0);

    let servers = state.db.get_all_mcp_servers().expect("get all mcp servers");
    let entry = servers.get("echo").expect("server imported");
    assert!(entry.apps.claude);
}

#[test]
fn import_mcp_from_claude_invalid_json_preserves_state() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let mcp_path = get_claude_mcp_path();
    fs::write(&mcp_path, "{\"mcpServers\":").expect("seed invalid ~/.claude.json");

    let state = create_test_state().expect("create test state");

    let err =
        McpService::import_from_claude(&state).expect_err("invalid json should bubble up error");
    match err {
        AppError::McpValidation(msg) => assert!(msg.contains("解析 ~/.claude.json 失败")),
        other => panic!("unexpected error variant: {other:?}"),
    }

    let servers = state.db.get_all_mcp_servers().expect("get all mcp servers");
    assert!(servers.is_empty());
}

#[test]
fn upsert_mcp_server_disabling_app_removes_from_claude_live_config() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let home = ensure_test_home();

    fs::create_dir_all(home.join(".claude")).expect("create ~/.claude dir");

    let state = create_test_state().expect("create test state");
    McpService::upsert_server(
        &state,
        McpServer {
            id: "echo".to_string(),
            name: "echo".to_string(),
            server: json!({
                "type": "stdio",
                "command": "echo"
            }),
            apps: McpApps { claude: true },
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        },
    )
    .expect("upsert should sync to Claude live config");

    let mcp_path = get_claude_mcp_path();
    let text = fs::read_to_string(&mcp_path).expect("read ~/.claude.json");
    let v: serde_json::Value = serde_json::from_str(&text).expect("parse ~/.claude.json");
    assert!(v.pointer("/mcpServers/echo").is_some());

    McpService::upsert_server(
        &state,
        McpServer {
            id: "echo".to_string(),
            name: "echo".to_string(),
            server: json!({
                "type": "stdio",
                "command": "echo"
            }),
            apps: McpApps { claude: false },
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        },
    )
    .expect("upsert disabling app should remove from Claude live config");

    let text = fs::read_to_string(&mcp_path).expect("read ~/.claude.json after disable");
    let v: serde_json::Value = serde_json::from_str(&text).expect("parse ~/.claude.json");
    assert!(v.pointer("/mcpServers/echo").is_none());
}

#[test]
fn enabling_claude_mcp_skips_when_claude_config_absent() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let home = ensure_test_home();

    assert!(!home.join(".claude").exists());
    assert!(!home.join(".claude.json").exists());

    let state = create_test_state().expect("create test state");

    McpService::upsert_server(
        &state,
        McpServer {
            id: "claude-server".to_string(),
            name: "Claude Server".to_string(),
            server: json!({
                "type": "stdio",
                "command": "echo"
            }),
            apps: McpApps { claude: false },
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        },
    )
    .expect("insert server without syncing");

    McpService::toggle_app(&state, "claude-server", AppType::Claude, true)
        .expect("toggle claude should succeed even when ~/.claude is missing");

    assert!(!home.join(".claude.json").exists());
}

#[test]
fn sync_all_enabled_removes_known_disabled_but_preserves_unknown_live_entries() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let mcp_path = get_claude_mcp_path();
    fs::write(
        &mcp_path,
        serde_json::to_string_pretty(&json!({
            "mcpServers": {
                "managed-disabled": {
                    "type": "stdio",
                    "command": "echo"
                },
                "external-only": {
                    "type": "stdio",
                    "command": "external"
                }
            }
        }))
        .expect("serialize claude mcp"),
    )
    .expect("seed claude mcp");

    let state = create_test_state().expect("create test state");

    state
        .db
        .save_mcp_server(&McpServer {
            id: "managed-disabled".to_string(),
            name: "Managed Disabled".to_string(),
            server: json!({
                "type": "stdio",
                "command": "echo"
            }),
            apps: McpApps { claude: false },
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        })
        .expect("save disabled server");
    state
        .db
        .save_mcp_server(&McpServer {
            id: "managed-enabled".to_string(),
            name: "Managed Enabled".to_string(),
            server: json!({
                "type": "stdio",
                "command": "managed"
            }),
            apps: McpApps { claude: true },
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        })
        .expect("save enabled server");

    McpService::sync_all_enabled(&state).expect("reconcile mcp");

    let text = fs::read_to_string(&mcp_path).expect("read claude mcp");
    let value: serde_json::Value = serde_json::from_str(&text).expect("parse claude mcp");
    let servers = value
        .get("mcpServers")
        .and_then(|entry| entry.as_object())
        .expect("mcpServers object");

    assert!(!servers.contains_key("managed-disabled"));
    assert!(servers.contains_key("managed-enabled"));
    assert!(servers.contains_key("external-only"));
}
