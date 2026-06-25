use super::{AppProviders, JsonStore, McpFile, SkillsFile};
use crate::app_config::{InstalledSkill, McpApps, MultiAppConfig, SkillApps};
use crate::error::AppError;
use crate::prompt::Prompt;
use crate::provider::{Provider, ProviderMeta};
use crate::services::skill::SkillRepo;
use indexmap::IndexMap;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;

impl JsonStore {
    pub(crate) fn migrate_from_legacy_sqlite_if_needed(dir: &Path) -> Result<(), AppError> {
        let db_path = dir.join("cc-switch.db");
        if !db_path.exists() || Self::has_any_json_data(dir) {
            return Ok(());
        }

        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| AppError::Database(format!("读取旧 SQLite 配置失败: {e}")))?;

        let providers = read_legacy_providers(&conn)?;
        let config = read_legacy_settings(&conn)?;
        let mcp = McpFile {
            servers: read_legacy_mcp_servers(&conn)?,
        };
        let prompts = read_legacy_prompts(&conn)?;
        let skills = SkillsFile {
            skills: read_legacy_skills(&conn)?,
            repos: read_legacy_skill_repos(&conn)?,
        };

        write_legacy_json(dir, "providers.json", &providers)?;
        write_legacy_json(dir, "config.json", &config)?;
        write_legacy_json(dir, "mcp.json", &mcp)?;
        write_legacy_json(dir, "prompts.json", &prompts)?;
        write_legacy_json(dir, "skills.json", &skills)?;
        log::info!("已将旧 SQLite 配置迁移到 JSON 文件存储");
        Ok(())
    }

    fn has_any_json_data(dir: &Path) -> bool {
        [
            "providers.json",
            "config.json",
            "mcp.json",
            "prompts.json",
            "skills.json",
        ]
        .iter()
        .any(|filename| {
            let path = dir.join(filename);
            path.exists()
                && std::fs::metadata(&path)
                    .map(|m| m.len() > 2)
                    .unwrap_or(false)
        })
    }

    /// 从 MultiAppConfig 批量导入数据（供集成测试和 Phase 2e 前的兼容层使用）
    pub fn migrate_from_json(&self, config: &MultiAppConfig) -> Result<(), AppError> {
        // 1. Providers
        for (app_type, manager) in &config.apps {
            for provider in manager.providers.values() {
                self.save_provider(app_type, provider)?;
            }
            if !manager.current.is_empty() && manager.providers.contains_key(&manager.current) {
                self.set_current_provider(app_type, &manager.current)?;
            }
        }

        // 2. MCP Servers
        if let Some(servers) = &config.mcp.servers {
            for server in servers.values() {
                self.save_mcp_server(server)?;
            }
        }

        // 3. Prompts
        for prompt in config.prompts.claude.prompts.values() {
            self.save_prompt("claude", prompt)?;
        }

        // 4. Skill Repos
        for repo in &config.skills.repos {
            self.save_skill_repo(repo)?;
        }

        // 5. Common Config Snippets
        if let Some(s) = &config.common_config_snippets.claude {
            self.set_config_snippet("claude", Some(s.clone()))?;
        }

        Ok(())
    }
}

fn write_legacy_json<T: serde::Serialize>(
    dir: &Path,
    filename: &str,
    value: &T,
) -> Result<(), AppError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| AppError::Config(format!("序列化迁移文件 {filename} 失败: {e}")))?;
    crate::config::atomic_write(&dir.join(filename), &bytes)
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, AppError> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [table],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(exists)
}

fn read_legacy_settings(conn: &Connection) -> Result<HashMap<String, String>, AppError> {
    if !table_exists(conn, "settings")? {
        return Ok(HashMap::new());
    }
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut settings = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(|e| AppError::Database(e.to_string()))?;
        settings.insert(key, value);
    }
    Ok(settings)
}

