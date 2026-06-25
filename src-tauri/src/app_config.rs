use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use crate::services::skill::SkillStore;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct McpApps {
    #[serde(default)]
    pub claude: bool,
}

impl McpApps {
    pub fn is_enabled_for(&self, app: &AppType) -> bool {
        match app {
            AppType::Claude => self.claude,
        }
    }

    pub fn set_enabled_for(&mut self, app: &AppType, enabled: bool) {
        match app {
            AppType::Claude => self.claude = enabled,
        }
    }

    pub fn enabled_apps(&self) -> Vec<AppType> {
        if self.claude {
            vec![AppType::Claude]
        } else {
            Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.claude
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillApps {
    #[serde(default)]
    pub claude: bool,
}

impl SkillApps {
    pub fn is_enabled_for(&self, app: &AppType) -> bool {
        match app {
            AppType::Claude => self.claude,
        }
    }

    pub fn set_enabled_for(&mut self, app: &AppType, enabled: bool) {
        match app {
            AppType::Claude => self.claude = enabled,
        }
    }

    pub fn enabled_apps(&self) -> Vec<AppType> {
        if self.claude {
            vec![AppType::Claude]
        } else {
            Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.claude
    }

    pub fn only(app: &AppType) -> Self {
        let mut apps = Self::default();
        apps.set_enabled_for(app, true);
        apps
    }

    pub fn from_labels(labels: &[String]) -> Self {
        let mut apps = Self::default();
        for label in labels {
            if let Ok(app) = label.parse::<AppType>() {
                apps.set_enabled_for(&app, true);
            }
        }
        apps
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme_url: Option<String>,
    pub apps: SkillApps,
    pub installed_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmanagedSkill {
    pub directory: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub found_in: Vec<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub server: serde_json::Value,
    pub apps: McpApps,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, serde_json::Value>,
}

impl McpConfig {
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRoot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<HashMap<String, McpServer>>,

    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub claude: McpConfig,
}

impl Default for McpRoot {
    fn default() -> Self {
        Self {
            servers: Some(HashMap::new()),
            claude: McpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptConfig {
    #[serde(default)]
    pub prompts: HashMap<String, crate::prompt::Prompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptRoot {
    #[serde(default)]
    pub claude: PromptConfig,
}

use crate::config::{copy_file, get_app_config_dir, get_app_config_path, write_json_file};
use crate::error::AppError;
use crate::prompt_files::prompt_file_path;
use crate::provider::ProviderManager;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Claude,
}

impl AppType {
    pub fn as_str(&self) -> &str {
        match self {
            AppType::Claude => "claude",
        }
    }

    pub fn all() -> impl Iterator<Item = AppType> {
        [AppType::Claude].into_iter()
    }
}

impl FromStr for AppType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "claude" => Ok(AppType::Claude),
            other => Err(AppError::localized(
                "unsupported_app",
                format!("不支持的应用标识: '{other}'。可选值: claude。"),
                format!("Unsupported app id: '{other}'. Allowed: claude."),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonConfigSnippets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude: Option<String>,
}

impl CommonConfigSnippets {
    pub fn get(&self, app: &AppType) -> Option<&String> {
        match app {
            AppType::Claude => self.claude.as_ref(),
        }
    }

    pub fn set(&mut self, app: &AppType, snippet: Option<String>) {
        match app {
            AppType::Claude => self.claude = snippet,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAppConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(flatten)]
    pub apps: HashMap<String, ProviderManager>,
    #[serde(default)]
    pub mcp: McpRoot,
    #[serde(default)]
    pub prompts: PromptRoot,
    #[serde(default)]
    pub skills: SkillStore,
    #[serde(default)]
    pub common_config_snippets: CommonConfigSnippets,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_common_config_snippet: Option<String>,
}

fn default_version() -> u32 {
    2
}

impl Default for MultiAppConfig {
    fn default() -> Self {
        let mut apps = HashMap::new();
        apps.insert("claude".to_string(), ProviderManager::default());

        Self {
            version: 2,
            apps,
            mcp: McpRoot::default(),
            prompts: PromptRoot::default(),
            skills: SkillStore::default(),
            common_config_snippets: CommonConfigSnippets::default(),
            claude_common_config_snippet: None,
        }
    }
}

impl MultiAppConfig {
    pub fn load() -> Result<Self, AppError> {
        let config_path = get_app_config_path();

        if !config_path.exists() {
            log::info!("配置文件不存在，创建新的 Claude 配置并自动导入提示词");
            let config = Self::default_with_auto_import()?;
            config.save()?;
            return Ok(config);
        }

        let content =
            std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;

        let value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| AppError::json(&config_path, e))?;
        let is_v1 = value.as_object().is_some_and(|map| {
            let has_providers = map.get("providers").map(|v| v.is_object()).unwrap_or(false);
            let has_current = map.get("current").map(|v| v.is_string()).unwrap_or(false);
            let has_apps = map.contains_key("apps");
            has_providers && has_current && !has_apps
        });
        if is_v1 {
            return Err(AppError::localized(
                "config.unsupported_v1",
                "检测到旧版 v1 配置格式。当前版本已不再支持运行时自动迁移。\n\n解决方案：\n1. 安装 v3.2.x 版本进行一次性自动迁移\n2. 或手动编辑 ~/.cc-config/config.json，将顶层结构调整为：\n   {\"version\": 2, \"claude\": {...}, \"mcp\": {...}}\n\n",
                "Detected legacy v1 config. Runtime auto-migration is no longer supported.\n\nSolutions:\n1. Install v3.2.x for one-time auto-migration\n2. Or manually edit ~/.cc-config/config.json to adjust the top-level structure:\n   {\"version\": 2, \"claude\": {...}, \"mcp\": {...}}\n\n",
            ));
        }

        let has_skills_in_config = value
            .as_object()
            .is_some_and(|map| map.contains_key("skills"));

        let mut config: Self =
            serde_json::from_value(value).map_err(|e| AppError::json(&config_path, e))?;
        let mut updated = false;

        if !has_skills_in_config {
            let skills_path = get_app_config_dir().join("skills.json");
            if skills_path.exists() {
                match std::fs::read_to_string(&skills_path) {
                    Ok(content) => match serde_json::from_str::<SkillStore>(&content) {
                        Ok(store) => {
                            config.skills = store;
                            updated = true;
                            log::info!("已从旧版 skills.json 导入 Claude Skills 配置");
                        }
                        Err(e) => {
                            log::warn!("解析旧版 skills.json 失败: {e}");
                        }
                    },
                    Err(e) => {
                        log::warn!("读取旧版 skills.json 失败: {e}");
                    }
                }
            }
        }

        if !config.apps.contains_key("claude") {
            config
                .apps
                .insert("claude".to_string(), ProviderManager::default());
            updated = true;
        }

        let migrated = config.migrate_mcp_to_unified()?;
        if migrated {
            log::info!("MCP 配置已迁移到统一结构，保存配置...");
            updated = true;
        }

        let imported_prompts = config.maybe_auto_import_prompts_for_existing_config()?;
        if imported_prompts {
            updated = true;
        }

        if let Some(old_claude_snippet) = config.claude_common_config_snippet.take() {
            config.common_config_snippets.claude = Some(old_claude_snippet);
            updated = true;
        }

        if updated {
            log::info!("配置结构已更新，保存配置...");
            config.save()?;
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<(), AppError> {
        let config_path = get_app_config_path();
        if config_path.exists() {
            let backup_path = get_app_config_dir().join("config.json.bak");
            if let Err(e) = copy_file(&config_path, &backup_path) {
                log::warn!("备份 config.json 到 .bak 失败: {e}");
            }
        }

        write_json_file(&config_path, self)?;
        Ok(())
    }

    pub fn get_manager(&self, app: &AppType) -> Option<&ProviderManager> {
        self.apps.get(app.as_str())
    }

    pub fn get_manager_mut(&mut self, app: &AppType) -> Option<&mut ProviderManager> {
        self.apps.get_mut(app.as_str())
    }

    pub fn ensure_app(&mut self, app: &AppType) {
        if !self.apps.contains_key(app.as_str()) {
            self.apps
                .insert(app.as_str().to_string(), ProviderManager::default());
        }
    }

    pub fn mcp_for(&self, app: &AppType) -> &McpConfig {
        match app {
            AppType::Claude => &self.mcp.claude,
        }
    }

    pub fn mcp_for_mut(&mut self, app: &AppType) -> &mut McpConfig {
        match app {
            AppType::Claude => &mut self.mcp.claude,
        }
    }

    fn default_with_auto_import() -> Result<Self, AppError> {
        log::info!("首次启动，创建默认配置并检测 Claude 提示词文件");

        let mut config = Self::default();
        Self::auto_import_prompt_if_exists(&mut config, AppType::Claude)?;

        Ok(config)
    }

    fn maybe_auto_import_prompts_for_existing_config(&mut self) -> Result<bool, AppError> {
        if !self.prompts.claude.prompts.is_empty() {
            return Ok(false);
        }

        log::info!(
            "检测到已存在配置文件且 Prompt 列表为空，将尝试从现有 Claude 提示词文件自动导入"
        );
        Self::auto_import_prompt_if_exists(self, AppType::Claude)
    }

    fn auto_import_prompt_if_exists(config: &mut Self, app: AppType) -> Result<bool, AppError> {
        let file_path = prompt_file_path(&app)?;

        if !file_path.exists() {
            log::debug!("提示词文件不存在，跳过自动导入: {file_path:?}");
            return Ok(false);
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("读取提示词文件失败: {file_path:?}, 错误: {e}");
                return Ok(false);
            }
        };

        if content.trim().is_empty() {
            log::debug!("提示词文件内容为空，跳过导入: {file_path:?}");
            return Ok(false);
        }

        log::info!("发现提示词文件，自动导入: {file_path:?}");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or_else(|_| {
                log::warn!("Failed to get system time, using 0 as timestamp");
                0
            });

        let id = format!("auto-imported-{timestamp}");
        let prompt = crate::prompt::Prompt {
            id: id.clone(),
            name: format!(
                "Auto-imported Prompt {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            ),
            content,
            description: Some("Automatically imported on first launch".to_string()),
            enabled: true,
            created_at: Some(timestamp),
            updated_at: Some(timestamp),
        };

        match app {
            AppType::Claude => {
                config.prompts.claude.prompts.insert(id, prompt);
            }
        }

        log::info!("自动导入完成: {}", app.as_str());
        Ok(true)
    }

    pub fn migrate_mcp_to_unified(&mut self) -> Result<bool, AppError> {
        if self.mcp.servers.is_some() {
            log::debug!("MCP 配置已是统一结构，跳过迁移");
            return Ok(false);
        }

        log::info!("检测到旧版 MCP 配置格式，开始迁移到统一结构...");

        let mut unified_servers: HashMap<String, McpServer> = HashMap::new();
        let mut conflicts = Vec::new();

        for (id, entry) in &self.mcp.claude.servers {
            let enabled = entry
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let name = entry
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string();

            let server = entry
                .get("server")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let description = entry
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let homepage = entry
                .get("homepage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let docs = entry
                .get("docs")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let tags = entry
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let mut apps = McpApps::default();
            apps.set_enabled_for(&AppType::Claude, enabled);

            if unified_servers
                .insert(
                    id.clone(),
                    McpServer {
                        id: id.clone(),
                        name,
                        server,
                        apps,
                        description,
                        homepage,
                        docs,
                        tags,
                    },
                )
                .is_some()
            {
                conflicts.push(format!("MCP '{id}' 存在重复配置，将使用最后一次读取的配置"));
            }
        }

        if !conflicts.is_empty() {
            log::warn!("MCP 迁移过程中检测到配置冲突：");
            for conflict in &conflicts {
                log::warn!("  - {conflict}");
            }
        }

        log::info!("MCP 迁移完成，共迁移 {} 个服务器", unified_servers.len());

        self.mcp.servers = Some(unified_servers);
        self.mcp.claude = McpConfig::default();

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    struct TempHome {
        #[allow(dead_code)]
        dir: TempDir,
        original_home: Option<String>,
        original_userprofile: Option<String>,
        original_test_home: Option<String>,
    }

    impl TempHome {
        fn new() -> Self {
            let dir = TempDir::new().expect("failed to create temp home");
            let original_home = env::var("HOME").ok();
            let original_userprofile = env::var("USERPROFILE").ok();
            let original_test_home = env::var("CC_CONFIG_TEST_HOME").ok();

            env::set_var("HOME", dir.path());
            env::set_var("USERPROFILE", dir.path());
            env::set_var("CC_CONFIG_TEST_HOME", dir.path());

            Self {
                dir,
                original_home,
                original_userprofile,
                original_test_home,
            }
        }
    }

    impl Drop for TempHome {
        fn drop(&mut self) {
            match &self.original_home {
                Some(value) => env::set_var("HOME", value),
                None => env::remove_var("HOME"),
            }

            match &self.original_userprofile {
                Some(value) => env::set_var("USERPROFILE", value),
                None => env::remove_var("USERPROFILE"),
            }

            match &self.original_test_home {
                Some(value) => env::set_var("CC_CONFIG_TEST_HOME", value),
                None => env::remove_var("CC_CONFIG_TEST_HOME"),
            }
        }
    }

    fn write_prompt_file(app: AppType, content: &str) {
        let path = crate::prompt_files::prompt_file_path(&app).expect("prompt path");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(path, content).expect("write prompt");
    }

    #[test]
    fn app_type_only_parses_claude() {
        assert_eq!("claude".parse::<AppType>().unwrap(), AppType::Claude);
        assert!("codex".parse::<AppType>().is_err());
        assert!("claude-desktop".parse::<AppType>().is_err());
    }

    #[test]
    #[serial]
    fn auto_imports_existing_prompt_when_config_missing() {
        let _home = TempHome::new();
        write_prompt_file(AppType::Claude, "# hello");

        let config = MultiAppConfig::load().expect("load config");

        assert_eq!(config.prompts.claude.prompts.len(), 1);
        let prompt = config
            .prompts
            .claude
            .prompts
            .values()
            .next()
            .expect("prompt exists");
        assert!(prompt.enabled);
        assert_eq!(prompt.content, "# hello");

        let config_path = crate::config::get_app_config_path();
        assert!(
            config_path.exists(),
            "auto import should persist config to disk"
        );
    }

    #[test]
    #[serial]
    fn skips_empty_prompt_files_during_import() {
        let _home = TempHome::new();
        write_prompt_file(AppType::Claude, "   \n  ");

        let config = MultiAppConfig::load().expect("load config");
        assert!(
            config.prompts.claude.prompts.is_empty(),
            "empty files must be ignored"
        );
    }

    #[test]
    #[serial]
    fn auto_import_happens_only_once() {
        let _home = TempHome::new();
        write_prompt_file(AppType::Claude, "first version");

        let first = MultiAppConfig::load().expect("load config");
        assert_eq!(first.prompts.claude.prompts.len(), 1);
        let claude_prompt = first
            .prompts
            .claude
            .prompts
            .values()
            .next()
            .expect("prompt exists")
            .content
            .clone();
        assert_eq!(claude_prompt, "first version");

        write_prompt_file(AppType::Claude, "second version");
        let second = MultiAppConfig::load().expect("load config again");

        assert_eq!(second.prompts.claude.prompts.len(), 1);
        let prompt = second
            .prompts
            .claude
            .prompts
            .values()
            .next()
            .expect("prompt exists");
        assert_eq!(
            prompt.content, "first version",
            "should not re-import when config already exists"
        );
    }
}
