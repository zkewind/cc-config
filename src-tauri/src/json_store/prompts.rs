use super::{rw_err, JsonStore};
use crate::error::AppError;
use crate::prompt::Prompt;
use indexmap::IndexMap;

impl JsonStore {
    pub fn get_prompts(&self, app_type: &str) -> Result<IndexMap<String, Prompt>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.prompts.get(app_type).cloned().unwrap_or_default())
    }

    pub fn save_prompt(&self, app_type: &str, prompt: &Prompt) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache
                .prompts
                .entry(app_type.to_string())
                .or_default()
                .insert(prompt.id.clone(), prompt.clone());
            serde_json::to_vec_pretty(&cache.prompts)
                .map_err(|e| AppError::Config(format!("序列化 prompts 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("prompts.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("prompts");
        Ok(())
    }

    pub fn delete_prompt(&self, app_type: &str, id: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            if let Some(app) = cache.prompts.get_mut(app_type) {
                app.shift_remove(id);
            }
            serde_json::to_vec_pretty(&cache.prompts)
                .map_err(|e| AppError::Config(format!("序列化 prompts 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("prompts.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("prompts");
        Ok(())
    }
}
