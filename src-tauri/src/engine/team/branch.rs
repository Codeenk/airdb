//! Branch-Aware Operations
//!
//! Track and manage branch context for team workflows

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Current branch context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchContext {
    pub branch_name: String,
    pub is_default: bool,
    pub is_preview: bool,
    pub remote: Option<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
}

impl BranchContext {
    /// Get current branch context from git
    pub fn current(project_dir: &Path) -> std::io::Result<Self> {
        let branch_name = get_current_branch(project_dir)?;
        let default_branch = get_default_branch(project_dir).unwrap_or_else(|_| "main".to_string());
        
        let (ahead, behind) = get_ahead_behind(project_dir, &branch_name, &default_branch)
            .unwrap_or((0, 0));
        
        Ok(Self {
            is_default: branch_name == default_branch,
            is_preview: branch_name.starts_with("preview/") || branch_name.contains("-preview"),
            remote: get_remote_name(project_dir).ok(),
            ahead_count: ahead,
            behind_count: behind,
            branch_name,
        })
    }

    /// Check if updates should be blocked on this branch
    pub fn should_block_updates(&self) -> bool {
        // Block updates on non-default branches
        !self.is_default && !self.is_preview
    }
}

/// Branch lock for preventing concurrent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchLock {
    pub branch_name: String,
    pub operation: String,
    pub locked_at: chrono::DateTime<chrono::Utc>,
    pub locked_by: String,
}

impl BranchLock {
    /// Create a new branch lock
    pub fn new(branch_name: &str, operation: &str, user: &str) -> Self {
        Self {
            branch_name: branch_name.to_string(),
            operation: operation.to_string(),
            locked_at: chrono::Utc::now(),
            locked_by: user.to_string(),
        }
    }

    /// Save lock to file
    pub fn save(&self, project_dir: &Path) -> std::io::Result<()> {
        let lock_path = project_dir.join(".airdb").join("branch.lock");
        fs::create_dir_all(lock_path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&lock_path, content)
    }

    /// Load lock from file
    pub fn load(project_dir: &Path) -> std::io::Result<Option<Self>> {
        let lock_path = project_dir.join(".airdb").join("branch.lock");
        if !lock_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&lock_path)?;
        let lock: BranchLock = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(lock))
    }

    /// Release the lock
    pub fn release(project_dir: &Path) -> std::io::Result<()> {
        let lock_path = project_dir.join(".airdb").join("branch.lock");
        if lock_path.exists() {
            fs::remove_file(&lock_path)?;
        }
        Ok(())
    }

    /// Check if lock is stale (older than 1 hour)
    pub fn is_stale(&self) -> bool {
        let age = chrono::Utc::now() - self.locked_at;
        age.num_hours() >= 1
    }
}

// Git helper functions
fn get_current_branch(project_dir: &Path) -> std::io::Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_dir)
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Not a git repository"))
    }
}

fn get_default_branch(project_dir: &Path) -> std::io::Result<String> {
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(project_dir)
        .output()?;
    
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        Ok(result.trim().split('/').last().unwrap_or("main").to_string())
    } else {
        Ok("main".to_string())
    }
}

fn get_remote_name(project_dir: &Path) -> std::io::Result<String> {
    let output = Command::new("git")
        .args(["remote"])
        .current_dir(project_dir)
        .output()?;
    
    if output.status.success() {
        let remotes = String::from_utf8_lossy(&output.stdout);
        Ok(remotes.lines().next().unwrap_or("origin").to_string())
    } else {
        Ok("origin".to_string())
    }
}

fn get_ahead_behind(project_dir: &Path, branch: &str, base: &str) -> std::io::Result<(u32, u32)> {
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", &format!("{}...{}", base, branch)])
        .current_dir(project_dir)
        .output()?;
    
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split_whitespace().collect();
        if parts.len() == 2 {
            let behind = parts[0].parse().unwrap_or(0);
            let ahead = parts[1].parse().unwrap_or(0);
            return Ok((ahead, behind));
        }
    }
    Ok((0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_branch_lock_lifecycle() {
        let dir = tempdir().unwrap();
        
        let lock = BranchLock::new("feature/test", "migration", "test-user");
        lock.save(dir.path()).unwrap();
        
        let loaded = BranchLock::load(dir.path()).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().branch_name, "feature/test");
        
        BranchLock::release(dir.path()).unwrap();
        
        let after = BranchLock::load(dir.path()).unwrap();
        assert!(after.is_none());
    }
}
