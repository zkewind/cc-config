use super::{rw_err, JsonStore};
use crate::error::AppError;

impl JsonStore {
    const LEGACY_COMMON_CONFIG_MIGRATED_KEY: &'static str = "common_config_legacy_migrated_v1";

    fn config_snippet_cleared_key(app_type: &str) -> String {
        format!("common_config_{app_type}_cleared")
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.config.get(key).cloned())
    }

    pub fn get_bool_flag(&self, key: &str) -> Result<bool, AppError> {
        Ok(matches!(
            self.get_setting(key)?.as_deref(),
            Some("true") | Some("1")
        ))
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache.config.insert(key.to_string(), value.to_string());
            serde_json::to_vec_pretty(&cache.config)
                .map_err(|e| AppError::Config(format!("序列化 config 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("config.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("settings");
        Ok(())
    }

    fn delete_setting(&self, key: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache.config.remove(key);
            serde_json::to_vec_pretty(&cache.config)
                .map_err(|e| AppError::Config(format!("序列化 config 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("config.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("settings");
        Ok(())
    }

    // ─── 通用配置片段 ─────────────────────────────────────────────────────────

    pub fn get_config_snippet(&self, app_type: &str) -> Result<Option<String>, AppError> {
        self.get_setting(&format!("common_config_{app_type}"))
    }

    pub fn is_config_snippet_cleared(&self, app_type: &str) -> Result<bool, AppError> {
        Ok(self
            .get_setting(&Self::config_snippet_cleared_key(app_type))?
            .as_deref()
            == Some("true"))
    }

    pub fn set_config_snippet_cleared(
        &self,
        app_type: &str,
        cleared: bool,
    ) -> Result<(), AppError> {
        let key = Self::config_snippet_cleared_key(app_type);
        if cleared {
            self.set_setting(&key, "true")
        } else {
            self.delete_setting(&key)
        }
    }

    pub fn should_auto_extract_config_snippet(&self, app_type: &str) -> Result<bool, AppError> {
        Ok(self.get_config_snippet(app_type)?.is_none()
            && !self.is_config_snippet_cleared(app_type)?)
    }

    pub fn is_legacy_common_config_migrated(&self) -> Result<bool, AppError> {
        Ok(self
            .get_setting(Self::LEGACY_COMMON_CONFIG_MIGRATED_KEY)?
            .as_deref()
            == Some("true"))
    }

    pub fn set_legacy_common_config_migrated(&self, migrated: bool) -> Result<(), AppError> {
        if migrated {
            self.set_setting(Self::LEGACY_COMMON_CONFIG_MIGRATED_KEY, "true")
        } else {
            self.delete_setting(Self::LEGACY_COMMON_CONFIG_MIGRATED_KEY)
        }
    }

    pub fn set_config_snippet(
        &self,
        app_type: &str,
        snippet: Option<String>,
    ) -> Result<(), AppError> {
        let key = format!("common_config_{app_type}");
        if let Some(value) = snippet {
            self.set_setting(&key, &value)
        } else {
            self.delete_setting(&key)
        }
    }

    // ─── 全局代理 ─────────────────────────────────────────────────────────────

    const GLOBAL_PROXY_URL_KEY: &'static str = "global_proxy_url";

    pub fn get_global_proxy_url(&self) -> Result<Option<String>, AppError> {
        self.get_setting(Self::GLOBAL_PROXY_URL_KEY)
    }

    pub fn set_global_proxy_url(&self, url: Option<&str>) -> Result<(), AppError> {
        match url {
            Some(u) if !u.trim().is_empty() => {
                self.set_setting(Self::GLOBAL_PROXY_URL_KEY, u.trim())
            }
            _ => self.delete_setting(Self::GLOBAL_PROXY_URL_KEY),
        }
    }

    // ─── 废弃代理接管（兼容旧调用） ──────────────────────────────────────────

    #[deprecated(since = "3.9.0", note = "使用 get_proxy_config_for_app().enabled 替代")]
    pub fn get_proxy_takeover_enabled(&self, app_type: &str) -> Result<bool, AppError> {
        let key = format!("proxy_takeover_{app_type}");
        Ok(self.get_setting(&key)?.as_deref() == Some("true"))
    }

    #[deprecated(
        since = "3.9.0",
        note = "使用 update_proxy_config_for_app() 修改 enabled 字段"
    )]
    pub fn set_proxy_takeover_enabled(
        &self,
        app_type: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        let key = format!("proxy_takeover_{app_type}");
        self.set_setting(&key, if enabled { "true" } else { "false" })
    }

    #[deprecated(since = "3.9.0", note = "使用 is_live_takeover_active() 替代")]
    pub fn has_any_proxy_takeover(&self) -> Result<bool, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache
            .config
            .iter()
            .any(|(k, v)| k.starts_with("proxy_takeover_") && v == "true"))
    }

    #[deprecated(
        since = "3.9.0",
        note = "使用 update_proxy_config_for_app() 清除各应用的 enabled 字段"
    )]
    pub fn clear_all_proxy_takeover(&self) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            for (k, v) in cache.config.iter_mut() {
                if k.starts_with("proxy_takeover_") {
                    *v = "false".to_string();
                }
            }
            serde_json::to_vec_pretty(&cache.config)
                .map_err(|e| AppError::Config(format!("序列化 config 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("config.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("settings");
        log::info!("已清除所有代理接管状态");
        Ok(())
    }

    // ─── 整流器配置 ───────────────────────────────────────────────────────────

    pub fn get_rectifier_config(&self) -> Result<crate::proxy_types::RectifierConfig, AppError> {
        match self.get_setting("rectifier_config")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::Database(format!("解析整流器配置失败: {e}"))),
            None => Ok(crate::proxy_types::RectifierConfig::default()),
        }
    }

    pub fn set_rectifier_config(
        &self,
        config: &crate::proxy_types::RectifierConfig,
    ) -> Result<(), AppError> {
        let json = serde_json::to_string(config)
            .map_err(|e| AppError::Database(format!("序列化整流器配置失败: {e}")))?;
        self.set_setting("rectifier_config", &json)
    }

    // ─── 优化器配置 ───────────────────────────────────────────────────────────

    pub fn get_optimizer_config(&self) -> Result<crate::proxy_types::OptimizerConfig, AppError> {
        match self.get_setting("optimizer_config")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::Database(format!("解析优化器配置失败: {e}"))),
            None => Ok(crate::proxy_types::OptimizerConfig::default()),
        }
    }

    pub fn set_optimizer_config(
        &self,
        config: &crate::proxy_types::OptimizerConfig,
    ) -> Result<(), AppError> {
        let json = serde_json::to_string(config)
            .map_err(|e| AppError::Database(format!("序列化优化器配置失败: {e}")))?;
        self.set_setting("optimizer_config", &json)
    }

    // ─── Copilot 优化器配置 ───────────────────────────────────────────────────

    pub fn get_copilot_optimizer_config(
        &self,
    ) -> Result<crate::proxy_types::CopilotOptimizerConfig, AppError> {
        match self.get_setting("copilot_optimizer_config")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::Database(format!("解析 Copilot 优化器配置失败: {e}"))),
            None => Ok(crate::proxy_types::CopilotOptimizerConfig::default()),
        }
    }

    pub fn set_copilot_optimizer_config(
        &self,
        config: &crate::proxy_types::CopilotOptimizerConfig,
    ) -> Result<(), AppError> {
        let json = serde_json::to_string(config)
            .map_err(|e| AppError::Database(format!("序列化 Copilot 优化器配置失败: {e}")))?;
        self.set_setting("copilot_optimizer_config", &json)
    }

    // ─── 日志配置 ─────────────────────────────────────────────────────────────

    pub fn get_log_config(&self) -> Result<crate::proxy_types::LogConfig, AppError> {
        match self.get_setting("log_config")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::Database(format!("解析日志配置失败: {e}"))),
            None => Ok(crate::proxy_types::LogConfig::default()),
        }
    }

    pub fn set_log_config(&self, config: &crate::proxy_types::LogConfig) -> Result<(), AppError> {
        let json = serde_json::to_string(config)
            .map_err(|e| AppError::Database(format!("序列化日志配置失败: {e}")))?;
        self.set_setting("log_config", &json)
    }

    // ─── 当前选中项目作用域（供托盘同步前端选中态） ─────────────────────────

    const CURRENT_PROJECT_SCOPE_KEY: &'static str = "current_project_scope";

    pub fn get_current_project_scope(&self) -> Result<Option<String>, AppError> {
        self.get_setting(Self::CURRENT_PROJECT_SCOPE_KEY)
    }

    pub fn set_current_project_scope(&self, path: Option<&str>) -> Result<(), AppError> {
        match path {
            Some(p) if !p.trim().is_empty() => {
                self.set_setting(Self::CURRENT_PROJECT_SCOPE_KEY, p.trim())
            }
            _ => self.delete_setting(Self::CURRENT_PROJECT_SCOPE_KEY),
        }
    }

    // ─── 托管项目路径 ─────────────────────────────────────────────────────────

    const MANAGED_PROJECT_PATHS_KEY: &'static str = "managed_project_paths";

    pub fn get_managed_project_paths(&self) -> Result<Vec<String>, AppError> {
        match self.get_setting(Self::MANAGED_PROJECT_PATHS_KEY)? {
            None => Ok(Vec::new()),
            Some(s) => serde_json::from_str(&s)
                .map_err(|e| AppError::Config(format!("解析项目路径列表失败: {e}"))),
        }
    }

    pub fn add_managed_project_path(&self, path: &str) -> Result<bool, AppError> {
        let mut paths = self.get_managed_project_paths()?;
        if paths.iter().any(|p| p == path) {
            return Ok(false);
        }
        paths.push(path.to_string());
        let json = serde_json::to_string(&paths)
            .map_err(|e| AppError::Config(format!("序列化项目路径列表失败: {e}")))?;
        self.set_setting(Self::MANAGED_PROJECT_PATHS_KEY, &json)?;
        Ok(true)
    }

    pub fn remove_managed_project_path(&self, path: &str) -> Result<bool, AppError> {
        let mut paths = self.get_managed_project_paths()?;
        let before = paths.len();
        paths.retain(|p| p != path);
        if paths.len() == before {
            return Ok(false);
        }
        let json = serde_json::to_string(&paths)
            .map_err(|e| AppError::Config(format!("序列化项目路径列表失败: {e}")))?;
        self.set_setting(Self::MANAGED_PROJECT_PATHS_KEY, &json)?;
        Ok(true)
    }

    // ─── 流式检查配置 ─────────────────────────────────────────────────────────

    pub fn get_stream_check_config(
        &self,
    ) -> Result<crate::services::stream_check::StreamCheckConfig, AppError> {
        match self.get_setting("stream_check_config")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::Message(format!("解析流检查配置失败: {e}"))),
            None => Ok(crate::services::stream_check::StreamCheckConfig::default()),
        }
    }

    pub fn save_stream_check_config(
        &self,
        config: &crate::services::stream_check::StreamCheckConfig,
    ) -> Result<(), AppError> {
        let json = serde_json::to_string(config)
            .map_err(|e| AppError::Message(format!("序列化流检查配置失败: {e}")))?;
        self.set_setting("stream_check_config", &json)
    }
}
