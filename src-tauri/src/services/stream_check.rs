//! 流式健康检查服务
//!
//! 使用流式 API 进行快速健康检查，只需接收首个 chunk 即判定成功。

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

use crate::app_config::AppType;
use crate::error::AppError;
use crate::provider::Provider;
use crate::providers::transform::anthropic_to_openai;
use crate::providers::transform_responses::anthropic_to_responses;
use crate::providers::{get_adapter, AuthInfo, AuthStrategy, ClaudeAdapter, ProviderAdapter};

/// 健康状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Operational,
    Degraded,
    Failed,
}

/// 流式检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCheckConfig {
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub degraded_threshold_ms: u64,
    pub claude_model: String,
    #[serde(default = "default_test_prompt")]
    pub test_prompt: String,
}

fn default_test_prompt() -> String {
    "Who are you?".to_string()
}

impl Default for StreamCheckConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 45,
            max_retries: 2,
            degraded_threshold_ms: 6000,
            claude_model: "claude-haiku-4-5-20251001".to_string(),
            test_prompt: default_test_prompt(),
        }
    }
}

/// 流式检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCheckResult {
    pub status: HealthStatus,
    pub success: bool,
    pub message: String,
    pub response_time_ms: Option<u64>,
    pub http_status: Option<u16>,
    pub model_used: String,
    pub tested_at: i64,
    pub retry_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_category: Option<String>,
}

/// 流式健康检查服务
pub struct StreamCheckService;

impl StreamCheckService {
    pub async fn check_with_retry(
        app_type: &AppType,
        provider: &Provider,
        config: &StreamCheckConfig,
        auth_override: Option<AuthInfo>,
        base_url_override: Option<String>,
        claude_api_format_override: Option<String>,
    ) -> Result<StreamCheckResult, AppError> {
        let effective_config = Self::merge_provider_config(provider, config);
        let mut last_result = None;

        for attempt in 0..=effective_config.max_retries {
            let result = Self::check_once(
                app_type,
                provider,
                &effective_config,
                auth_override.clone(),
                base_url_override.clone(),
                claude_api_format_override.clone(),
            )
            .await;

            match &result {
                Ok(r) if r.success => {
                    return Ok(StreamCheckResult {
                        retry_count: attempt,
                        ..r.clone()
                    });
                }
                Ok(r) => {
                    if Self::should_retry(&r.message) && attempt < effective_config.max_retries {
                        last_result = Some(r.clone());
                        continue;
                    }
                    return Ok(StreamCheckResult {
                        retry_count: attempt,
                        ..r.clone()
                    });
                }
                Err(e) => {
                    if Self::should_retry(&e.to_string()) && attempt < effective_config.max_retries
                    {
                        continue;
                    }
                    return Err(AppError::Message(e.to_string()));
                }
            }
        }

        Ok(last_result.unwrap_or_else(|| StreamCheckResult {
            status: HealthStatus::Failed,
            success: false,
            message: "Check failed".to_string(),
            response_time_ms: None,
            http_status: None,
            model_used: String::new(),
            tested_at: chrono::Utc::now().timestamp(),
            retry_count: effective_config.max_retries,
            error_category: None,
        }))
    }

    fn merge_provider_config(
        provider: &Provider,
        global_config: &StreamCheckConfig,
    ) -> StreamCheckConfig {
        let test_config = provider
            .meta
            .as_ref()
            .and_then(|m| m.test_config.as_ref())
            .filter(|tc| tc.enabled);

        match test_config {
            Some(tc) => StreamCheckConfig {
                timeout_secs: tc.timeout_secs.unwrap_or(global_config.timeout_secs),
                max_retries: tc.max_retries.unwrap_or(global_config.max_retries),
                degraded_threshold_ms: tc
                    .degraded_threshold_ms
                    .unwrap_or(global_config.degraded_threshold_ms),
                claude_model: tc
                    .test_model
                    .clone()
                    .unwrap_or_else(|| global_config.claude_model.clone()),
                test_prompt: tc
                    .test_prompt
                    .clone()
                    .unwrap_or_else(|| global_config.test_prompt.clone()),
            },
            None => global_config.clone(),
        }
    }

