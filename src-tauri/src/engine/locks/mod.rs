//! Operation Lock System
//!
//! Prevents conflicting operations during updates, migrations, backups, etc.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

/// Lock types for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    Migration,
    Backup,
    Serve,
    Update,
    BranchPreview,
}

impl LockType {
    pub fn filename(&self) -> &'static str {
        match self {
            LockType::Migration => "migration.lock",
            LockType::Backup => "backup.lock",
            LockType::Serve => "serve.lock",
            LockType::Update => "update.lock",
            LockType::BranchPreview => "branch_preview.lock",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LockType::Migration => "Database migration in progress",
            LockType::Backup => "Backup operation in progress",
            LockType::Serve => "API server running",
            LockType::Update => "Application update in progress",
            LockType::BranchPreview => "Branch preview server active",
        }
    }
}

/// Information stored in a lock file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_type: LockType,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub description: String,
    pub timeout_secs: Option<u64>,
}

impl LockInfo {
    pub fn new(lock_type: LockType) -> Self {
        Self {
            lock_type,
            pid: std::process::id(),
            started_at: Utc::now(),
            description: lock_type.description().to_string(),
            timeout_secs: None,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Check if lock has expired
    pub fn is_expired(&self) -> bool {
        if let Some(timeout) = self.timeout_secs {
            let elapsed = Utc::now().signed_duration_since(self.started_at);
            elapsed.num_seconds() as u64 > timeout
        } else {
            false
        }
    }

    /// Check if the process that created the lock is still running
    pub fn is_process_alive(&self) -> bool {
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill")
                .args(["-0", &self.pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        
        #[cfg(windows)]
        {
            // On Windows, check if process exists
            use std::process::Command;
            Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", self.pid)])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(&self.pid.to_string()))
                .unwrap_or(false)
        }
        
        #[cfg(not(any(unix, windows)))]
        true
    }
}

/// Operation lock manager
pub struct OperationLock {
    locks_dir: PathBuf,
}

impl OperationLock {
    /// Create a new lock manager for a project
    pub fn new(project_dir: &Path) -> Self {
        Self {
            locks_dir: project_dir.join(".airdb").join("locks"),
        }
    }

    /// Get the path for a specific lock type
    fn lock_path(&self, lock_type: LockType) -> PathBuf {
        self.locks_dir.join(lock_type.filename())
    }

    /// Acquire a lock
    pub fn acquire(&self, lock_type: LockType) -> Result<LockGuard, LockError> {
        // Check for blocking locks first
        self.check_blocking_locks(lock_type)?;
        
        // Create locks directory
        fs::create_dir_all(&self.locks_dir)
            .map_err(|e| LockError::IoError(e.to_string()))?;
        
        let lock_path = self.lock_path(lock_type);
        
        // Check if lock already exists
        if let Some(existing) = self.read_lock(&lock_path) {
            if existing.is_process_alive() && !existing.is_expired() {
                return Err(LockError::AlreadyLocked {
                    lock_type,
                    pid: existing.pid,
                    description: existing.description,
                });
            }
            // Stale lock, remove it
            let _ = fs::remove_file(&lock_path);
        }
        
        // Create new lock
        let info = LockInfo::new(lock_type);
        let content = serde_json::to_string_pretty(&info)
            .map_err(|e| LockError::SerializeError(e.to_string()))?;
        
        fs::write(&lock_path, content)
            .map_err(|e| LockError::IoError(e.to_string()))?;
        
        Ok(LockGuard {
            lock_path,
            lock_type,
        })
    }

    /// Read a lock file
    fn read_lock(&self, path: &Path) -> Option<LockInfo> {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Check for locks that would block the requested operation
    fn check_blocking_locks(&self, requested: LockType) -> Result<(), LockError> {
        let blocking = self.get_blocking_lock_types(requested);
        
        for lock_type in blocking {
            let lock_path = self.lock_path(lock_type);
            if let Some(info) = self.read_lock(&lock_path) {
                if info.is_process_alive() && !info.is_expired() {
                    return Err(LockError::BlockedBy {
                        requested,
                        blocking: lock_type,
                        description: info.description,
                    });
                }
            }
        }
        
        Ok(())
    }

    /// Get lock types that would block the requested operation
    fn get_blocking_lock_types(&self, requested: LockType) -> Vec<LockType> {
        match requested {
            LockType::Update => vec![
                LockType::Migration,
                LockType::Backup,
                LockType::Serve,
                LockType::BranchPreview,
            ],
            LockType::Migration => vec![
                LockType::Update,
                LockType::Backup,
            ],
            LockType::Backup => vec![
                LockType::Update,
                LockType::Migration,
            ],
            LockType::Serve => vec![
                LockType::Update,
            ],
            LockType::BranchPreview => vec![
                LockType::Update,
            ],
        }
    }

    /// Check if any locks are active
    pub fn get_active_locks(&self) -> Vec<LockInfo> {
        let lock_types = [
            LockType::Migration,
            LockType::Backup,
            LockType::Serve,
            LockType::Update,
            LockType::BranchPreview,
        ];
        
        let mut active = Vec::new();
        for lock_type in lock_types {
            let lock_path = self.lock_path(lock_type);
            if let Some(info) = self.read_lock(&lock_path) {
                if info.is_process_alive() && !info.is_expired() {
                    active.push(info);
                }
            }
        }
        active
    }

    /// Check if update is blocked
    pub fn is_update_blocked(&self) -> Option<String> {
        let blocking = self.get_blocking_lock_types(LockType::Update);
        
        for lock_type in blocking {
            let lock_path = self.lock_path(lock_type);
            if let Some(info) = self.read_lock(&lock_path) {
                if info.is_process_alive() && !info.is_expired() {
                    return Some(info.description);
                }
            }
        }
        None
    }

    /// Force release all locks (admin use only)
    pub fn force_release_all(&self) -> std::io::Result<()> {
        if self.locks_dir.exists() {
            for entry in fs::read_dir(&self.locks_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "lock").unwrap_or(false) {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }
}

/// RAII guard that releases lock on drop
pub struct LockGuard {
    lock_path: PathBuf,
    lock_type: LockType,
}

impl LockGuard {
    pub fn lock_type(&self) -> LockType {
        self.lock_type
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

/// Lock errors
#[derive(Debug, Clone)]
pub enum LockError {
    AlreadyLocked {
        lock_type: LockType,
        pid: u32,
        description: String,
    },
    BlockedBy {
        requested: LockType,
        blocking: LockType,
        description: String,
    },
    IoError(String),
    SerializeError(String),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::AlreadyLocked { lock_type, pid, description } => {
                write!(f, "{:?} lock held by PID {}: {}", lock_type, pid, description)
            }
            LockError::BlockedBy { requested, blocking, description } => {
                write!(f, "Cannot acquire {:?} lock: blocked by {:?} ({})", 
                       requested, blocking, description)
            }
            LockError::IoError(e) => write!(f, "IO error: {}", e),
            LockError::SerializeError(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

impl std::error::Error for LockError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_acquire_release() {
        let dir = tempdir().unwrap();
        let locks = OperationLock::new(dir.path());
        
        let guard = locks.acquire(LockType::Migration).unwrap();
        assert_eq!(guard.lock_type(), LockType::Migration);
        
        // Can't acquire same lock twice
        assert!(locks.acquire(LockType::Migration).is_err());
        
        drop(guard);
        
        // Can acquire after release
        assert!(locks.acquire(LockType::Migration).is_ok());
    }

    #[test]
    fn test_blocking_locks() {
        let dir = tempdir().unwrap();
        let locks = OperationLock::new(dir.path());
        
        let _migration = locks.acquire(LockType::Migration).unwrap();
        
        // Update should be blocked by migration
        assert!(locks.acquire(LockType::Update).is_err());
        assert!(locks.is_update_blocked().is_some());
    }

    #[test]
    fn test_get_active_locks() {
        let dir = tempdir().unwrap();
        let locks = OperationLock::new(dir.path());
        
        assert!(locks.get_active_locks().is_empty());
        
        let _guard = locks.acquire(LockType::Serve).unwrap();
        
        let active = locks.get_active_locks();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].lock_type, LockType::Serve);
    }
}