fn read_legacy_providers(conn: &Connection) -> Result<HashMap<String, AppProviders>, AppError> {
    if !table_exists(conn, "providers")? {
        return Ok(HashMap::new());
    }
    let mut stmt = conn.prepare(
        "SELECT id, app_type, name, settings_config, website_url, category, created_at, sort_index, notes, icon, icon_color, meta, is_current
         FROM providers ORDER BY app_type ASC, COALESCE(sort_index, 999999), created_at ASC, id ASC",
    ).map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let app_type: String = row.get(1)?;
            let settings_config_str: String = row.get(3)?;
            let meta_str: String = row.get(11)?;
            let settings_config =
                serde_json::from_str(&settings_config_str).unwrap_or(serde_json::Value::Null);
            let meta: ProviderMeta = serde_json::from_str(&meta_str).unwrap_or_default();
            let provider = Provider {
                id: id.clone(),
                name: row.get(2)?,
                settings_config,
                website_url: row.get(4)?,
                category: row.get(5)?,
                created_at: row.get(6)?,
                sort_index: row.get(7)?,
                notes: row.get(8)?,
                icon: row.get(9)?,
                icon_color: row.get(10)?,
                meta: Some(meta),
            };
            Ok((app_type, id, provider, row.get::<_, bool>(12)?))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;

    let endpoint_map = read_legacy_provider_endpoints(conn)?;
    let mut providers: HashMap<String, AppProviders> = HashMap::new();
    for row in rows {
        let (app_type, id, mut provider, is_current) =
            row.map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(endpoints) = endpoint_map.get(&(app_type.clone(), id.clone())) {
            let meta = provider.meta.get_or_insert_with(Default::default);
            meta.custom_endpoints = endpoints.clone();
        }
        let app = providers.entry(app_type).or_default();
        if is_current {
            app.current_id = Some(id.clone());
        }
        app.providers.insert(id, provider);
    }
    Ok(providers)
}

type EndpointMap = HashMap<(String, String), HashMap<String, crate::settings::CustomEndpoint>>;

fn read_legacy_provider_endpoints(conn: &Connection) -> Result<EndpointMap, AppError> {
    if !table_exists(conn, "provider_endpoints")? {
        return Ok(HashMap::new());
    }
    let mut stmt = conn
        .prepare("SELECT provider_id, app_type, url, added_at FROM provider_endpoints ORDER BY added_at ASC, url ASC")
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            let provider_id: String = row.get(0)?;
            let app_type: String = row.get(1)?;
            let url: String = row.get(2)?;
            let added_at: Option<i64> = row.get(3)?;
            Ok((
                (app_type, provider_id),
                url.clone(),
                crate::settings::CustomEndpoint {
                    url,
                    added_at: added_at.unwrap_or(0),
                    last_used: None,
                },
            ))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut map: EndpointMap = HashMap::new();
    for row in rows {
        let (key, url, endpoint) = row.map_err(|e| AppError::Database(e.to_string()))?;
        map.entry(key).or_default().insert(url, endpoint);
    }
    Ok(map)
}

fn read_legacy_mcp_servers(
    conn: &Connection,
) -> Result<IndexMap<String, crate::app_config::McpServer>, AppError> {
    if !table_exists(conn, "mcp_servers")? {
        return Ok(IndexMap::new());
    }
    let mut stmt = conn.prepare(
        "SELECT id, name, server_config, description, homepage, docs, tags, enabled_claude, enabled_codex, enabled_gemini, enabled_opencode, enabled_hermes
         FROM mcp_servers ORDER BY name ASC, id ASC",
    ).map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let server_config_str: String = row.get(2)?;
            let tags_str: String = row.get(6)?;
            Ok((
                id.clone(),
                crate::app_config::McpServer {
                    id,
                    name: row.get(1)?,
                    server: serde_json::from_str(&server_config_str).unwrap_or_default(),
                    apps: McpApps {
                        claude: row.get(7)?,
                    },
                    description: row.get(3)?,
                    homepage: row.get(4)?,
                    docs: row.get(5)?,
                    tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                },
            ))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut servers = IndexMap::new();
    for row in rows {
        let (id, server) = row.map_err(|e| AppError::Database(e.to_string()))?;
        servers.insert(id, server);
    }
    Ok(servers)
}

