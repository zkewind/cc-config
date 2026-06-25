use indexmap::IndexMap;
use tauri::State;

use crate::app_config::AppType;
use crate::error::AppError;
use crate::provider::Provider;
use crate::services::{
    EndpointLatency, ProviderService, ProviderSortUpdate, SpeedtestService, SwitchResult,
};
use crate::store::AppState;
use std::str::FromStr;

// 常量定义
const TEMPLATE_TYPE_TOKEN_PLAN: &str = "token_plan";
const TEMPLATE_TYPE_BALANCE: &str = "balance";
#[allow(dead_code)]
const COPILOT_UNIT_PREMIUM: &str = "requests";

/// 获取所有供应商
#[tauri::command]
pub fn get_providers(
    state: State<'_, AppState>,
    app: String,
) -> Result<IndexMap<String, Provider>, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::list(state.inner(), app_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_provider(state: State<'_, AppState>, app: String) -> Result<String, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::current(state.inner(), app_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_provider(
    state: State<'_, AppState>,
    app: String,
    provider: Provider,
    #[allow(non_snake_case)] addToLive: Option<bool>,
) -> Result<bool, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::add(state.inner(), app_type, provider, addToLive.unwrap_or(true))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_provider(
    state: State<'_, AppState>,
    app: String,
    provider: Provider,
    #[allow(non_snake_case)] originalId: Option<String>,
) -> Result<bool, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::update(state.inner(), app_type, originalId.as_deref(), provider)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn regenerate_provider_name(
    state: State<'_, AppState>,
    app: String,
    provider_id: String,
) -> Result<String, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::regenerate_name(state.inner(), app_type, &provider_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_provider(
    state: State<'_, AppState>,
    app: String,
    id: String,
) -> Result<bool, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::delete(state.inner(), app_type, &id)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_provider_from_live_config(
    state: tauri::State<'_, AppState>,
    app: String,
    id: String,
) -> Result<bool, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::remove_from_live_config(state.inner(), app_type, &id)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

fn switch_provider_internal(
    state: &AppState,
    app_type: AppType,
    id: &str,
) -> Result<SwitchResult, AppError> {
    ProviderService::switch(state, app_type, id)
}

#[cfg_attr(not(feature = "test-hooks"), doc(hidden))]
pub fn switch_provider_test_hook(
    state: &AppState,
    app_type: AppType,
    id: &str,
) -> Result<SwitchResult, AppError> {
    switch_provider_internal(state, app_type, id)
}

#[tauri::command]
pub fn switch_provider(
    state: State<'_, AppState>,
    app: String,
    id: String,
) -> Result<SwitchResult, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    switch_provider_internal(&state, app_type, &id).map_err(|e| e.to_string())
}

fn import_default_config_internal(state: &AppState, app_type: AppType) -> Result<bool, AppError> {
    let imported = ProviderService::import_default_config(state, app_type.clone())?;

    if imported {
        // Extract common config snippet (mirrors old startup logic in lib.rs)
        if state
            .db
            .should_auto_extract_config_snippet(app_type.as_str())?
        {
            match ProviderService::extract_common_config_snippet(state, app_type.clone()) {
                Ok(snippet) if !snippet.is_empty() && snippet != "{}" => {
                    let _ = state
                        .db
                        .set_config_snippet(app_type.as_str(), Some(snippet));
                    let _ = state
                        .db
                        .set_config_snippet_cleared(app_type.as_str(), false);
                }
                _ => {}
            }
        }

        ProviderService::migrate_legacy_common_config_usage_if_needed(state, app_type.clone())?;
    }

    Ok(imported)
}

#[cfg_attr(not(feature = "test-hooks"), doc(hidden))]
pub fn import_default_config_test_hook(
    state: &AppState,
    app_type: AppType,
) -> Result<bool, AppError> {
    import_default_config_internal(state, app_type)
}

#[tauri::command]
pub fn import_default_config(state: State<'_, AppState>, app: String) -> Result<bool, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    import_default_config_internal(&state, app_type).map_err(Into::into)
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn queryProviderUsage(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerId: String,
    app: String,
) -> Result<crate::provider::UsageResult, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    query_provider_usage_inner(&state, app_type, &providerId).await
}

async fn query_provider_usage_inner(
    state: &AppState,
    app_type: AppType,
    provider_id: &str,
) -> Result<crate::provider::UsageResult, String> {
    // 从数据库读取供应商信息，检查特殊模板类型
    let providers = state
        .db
        .get_all_providers(app_type.as_str())
        .map_err(|e| format!("Failed to get providers: {e}"))?;
    let provider = providers.get(provider_id);
    let usage_script = provider
        .and_then(|p| p.meta.as_ref())
        .and_then(|m| m.usage_script.as_ref());
    let template_type = usage_script
        .and_then(|s| s.template_type.as_deref())
        .unwrap_or("");

    // ── Coding Plan 专用路径 ──
    if template_type == TEMPLATE_TYPE_TOKEN_PLAN {
        // 从供应商配置中提取 API Key 和 Base URL
        let settings_config = provider
            .map(|p| &p.settings_config)
            .cloned()
            .unwrap_or_default();
        let env = settings_config.get("env");
        let base_url = env
            .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let api_key = env
            .and_then(|e| {
                e.get("ANTHROPIC_AUTH_TOKEN")
                    .or_else(|| e.get("ANTHROPIC_API_KEY"))
            })
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let quota = crate::services::coding_plan::get_coding_plan_quota(base_url, api_key)
            .await
            .map_err(|e| format!("Failed to query coding plan: {e}"))?;

        // 将 SubscriptionQuota 转换为 UsageResult
        if !quota.success {
            return Ok(crate::provider::UsageResult {
                success: false,
                data: None,
                error: quota.error,
            });
        }

        let data: Vec<crate::provider::UsageData> = quota
            .tiers
            .iter()
            .map(|tier| {
                let total = 100.0;
                let used = tier.utilization;
                let remaining = total - used;
                crate::provider::UsageData {
                    plan_name: Some(tier.name.clone()),
                    remaining: Some(remaining),
                    total: Some(total),
                    used: Some(used),
                    unit: Some("%".to_string()),
                    is_valid: Some(true),
                    invalid_message: None,
                    extra: tier.resets_at.clone(),
                }
            })
            .collect();

        return Ok(crate::provider::UsageResult {
            success: true,
            data: if data.is_empty() { None } else { Some(data) },
            error: None,
        });
    }

    // ── 官方余额查询路径 ──
    if template_type == TEMPLATE_TYPE_BALANCE {
        let settings_config = provider
            .map(|p| &p.settings_config)
            .cloned()
            .unwrap_or_default();
        let env = settings_config.get("env");
        let base_url = env
            .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let api_key = env
            .and_then(|e| {
                e.get("ANTHROPIC_AUTH_TOKEN")
                    .or_else(|| e.get("ANTHROPIC_API_KEY"))
            })
            .and_then(|v| v.as_str())
            .unwrap_or("");

        return crate::services::balance::get_balance(base_url, api_key)
            .await
            .map_err(|e| format!("Failed to query balance: {e}"));
    }

    // ── 通用 JS 脚本路径 ──
    ProviderService::query_usage(state, app_type, provider_id)
        .await
        .map_err(|e| e.to_string())
}

#[allow(non_snake_case)]
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn testUsageScript(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerId: String,
    app: String,
    #[allow(non_snake_case)] scriptCode: String,
    timeout: Option<u64>,
    #[allow(non_snake_case)] apiKey: Option<String>,
    #[allow(non_snake_case)] baseUrl: Option<String>,
    #[allow(non_snake_case)] accessToken: Option<String>,
    #[allow(non_snake_case)] userId: Option<String>,
    #[allow(non_snake_case)] templateType: Option<String>,
) -> Result<crate::provider::UsageResult, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::test_usage_script(
        state.inner(),
        app_type,
        &providerId,
        &scriptCode,
        timeout.unwrap_or(10),
        apiKey.as_deref(),
        baseUrl.as_deref(),
        accessToken.as_deref(),
        userId.as_deref(),
        templateType.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_live_provider_settings(app: String) -> Result<serde_json::Value, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::read_live_settings(app_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_api_endpoints(
    urls: Vec<String>,
    #[allow(non_snake_case)] timeoutSecs: Option<u64>,
) -> Result<Vec<EndpointLatency>, String> {
    SpeedtestService::test_endpoints(urls, timeoutSecs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custom_endpoints(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] providerId: String,
) -> Result<Vec<crate::settings::CustomEndpoint>, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::get_custom_endpoints(state.inner(), app_type, &providerId)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_custom_endpoint(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] providerId: String,
    url: String,
) -> Result<(), String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::add_custom_endpoint(state.inner(), app_type, &providerId, url)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_custom_endpoint(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] providerId: String,
    url: String,
) -> Result<(), String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::remove_custom_endpoint(state.inner(), app_type, &providerId, url)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_endpoint_last_used(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] providerId: String,
    url: String,
) -> Result<(), String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::update_endpoint_last_used(state.inner(), app_type, &providerId, url)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_providers_sort_order(
    state: State<'_, AppState>,
    app: String,
    updates: Vec<ProviderSortUpdate>,
) -> Result<bool, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    ProviderService::update_sort_order(state.inner(), app_type, updates).map_err(|e| e.to_string())
}

