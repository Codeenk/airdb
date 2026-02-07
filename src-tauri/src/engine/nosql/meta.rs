//! Meta information for NoSQL storage format
//! 
//! Contains version info for safe updates/rollbacks

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

use super::error::{NoSqlError, Result};

/// Current format version supported by this engine
pub const CURRENT_FORMAT_VERSION: u32 = 1;

/// Minimum format version we can read
pub const MIN_FORMAT_VERSION: u32 = 1;

/// Meta information stored in _meta.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// Engine identifier
    pub engine: String,
    
    /// Format version number
    pub format_version: u32,
    
    /// Minimum app version required to read this data
    pub min_app_version: String,
    
    /// When this store was created
    pub created_at: DateTime<Utc>,
    
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Meta {
    /// Create new meta for a fresh NoSQL store
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            engine: "airdb-nosql".to_string(),
            format_version: CURRENT_FORMAT_VERSION,
            min_app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            modified_at: now,
            description: None,
        }
    }

    /// Load meta from a directory
    pub fn load(base_path: &Path) -> Result<Self> {
        let meta_path = base_path.join("_meta.json");
        let content = fs::read_to_string(&meta_path)?;
        let meta: Meta = serde_json::from_str(&content)?;
        
        // Version compatibility check
        if meta.format_version < MIN_FORMAT_VERSION || meta.format_version > CURRENT_FORMAT_VERSION {
            return Err(NoSqlError::UnsupportedFormatVersion {
                found: meta.format_version,
                min: MIN_FORMAT_VERSION,
                max: CURRENT_FORMAT_VERSION,
            });
        }
        
        Ok(meta)
    }

    /// Save meta to a directory
    pub fn save(&self, base_path: &Path) -> Result<()> {
        let meta_path = base_path.join("_meta.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(meta_path, content)?;
        Ok(())
    }

    /// Update modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Check if this meta is compatible with current app version
    pub fn check_app_compatibility(&self) -> Result<()> {
        let current_version = env!("CARGO_PKG_VERSION");
        
        // Simple version comparison (for production, use semver crate)
        if self.min_app_version > current_version.to_string() {
            return Err(NoSqlError::AppVersionTooOld {
                app: current_version.to_string(),
                required: self.min_app_version.clone(),
            });
        }
        
        Ok(())
    }
}

impl Default for Meta {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_meta_save_load() {
        let dir = tempdir().unwrap();
        let meta = Meta::new();
        
        meta.save(dir.path()).unwrap();
        let loaded = Meta::load(dir.path()).unwrap();
        
        assert_eq!(loaded.engine, "airdb-nosql");
        assert_eq!(loaded.format_version, CURRENT_FORMAT_VERSION);
    }
}
