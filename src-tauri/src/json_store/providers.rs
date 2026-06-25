use super::{rw_err, JsonStore};
use crate::error::AppError;
use crate::provider::Provider;
use indexmap::IndexMap;
use std::collections::HashSet;

impl JsonStore {
    pub fn get_all_providers(
        &self,
        app_type: &str,
    ) -> Result<IndexMap<String, Provider>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache
            .providers
            .get(app_type)
            .map(|a| a.providers.clone())
            .unwrap_or_default())
    }

    pub fn get_current_provider(&self, app_type: &str) -> Result<Option<String>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache
            .providers
            .get(app_type)
            .and_then(|a| a.current_id.clone()))
    }

    pub fn get_provider_by_id(
        &self,
        id: &str,
        app_type: &str,
    ) -> Result<Option<Provider>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache
            .providers
            .get(app_type)
            .and_then(|a| a.providers.get(id))
            .cloned())
    }

    pub fn save_provider(&self, app_type: &str, provider: &Provider) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache
                .providers
                .entry(app_type.to_string())
                .or_default()
                .providers
                .insert(provider.id.clone(), provider.clone());
            serde_json::to_vec_pretty(&cache.providers)
                .map_err(|e| AppError::Config(format!("序列化 providers 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("providers.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("providers");
        Ok(())
    }

    pub fn delete_provider(&self, app_type: &str, id: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            if let Some(app) = cache.providers.get_mut(app_type) {
                app.providers.shift_remove(id);
                if app.current_id.as_deref() == Some(id) {
                    app.current_id = None;
                }
            }
            serde_json::to_vec_pretty(&cache.providers)
                .map_err(|e| AppError::Config(format!("序列化 providers 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("providers.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("providers");
        Ok(())
    }

    pub fn set_current_provider(&self, app_type: &str, id: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache
                .providers
                .entry(app_type.to_string())
                .or_default()
                .current_id = Some(id.to_string());
            serde_json::to_vec_pretty(&cache.providers)
                .map_err(|e| AppError::Config(format!("序列化 providers 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("providers.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("providers");
        Ok(())
    }

    pub fn update_provider_settings_config(
        &self,
        app_type: &str,
        provider_id: &str,
        settings_config: &serde_json::Value,
    ) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            if let Some(app) = cache.providers.get_mut(app_type) {
                if let Some(p) = app.providers.get_mut(provider_id) {
                    p.settings_config = settings_config.clone();
                }
            }
            serde_json::to_vec_pretty(&cache.providers)
                .map_err(|e| AppError::Config(format!("序列化 providers 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("providers.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("providers");
        Ok(())
    }

    pub fn add_custom_endpoint(
        &self,
        app_type: &str,
        provider_id: &str,
        url: &str,
    ) -> Result<(), AppError> {
        let added_at = chrono::Utc::now().timestamp_millis();
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            if let Some(app) = cache.providers.get_mut(app_type) {
                if let Some(p) = app.providers.get_mut(provider_id) {
                    let meta = p.meta.get_or_insert_with(Default::default);
                    meta.custom_endpoints.insert(
                        url.to_string(),
                        crate::settings::CustomEndpoint {
                            url: url.to_string(),
                            added_at,
                            last_used: None,
                        },
                    );
                }
            }
            serde_json::to_vec_pretty(&cache.providers)
                .map_err(|e| AppError::Config(format!("序列化 providers 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("providers.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("providers");
        Ok(())
    }

    pub fn remove_custom_endpoint(
        &self,
        app_type: &str,
        provider_id: &str,
        url: &str,
    ) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            if let Some(app) = cache.providers.get_mut(app_type) {
                if let Some(p) = app.providers.get_mut(provider_id) {
                    if let Some(meta) = &mut p.meta {
                        meta.custom_endpoints.remove(url);
                    }
                }
            }
            serde_json::to_vec_pretty(&cache.providers)
                .map_err(|e| AppError::Config(format!("序列化 providers 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("providers.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("providers");
        Ok(())
    }

    pub fn is_providers_empty(&self) -> Result<bool, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.providers.values().all(|a| a.providers.is_empty()))
    }

    pub fn get_provider_ids(&self, app_type: &str) -> Result<HashSet<String>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache
            .providers
            .get(app_type)
            .map(|a| a.providers.keys().cloned().collect())
            .unwrap_or_default())
    }

    pub fn has_any_provider_for_app(&self, app_type: &str) -> Result<bool, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache
            .providers
            .get(app_type)
            .map(|a| !a.providers.is_empty())
            .unwrap_or(false))
    }

    pub fn has_non_official_seed_provider(&self, app_type: &str) -> Result<bool, AppError> {
        use crate::json_store::providers_seed::is_official_seed_id;
        let cache = self.cache.read().map_err(rw_err)?;
        if let Some(app) = cache.providers.get(app_type) {
            for id in app.providers.keys() {
                if !is_official_seed_id(id) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn next_sort_index_for_app(&self, app_type: &str) -> Result<usize, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        let max = cache
            .providers
            .get(app_type)
            .and_then(|a| a.providers.values().filter_map(|p| p.sort_index).max());
        Ok(max.map(|v| v + 1).unwrap_or(0))
    }

    pub fn init_default_official_providers(&self) -> Result<usize, AppError> {
        use crate::json_store::providers_seed::OFFICIAL_SEEDS;

        if self
            .get_bool_flag("official_providers_seeded")
            .unwrap_or(false)
        {
            return Ok(0);
        }

        let mut inserted = 0_usize;
        let now_ms = chrono::Utc::now().timestamp_millis();

        for seed in OFFICIAL_SEEDS {
            let app_type_str = seed.app_type.as_str();

            if self.get_provider_by_id(seed.id, app_type_str)?.is_some() {
                continue;
            }

            let next_sort = self.next_sort_index_for_app(app_type_str)?;

            let settings_config: serde_json::Value =
                serde_json::from_str(seed.settings_config_json).map_err(|e| {
                    AppError::Database(format!("Seed JSON parse failed for {}: {e}", seed.id))
                })?;

            let mut provider = Provider::with_id(
                seed.id.to_string(),
                seed.name.to_string(),
                settings_config,
                Some(seed.website_url.to_string()),
            );
            provider.category = Some("official".to_string());
            provider.icon = Some(seed.icon.to_string());
            provider.icon_color = Some(seed.icon_color.to_string());
            provider.sort_index = Some(next_sort);
            provider.created_at = Some(now_ms);

            self.save_provider(app_type_str, &provider)?;
            inserted += 1;
            log::info!(
                "✓ Seeded official provider: {} ({})",
                seed.name,
                app_type_str
            );
        }

        self.set_setting("official_providers_seeded", "true")?;
        Ok(inserted)
    }
}
