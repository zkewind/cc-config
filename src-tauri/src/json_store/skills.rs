use super::{rw_err, JsonStore};
use crate::app_config::{InstalledSkill, SkillApps};
use crate::error::AppError;
use crate::services::skill::SkillRepo;
use indexmap::IndexMap;

impl JsonStore {
    pub fn get_all_installed_skills(&self) -> Result<IndexMap<String, InstalledSkill>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.skills.skills.clone())
    }

    pub fn get_installed_skill(&self, id: &str) -> Result<Option<InstalledSkill>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.skills.skills.get(id).cloned())
    }

    pub fn save_skill(&self, skill: &InstalledSkill) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache.skills.skills.insert(skill.id.clone(), skill.clone());
            serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("skills");
        Ok(())
    }

    pub fn delete_skill(&self, id: &str) -> Result<bool, AppError> {
        let (existed, bytes) = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            let existed = cache.skills.skills.shift_remove(id).is_some();
            let bytes = serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?;
            (existed, bytes)
        };
        crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("skills");
        Ok(existed)
    }

    pub fn clear_skills(&self) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache.skills.skills.clear();
            serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("skills");
        Ok(())
    }

    pub fn update_skill_apps(&self, id: &str, apps: &SkillApps) -> Result<bool, AppError> {
        let (existed, bytes) = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            let existed = if let Some(skill) = cache.skills.skills.get_mut(id) {
                skill.apps = apps.clone();
                true
            } else {
                false
            };
            let bytes = serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?;
            (existed, bytes)
        };
        if existed {
            crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
            crate::services::webdav_auto_sync::notify_db_changed("skills");
        }
        Ok(existed)
    }

    pub fn update_skill_hash(
        &self,
        id: &str,
        content_hash: &str,
        updated_at: i64,
    ) -> Result<bool, AppError> {
        let (existed, bytes) = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            let existed = if let Some(skill) = cache.skills.skills.get_mut(id) {
                skill.content_hash = Some(content_hash.to_string());
                skill.updated_at = updated_at;
                true
            } else {
                false
            };
            let bytes = serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?;
            (existed, bytes)
        };
        if existed {
            crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
            crate::services::webdav_auto_sync::notify_db_changed("skills");
        }
        Ok(existed)
    }

    // ─── Skill repos ──────────────────────────────────────────────────────────

    pub fn get_skill_repos(&self) -> Result<Vec<SkillRepo>, AppError> {
        let cache = self.cache.read().map_err(rw_err)?;
        Ok(cache.skills.repos.clone())
    }

    pub fn save_skill_repo(&self, repo: &SkillRepo) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            // Upsert by (owner, name)
            if let Some(existing) = cache
                .skills
                .repos
                .iter_mut()
                .find(|r| r.owner == repo.owner && r.name == repo.name)
            {
                *existing = repo.clone();
            } else {
                cache.skills.repos.push(repo.clone());
            }
            serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("skills");
        Ok(())
    }

    pub fn delete_skill_repo(&self, owner: &str, name: &str) -> Result<(), AppError> {
        let bytes = {
            let mut cache = self.cache.write().map_err(rw_err)?;
            cache
                .skills
                .repos
                .retain(|r| !(r.owner == owner && r.name == name));
            serde_json::to_vec_pretty(&cache.skills)
                .map_err(|e| AppError::Config(format!("序列化 skills 失败: {e}")))?
        };
        crate::config::atomic_write(&self.dir.join("skills.json"), &bytes)?;
        crate::services::webdav_auto_sync::notify_db_changed("skills");
        Ok(())
    }

    pub fn init_default_skill_repos(&self) -> Result<usize, AppError> {
        let existing = self.get_skill_repos()?;
        let existing_keys: std::collections::HashSet<(String, String)> = existing
            .iter()
            .map(|r| (r.owner.clone(), r.name.clone()))
            .collect();

        let default_store = crate::services::skill::SkillStore::default();
        let mut count = 0;

        for repo in &default_store.repos {
            let key = (repo.owner.clone(), repo.name.clone());
            if !existing_keys.contains(&key) {
                self.save_skill_repo(repo)?;
                count += 1;
                log::info!("补充默认 Skill 仓库: {}/{}", repo.owner, repo.name);
            }
        }

        if count > 0 {
            log::info!("补充默认 Skill 仓库完成，新增 {count} 个");
        }
        Ok(count)
    }
}
