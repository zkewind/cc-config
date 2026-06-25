use super::{rw_err, JsonStore};
use crate::app_config::McpServer;
use crate::error::AppError;
use indexmap::IndexMap;

impl JsonStore {
    pub fn get_all_mcp_servers(&self) -> Result<IndexMap<String, McpServer>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.mcp.servers.clone())
    }

    pub fn save_mcp_server(&self, server: &McpServer) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache.mcp.servers.insert(server.id.clone(), server.clone());
            serde_json::to_vec_pretty(&cache.mcp)
                .map_err(|e| AppError::Config(format!("序列化 mcp 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("mcp.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("mcp_servers");
        Ok(())
    }

    pub fn delete_mcp_server(&self, id: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache.mcp.servers.shift_remove(id);
            serde_json::to_vec_pretty(&cache.mcp)
                .map_err(|e| AppError::Config(format!("序列化 mcp 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("mcp.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("mcp_servers");
        Ok(())
    }
}