fn read_legacy_prompts(
    conn: &Connection,
) -> Result<HashMap<String, IndexMap<String, Prompt>>, AppError> {
    if !table_exists(conn, "prompts")? {
        return Ok(HashMap::new());
    }
    let mut stmt = conn
        .prepare(
            "SELECT id, app_type, name, content, description, enabled, created_at, updated_at
         FROM prompts ORDER BY app_type ASC, created_at ASC, id ASC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let app_type: String = row.get(1)?;
            Ok((
                app_type,
                id.clone(),
                Prompt {
                    id,
                    name: row.get(2)?,
                    content: row.get(3)?,
                    description: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                },
            ))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut prompts: HashMap<String, IndexMap<String, Prompt>> = HashMap::new();
    for row in rows {
        let (app_type, id, prompt) = row.map_err(|e| AppError::Database(e.to_string()))?;
        prompts.entry(app_type).or_default().insert(id, prompt);
    }
    Ok(prompts)
}

fn read_legacy_skills(conn: &Connection) -> Result<IndexMap<String, InstalledSkill>, AppError> {
    if !table_exists(conn, "skills")? {
        return Ok(IndexMap::new());
    }
    let mut stmt = conn.prepare(
        "SELECT id, name, description, directory, repo_owner, repo_name, repo_branch, readme_url,
                enabled_claude, enabled_codex, enabled_gemini, enabled_opencode, enabled_hermes,
                installed_at, content_hash, updated_at
         FROM skills ORDER BY name ASC",
    ).map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(InstalledSkill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                directory: row.get(3)?,
                repo_owner: row.get(4)?,
                repo_name: row.get(5)?,
                repo_branch: row.get(6)?,
                readme_url: row.get(7)?,
                apps: SkillApps {
                    claude: row.get(8)?,
                },
                installed_at: row.get(13)?,
                content_hash: row.get(14)?,
                updated_at: row.get::<_, i64>(15).unwrap_or(0),
            })
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut skills = IndexMap::new();
    for row in rows {
        let skill = row.map_err(|e| AppError::Database(e.to_string()))?;
        skills.insert(skill.id.clone(), skill);
    }
    Ok(skills)
}