/// 将某个供应商配置写入指定项目目录的 .claude/settings.json
///
/// 采用深度合并策略：保留项目文件中已有的其他配置（如 mcpServers），
/// 仅覆盖供应商相关字段（apiKey、baseApiUrl、env 等）。
#[tauri::command]
pub fn switch_provider_for_project(
    state: State<'_, AppState>,
    app: String,
    id: String,
    #[allow(non_snake_case)] projectPath: String,
) -> Result<SwitchResult, String> {
    use crate::services::provider::{json_deep_merge, sanitize_claude_settings_for_live};

    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;

    let provider = state
        .db
        .get_provider_by_id(&id, app_type.as_str())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("供应商不存在: {id}"))?;

    let settings_path = std::path::Path::new(&projectPath)
        .join(".claude")
        .join("settings.json");

    // 读取已有的项目配置（不存在则为空对象）
    let mut existing: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("读取项目配置失败: {e}"))?;
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Object(Default::default()))
    } else {
        serde_json::Value::Object(Default::default())
    };

    // 将供应商配置深度合并到已有配置上
    let sanitized = sanitize_claude_settings_for_live(&provider.settings_config);
    json_deep_merge(&mut existing, &sanitized);

    // 写入项目 settings.json（自动创建 .claude 目录）
    crate::config::write_json_file(&settings_path, &existing).map_err(|e| e.to_string())?;

    Ok(SwitchResult::default())
}

