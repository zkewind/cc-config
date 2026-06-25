use serde::{Deserialize, Serialize};

/// 整流器配置
///
/// 存储在 settings 表中
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RectifierConfig {
    /// 总开关：是否启用整流器（默认开启）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 请求整流：启用 thinking 签名整流器（默认开启）
    #[serde(default = "default_true")]
    pub request_thinking_signature: bool,
    /// 请求整流：启用 thinking budget 整流器（默认开启）
    #[serde(default = "default_true")]
    pub request_thinking_budget: bool,
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for RectifierConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            request_thinking_signature: true,
            request_thinking_budget: true,
        }
    }
}

/// 请求优化器配置
///
/// 存储在 settings 表中，key = "optimizer_config"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub thinking_optimizer: bool,
    #[serde(default = "default_true")]
    pub cache_injection: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: String,
}

fn default_cache_ttl() -> String {
    "1h".to_string()
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            thinking_optimizer: true,
            cache_injection: true,
            cache_ttl: "1h".to_string(),
        }
    }
}

/// Copilot 优化器配置
///
/// 存储在 settings 表中，key = "copilot_optimizer_config"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotOptimizerConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub request_classification: bool,
    #[serde(default = "default_true")]
    pub tool_result_merging: bool,
    #[serde(default = "default_true")]
    pub compact_detection: bool,
    #[serde(default = "default_true")]
    pub deterministic_request_id: bool,
    #[serde(default = "default_true")]
    pub subagent_detection: bool,
    #[serde(default = "default_true")]
    pub warmup_downgrade: bool,
    #[serde(default = "default_warmup_model")]
    pub warmup_model: String,
    #[serde(default = "default_true")]
    pub strip_thinking: bool,
}

fn default_warmup_model() -> String {
    "gpt-5-mini".to_string()
}

impl Default for CopilotOptimizerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            request_classification: true,
            tool_result_merging: true,
            compact_detection: true,
            deterministic_request_id: true,
            subagent_detection: true,
            warmup_downgrade: true,
            warmup_model: "gpt-5-mini".to_string(),
            strip_thinking: true,
        }
    }
}

/// 日志配置
///
/// 存储在 settings 表的 log_config 字段中（JSON 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
        }
    }
}

impl LogConfig {
    pub fn to_level_filter(&self) -> log::LevelFilter {
        if !self.enabled {
            return log::LevelFilter::Off;
        }
        match self.level.to_lowercase().as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Info,
        }
    }
}