fn read_legacy_skill_repos(conn: &Connection) -> Result<Vec<SkillRepo>, AppError> {
    if !table_exists(conn, "skill_repos")? {
        return Ok(Vec::new());
    }
    let mut stmt = conn
        .prepare(
            "SELECT owner, name, branch, enabled FROM skill_repos ORDER BY owner ASC, name ASC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SkillRepo {
                owner: row.get(0)?,
                name: row.get(1)?,
                branch: row.get(2)?,
                enabled: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut repos = Vec::new();
    for row in rows {
        repos.push(row.map_err(|e| AppError::Database(e.to_string()))?);
    }
    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use serde_json::json;

    fn create_legacy_db(dir: &Path) -> Connection {
        std::fs::create_dir_all(dir).unwrap();
        let conn = Connection::open(dir.join("cc-switch.db")).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE providers (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                name TEXT NOT NULL,
                settings_config TEXT NOT NULL,
                website_url TEXT,
                category TEXT,
                created_at INTEGER,
                sort_index INTEGER,
                notes TEXT,
                icon TEXT,
                icon_color TEXT,
                meta TEXT NOT NULL DEFAULT '{}',
                is_current BOOLEAN NOT NULL DEFAULT 0,
                in_failover_queue BOOLEAN NOT NULL DEFAULT 0,
                PRIMARY KEY (id, app_type)
            );
            CREATE TABLE provider_endpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider_id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                url TEXT NOT NULL,
                added_at INTEGER
            );
            CREATE TABLE mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                server_config TEXT NOT NULL,
                description TEXT,
                homepage TEXT,
                docs TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                enabled_claude BOOLEAN NOT NULL DEFAULT 0,
                enabled_codex BOOLEAN NOT NULL DEFAULT 0,
                enabled_gemini BOOLEAN NOT NULL DEFAULT 0,
                enabled_opencode BOOLEAN NOT NULL DEFAULT 0,
                enabled_hermes BOOLEAN NOT NULL DEFAULT 0
            );
            CREATE TABLE prompts (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                description TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER,
                updated_at INTEGER,
                PRIMARY KEY (id, app_type)
            );
            CREATE TABLE skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                directory TEXT NOT NULL,
                repo_owner TEXT,
                repo_name TEXT,
                repo_branch TEXT,
                readme_url TEXT,
                enabled_claude BOOLEAN NOT NULL DEFAULT 0,
                enabled_codex BOOLEAN NOT NULL DEFAULT 0,
                enabled_gemini BOOLEAN NOT NULL DEFAULT 0,
                enabled_opencode BOOLEAN NOT NULL DEFAULT 0,
                enabled_hermes BOOLEAN NOT NULL DEFAULT 0,
                installed_at INTEGER NOT NULL,
                content_hash TEXT,
                updated_at INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE skill_repos (
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                branch TEXT NOT NULL DEFAULT 'main',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                PRIMARY KEY (owner, name)
            );
            CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn migrates_legacy_sqlite_to_domain_json_files() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        let conn = create_legacy_db(dir);
        conn.execute(
            "INSERT INTO providers (id, app_type, name, settings_config, website_url, category, created_at, sort_index, notes, icon, icon_color, meta, is_current)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                "p1",
                "claude",
                "Provider One",
                json!({"env":{"ANTHROPIC_AUTH_TOKEN":"sk"}}).to_string(),
                "https://example.com",
                "custom",
                100_i64,
                1_i64,
                "note",
                "anthropic",
                "#fff",
                "{}",
                true,
            ],
        ).unwrap();
        conn.execute(
            "INSERT INTO provider_endpoints (provider_id, app_type, url, added_at) VALUES (?1, ?2, ?3, ?4)",
            params!["p1", "claude", "https://api.example.com", 123_i64],
        ).unwrap();
        conn.execute(
            "INSERT INTO mcp_servers (id, name, server_config, tags, enabled_claude) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["m1", "MCP One", json!({"command":"node"}).to_string(), json!(["dev"]).to_string(), true],
        ).unwrap();
        conn.execute(
            "INSERT INTO prompts (id, app_type, name, content, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["prompt1", "claude", "Prompt One", "content", true, 10_i64, 20_i64],
        ).unwrap();
        conn.execute(
            "INSERT INTO skills (id, name, directory, enabled_claude, installed_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["skill1", "Skill One", "skill-one", true, 10_i64, 20_i64],
        ).unwrap();
        conn.execute(
            "INSERT INTO skill_repos (owner, name, branch, enabled) VALUES (?1, ?2, ?3, ?4)",
            params!["owner", "repo", "main", true],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            params!["global_proxy_url", "http://127.0.0.1:7890"],
        )
        .unwrap();
        drop(conn);

        JsonStore::migrate_from_legacy_sqlite_if_needed(dir).unwrap();
        let store = JsonStore {
            dir: dir.to_path_buf(),
            cache: std::sync::RwLock::new(JsonStore::load_all(dir).unwrap()),
        };

        let providers = store.get_all_providers("claude").unwrap();
        assert_eq!(
            store.get_current_provider("claude").unwrap().as_deref(),
            Some("p1")
        );
        assert_eq!(providers["p1"].name, "Provider One");
        assert!(providers["p1"]
            .meta
            .as_ref()
            .unwrap()
            .custom_endpoints
            .contains_key("https://api.example.com"));
        assert_eq!(
            store.get_setting("global_proxy_url").unwrap().as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert!(store.get_all_mcp_servers().unwrap().contains_key("m1"));
        assert!(store.get_prompts("claude").unwrap().contains_key("prompt1"));
        assert!(store
            .get_all_installed_skills()
            .unwrap()
            .contains_key("skill1"));
        assert_eq!(store.get_skill_repos().unwrap().len(), 1);
    }

    #[test]
    fn legacy_sqlite_migration_does_not_overwrite_existing_json() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        let conn = create_legacy_db(dir);
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            params!["global_proxy_url", "http://legacy"],
        )
        .unwrap();
        drop(conn);
        crate::config::atomic_write(
            &dir.join("config.json"),
            br#"{"global_proxy_url":"http://json"}"#,
        )
        .unwrap();

        JsonStore::migrate_from_legacy_sqlite_if_needed(dir).unwrap();
        let config: HashMap<String, String> = JsonStore::load_json(dir.join("config.json"));
        assert_eq!(
            config.get("global_proxy_url").map(String::as_str),
            Some("http://json")
        );
    }
}
