//! Update State Machine
//! 
//! Manages the update lifecycle with explicit states to prevent half-baked updates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Current status of the update process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateStatus {
    /// No update in progress
    Idle,
    /// Checking for updates
    Checking,
    /// Downloading update
    Downloading {
        progress: f32,
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    /// Verifying checksum and signature
    Verifying,
    /// Update downloaded and verified, ready to switch on restart
    ReadyToSwitch,
    /// Update failed
    Failed { reason: String },
    /// Rolled back to previous version
    RolledBack { reason: String },
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Complete update state persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    /// Currently running version
    pub current_version: String,
    /// Version pending installation (downloaded but not switched)
    pub pending_version: Option<String>,
    /// Last known good version (for rollback)
    pub last_good_version: String,
    /// Current update status
    pub update_status: UpdateStatus,
    /// Last time we checked for updates
    pub last_check: Option<DateTime<Utc>>,
    /// Update channel (stable, beta, nightly)
    pub channel: String,
    /// Number of consecutive failed boots (for rollback detection)
    pub failed_boot_count: u32,
    /// Maximum failed boots before rollback
    pub max_failed_boots: u32,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            pending_version: None,
            last_good_version: env!("CARGO_PKG_VERSION").to_string(),
            update_status: UpdateStatus::Idle,
            last_check: None,
            channel: "stable".to_string(),
            failed_boot_count: 0,
            max_failed_boots: 3,
        }
    }
}

impl UpdateState {
    /// Load state from disk, or create default if not exists
    pub fn load(state_path: &Path) -> Result<Self, StateError> {
        if state_path.exists() {
            let content = fs::read_to_string(state_path)
                .map_err(|e| StateError::ReadError(e.to_string()))?;
            serde_json::from_str(&content)
                .map_err(|e| StateError::ParseError(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to disk atomically
    pub fn save(&self, state_path: &Path) -> Result<(), StateError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| StateError::SerializeError(e.to_string()))?;
        
        // Write to temp file first, then rename (atomic)
        let temp_path = state_path.with_extension("tmp");
        fs::write(&temp_path, &content)
            .map_err(|e| StateError::WriteError(e.to_string()))?;
        fs::rename(&temp_path, state_path)
            .map_err(|e| StateError::WriteError(e.to_string()))?;
        
        Ok(())
    }

    /// Transition to checking state
    pub fn start_checking(&mut self) {
        self.update_status = UpdateStatus::Checking;
        self.last_check = Some(Utc::now());
    }

    /// Transition to downloading state
    pub fn start_downloading(&mut self) {
        self.update_status = UpdateStatus::Downloading {
            progress: 0.0,
            bytes_downloaded: 0,
            total_bytes: 0,
        };
    }

    /// Update download progress
    pub fn update_progress(&mut self, downloaded: u64, total: u64) {
        let progress = if total > 0 {
            (downloaded as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        self.update_status = UpdateStatus::Downloading {
            progress,
            bytes_downloaded: downloaded,
            total_bytes: total,
        };
    }

    /// Transition to verifying state
    pub fn start_verifying(&mut self) {
        self.update_status = UpdateStatus::Verifying;
    }

    /// Mark update as ready to switch
    pub fn mark_ready(&mut self, version: String) {
        self.pending_version = Some(version);
        self.update_status = UpdateStatus::ReadyToSwitch;
    }

    /// Mark update as failed
    pub fn mark_failed(&mut self, reason: String) {
        self.pending_version = None;
        self.update_status = UpdateStatus::Failed { reason };
    }

    /// Mark as rolled back
    pub fn mark_rolled_back(&mut self, reason: String) {
        self.current_version = self.last_good_version.clone();
        self.pending_version = None;
        self.update_status = UpdateStatus::RolledBack { reason };
    }

    /// Complete the switch to new version
    pub fn complete_switch(&mut self) {
        if let Some(version) = self.pending_version.take() {
            self.last_good_version = self.current_version.clone();
            self.current_version = version;
            self.update_status = UpdateStatus::Idle;
            self.failed_boot_count = 0;
        }
    }

    /// Record a failed boot attempt
    pub fn record_failed_boot(&mut self) -> bool {
        self.failed_boot_count += 1;
        self.failed_boot_count >= self.max_failed_boots
    }

    /// Mark boot as successful
    pub fn mark_boot_successful(&mut self) {
        self.failed_boot_count = 0;
        self.update_status = UpdateStatus::Idle;
    }

    /// Check if we should rollback
    pub fn should_rollback(&self) -> bool {
        self.failed_boot_count >= self.max_failed_boots
    }

    /// Check if update is available
    pub fn has_pending_update(&self) -> bool {
        matches!(self.update_status, UpdateStatus::ReadyToSwitch)
    }

    /// Reset to idle state
    pub fn reset(&mut self) {
        self.update_status = UpdateStatus::Idle;
        self.pending_version = None;
    }
}

/// State machine errors
#[derive(Debug, Clone)]
pub enum StateError {
    ReadError(String),
    WriteError(String),
    ParseError(String),
    SerializeError(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(e) => write!(f, "Failed to read state: {}", e),
            Self::WriteError(e) => write!(f, "Failed to write state: {}", e),
            Self::ParseError(e) => write!(f, "Failed to parse state: {}", e),
            Self::SerializeError(e) => write!(f, "Failed to serialize state: {}", e),
        }
    }
}

impl std::error::Error for StateError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_state_default() {
        let state = UpdateState::default();
        assert_eq!(state.update_status, UpdateStatus::Idle);
        assert!(state.pending_version.is_none());
    }

    #[test]
    fn test_state_transitions() {
        let mut state = UpdateState::default();
        
        state.start_checking();
        assert!(matches!(state.update_status, UpdateStatus::Checking));
        
        state.start_downloading();
        assert!(matches!(state.update_status, UpdateStatus::Downloading { .. }));
        
        state.update_progress(50, 100);
        if let UpdateStatus::Downloading { progress, .. } = state.update_status {
            assert_eq!(progress, 50.0);
        }
        
        state.start_verifying();
        assert!(matches!(state.update_status, UpdateStatus::Verifying));
        
        state.mark_ready("0.2.0".to_string());
        assert!(matches!(state.update_status, UpdateStatus::ReadyToSwitch));
        assert_eq!(state.pending_version, Some("0.2.0".to_string()));
    }

    #[test]
    fn test_state_persistence() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        
        let mut state = UpdateState::default();
        state.current_version = "1.0.0".to_string();
        state.save(&state_path).unwrap();
        
        let loaded = UpdateState::load(&state_path).unwrap();
        assert_eq!(loaded.current_version, "1.0.0");
    }

    #[test]
    fn test_rollback_detection() {
        let mut state = UpdateState::default();
        state.max_failed_boots = 3;
        
        assert!(!state.record_failed_boot()); // 1
        assert!(!state.record_failed_boot()); // 2
        assert!(state.record_failed_boot());  // 3 - should rollback
        assert!(state.should_rollback());
    }
}