/// 从指定项目目录的 .claude/settings.json 中导入供应商配置。
///
/// 逻辑：
/// 1. 读取项目 settings.json，提取 apiKey + baseApiUrl（含 env 变体）。
/// 2. 若无任何 API 配置则返回空字符串。
/// 3. 与现有供应商逐一比对：key 和 url 都匹配则直接用该供应商。
/// 4. 无匹配时以项目目录名创建新供应商（不写入全局 live 配置）。
/// 5. 将匹配/新建的供应商 ID 记为该项目的当前供应商，并返回 ID。
#[tauri::command]
pub fn import_project_provider(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] projectPath: String,
) -> Result<String, String> {
    import_project_provider_internal(&state, &app, &projectPath)
}

#[cfg_attr(not(feature = "test-hooks"), doc(hidden))]
pub fn import_project_provider_test_hook(
    state: &AppState,
    app: &str,
    project_path: &str,
) -> Result<String, String> {
    import_project_provider_internal(state, app, project_path)
}

/// 解析指定项目当前"实际生效"的供应商：
/// 读取项目 .claude/settings.json，按 key+url 比对现有供应商，
/// 匹配则返回该 ID，不匹配则导入新建返回新 ID，无配置返回空串。
/// 全程不写 project_provider_* 字段（真相源为 live 文件本身）。
#[tauri::command]
pub fn resolve_current_provider_for_project(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] projectPath: String,
) -> Result<String, String> {
    resolve_provider_from_project_internal(&state, &app, &projectPath)
}

