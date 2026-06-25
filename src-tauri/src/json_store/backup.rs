use super::JsonStore;
use crate::config::get_app_config_dir;
use crate::error::AppError;

/// 备份条目（供 UI 展示）
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
}
use chrono::{DateTime, Utc};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const JSON_FILES: &[&str] = &[
    "providers.json",
    "config.json",
    "mcp.json",
    "prompts.json",
    "skills.json",
];

impl JsonStore {
    fn backup_dir() -> PathBuf {
        get_app_config_dir().join("backups")
    }

    fn validate_zip_filename(filename: &str) -> Result<(), AppError> {
        if filename.contains("..")
            || filename.contains('/')
            || filename.contains('\\')
            || !filename.ends_with(".zip")
        {
            return Err(AppError::InvalidInput(
                "Invalid backup filename".to_string(),
            ));
        }
        Ok(())
    }

    fn zip_config_files(&self) -> Result<Vec<u8>, AppError> {
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            for name in JSON_FILES {
                let path = self.dir.join(name);
                if !path.exists() {
                    continue;
                }
                let content = fs::read(&path).map_err(|e| AppError::io(&path, e))?;
                zip.start_file(*name, options)
                    .map_err(|e| AppError::Config(format!("Zip error: {e}")))?;
                zip.write_all(&content)
                    .map_err(|e| AppError::Config(format!("Zip write error: {e}")))?;
            }
            zip.finish()
                .map_err(|e| AppError::Config(format!("Zip finish error: {e}")))?;
        }
        Ok(buf)
    }

    fn extract_zip_to_config(&self, zip_bytes: Vec<u8>) -> Result<(), AppError> {
        let cursor = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| AppError::Config(format!("Invalid zip archive: {e}")))?;

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| AppError::Config(format!("Zip read error: {e}")))?;
            let name = entry.name().to_string();
            if !JSON_FILES.contains(&name.as_str()) {
                continue;
            }
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .map_err(|e| AppError::Config(format!("Zip extract error: {e}")))?;
            crate::config::atomic_write(&self.dir.join(&name), &content)?;
        }
        Ok(())
    }

    fn cleanup_zip_backups(dir: &Path) -> Result<(), AppError> {
        let retain = crate::settings::effective_backup_retain_count();
        let entries: Vec<_> = match fs::read_dir(dir) {
            Ok(iter) => iter
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "zip")
                        .unwrap_or(false)
                })
                .collect(),
            Err(_) => return Ok(()),
        };

        if entries.len() <= retain {
            return Ok(());
        }

        let remove_count = entries.len().saturating_sub(retain);
        let mut sorted = entries;
        sorted.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

        for entry in sorted.into_iter().take(remove_count) {
            if let Err(err) = fs::remove_file(entry.path()) {
                log::warn!("删除旧备份失败 {}: {}", entry.path().display(), err);
            }
        }
        Ok(())
    }

    // ─── 公开接口 ─────────────────────────────────────────────────────────────

    pub fn backup_database_file(&self) -> Result<Option<PathBuf>, AppError> {
        let backup_dir = Self::backup_dir();
        fs::create_dir_all(&backup_dir).map_err(|e| AppError::io(&backup_dir, e))?;

        let base = format!("cc-config_{}", Utc::now().format("%Y%m%d_%H%M%S"));
        let mut filename = format!("{base}.zip");
        let mut backup_path = backup_dir.join(&filename);
        let mut counter = 1u32;
        while backup_path.exists() {
            filename = format!("{base}_{counter}.zip");
            backup_path = backup_dir.join(&filename);
            counter += 1;
        }

        let zip_bytes = self.zip_config_files()?;
        crate::config::atomic_write(&backup_path, &zip_bytes)?;

        Self::cleanup_zip_backups(&backup_dir)?;
        Ok(Some(backup_path))
    }

    pub fn periodic_backup_if_needed(&self) -> Result<(), AppError> {
        let interval_hours = crate::settings::effective_backup_interval_hours();
        if interval_hours == 0 {
            return Ok(());
        }

        let backup_dir = Self::backup_dir();
        if !backup_dir.exists() {
            self.backup_database_file()?;
            return Ok(());
        }

        let latest = fs::read_dir(&backup_dir).ok().and_then(|iter| {
            iter.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "zip")
                        .unwrap_or(false)
                })
                .filter_map(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
                .max()
        });

        let interval_secs = u64::from(interval_hours) * 3600;
        let needs_backup = match latest {
            None => true,
            Some(last_modified) => {
                last_modified.elapsed().unwrap_or_default()
                    > std::time::Duration::from_secs(interval_secs)
            }
        };

        if needs_backup {
            log::info!(
                "Periodic backup: latest is older than {interval_hours}h, creating new backup"
            );
            self.backup_database_file()?;
        }

        Ok(())
    }

    pub fn restore_from_backup(&self, filename: &str) -> Result<String, AppError> {
        Self::validate_zip_filename(filename)?;

        let backup_path = Self::backup_dir().join(filename);
        if !backup_path.exists() {
            return Err(AppError::InvalidInput(format!(
                "Backup file not found: {filename}"
            )));
        }

        // Safety backup before overwriting
        let safety_path = self.backup_database_file()?;
        let safety_id = safety_path
            .as_ref()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_default();

        let zip_bytes = fs::read(&backup_path).map_err(|e| AppError::io(&backup_path, e))?;
        self.extract_zip_to_config(zip_bytes)?;
        self.reload_cache()?;

        Ok(safety_id)
    }

    pub(crate) fn reload_cache(&self) -> Result<(), AppError> {
        let new_cache = Self::load_all(&self.dir)?;
        let mut cache = self.cache.write().map_err(super::rw_err)?;
        *cache = new_cache;
        Ok(())
    }

    pub fn list_backups() -> Result<Vec<BackupEntry>, AppError> {
        let backup_dir = Self::backup_dir();
        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut entries: Vec<BackupEntry> = fs::read_dir(&backup_dir)
            .map_err(|e| AppError::io(&backup_dir, e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "zip")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let metadata = e.metadata().ok()?;
                let filename = e.file_name().to_string_lossy().to_string();
                let size_bytes = metadata.len();
                let created_at = metadata
                    .modified()
                    .ok()
                    .map(|t| {
                        let dt: DateTime<Utc> = t.into();
                        dt.to_rfc3339()
                    })
                    .unwrap_or_default();
                Some(BackupEntry {
                    filename,
                    size_bytes,
                    created_at,
                })
            })
            .collect();

        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(entries)
    }

    pub fn rename_backup(old_filename: &str, new_name: &str) -> Result<String, AppError> {
        Self::validate_zip_filename(old_filename)?;

        let new_filename = if new_name.ends_with(".zip") {
            new_name.to_string()
        } else {
            format!("{new_name}.zip")
        };

        if new_filename.contains("..") || new_filename.contains('/') || new_filename.contains('\\')
        {
            return Err(AppError::InvalidInput(
                "Invalid new backup name".to_string(),
            ));
        }

        let backup_dir = Self::backup_dir();
        let old_path = backup_dir.join(old_filename);
        let new_path = backup_dir.join(&new_filename);

        if !old_path.exists() {
            return Err(AppError::InvalidInput(format!(
                "Backup not found: {old_filename}"
            )));
        }
        if new_path.exists() {
            return Err(AppError::InvalidInput(format!(
                "Backup name already in use: {new_filename}"
            )));
        }

        fs::rename(&old_path, &new_path).map_err(|e| AppError::io(&old_path, e))?;
        Ok(new_filename)
    }

    pub fn delete_backup(filename: &str) -> Result<(), AppError> {
        Self::validate_zip_filename(filename)?;

        let backup_path = Self::backup_dir().join(filename);
        if !backup_path.exists() {
            return Err(AppError::InvalidInput(format!(
                "Backup not found: {filename}"
            )));
        }

        fs::remove_file(&backup_path).map_err(|e| AppError::io(&backup_path, e))?;
        Ok(())
    }

    /// 导出所有配置为可读 JSON bundle 文件（用户可手动编辑）
    pub fn export_sql(&self, target_path: &Path) -> Result<(), AppError> {
        if let Some(parent) = target_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
            }
        }
        let json_str = self.export_sql_string_for_sync()?;
        crate::config::atomic_write(target_path, json_str.as_bytes())
    }

    /// 从 cc-config JSON bundle 导入配置文件，返回安全备份 ID
    pub fn import_sql(&self, source_path: &Path) -> Result<String, AppError> {
        if !source_path.exists() {
            return Err(AppError::InvalidInput(format!(
                "File not found: {}",
                source_path.display()
            )));
        }

        let content = fs::read_to_string(source_path).map_err(|e| AppError::io(source_path, e))?;

        // 验证格式是否为 cc-config JSON bundle
        let bundle: serde_json::Value = serde_json::from_str(&content).map_err(|_| {
            AppError::localized(
                "backup.sql.invalid_format",
                "不是有效的 cc-config 配置文件",
                "Not a valid cc-config configuration file",
            )
        })?;
        if bundle.get("format").and_then(|v| v.as_str()) != Some("cc-config-json-v1") {
            return Err(AppError::localized(
                "backup.sql.invalid_format",
                "不是有效的 cc-config 配置文件",
                "Not a valid cc-config configuration file",
            ));
        }

        self.import_sql_string_for_sync(&content)
    }
}
