#![allow(non_snake_case)]

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::app_config::AppType;
use crate::config::{self, get_claude_settings_path, ConfigStatus};
use crate::settings;
use crate::store::AppState;
use std::str::FromStr;

#[tauri::command]
pub async fn get_claude_config_status() -> Result<ConfigStatus, String> {
    Ok(config::get_claude_config_status())
}

fn invalid_json_format_error(error: serde_json::Error) -> String {
    let lang = settings::get_settings()
        .language
        .unwrap_or_else(|| "zh".to_string());

    match lang.as_str() {
        "en" => format!("Invalid JSON format: {error}"),
        _ => format!("无效的 JSON 格式: {error}"),
    }
}

fn validate_common_config_snippet(app_type: &str, snippet: &str) -> Result<(), String> {
    if snippet.trim().is_empty() {
        return Ok(());
    }

    if app_type == "claude" {
        serde_json::from_str::<serde_json::Value>(snippet).map_err(invalid_json_format_error)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_config_status(
    _state: State<'_, AppState>,
    app: String,
) -> Result<ConfigStatus, String> {
    match AppType::from_str(&app).map_err(|e| e.to_string())? {
        AppType::Claude => Ok(config::get_claude_config_status()),
    }
}

#[tauri::command]
pub async fn get_claude_code_config_path() -> Result<String, String> {
    Ok(get_claude_settings_path().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_config_dir(app: String) -> Result<String, String> {
    let dir = match AppType::from_str(&app).map_err(|e| e.to_string())? {
        AppType::Claude => config::get_claude_config_dir(),
    };

    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_config_folder(handle: AppHandle, app: String) -> Result<bool, String> {
    let config_dir = match AppType::from_str(&app).map_err(|e| e.to_string())? {
        AppType::Claude => config::get_claude_config_dir(),
    };

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    handle
        .opener()
        .open_path(config_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("打开文件夹失败: {e}"))?;

    Ok(true)
}

#[tauri::command]
pub async fn pick_directory(
    app: AppHandle,
    #[allow(non_snake_case)] defaultPath: Option<String>,
) -> Result<Option<String>, String> {
    let initial = defaultPath
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut builder = app.dialog().file();
        if let Some(path) = initial {
            builder = builder.set_directory(path);
        }
        builder.blocking_pick_folder()
    })
    .await
    .map_err(|e| format!("弹出目录选择器失败: {e}"))?;

    match result {
        Some(file_path) => {
            let resolved = file_path
                .simplified()
                .into_path()
                .map_err(|e| format!("解析选择的目录失败: {e}"))?;
            Ok(Some(resolved.to_string_lossy().to_string()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_app_config_path() -> Result<String, String> {
    let config_path = config::get_app_config_path();
    Ok(config_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_app_config_folder(handle: AppHandle) -> Result<bool, String> {
    let config_dir = config::get_app_config_dir();

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    handle
        .opener()
        .open_path(config_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("打开文件夹失败: {e}"))?;

    Ok(true)
}

#[tauri::command]
pub async fn get_claude_common_config_snippet(
    state: tauri::State<'_, crate::store::AppState>,
) -> Result<Option<String>, String> {
    state
        .db
        .get_config_snippet("claude")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_claude_common_config_snippet(
    snippet: String,
    state: tauri::State<'_, crate::store::AppState>,
) -> Result<(), String> {
    set_common_config_snippet("claude".to_string(), snippet, state).await
}

#[tauri::command]
pub async fn get_common_config_snippet(
    app_type: String,
    state: tauri::State<'_, crate::store::AppState>,
) -> Result<Option<String>, String> {
    let app = AppType::from_str(&app_type).map_err(|e| e.to_string())?;
    state
        .db
        .get_config_snippet(app.as_str())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_common_config_snippet(
    app_type: String,
    snippet: String,
    state: tauri::State<'_, crate::store::AppState>,
) -> Result<(), String> {
    let app = AppType::from_str(&app_type).map_err(|e| e.to_string())?;
    let app_key = app.as_str();
    let is_cleared = snippet.trim().is_empty();
    let old_snippet = state
        .db
        .get_config_snippet(app_key)
        .map_err(|e| e.to_string())?;

    validate_common_config_snippet(app_key, &snippet)?;

    let value = if is_cleared { None } else { Some(snippet) };

    if let Some(legacy_snippet) = old_snippet
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        crate::services::provider::ProviderService::migrate_legacy_common_config_usage(
            state.inner(),
            app.clone(),
            legacy_snippet,
        )
        .map_err(|e| e.to_string())?;
    }

    state
        .db
        .set_config_snippet(app_key, value)
        .map_err(|e| e.to_string())?;
    state
        .db
        .set_config_snippet_cleared(app_key, is_cleared)
        .map_err(|e| e.to_string())?;

    crate::services::provider::ProviderService::sync_current_provider_for_app(state.inner(), app)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn extract_common_config_snippet(
    appType: String,
    settingsConfig: Option<String>,
    state: tauri::State<'_, crate::store::AppState>,
) -> Result<String, String> {
    let app = AppType::from_str(&appType).map_err(|e| e.to_string())?;

    if let Some(settings_config) = settingsConfig.filter(|s| !s.trim().is_empty()) {
        let settings: serde_json::Value =
            serde_json::from_str(&settings_config).map_err(invalid_json_format_error)?;

        return crate::services::provider::ProviderService::extract_common_config_snippet_from_settings(
            app,
            &settings,
        )
        .map_err(|e| e.to_string());
    }

    crate::services::provider::ProviderService::extract_common_config_snippet(&state, app)
        .map_err(|e| e.to_string())
}

/// 读取指定项目目录下的 .claude/settings.json
/// 返回 pretty-printed JSON 字符串，若文件不存在则返回 null
#[tauri::command]
pub async fn read_project_settings(
    #[allow(non_snake_case)] projectPath: String,
) -> Result<Option<String>, String> {
    let project_path = std::path::Path::new(&projectPath);
    let settings_path = project_path.join(".claude").join("settings.json");

    if !settings_path.exists() {
        return Ok(None);
    }

    let content =
        std::fs::read_to_string(&settings_path).map_err(|e| format!("读取项目配置失败: {e}"))?;

    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析项目配置 JSON 失败: {e}"))?;

    let pretty =
        serde_json::to_string_pretty(&value).map_err(|e| format!("格式化项目配置失败: {e}"))?;

    Ok(Some(pretty))
}

/// 写入指定项目目录下的 .claude/settings.json
/// 若 .claude 目录不存在则自动创建
#[tauri::command]
pub async fn save_project_settings(
    #[allow(non_snake_case)] projectPath: String,
    content: String,
) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("无效的 JSON 格式: {e}"))?;

    let project_path = std::path::Path::new(&projectPath);
    let settings_path = project_path.join(".claude").join("settings.json");

    config::write_json_file(&settings_path, &value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::validate_common_config_snippet;

    #[test]
    fn validate_common_config_snippet_rejects_invalid_claude_snippet() {
        let err = validate_common_config_snippet("claude", "{")
            .expect_err("invalid claude snippet should be rejected");
        assert!(err.contains("JSON") || err.contains("格式"));
    }
}
