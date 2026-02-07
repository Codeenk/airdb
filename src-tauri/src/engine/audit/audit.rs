//! Immutable Audit Log
//!
//! Append-only, line-delimited JSON audit log

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Current audit log format version
pub const AUDIT_VERSION: u32 = 1;

/// Audit action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Document operations
    Insert,
    Update,
    Delete,
    
    // Schema operations
    SchemaCreate,
    SchemaUpdate,
    SchemaMigrate,
    
    // Collection operations
    CollectionCreate,
    CollectionDrop,
    
    // Relation operations
    RelationCreate,
    RelationDelete,
    
    // Access operations
    PolicyUpdate,
    RoleAssign,
    
    // System operations
    AppUpdate,
    Backup,
    Restore,
    
    // Custom
    Custom(String),
}

/// A single audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Format version for compatibility
    pub version: u32,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Action performed
    pub action: AuditAction,
    
    /// Resource type (collection, table, etc.)
    pub resource_type: String,
    
    /// Resource name
    pub resource_name: String,
    
    /// User or API key that performed the action
    pub actor: Option<String>,
    
    /// Additional metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    
    /// Before state (for updates)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<serde_json::Value>,
    
    /// After state (for updates)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<serde_json::Value>,
}

impl AuditEntry {
    pub fn new(action: AuditAction, resource_type: &str, resource_name: &str) -> Self {
        Self {
            version: AUDIT_VERSION,
            timestamp: Utc::now(),
            action,
            resource_type: resource_type.to_string(),
            resource_name: resource_name.to_string(),
            actor: None,
            metadata: None,
            before: None,
            after: None,
        }
    }

    pub fn with_actor(mut self, actor: &str) -> Self {
        self.actor = Some(actor.to_string());
        self
    }

    pub fn with_before(mut self, before: serde_json::Value) -> Self {
        self.before = Some(before);
        self
    }

    pub fn with_after(mut self, after: serde_json::Value) -> Self {
        self.after = Some(after);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// The audit log manager
pub struct AuditLog {
    log_path: PathBuf,
}

impl AuditLog {
    /// Create a new audit log for a project
    pub fn new(project_dir: &Path) -> std::io::Result<Self> {
        let audit_dir = project_dir.join("audit");
        fs::create_dir_all(&audit_dir)?;
        
        let log_path = audit_dir.join("audit.jsonl");
        
        Ok(Self { log_path })
    }

    /// Append an entry to the audit log
    pub fn append(&self, entry: &AuditEntry) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        
        let line = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Query all entries
    pub fn query_all(&self) -> std::io::Result<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(vec![]);
        }

        let file = fs::File::open(&self.log_path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: AuditEntry = serde_json::from_str(&line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            
            // Version compatibility - older entries are readable
            if entry.version <= AUDIT_VERSION {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    /// Query entries filtered by action
    pub fn query_by_action(&self, action: &AuditAction) -> std::io::Result<Vec<AuditEntry>> {
        let all = self.query_all()?;
        let action_str = serde_json::to_string(action)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        Ok(all.into_iter()
            .filter(|e| {
                let e_str = serde_json::to_string(&e.action).unwrap_or_default();
                e_str == action_str
            })
            .collect())
    }

    /// Query entries for a specific resource
    pub fn query_by_resource(&self, resource_type: &str, resource_name: &str) -> std::io::Result<Vec<AuditEntry>> {
        let all = self.query_all()?;
        
        Ok(all.into_iter()
            .filter(|e| e.resource_type == resource_type && e.resource_name == resource_name)
            .collect())
    }

    /// Get entry count
    pub fn count(&self) -> std::io::Result<usize> {
        if !self.log_path.exists() {
            return Ok(0);
        }

        let file = fs::File::open(&self.log_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audit_log_append_query() {
        let dir = tempdir().unwrap();
        let log = AuditLog::new(dir.path()).unwrap();
        
        let entry = AuditEntry::new(AuditAction::Insert, "collection", "users")
            .with_actor("test-user");
        
        log.append(&entry).unwrap();
        
        let entries = log.query_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].resource_name, "users");
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::new(AuditAction::Update, "document", "doc123")
            .with_before(serde_json::json!({"old": true}))
            .with_after(serde_json::json!({"old": false}))
            .with_metadata(serde_json::json!({"reason": "test"}));
        
        assert!(entry.before.is_some());
        assert!(entry.after.is_some());
        assert!(entry.metadata.is_some());
    }
}
