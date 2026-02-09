//! Update-Linked Backups
//!
//! Pre-update auto-backup with version tagging

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Backup options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    /// Include SQL data
    #[serde(default = "default_true")]
    pub include_sql: bool,
    
    /// Include NoSQL collections
    #[serde(default = "default_true")]
    pub include_nosql: bool,
    
    /// Include configuration
    #[serde(default = "default_true")]
    pub include_config: bool,
    
    /// Include audit logs
    #[serde(default)]
    pub include_audit: bool,
    
    /// Compression level (0 = none, 9 = max)
    #[serde(default)]
    pub compression: u8,
}

fn default_true() -> bool { true }

impl Default for BackupOptions {
    fn default() -> Self {
        Self {
            include_sql: true,
            include_nosql: true,
            include_config: true,
            include_audit: false,
            compression: 0,
        }
    }
}

/// Metadata for a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// Backup ID (timestamp-based)
    pub id: String,
    
    /// Timestamp
    pub created_at: DateTime<Utc>,
    
    /// App version at backup time
    pub app_version: String,
    
    /// Reason for backup
    pub reason: String,
    
    /// Link to update version (if pre-update backup)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_update_version: Option<String>,
    
    /// Schema version at backup time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    
    /// Files included
    pub files: Vec<String>,
    
    /// Total size in bytes
    pub total_size: u64,
    
    /// Verification checksum
    pub checksum: String,
}

/// The backup manager
pub struct BackupManager {
    project_dir: PathBuf,
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(project_dir: &Path) -> std::io::Result<Self> {
        let backup_dir = project_dir.join("backups");
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self {
            project_dir: project_dir.to_path_buf(),
            backup_dir,
        })
    }

    /// Create a backup before update
    pub fn create_pre_update_backup(
        &self,
        update_version: &str,
        options: &BackupOptions,
    ) -> std::io::Result<Backup> {
        let id = format!(
            "pre-update-{}-{}",
            update_version.replace('.', "-"),
            Utc::now().format("%Y%m%d-%H%M%S")
        );
        
        self.create_backup(&id, &format!("Pre-update to v{}", update_version), Some(update_version), options)
    }

    /// Create a manual backup
    pub fn create_backup(
        &self,
        id: &str,
        reason: &str,
        linked_version: Option<&str>,
        options: &BackupOptions,
    ) -> std::io::Result<Backup> {
        let backup_path = self.backup_dir.join(id);
        fs::create_dir_all(&backup_path)?;
        
        let mut files = Vec::new();
        let mut total_size = 0u64;
        
        // Copy SQL data
        if options.include_sql {
            let sql_src = self.project_dir.join("sql");
            if sql_src.exists() {
                let copied = copy_dir_recursive(&sql_src, &backup_path.join("sql"))?;
                files.extend(copied.0);
                total_size += copied.1;
            }
        }
        
        // Copy NoSQL data
        if options.include_nosql {
            let nosql_src = self.project_dir.join("nosql");
            if nosql_src.exists() {
                let copied = copy_dir_recursive(&nosql_src, &backup_path.join("nosql"))?;
                files.extend(copied.0);
                total_size += copied.1;
            }
        }
        
        // Copy config
        if options.include_config {
            let config_path = self.project_dir.join("airdb.config.json");
            if config_path.exists() {
                let dest = backup_path.join("airdb.config.json");
                fs::copy(&config_path, &dest)?;
                files.push("airdb.config.json".to_string());
                total_size += fs::metadata(&dest)?.len();
            }
        }
        
        // Copy audit logs
        if options.include_audit {
            let audit_src = self.project_dir.join("audit");
            if audit_src.exists() {
                let copied = copy_dir_recursive(&audit_src, &backup_path.join("audit"))?;
                files.extend(copied.0);
                total_size += copied.1;
            }
        }
        
        // Calculate checksum
        let checksum = calculate_dir_checksum(&backup_path)?;
        
        let backup = Backup {
            id: id.to_string(),
            created_at: Utc::now(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            reason: reason.to_string(),
            linked_update_version: linked_version.map(String::from),
            schema_version: None,
            files,
            total_size,
            checksum,
        };
        
        // Save metadata
        let meta_path = backup_path.join("backup.json");
        let meta_content = serde_json::to_string_pretty(&backup)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&meta_path, meta_content)?;
        
        Ok(backup)
    }

    /// List all backups
    pub fn list(&self) -> std::io::Result<Vec<Backup>> {
        let mut backups = Vec::new();
        
        if !self.backup_dir.exists() {
            return Ok(backups);
        }
        
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let meta_path = path.join("backup.json");
                if meta_path.exists() {
                    let content = fs::read_to_string(&meta_path)?;
                    if let Ok(backup) = serde_json::from_str(&content) {
                        backups.push(backup);
                    }
                }
            }
        }
        
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    /// Find backup linked to a specific update version
    pub fn find_for_update(&self, update_version: &str) -> std::io::Result<Option<Backup>> {
        let backups = self.list()?;
        Ok(backups.into_iter()
            .find(|b| b.linked_update_version.as_deref() == Some(update_version)))
    }

    /// Restore from a backup (aligned rollback)
    pub fn restore(&self, backup_id: &str) -> std::io::Result<()> {
        let backup_path = self.backup_dir.join(backup_id);
        
        if !backup_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Backup {} not found", backup_id)
            ));
        }
        
        // Load and verify backup
        let meta_path = backup_path.join("backup.json");
        let content = fs::read_to_string(&meta_path)?;
        let backup: Backup = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        // Verify checksum
        let current_checksum = calculate_dir_checksum(&backup_path)?;
        if current_checksum != backup.checksum {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Backup verification failed: checksum mismatch"
            ));
        }
        
        // Restore SQL
        let sql_backup = backup_path.join("sql");
        if sql_backup.exists() {
            let sql_dest = self.project_dir.join("sql");
            if sql_dest.exists() {
                fs::remove_dir_all(&sql_dest)?;
            }
            copy_dir_recursive(&sql_backup, &sql_dest)?;
        }
        
        // Restore NoSQL
        let nosql_backup = backup_path.join("nosql");
        if nosql_backup.exists() {
            let nosql_dest = self.project_dir.join("nosql");
            if nosql_dest.exists() {
                fs::remove_dir_all(&nosql_dest)?;
            }
            copy_dir_recursive(&nosql_backup, &nosql_dest)?;
        }
        
        // Restore config
        let config_backup = backup_path.join("airdb.config.json");
        if config_backup.exists() {
            fs::copy(&config_backup, self.project_dir.join("airdb.config.json"))?;
        }
        
        Ok(())
    }

    /// Verify a backup's integrity
    pub fn verify(&self, backup_id: &str) -> std::io::Result<bool> {
        let backup_path = self.backup_dir.join(backup_id);
        
        let meta_path = backup_path.join("backup.json");
        let content = fs::read_to_string(&meta_path)?;
        let backup: Backup = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let current_checksum = calculate_dir_checksum(&backup_path)?;
        Ok(current_checksum == backup.checksum)
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<(Vec<String>, u64)> {
    let mut files = Vec::new();
    let mut total_size = 0u64;
    
    fs::create_dir_all(dest)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if path.is_dir() {
            let (sub_files, sub_size) = copy_dir_recursive(&path, &dest_path)?;
            files.extend(sub_files);
            total_size += sub_size;
        } else {
            fs::copy(&path, &dest_path)?;
            let size = fs::metadata(&dest_path)?.len();
            files.push(path.file_name().unwrap().to_string_lossy().to_string());
            total_size += size;
        }
    }
    
    Ok((files, total_size))
}

