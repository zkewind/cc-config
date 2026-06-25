//! Claude (Anthropic) Provider Adapter
//!
//! 支持透传模式和 OpenAI 格式转换模式
//!
//! ## API 格式
//! - **anthropic** (默认): Anthropic Messages API 格式，直接透传
//! - **openai_chat**: OpenAI Chat Completions 格式，需要 Anthropic ↔ OpenAI 转换
//! - **openai_responses**: OpenAI Responses API 格式，需要 Anthropic ↔ Responses 转换
//!
//! ## 认证模式
//! - **Claude**: Anthropic 官方 API (x-api-key + anthropic-version)
//! - **ClaudeAuth**: 中转服务 (仅 Bearer 认证，无 x-api-key)
//! - **OpenRouter**: 已支持 Claude Code 兼容接口，默认透传

use super::{AuthInfo, AuthStrategy, ProviderAdapter, ProviderType};
use crate::provider::Provider;
use crate::providers::proxy_error::ProxyError;

/// 获取 Claude 供应商的 API 格式
///
/// 供 handler/forwarder 外部使用的公开函数。
/// 优先级：meta.apiFormat > settings_config.api_format > openrouter_compat_mode > 默认 "anthropic"
#[allow(dead_code)]
pub fn get_claude_api_format(provider: &Provider) -> &'static str {
    // 1) Preferred: meta.apiFormat (SSOT, never written to Claude Code config)
    if let Some(meta) = provider.meta.as_ref() {
        if let Some(api_format) = meta.api_format.as_deref() {
            return match api_format {
                "openai_chat" => "openai_chat",
                "openai_responses" => "openai_responses",
                _ => "anthropic",
            };
        }
    }

    // 2) Backward compatibility: legacy settings_config.api_format
    if let Some(api_format) = provider
        .settings_config
        .get("api_format")
        .and_then(|v| v.as_str())
    {
        return match api_format {
            "openai_chat" => "openai_chat",
            "openai_responses" => "openai_responses",
            _ => "anthropic",
        };
    }

    // 3) Backward compatibility: legacy openrouter_compat_mode (bool/number/string)
    let raw = provider.settings_config.get("openrouter_compat_mode");
    let enabled = match raw {
        Some(serde_json::Value::Bool(v)) => *v,
        Some(serde_json::Value::Number(num)) => num.as_i64().unwrap_or(0) != 0,
        Some(serde_json::Value::String(value)) => {
            let normalized = value.trim().to_lowercase();
            normalized == "true" || normalized == "1"
        }
        _ => false,
    };

    if enabled {
        "openai_chat"
    } else {
        "anthropic"
    }
}

#[allow(dead_code)]
pub fn claude_api_format_needs_transform(api_format: &str) -> bool {
    matches!(api_format, "openai_chat" | "openai_responses")
}

#[allow(dead_code)]
fn is_reasoning_content_compatible_identifier(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("moonshot") || value.contains("kimi") || value.contains("deepseek")
}

#[allow(dead_code)]
fn should_preserve_reasoning_content_for_openai_chat(
    provider: &Provider,
    body: &serde_json::Value,
) -> bool {
    if body
        .get("model")
        .and_then(|m| m.as_str())
        .is_some_and(is_reasoning_content_compatible_identifier)
    {
        return true;
    }

    let settings = &provider.settings_config;
    let base_urls = [
        settings
            .get("env")
            .and_then(|env| env.get("ANTHROPIC_BASE_URL"))
            .and_then(|v| v.as_str()),
        settings.get("base_url").and_then(|v| v.as_str()),
        settings.get("baseURL").and_then(|v| v.as_str()),
        settings.get("apiEndpoint").and_then(|v| v.as_str()),
    ];

    base_urls
        .into_iter()
        .flatten()
        .any(is_reasoning_content_compatible_identifier)
}

