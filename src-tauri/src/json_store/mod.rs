//! JSON 文件存储层 - 替代 SQLite 的分域多文件存储方案
//!
//! 文件布局（~/.cc-config/）：
//! - providers.json          per-app 供应商列表 + current_id
//! - config.json             键值设置（替代 settings 表）
//! - mcp.json                MCP 服务器
//! - prompts.json            per-app 提示词
//! - skills.json             Skills + Repos

pub mod backup;
pub mod mcp;
pub mod migration;
pub mod prompts;
pub mod providers;
pub mod providers_seed;
pub mod settings;
pub mod skills;

use crate::app_config::{InstalledSkill, McpServer};
use crate::config::get_app_config_dir;
use crate::error::AppError;
use crate::prompt::Prompt;
use crate::provider::Provider;
use crate::services::skill::SkillRepo;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

// ─── Per-app provider bundle ──────────────────────────────────────────────────

#[derive(Default, Serialize, Deserialize, Clone)]
pub(crate) struct AppProviders {
    pub current_id: Option<String>,
    pub providers: IndexMap<String, Provider>,
}

// ─── File-level serde types ───────────────────────────────────────────────────

#[derive(Default, Serialize, Deserialize, Clone)]
pub(crate) struct McpFile {
    pub servers: IndexMap<String, McpServer>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub(crate) struct SkillsFile {
    pub skills: IndexMap<String, InstalledSkill>,
    pub repos: Vec<SkillRepo>,
}

// ─── In-memory cache ─────────────────────────────────────────────────────────

pub(crate) struct StoreCache {
    pub providers: HashMap<String, AppProviders>,
    pub config: HashMap<String, String>,
    pub mcp: McpFile,
    pub prompts: HashMap<String, IndexMap<String, Prompt>>,
    pub skills: SkillsFile,
}

// ─── JsonStore ────────────────────────────────────────────────────────────────

pub struct JsonStore {
    pub(crate) dir: PathBuf,
    pub(crate) cache: RwLock<StoreCache>,
}

/// 统一的 RwLock 中毒错误转换
pub(crate) fn rw_err<T>(_: T) -> AppError {
    AppError::Database("RwLock poisoned".to_string())
}

impl JsonStore {
    pub fn init() -> Result<Self, AppError> {
        let dir = get_app_config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| AppError::io(&dir, e))?;
        Self::migrate_from_legacy_sqlite_if_needed(&dir)?;
        let cache = Self::load_all(&dir)?;
        Ok(Self {
            dir,
            cache: RwLock::new(cache),
        })
    }

    fn load_all(dir: &std::path::Path) -> Result<StoreCache, AppError> {
        Ok(StoreCache {
            providers: Self::load_json(dir.join("providers.json")),
            config: Self::load_json(dir.join("config.json")),
            mcp: Self::load_json(dir.join("mcp.json")),
            prompts: Self::load_json(dir.join("prompts.json")),
            skills: Self::load_json(dir.join("skills.json")),
        })
    }

    /// 加载 JSON 文件到指定类型，文件不存在或解析失败时返回 Default
    pub(crate) fn load_json<T: Default + for<'de> Deserialize<'de>>(path: PathBuf) -> T {
        if !path.exists() {
            return T::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    // ─── 实用方法 ─────────────────────────────────────────────────────────────

    pub fn is_mcp_table_empty(&self) -> Result<bool, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.mcp.servers.is_empty())
    }

    pub fn is_prompts_table_empty(&self) -> Result<bool, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.prompts.values().all(|p| p.is_empty()))
    }

    /// 导出所有配置为 JSON bundle 字符串（WebDAV 同步上传）
    pub fn export_sql_string_for_sync(&self) -> Result<String, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        let bundle = serde_json::json!({
            "format": "cc-config-json-v1",
            "providers": cache.providers,
            "config": cache.config,
            "mcp": cache.mcp,
            "prompts": cache.prompts,
            "skills": cache.skills,
        });
        serde_json::to_string_pretty(&bundle)
            .map_err(|e| AppError::Config(format!("JSON bundle 序列化失败: {e}")))
    }

    /// 从 JSON bundle 字符串导入配置（WebDAV 同步下载），返回安全备份 ID
    pub fn import_sql_string_for_sync(&self, json_str: &str) -> Result<String, AppError> {
        let bundle: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| AppError::Config(format!("JSON bundle 解析失败: {e}")))?;

        // Safety backup
        let safety_path = self.backup_database_file()?;
        let safety_id = safety_path
            .as_ref()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_default();

        let write_if_present = |key: &str, filename: &str| -> Result<(), AppError> {
            if let Some(val) = bundle.get(key) {
                let bytes = serde_json::to_vec_pretty(val)
                    .map_err(|e| AppError::Config(format!("序列化 {filename} 失败: {e}")))?;
                crate::config::atomic_write(&self.dir.join(filename), &bytes)?;
            }
            Ok(())
        };

        write_if_present("providers", "providers.json")?;
        write_if_present("config", "config.json")?;
        write_if_present("mcp", "mcp.json")?;
        write_if_present("prompts", "prompts.json")?;
        write_if_present("skills", "skills.json")?;

        self.reload_cache()?;
        Ok(safety_id)
    }

    /// 仅用于测试：在独立临时目录创建 JsonStore
    pub fn memory() -> Result<Self, AppError> {
        Self::init()
    }
}