    async fn check_once(
        app_type: &AppType,
        provider: &Provider,
        config: &StreamCheckConfig,
        auth_override: Option<AuthInfo>,
        base_url_override: Option<String>,
        claude_api_format_override: Option<String>,
    ) -> Result<StreamCheckResult, AppError> {
        let start = Instant::now();
        let adapter: Box<dyn ProviderAdapter> = get_adapter(app_type);

        let base_url = match base_url_override {
            Some(base_url) => base_url,
            None => adapter
                .extract_base_url(provider)
                .map_err(|e| AppError::Message(format!("Failed to extract base_url: {e}")))?,
        };

        let auth = auth_override
            .or_else(|| adapter.extract_auth(provider))
            .ok_or_else(|| AppError::Message("API Key not found".to_string()))?;

        let client = crate::http_client::get();
        let request_timeout = std::time::Duration::from_secs(config.timeout_secs);
        let model_to_test = Self::resolve_test_model(app_type, provider, config);
        let result = Self::check_claude_stream(
            &client,
            &base_url,
            &auth,
            &model_to_test,
            &config.test_prompt,
            request_timeout,
            provider,
            claude_api_format_override.as_deref(),
            None,
        )
        .await;

        let response_time = start.elapsed().as_millis() as u64;
        Ok(Self::build_stream_check_result(
            result,
            response_time,
            config.degraded_threshold_ms,
            &model_to_test,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    async fn check_claude_stream(
        client: &Client,
        base_url: &str,
        auth: &AuthInfo,
        model: &str,
        test_prompt: &str,
        timeout: std::time::Duration,
        provider: &Provider,
        claude_api_format_override: Option<&str>,
        extra_headers: Option<&serde_json::Map<String, serde_json::Value>>,
    ) -> Result<(u16, String), AppError> {
        let base = base_url.trim_end_matches('/');
        let api_format = provider
            .meta
            .as_ref()
            .and_then(|m| m.api_format.as_deref())
            .or_else(|| {
                provider
                    .settings_config
                    .get("api_format")
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("anthropic");
        let effective_api_format = claude_api_format_override.unwrap_or(api_format);
        let is_full_url = provider
            .meta
            .as_ref()
            .and_then(|meta| meta.is_full_url)
            .unwrap_or(false);
        let is_openai_chat = effective_api_format == "openai_chat";
        let is_openai_responses = effective_api_format == "openai_responses";
        let url = Self::resolve_claude_stream_url(
            base,
            auth.strategy,
            effective_api_format,
            is_full_url,
            model,
        );
        let max_tokens = if is_openai_responses { 16 } else { 1 };
        let anthropic_body = json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": [{ "role": "user", "content": test_prompt }],
            "stream": true
        });
        let body = if is_openai_responses {
            anthropic_to_responses(anthropic_body, Some(&provider.id), false, false)
                .map_err(|e| AppError::Message(format!("Failed to build test request: {e}")))?
        } else if is_openai_chat {
            anthropic_to_openai(anthropic_body)
                .map_err(|e| AppError::Message(format!("Failed to build test request: {e}")))?
        } else {
            anthropic_body
        };

        let mut request_builder = client.post(&url);
        if is_openai_chat || is_openai_responses {
            request_builder = request_builder
                .header("authorization", format!("Bearer {}", auth.api_key))
                .header("content-type", "application/json")
                .header("accept", "text/event-stream")
                .header("accept-encoding", "identity");
        } else {
            let os_name = Self::get_os_name();
            let arch_name = Self::get_arch_name();
            for (name, value) in ClaudeAdapter::new().get_auth_headers(auth) {
                request_builder = request_builder.header(name, value);
            }
            request_builder = request_builder
                .header("anthropic-version", "2023-06-01")
                .header(
                    "anthropic-beta",
                    "claude-code-20250219,interleaved-thinking-2025-05-14",
                )
                .header("anthropic-dangerous-direct-browser-access", "true")
                .header("content-type", "application/json")
                .header("accept", "application/json")
                .header("accept-encoding", "identity")
                .header("accept-language", "*")
                .header("user-agent", "claude-cli/2.1.2 (external, cli)")
                .header("x-app", "cli")
                .header("x-stainless-lang", "js")
                .header("x-stainless-package-version", "0.70.0")
                .header("x-stainless-os", os_name)
                .header("x-stainless-arch", arch_name)
                .header("x-stainless-runtime", "node")
                .header("x-stainless-runtime-version", "v22.20.0")
                .header("x-stainless-retry-count", "0")
                .header("x-stainless-timeout", "600")
                .header("sec-fetch-mode", "cors");
        }

        if let Some(headers) = extra_headers {
            for (key, value) in headers {
                if let Some(v) = value.as_str() {
                    request_builder = request_builder.header(key.as_str(), v);
                }
            }
        }

        let response = request_builder
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Self::http_status_error(status, error_text));
        }

        let mut stream = response.bytes_stream();
        if let Some(chunk) = stream.next().await {
            match chunk {
                Ok(_) => Ok((status, model.to_string())),
                Err(e) => Err(AppError::Message(format!("Stream read failed: {e}"))),
            }
        } else {
            Err(AppError::Message("No response data received".to_string()))
        }
    }

    fn build_stream_check_result(
        result: Result<(u16, String), AppError>,
        response_time: u64,
        degraded_threshold_ms: u64,
        model_tested: &str,
    ) -> StreamCheckResult {
        let tested_at = chrono::Utc::now().timestamp();
        match result {
            Ok((status_code, model)) => StreamCheckResult {
                status: Self::determine_status(response_time, degraded_threshold_ms),
                success: true,
                message: "Check succeeded".to_string(),
                response_time_ms: Some(response_time),
                http_status: Some(status_code),
                model_used: model,
                tested_at,
                retry_count: 0,
                error_category: None,
            },
            Err(e) => {
                let (http_status, message, error_category) = match &e {
                    AppError::HttpStatus { status, body } => {
                        let category = Self::detect_error_category(*status, body);
                        (
                            Some(*status),
                            Self::classify_http_status(*status).to_string(),
                            category.map(|s| s.to_string()),
                        )
                    }
                    _ => (None, e.to_string(), None),
                };
                StreamCheckResult {
                    status: HealthStatus::Failed,
                    success: false,
                    message,
                    response_time_ms: Some(response_time),
                    http_status,
                    model_used: model_tested.to_string(),
                    tested_at,
                    retry_count: 0,
                    error_category,
                }
            }
        }
    }

    pub(crate) fn detect_error_category(status: u16, body: &str) -> Option<&'static str> {
        if !(400..500).contains(&status) {
            return None;
        }
        let lower = body.to_lowercase();
        let qianfan_quota_indicators = [
            "coding_plan_hour_quota_exceeded",
            "coding_plan_week_quota_exceeded",
            "coding_plan_month_quota_exceeded",
        ];
        if qianfan_quota_indicators.iter().any(|s| lower.contains(s)) {
            return Some("quotaExceeded");
        }
        if !lower.contains("model") {
            return None;
        }
        let indicators = [
            "model_not_found",
            "model not found",
            "does not exist",
            "invalid_model",
            "invalid model",
            "unknown_model",
            "unknown model",
            "is not a valid model",
            "not_found_error",
        ];
        if indicators.iter().any(|s| lower.contains(s)) {
            return Some("modelNotFound");
        }
        None
    }

