//! Provider Adapters Module
//!
//! 供应商适配器模块，提供统一的接口抽象不同上游供应商的处理逻辑。

mod adapter;
mod auth;
mod claude;
pub mod copilot_optimizer;
pub mod models;
pub mod proxy_error;
pub(crate) mod sse;
pub mod streaming;
pub mod streaming_responses;
pub mod transform;
pub mod transform_responses;

use crate::app_config::AppType;
use crate::provider::Provider;
use serde::{Deserialize, Serialize};

pub use adapter::ProviderAdapter;
pub use auth::{AuthInfo, AuthStrategy};
pub use claude::ClaudeAdapter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Claude,
    ClaudeAuth,
    OpenRouter,
}

impl ProviderType {
    #[allow(dead_code)]
    pub fn needs_transform(&self) -> bool {
        match self {
            ProviderType::OpenRouter => false,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn default_endpoint(&self) -> &'static str {
        match self {
            ProviderType::Claude | ProviderType::ClaudeAuth => "https://api.anthropic.com",
            ProviderType::OpenRouter => "https://openrouter.ai/api",
        }
    }

    #[allow(dead_code)]
    pub fn from_app_type_and_config(app_type: &AppType, provider: &Provider) -> Self {
        match app_type {
            AppType::Claude => {
                let adapter = ClaudeAdapter::new();
                if let Ok(base_url) = adapter.extract_base_url(provider) {
                    if base_url.contains("openrouter.ai") {
                        return ProviderType::OpenRouter;
                    }
                }

                if let Some(auth_mode) = provider
                    .settings_config
                    .get("auth_mode")
                    .and_then(|v| v.as_str())
                {
                    if auth_mode == "bearer_only" {
                        return ProviderType::ClaudeAuth;
                    }
                }

                if let Some(env) = provider.settings_config.get("env") {
                    if let Some(auth_mode) = env.get("AUTH_MODE").and_then(|v| v.as_str()) {
                        if auth_mode == "bearer_only" {
                            return ProviderType::ClaudeAuth;
                        }
                    }
                }

                ProviderType::Claude
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Claude => "claude",
            ProviderType::ClaudeAuth => "claude_auth",
            ProviderType::OpenRouter => "openrouter",
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(ProviderType::Claude),
            "claude_auth" | "claude-auth" => Ok(ProviderType::ClaudeAuth),
            "openrouter" => Ok(ProviderType::OpenRouter),
            _ => Err(format!("Invalid provider type: {s}")),
        }
    }
}

pub fn get_adapter(app_type: &AppType) -> Box<dyn ProviderAdapter> {
    match app_type {
        AppType::Claude => Box::new(ClaudeAdapter::new()),
    }
}

#[allow(dead_code)]
pub fn get_adapter_for_provider_type(provider_type: &ProviderType) -> Box<dyn ProviderAdapter> {
    match provider_type {
        ProviderType::Claude | ProviderType::ClaudeAuth | ProviderType::OpenRouter => {
            Box::new(ClaudeAdapter::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_provider(config: serde_json::Value) -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            settings_config: config,
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
        }
    }

    #[test]
    fn provider_type_defaults_are_claude_only() {
        assert_eq!(
            ProviderType::Claude.default_endpoint(),
            "https://api.anthropic.com"
        );
        assert_eq!(
            ProviderType::ClaudeAuth.default_endpoint(),
            "https://api.anthropic.com"
        );
        assert_eq!(
            ProviderType::OpenRouter.default_endpoint(),
            "https://openrouter.ai/api"
        );
    }

    #[test]
    fn provider_type_from_str_accepts_supported_values() {
        assert_eq!(
            "claude".parse::<ProviderType>().unwrap(),
            ProviderType::Claude
        );
        assert_eq!(
            "claude-auth".parse::<ProviderType>().unwrap(),
            ProviderType::ClaudeAuth
        );
        assert_eq!(
            "openrouter".parse::<ProviderType>().unwrap(),
            ProviderType::OpenRouter
        );
        assert!("codex".parse::<ProviderType>().is_err());
    }

    #[test]
    fn from_app_type_detects_openrouter() {
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://openrouter.ai/api",
                "OPENROUTER_API_KEY": "sk-or-test"
            }
        }));

        let provider_type = ProviderType::from_app_type_and_config(&AppType::Claude, &provider);
        assert_eq!(provider_type, ProviderType::OpenRouter);
    }

    #[test]
    fn from_app_type_detects_bearer_only_auth() {
        let provider = create_provider(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://some-proxy.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-test"
            },
            "auth_mode": "bearer_only"
        }));

        let provider_type = ProviderType::from_app_type_and_config(&AppType::Claude, &provider);
        assert_eq!(provider_type, ProviderType::ClaudeAuth);
    }
}