#[allow(dead_code)]
pub fn transform_claude_request_for_api_format(
    body: serde_json::Value,
    provider: &Provider,
    api_format: &str,
) -> Result<serde_json::Value, ProxyError> {
    match api_format {
        "openai_responses" => super::transform_responses::anthropic_to_responses(
            body,
            Some(&provider.id),
            false,
            false,
        ),
        "openai_chat" => {
            let preserve_reasoning_content =
                should_preserve_reasoning_content_for_openai_chat(provider, &body);
            let mut result = super::transform::anthropic_to_openai_with_reasoning_content(
                body,
                preserve_reasoning_content,
            )?;
            // Inject prompt_cache_key only if explicitly configured in meta
            if let Some(key) = provider
                .meta
                .as_ref()
                .and_then(|m| m.prompt_cache_key.as_deref())
            {
                result["prompt_cache_key"] = serde_json::json!(key);
            }
            Ok(result)
        }
        _ => Ok(body),
    }
}

/// Claude 适配器
pub struct ClaudeAdapter;

impl ClaudeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// 获取供应商类型
    ///
    /// 根据 base_url 和 auth_mode 检测具体的供应商类型：
    /// - OpenRouter: base_url 包含 openrouter.ai
    /// - ClaudeAuth: auth_mode 为 bearer_only
    /// - Claude: 默认 Anthropic 官方
    pub fn provider_type(&self, provider: &Provider) -> ProviderType {
        // 检测 OpenRouter
        if self.is_openrouter(provider) {
            return ProviderType::OpenRouter;
        }

        // 检测 ClaudeAuth (仅 Bearer 认证)
        if self.is_bearer_only_mode(provider) {
            return ProviderType::ClaudeAuth;
        }

        ProviderType::Claude
    }

    /// 检测是否使用 OpenRouter
    fn is_openrouter(&self, provider: &Provider) -> bool {
        if let Ok(base_url) = self.extract_base_url(provider) {
            return base_url.contains("openrouter.ai");
        }
        false
    }

    /// 获取 API 格式
    ///
    /// 从 provider.meta.api_format 读取格式设置：
    /// - "anthropic" (默认): Anthropic Messages API 格式，直接透传
    /// - "openai_chat": OpenAI Chat Completions 格式，需要格式转换
    /// - "openai_responses": OpenAI Responses API 格式，需要格式转换
    #[allow(dead_code)]
    fn get_api_format(&self, provider: &Provider) -> &'static str {
        get_claude_api_format(provider)
    }

    /// 检测是否为仅 Bearer 认证模式
    fn is_bearer_only_mode(&self, provider: &Provider) -> bool {
        // 检查 settings_config 中的 auth_mode
        if let Some(auth_mode) = provider
            .settings_config
            .get("auth_mode")
            .and_then(|v| v.as_str())
        {
            if auth_mode == "bearer_only" {
                return true;
            }
        }

        // 检查 env 中的 AUTH_MODE
        if let Some(env) = provider.settings_config.get("env") {
            if let Some(auth_mode) = env.get("AUTH_MODE").and_then(|v| v.as_str()) {
                if auth_mode == "bearer_only" {
                    return true;
                }
            }
        }

        false
    }

    /// 从 Provider 配置中提取 API Key
    fn extract_key(&self, provider: &Provider) -> Option<String> {
        if let Some(env) = provider.settings_config.get("env") {
            // Anthropic 标准 key
            if let Some(key) = env
                .get("ANTHROPIC_AUTH_TOKEN")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                log::debug!("[Claude] 使用 ANTHROPIC_AUTH_TOKEN");
                return Some(key.to_string());
            }
            if let Some(key) = env
                .get("ANTHROPIC_API_KEY")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                log::debug!("[Claude] 使用 ANTHROPIC_API_KEY");
                return Some(key.to_string());
            }
            // OpenRouter key
            if let Some(key) = env
                .get("OPENROUTER_API_KEY")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                log::debug!("[Claude] 使用 OPENROUTER_API_KEY");
                return Some(key.to_string());
            }
            // 备选 OpenAI key (用于 OpenRouter)
            if let Some(key) = env
                .get("OPENAI_API_KEY")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                log::debug!("[Claude] 使用 OPENAI_API_KEY");
                return Some(key.to_string());
            }
            // Gemini Native key
            if let Some(key) = env
                .get("GEMINI_API_KEY")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                log::debug!("[Claude] 使用 GEMINI_API_KEY");
                return Some(key.to_string());
            }
        }

        // 尝试直接获取
        if let Some(key) = provider
            .settings_config
            .get("apiKey")
            .or_else(|| provider.settings_config.get("api_key"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            log::debug!("[Claude] 使用 apiKey/api_key");
            return Some(key.to_string());
        }

        log::warn!("[Claude] 未找到有效的 API Key");
        None
    }

    /// 根据 env 中填写的变量名推断 Anthropic 默认走哪种鉴权策略。
    ///
    /// 与 Anthropic SDK 原生语义保持一致：
    /// - `ANTHROPIC_AUTH_TOKEN` → `ClaudeAuth`（发送 `Authorization: Bearer`）
    /// - `ANTHROPIC_API_KEY`    → `Anthropic` （发送 `x-api-key`）
    ///
    /// 优先级与 [`extract_key`] 一致；两者都缺时返回 `None` 由调用方决定 fallback。
    fn infer_anthropic_auth_strategy(&self, provider: &Provider) -> Option<AuthStrategy> {
        let env = provider.settings_config.get("env")?;

        let has_value = |key: &str| -> bool {
            env.get(key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some()
        };

        if has_value("ANTHROPIC_AUTH_TOKEN") {
            return Some(AuthStrategy::ClaudeAuth);
        }
        if has_value("ANTHROPIC_API_KEY") {
            return Some(AuthStrategy::Anthropic);
        }
        None
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "Claude"
    }

    fn extract_base_url(&self, provider: &Provider) -> Result<String, ProxyError> {
        // 1. 从 env 中获取
        if let Some(env) = provider.settings_config.get("env") {
            if let Some(url) = env.get("ANTHROPIC_BASE_URL").and_then(|v| v.as_str()) {
                return Ok(url.trim_end_matches('/').to_string());
            }
        }

        // 2. 尝试直接获取
        if let Some(url) = provider
            .settings_config
            .get("base_url")
            .and_then(|v| v.as_str())
        {
            return Ok(url.trim_end_matches('/').to_string());
        }

        if let Some(url) = provider
            .settings_config
            .get("baseURL")
            .and_then(|v| v.as_str())
        {
            return Ok(url.trim_end_matches('/').to_string());
        }

        if let Some(url) = provider
            .settings_config
            .get("apiEndpoint")
            .and_then(|v| v.as_str())
        {
            return Ok(url.trim_end_matches('/').to_string());
        }

        Err(ProxyError::ConfigError(
            "Claude Provider 缺少 base_url 配置".to_string(),
        ))
    }

    fn extract_auth(&self, provider: &Provider) -> Option<AuthInfo> {
        let provider_type = self.provider_type(provider);

        let key = self.extract_key(provider)?;

        match provider_type {
            ProviderType::OpenRouter => Some(AuthInfo::new(key, AuthStrategy::Bearer)),
            ProviderType::ClaudeAuth => Some(AuthInfo::new(key, AuthStrategy::ClaudeAuth)),
            ProviderType::Claude => {
                // 按 env 中的变量名推断鉴权策略，对齐 Anthropic SDK 语义：
                // ANTHROPIC_AUTH_TOKEN → Authorization: Bearer
                // ANTHROPIC_API_KEY    → x-api-key
                // 其他来源（apiKey 直填等）默认走 x-api-key（Anthropic 官方协议）。
                let strategy = self
                    .infer_anthropic_auth_strategy(provider)
                    .unwrap_or(AuthStrategy::Anthropic);
                Some(AuthInfo::new(key, strategy))
            }
        }
    }

    fn build_url(&self, base_url: &str, endpoint: &str) -> String {
        // NOTE:
        // 过去 OpenRouter 只有 OpenAI Chat Completions 兼容接口，需要把 Claude 的 `/v1/messages`
        // 映射到 `/v1/chat/completions`，并做 Anthropic ↔ OpenAI 的格式转换。
        //
        // 现在 OpenRouter 已推出 Claude Code 兼容接口，因此默认直接透传 endpoint。
        // 如需回退旧逻辑，可在 forwarder 中根据 needs_transform 改写 endpoint。
        //
        let mut base = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        // 去除重复的 /v1/v1（可能由 base_url 与 endpoint 都带版本导致）
        while base.contains("/v1/v1") {
            base = base.replace("/v1/v1", "/v1");
        }

        base
    }

    fn get_auth_headers(&self, auth: &AuthInfo) -> Vec<(http::HeaderName, http::HeaderValue)> {
        use http::{HeaderName, HeaderValue};
        // 注意：anthropic-version 由 forwarder.rs 统一处理（透传客户端值或设置默认值）
        let bearer = format!("Bearer {}", auth.api_key);
        match auth.strategy {
            AuthStrategy::Anthropic => {
                vec![(
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_str(&auth.api_key).unwrap(),
                )]
            }
            AuthStrategy::ClaudeAuth | AuthStrategy::Bearer => {
                vec![(
                    HeaderName::from_static("authorization"),
                    HeaderValue::from_str(&bearer).unwrap(),
                )]
            }
            AuthStrategy::Google => vec![(
                HeaderName::from_static("x-goog-api-key"),
                HeaderValue::from_str(&auth.api_key).unwrap(),
            )],
            AuthStrategy::GoogleOAuth => {
                let token = auth.access_token.as_ref().unwrap_or(&auth.api_key);
                vec![
                    (
                        HeaderName::from_static("authorization"),
                        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
                    ),
                    (
                        HeaderName::from_static("x-goog-api-client"),
                        HeaderValue::from_static("GeminiCLI/1.0"),
                    ),
                ]
            }
        }
    }

    fn needs_transform(&self, provider: &Provider) -> bool {
        // 根据 api_format 配置决定是否需要格式转换
        // - "anthropic" (默认): 直接透传，无需转换
        // - "openai_chat": 需要 Anthropic ↔ OpenAI Chat Completions 格式转换
        // - "openai_responses": 需要 Anthropic ↔ OpenAI Responses API 格式转换
        matches!(
            self.get_api_format(provider),
            "openai_chat" | "openai_responses"
        )
    }

    fn transform_request(
        &self,
        body: serde_json::Value,
        provider: &Provider,
    ) -> Result<serde_json::Value, ProxyError> {
        transform_claude_request_for_api_format(body, provider, self.get_api_format(provider))
    }

    fn transform_response(&self, body: serde_json::Value) -> Result<serde_json::Value, ProxyError> {
        // Heuristic: detect response format by presence of top-level fields.
        // The ProviderAdapter trait's transform_response doesn't receive the Provider
        // config, so we can't check api_format here. Instead we rely on the fact that
        // Responses API always returns "output" while Chat Completions returns "choices".
        // This is safe because the two formats are structurally disjoint.
        if body.get("output").is_some() {
            super::transform_responses::responses_to_anthropic(body)
        } else {
            super::transform::openai_to_anthropic(body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderMeta;
    use serde_json::json;

    fn create_provider(config: serde_json::Value) -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test Claude".to_string(),
            settings_config: config,
            website_url: None,
            category: Some("claude".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
        }
    }

    fn create_provider_with_meta(config: serde_json::Value, meta: ProviderMeta) -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test Claude".to_string(),
            settings_config: config,
            website_url: None,
            category: Some("claude".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: Some(meta),
            icon: None,
            icon_color: None,
        }
    }

    #[test]
    fn test_extract_base_url_from_env() {
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com"
            }
        }));

        let url = adapter.extract_base_url(&provider).unwrap();
        assert_eq!(url, "https://api.anthropic.com");
    }

    #[test]
    fn test_extract_auth_anthropic_auth_token_uses_claude_auth_strategy() {
        // ANTHROPIC_AUTH_TOKEN 在 Anthropic SDK 里语义就是 Authorization: Bearer，
        // 因此走 ClaudeAuth strategy 而不是 Anthropic（x-api-key）。
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-ant-test-key"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-ant-test-key");
        assert_eq!(auth.strategy, AuthStrategy::ClaudeAuth);
    }

    #[test]
    fn test_extract_auth_anthropic_api_key() {
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
                "ANTHROPIC_API_KEY": "sk-ant-test-key"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-ant-test-key");
        assert_eq!(auth.strategy, AuthStrategy::Anthropic);
    }

    #[test]
    fn test_extract_auth_both_env_vars_prefer_auth_token() {
        // 两个变量都填时，extract_key 选 AUTH_TOKEN，strategy 推断也必须保持一致。
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-from-auth-token",
                "ANTHROPIC_API_KEY": "sk-from-api-key"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-from-auth-token");
        assert_eq!(auth.strategy, AuthStrategy::ClaudeAuth);
    }

    #[test]
    fn test_extract_auth_apikey_field_fallback_uses_anthropic_strategy() {
        // 当用户没填任一 ANTHROPIC_* env，而是直接使用 apiKey 字段时，
        // 视为没有显式语义偏好，默认走 Anthropic 官方协议（x-api-key）。
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "apiKey": "sk-direct",
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-direct");
        assert_eq!(auth.strategy, AuthStrategy::Anthropic);
    }

    #[test]
    fn test_get_auth_headers_anthropic_emits_x_api_key() {
        let adapter = ClaudeAdapter::new();
        let auth = AuthInfo::new("sk-ant-test".to_string(), AuthStrategy::Anthropic);

        let headers = adapter.get_auth_headers(&auth);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0.as_str(), "x-api-key");
        assert_eq!(headers[0].1.to_str().unwrap(), "sk-ant-test");
    }

    #[test]
    fn test_get_auth_headers_claude_auth_emits_authorization_bearer() {
        let adapter = ClaudeAdapter::new();
        let auth = AuthInfo::new("sk-relay-test".to_string(), AuthStrategy::ClaudeAuth);

        let headers = adapter.get_auth_headers(&auth);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0.as_str(), "authorization");
        assert_eq!(headers[0].1.to_str().unwrap(), "Bearer sk-relay-test");
    }

    #[test]
    fn test_get_auth_headers_bearer_emits_authorization_bearer() {
        let adapter = ClaudeAdapter::new();
        let auth = AuthInfo::new("sk-or-test".to_string(), AuthStrategy::Bearer);

        let headers = adapter.get_auth_headers(&auth);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0.as_str(), "authorization");
        assert_eq!(headers[0].1.to_str().unwrap(), "Bearer sk-or-test");
    }

    #[test]
    fn test_extract_auth_openrouter() {
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://openrouter.ai/api",
                "OPENROUTER_API_KEY": "sk-or-test-key"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-or-test-key");
        assert_eq!(auth.strategy, AuthStrategy::Bearer);
    }

    #[test]
    fn test_extract_auth_claude_auth_mode() {
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://some-proxy.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-proxy-key"
            },
            "auth_mode": "bearer_only"
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-proxy-key");
        assert_eq!(auth.strategy, AuthStrategy::ClaudeAuth);
    }

    #[test]
    fn test_extract_auth_claude_auth_env_mode() {
        let adapter = ClaudeAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://some-proxy.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-proxy-key",
                "AUTH_MODE": "bearer_only"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-proxy-key");
        assert_eq!(auth.strategy, AuthStrategy::ClaudeAuth);
    }

    #[test]
    fn test_provider_type_detection() {
        let adapter = ClaudeAdapter::new();

        // Anthropic 官方
        let anthropic = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-ant-test"
            }
        }));
        assert_eq!(adapter.provider_type(&anthropic), ProviderType::Claude);

        // OpenRouter
        let openrouter = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://openrouter.ai/api",
                "OPENROUTER_API_KEY": "sk-or-test"
            }
        }));
        assert_eq!(adapter.provider_type(&openrouter), ProviderType::OpenRouter);

        // ClaudeAuth
        let claude_auth = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://some-proxy.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-test"
            },
            "auth_mode": "bearer_only"
        }));
        assert_eq!(
            adapter.provider_type(&claude_auth),
            ProviderType::ClaudeAuth
        );
    }

    #[test]
    fn test_build_url_anthropic() {
        let adapter = ClaudeAdapter::new();
        let url = adapter.build_url("https://api.anthropic.com", "/v1/messages");
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn test_build_url_openrouter() {
        let adapter = ClaudeAdapter::new();
        let url = adapter.build_url("https://openrouter.ai/api", "/v1/messages");
        assert_eq!(url, "https://openrouter.ai/api/v1/messages");
    }

    #[test]
    fn test_build_url_no_beta_for_other_endpoints() {
        let adapter = ClaudeAdapter::new();
        let url = adapter.build_url("https://api.anthropic.com", "/v1/complete");
        assert_eq!(url, "https://api.anthropic.com/v1/complete");
    }

    #[test]
    fn test_build_url_preserve_existing_query() {
        let adapter = ClaudeAdapter::new();
        let url = adapter.build_url("https://api.anthropic.com", "/v1/messages?foo=bar");
        assert_eq!(url, "https://api.anthropic.com/v1/messages?foo=bar");
    }

    #[test]
    fn test_build_url_no_beta_for_openai_chat_completions() {
        let adapter = ClaudeAdapter::new();
        let url = adapter.build_url("https://integrate.api.nvidia.com", "/v1/chat/completions");
        assert_eq!(url, "https://integrate.api.nvidia.com/v1/chat/completions");
    }

    #[test]
    fn test_needs_transform() {
        let adapter = ClaudeAdapter::new();

        // Default: no transform (anthropic format) - no meta
        let anthropic_provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com"
            }
        }));
        assert!(!adapter.needs_transform(&anthropic_provider));

        // Explicit anthropic format in meta: no transform
        let explicit_anthropic = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com"
                }
            }),
            ProviderMeta {
                api_format: Some("anthropic".to_string()),
                ..Default::default()
            },
        );
        assert!(!adapter.needs_transform(&explicit_anthropic));

        // Legacy settings_config.api_format: openai_chat should enable transform
        let legacy_settings_api_format = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.example.com"
            },
            "api_format": "openai_chat"
        }));
        assert!(adapter.needs_transform(&legacy_settings_api_format));

        // Legacy openrouter_compat_mode: bool/number/string should enable transform
        let legacy_openrouter_bool = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.example.com"
            },
            "openrouter_compat_mode": true
        }));
        assert!(adapter.needs_transform(&legacy_openrouter_bool));

        let legacy_openrouter_num = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.example.com"
            },
            "openrouter_compat_mode": 1
        }));
        assert!(adapter.needs_transform(&legacy_openrouter_num));

        let legacy_openrouter_str = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.example.com"
            },
            "openrouter_compat_mode": "true"
        }));
        assert!(adapter.needs_transform(&legacy_openrouter_str));

        // OpenAI Chat format in meta: needs transform
        let openai_chat_provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_chat".to_string()),
                ..Default::default()
            },
        );
        assert!(adapter.needs_transform(&openai_chat_provider));

        // OpenAI Responses format in meta: needs transform
        let openai_responses_provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_responses".to_string()),
                ..Default::default()
            },
        );
        assert!(adapter.needs_transform(&openai_responses_provider));

        // meta takes precedence over legacy settings_config fields
        let meta_precedence_over_settings = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com"
                },
                "api_format": "openai_chat",
                "openrouter_compat_mode": true
            }),
            ProviderMeta {
                api_format: Some("anthropic".to_string()),
                ..Default::default()
            },
        );
        assert!(!adapter.needs_transform(&meta_precedence_over_settings));

        // Unknown format in meta: default to anthropic (no transform)
        let unknown_format = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com"
                }
            }),
            ProviderMeta {
                api_format: Some("unknown".to_string()),
                ..Default::default()
            },
        );
        assert!(!adapter.needs_transform(&unknown_format));
    }

    #[test]
    fn test_transform_claude_request_for_api_format_openai_chat_skips_prompt_cache_key_by_default()
    {
        let provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com",
                    "ANTHROPIC_API_KEY": "test-key"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_chat".to_string()),
                ..Default::default()
            },
        );
        let body = json!({
            "model": "gpt-5.4",
            "messages": [{ "role": "user", "content": "hello" }],
            "max_tokens": 64
        });

        let transformed =
            transform_claude_request_for_api_format(body, &provider, "openai_chat").unwrap();

        assert!(transformed.get("prompt_cache_key").is_none());
    }

    #[test]
    fn test_transform_claude_request_for_api_format_openai_chat_keeps_explicit_prompt_cache_key() {
        let provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com",
                    "ANTHROPIC_API_KEY": "test-key"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_chat".to_string()),
                prompt_cache_key: Some("claude-cache-route".to_string()),
                ..Default::default()
            },
        );
        let body = json!({
            "model": "gpt-5.4",
            "messages": [{ "role": "user", "content": "hello" }],
            "max_tokens": 64
        });

        let transformed =
            transform_claude_request_for_api_format(body, &provider, "openai_chat").unwrap();

        assert_eq!(transformed["prompt_cache_key"], "claude-cache-route");
    }

    #[test]
    fn test_transform_openai_chat_skips_reasoning_content_for_generic_provider() {
        let provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.example.com",
                    "ANTHROPIC_API_KEY": "test-key"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_chat".to_string()),
                ..Default::default()
            },
        );
        let body = json!({
            "model": "gpt-5.4",
            "max_tokens": 64,
            "messages": [{
                "role": "assistant",
                "content": [
                    {"type": "thinking", "thinking": "I should call the tool."},
                    {"type": "tool_use", "id": "call_123", "name": "get_weather", "input": {"location": "Tokyo"}}
                ]
            }]
        });

        let transformed =
            transform_claude_request_for_api_format(body, &provider, "openai_chat").unwrap();

        let msg = &transformed["messages"][0];
        assert!(msg.get("tool_calls").is_some());
        assert!(msg.get("reasoning_content").is_none());
    }

    #[test]
    fn test_transform_openai_chat_preserves_reasoning_content_for_kimi_provider() {
        let provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.moonshot.cn/v1",
                    "ANTHROPIC_API_KEY": "test-key"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_chat".to_string()),
                ..Default::default()
            },
        );
        let body = json!({
            "model": "kimi-k2.6",
            "max_tokens": 64,
            "messages": [{
                "role": "assistant",
                "content": [
                    {"type": "thinking", "thinking": "I should call the tool."},
                    {"type": "tool_use", "id": "call_123", "name": "get_weather", "input": {"location": "Tokyo"}}
                ]
            }]
        });

        let transformed =
            transform_claude_request_for_api_format(body, &provider, "openai_chat").unwrap();

        let msg = &transformed["messages"][0];
        assert_eq!(msg["reasoning_content"], "I should call the tool.");
        assert!(msg.get("tool_calls").is_some());
    }

    #[test]
    fn test_transform_openai_chat_preserves_reasoning_content_for_deepseek_provider() {
        let provider = create_provider_with_meta(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.deepseek.com/v1",
                    "ANTHROPIC_API_KEY": "test-key"
                }
            }),
            ProviderMeta {
                api_format: Some("openai_chat".to_string()),
                ..Default::default()
            },
        );
        let body = json!({
            "model": "deepseek-v4-flash",
            "max_tokens": 64,
            "messages": [{
                "role": "assistant",
                "content": [
                    {"type": "thinking", "thinking": "I should call the tool."},
                    {"type": "tool_use", "id": "call_123", "name": "get_weather", "input": {"location": "Tokyo"}}
                ]
            }]
        });

        let transformed =
            transform_claude_request_for_api_format(body, &provider, "openai_chat").unwrap();

        let msg = &transformed["messages"][0];
        assert_eq!(msg["reasoning_content"], "I should call the tool.");
        assert!(msg.get("tool_calls").is_some());
    }
}