/// Calculate a simple checksum for a directory
fn calculate_dir_checksum(dir: &Path) -> std::io::Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    
    fn hash_dir(path: &Path, hasher: &mut DefaultHasher) -> std::io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                
                // Skip backup.json itself
                if path.file_name().map(|n| n == "backup.json").unwrap_or(false) {
                    continue;
                }
                
                if path.is_dir() {
                    hash_dir(&path, hasher)?;
                } else {
                    let mut file = File::open(&path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    buffer.hash(hasher);
                }
            }
        }
        Ok(())
    }
    
    hash_dir(dir, &mut hasher)?;
    Ok(format!("{:016x}", hasher.finish()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_backup_create_list() {
        let dir = tempdir().unwrap();
        
        // Create some test data
        fs::create_dir_all(dir.path().join("nosql/test")).unwrap();
        fs::write(dir.path().join("nosql/test/doc.json"), r#"{"test": true}"#).unwrap();
        
        let manager = BackupManager::new(dir.path()).unwrap();
        
        let backup = manager.create_backup(
            "test-backup",
            "Test backup",
            None,
            &BackupOptions::default()
        ).unwrap();
        
        assert!(!backup.checksum.is_empty());
        
        let backups = manager.list().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, "test-backup");
    }

    #[test]
    fn test_backup_verify() {
        let dir = tempdir().unwrap();
        
        fs::create_dir_all(dir.path().join("nosql")).unwrap();
        fs::write(dir.path().join("nosql/data.json"), "test").unwrap();
        
        let manager = BackupManager::new(dir.path()).unwrap();
        manager.create_backup("verify-test", "Test", None, &BackupOptions::default()).unwrap();
        
        assert!(manager.verify("verify-test").unwrap());
    }
}