#[cfg_attr(not(feature = "test-hooks"), doc(hidden))]
pub fn resolve_current_provider_for_project_test_hook(
    state: &AppState,
    app: &str,
    project_path: &str,
) -> Result<String, String> {
    resolve_provider_from_project_internal(state, app, project_path)
}

/// 从项目 .claude/settings.json 解析"实际生效的供应商"：
/// - 文件不存在 / 读失败 / JSON 解析失败 / 无 api 配置 → 返回空串（容错，不抛错）
/// - key+url 命中现有供应商 → 返回该供应商 ID（不写库、不写字段）
/// - 无命中 → 按"域名_*key末两位"命名规则导入为新供应商 → 返回新 ID（不写字段）
///
/// import 与 resolve 两个命令共用此函数；自此 project_provider_* 字段无写入路径。
fn resolve_provider_from_project_internal(
    state: &AppState,
    app: &str,
    project_path: &str,
) -> Result<String, String> {
    use crate::services::provider::sanitize_claude_settings_for_live;
    use uuid::Uuid;

    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    let project_path = project_path.to_string();

    let settings_path = std::path::Path::new(&project_path)
        .join(".claude")
        .join("settings.json");

    if !settings_path.exists() {
        return Ok(String::new());
    }

    // 容错：读失败 / 解析失败一律降级为空串，不向前端抛错
    let Ok(content) = std::fs::read_to_string(&settings_path) else {
        return Ok(String::new());
    };
    let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Ok(String::new());
    };

    let project_key = extract_api_key_from_settings(&settings);
    let project_url = extract_base_url_from_settings(&settings);

    if project_key.is_none() && project_url.is_none() {
        return Ok(String::new());
    }

    // 比对现有供应商
    let existing = state
        .db
        .get_all_providers(app_type.as_str())
        .map_err(|e| e.to_string())?;
    for (id, provider) in &existing {
        let prov_key = extract_api_key_from_settings(&provider.settings_config);
        let prov_url = extract_base_url_from_settings(&provider.settings_config);
        if prov_key == project_key && prov_url == project_url {
            return Ok(id.clone());
        }
    }

    // 无匹配 —— 导入新建（不写 project_provider_* 字段）
    let provider_name = ProviderService::name_from_settings(&settings).unwrap_or_else(|| {
        project_url
            .as_deref()
            .and_then(|url| ::url::Url::parse(url).ok())
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .filter(|h| !h.is_empty())
            .unwrap_or_else(|| {
                std::path::Path::new(&project_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "导入的项目".to_string())
            })
    });
    let sanitized = sanitize_claude_settings_for_live(&settings);
    let new_id = Uuid::new_v4().to_string();
    let new_provider = crate::provider::Provider {
        id: new_id.clone(),
        name: provider_name,
        settings_config: sanitized,
        website_url: None,
        category: Some("imported".to_string()),
        created_at: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        ),
        sort_index: None,
        notes: None,
        meta: None,
        icon: None,
        icon_color: None,
    };
    state
        .db
        .save_provider(app_type.as_str(), &new_provider)
        .map_err(|e| e.to_string())?;
    Ok(new_id)
}