    fn determine_status(latency_ms: u64, threshold: u64) -> HealthStatus {
        if latency_ms <= threshold {
            HealthStatus::Operational
        } else {
            HealthStatus::Degraded
        }
    }

    fn should_retry(msg: &str) -> bool {
        let lower = msg.to_lowercase();
        lower.contains("timeout") || lower.contains("abort") || lower.contains("timed out")
    }

    fn map_request_error(e: reqwest::Error) -> AppError {
        if e.is_timeout() {
            AppError::Message("Request timeout".to_string())
        } else if e.is_connect() {
            AppError::Message(format!("Connection failed: {e}"))
        } else {
            AppError::Message(e.to_string())
        }
    }

    fn http_status_error(status: u16, body: String) -> AppError {
        let body = if body.len() > 200 {
            let mut end = 200;
            while end > 0 && !body.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}…", &body[..end])
        } else {
            body
        };
        AppError::HttpStatus { status, body }
    }

    pub(crate) fn classify_http_status(status: u16) -> &'static str {
        match status {
            400 => "Bad request (400)",
            401 => "Auth rejected (401)",
            402 => "Payment required (402)",
            403 => "Access denied (403)",
            404 => "Not found (404)",
            429 => "Rate limited (429)",
            500 => "Internal server error (500)",
            502 => "Bad gateway (502)",
            503 => "Service unavailable (503)",
            504 => "Gateway timeout (504)",
            s if (500..600).contains(&s) => "Server error",
            _ => "HTTP error",
        }
    }

    fn resolve_test_model(
        app_type: &AppType,
        provider: &Provider,
        config: &StreamCheckConfig,
    ) -> String {
        match app_type {
            AppType::Claude => Self::extract_env_model(provider, "ANTHROPIC_MODEL")
                .unwrap_or_else(|| config.claude_model.clone()),
        }
    }

    fn extract_env_model(provider: &Provider, key: &str) -> Option<String> {
        provider
            .settings_config
            .get("env")
            .and_then(|env| env.get(key))
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn get_os_name() -> &'static str {
        match std::env::consts::OS {
            "macos" => "MacOS",
            "linux" => "Linux",
            "windows" => "Windows",
            other => other,
        }
    }

    fn get_arch_name() -> &'static str {
        match std::env::consts::ARCH {
            "aarch64" => "arm64",
            "x86_64" => "x86_64",
            "x86" => "x86",
            other => other,
        }
    }

    fn resolve_claude_stream_url(
        base_url: &str,
        _auth_strategy: AuthStrategy,
        api_format: &str,
        is_full_url: bool,
        _model: &str,
    ) -> String {
        if is_full_url {
            return base_url.to_string();
        }

        let base = base_url.trim_end_matches('/');
        if api_format == "openai_responses" {
            if base.ends_with("/v1") {
                format!("{base}/responses")
            } else {
                format!("{base}/v1/responses")
            }
        } else if api_format == "openai_chat" {
            if base.ends_with("/v1") {
                format!("{base}/chat/completions")
            } else {
                format!("{base}/v1/chat/completions")
            }
        } else if base.ends_with("/v1") {
            format!("{base}/messages")
        } else {
            format!("{base}/v1/messages")
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_effective_test_model(
        app_type: &AppType,
        provider: &Provider,
        config: &StreamCheckConfig,
    ) -> String {
        let effective_config = Self::merge_provider_config(provider, config);
        Self::resolve_test_model(app_type, provider, &effective_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_status() {
        assert_eq!(
            StreamCheckService::determine_status(3000, 6000),
            HealthStatus::Operational
        );
        assert_eq!(
            StreamCheckService::determine_status(6000, 6000),
            HealthStatus::Operational
        );
        assert_eq!(
            StreamCheckService::determine_status(6001, 6000),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn test_should_retry() {
        assert!(StreamCheckService::should_retry("Request timeout"));
        assert!(StreamCheckService::should_retry("request timed out"));
        assert!(StreamCheckService::should_retry("connection abort"));
        assert!(!StreamCheckService::should_retry("API Key invalid"));
    }

    #[test]
    fn test_default_config() {
        let config = StreamCheckConfig::default();
        assert_eq!(config.timeout_secs, 45);
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.degraded_threshold_ms, 6000);
    }

    #[test]
    fn test_detect_model_not_found() {
        let openai_404 = r#"{"error":{"message":"The model `gpt-5.1-codex` does not exist or you do not have access to it","type":"invalid_request_error","param":null,"code":"model_not_found"}}"#;
        assert_eq!(
            StreamCheckService::detect_error_category(404, openai_404),
            Some("modelNotFound")
        );

        let anthropic_404 = r#"{"type":"error","error":{"type":"not_found_error","message":"model: claude-deprecated"}}"#;
        assert_eq!(
            StreamCheckService::detect_error_category(404, anthropic_404),
            Some("modelNotFound")
        );

        let bad_req = r#"{"error":{"message":"invalid model specified"}}"#;
        assert_eq!(
            StreamCheckService::detect_error_category(400, bad_req),
            Some("modelNotFound")
        );

        let generic_404 = r#"{"error":"Not Found"}"#;
        assert_eq!(
            StreamCheckService::detect_error_category(404, generic_404),
            None
        );

        let server_error = r#"{"error":"model does not exist"}"#;
        assert_eq!(
            StreamCheckService::detect_error_category(500, server_error),
            None
        );

        let auth_err = r#"{"error":"Invalid API key"}"#;
        assert_eq!(
            StreamCheckService::detect_error_category(401, auth_err),
            None
        );
    }

    #[test]
    fn test_detect_qianfan_coding_plan_quota_errors() {
        let cases = [
            r#"{"error":{"code":"coding_plan_hour_quota_exceeded","message":"hour quota exceeded"}}"#,
            r#"{"error":{"code":"coding_plan_week_quota_exceeded","message":"week quota exceeded"}}"#,
            r#"{"error":{"code":"coding_plan_month_quota_exceeded","message":"month quota exceeded"}}"#,
        ];

        for body in cases {
            assert_eq!(
                StreamCheckService::detect_error_category(429, body),
                Some("quotaExceeded")
            );
        }
    }

    #[test]
    fn test_get_os_name() {
        let os_name = StreamCheckService::get_os_name();
        assert!(!os_name.is_empty());
        #[cfg(target_os = "macos")]
        assert_eq!(os_name, "MacOS");
        #[cfg(target_os = "linux")]
        assert_eq!(os_name, "Linux");
        #[cfg(target_os = "windows")]
        assert_eq!(os_name, "Windows");
    }

    #[test]
    fn test_get_arch_name() {
        let arch_name = StreamCheckService::get_arch_name();
        assert!(!arch_name.is_empty());
        #[cfg(target_arch = "aarch64")]
        assert_eq!(arch_name, "arm64");
        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch_name, "x86_64");
    }

    #[test]
    fn test_auth_strategy_imports() {
        let anthropic = AuthStrategy::Anthropic;
        let claude_auth = AuthStrategy::ClaudeAuth;
        let bearer = AuthStrategy::Bearer;

        assert_ne!(anthropic, claude_auth);
        assert_ne!(anthropic, bearer);
        assert_ne!(claude_auth, bearer);
        assert_eq!(anthropic, AuthStrategy::Anthropic);
        assert_eq!(claude_auth, AuthStrategy::ClaudeAuth);
        assert_eq!(bearer, AuthStrategy::Bearer);
    }

    #[test]
    fn test_resolve_claude_stream_url_for_full_url_mode() {
        let url = StreamCheckService::resolve_claude_stream_url(
            "https://relay.example/v1/chat/completions",
            AuthStrategy::Bearer,
            "openai_chat",
            true,
            "gpt-5.4",
        );

        assert_eq!(url, "https://relay.example/v1/chat/completions");
    }

    #[test]
    fn test_resolve_claude_stream_url_for_openai_chat() {
        let url = StreamCheckService::resolve_claude_stream_url(
            "https://example.com/v1",
            AuthStrategy::Bearer,
            "openai_chat",
            false,
            "gpt-5.4",
        );

        assert_eq!(url, "https://example.com/v1/chat/completions");
    }

    #[test]
    fn test_resolve_claude_stream_url_for_openai_responses() {
        let url = StreamCheckService::resolve_claude_stream_url(
            "https://example.com/v1",
            AuthStrategy::Bearer,
            "openai_responses",
            false,
            "gpt-5.4",
        );

        assert_eq!(url, "https://example.com/v1/responses");
    }

    #[test]
    fn test_resolve_claude_stream_url_for_anthropic() {
        let url = StreamCheckService::resolve_claude_stream_url(
            "https://api.anthropic.com",
            AuthStrategy::Anthropic,
            "anthropic",
            false,
            "claude-sonnet-4-6",
        );

        assert_eq!(url, "https://api.anthropic.com/v1/messages");
    }
}