fn import_project_provider_internal(
    state: &AppState,
    app: &str,
    project_path: &str,
) -> Result<String, String> {
    resolve_provider_from_project_internal(state, app, project_path)
}

/// 从 Claude settings JSON 中提取 API key（支持 apiKey / env.ANTHROPIC_AUTH_TOKEN / env.ANTHROPIC_API_KEY）
pub(crate) fn extract_api_key_from_settings(settings: &serde_json::Value) -> Option<String> {
    if let Some(key) = settings.get("apiKey").and_then(|v| v.as_str()) {
        if !key.trim().is_empty() {
            return Some(key.to_string());
        }
    }
    let env = settings.get("env")?;
    for field in &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"] {
        if let Some(key) = env.get(field).and_then(|v| v.as_str()) {
            if !key.trim().is_empty() {
                return Some(key.to_string());
            }
        }
    }
    None
}

/// 从 Claude settings JSON 中提取 base URL（支持 baseApiUrl / env.ANTHROPIC_BASE_URL）
pub(crate) fn extract_base_url_from_settings(settings: &serde_json::Value) -> Option<String> {
    if let Some(url) = settings.get("baseApiUrl").and_then(|v| v.as_str()) {
        if !url.trim().is_empty() {
            return Some(url.to_string());
        }
    }
    if let Some(url) = settings
        .get("env")
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .and_then(|v| v.as_str())
    {
        if !url.trim().is_empty() {
            return Some(url.to_string());
        }
    }
    None
}

#[cfg(test)]
mod project_import_tests {
    use super::{extract_api_key_from_settings, extract_base_url_from_settings};
    use serde_json::json;

    #[test]
    fn extract_api_key_from_env_auth_token() {
        let settings = json!({
            "env": { "ANTHROPIC_AUTH_TOKEN": "sk-test-123" }
        });
        assert_eq!(
            extract_api_key_from_settings(&settings),
            Some("sk-test-123".to_string())
        );
    }

    #[test]
    fn extract_api_key_from_env_api_key() {
        let settings = json!({
            "env": { "ANTHROPIC_API_KEY": "sk-test-456" }
        });
        assert_eq!(
            extract_api_key_from_settings(&settings),
            Some("sk-test-456".to_string())
        );
    }

    #[test]
    fn extract_api_key_from_top_level_field() {
        let settings = json!({ "apiKey": "sk-top-level" });
        assert_eq!(
            extract_api_key_from_settings(&settings),
            Some("sk-top-level".to_string())
        );
    }

    #[test]
    fn extract_base_url_from_env() {
        let settings = json!({
            "env": { "ANTHROPIC_BASE_URL": "https://api.example.com/v1" }
        });
        assert_eq!(
            extract_base_url_from_settings(&settings),
            Some("https://api.example.com/v1".to_string())
        );
    }

    #[test]
    fn extract_returns_none_when_missing() {
        let settings = json!({ "model": "claude-3" });
        assert_eq!(extract_api_key_from_settings(&settings), None);
        assert_eq!(extract_base_url_from_settings(&settings), None);
    }

    #[test]
    fn extract_ignores_empty_string_values() {
        let settings = json!({
            "env": { "ANTHROPIC_AUTH_TOKEN": "", "ANTHROPIC_BASE_URL": "" }
        });
        assert_eq!(extract_api_key_from_settings(&settings), None);
        assert_eq!(extract_base_url_from_settings(&settings), None);
    }
}

/// 获取指定项目目录当前使用的供应商 ID（空字符串表示未设置）
#[tauri::command]
pub fn get_current_provider_for_project(
    state: State<'_, AppState>,
    app: String,
    #[allow(non_snake_case)] projectPath: String,
) -> Result<String, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    let config_key = format!("project_provider_{}_{}", app_type.as_str(), projectPath);
    Ok(state
        .db
        .get_setting(&config_key)
        .map_err(|e| e.to_string())?
        .unwrap_or_default())
}
