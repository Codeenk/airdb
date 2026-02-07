//! AirDB Bootstrapper
//! 
//! A tiny launcher that resolves and launches the correct AirDB version.
//! This binary should never be updated directly - it only reads state
//! and launches the appropriate version.
//!
//! ## Responsibilities
//! 1. Read state.json to determine current version
//! 2. Handle pending version switches
//! 3. Track failed boot attempts for rollback
//! 4. Launch the correct binary
//!
//! ## Design Principles
//! - Never updates itself
//! - Minimal dependencies
//! - Fast startup
//! - Rollback-safe

use std::env;
use std::process::{Command, ExitCode};
use std::path::PathBuf;

mod bootstrap_core {
    use std::path::{Path, PathBuf};
    use std::fs;
    use serde::{Deserialize, Serialize};

    /// Minimal state for bootstrapper (subset of full UpdateState)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BootState {
        pub current_version: String,
        pub pending_version: Option<String>,
        pub last_good_version: String,
        pub failed_boot_count: u32,
        pub max_failed_boots: u32,
    }

    impl Default for BootState {
        fn default() -> Self {
            Self {
                current_version: env!("CARGO_PKG_VERSION").to_string(),
                pending_version: None,
                last_good_version: env!("CARGO_PKG_VERSION").to_string(),
                failed_boot_count: 0,
                max_failed_boots: 3,
            }
        }
    }

    impl BootState {
        pub fn load(path: &Path) -> Option<Self> {
            let content = fs::read_to_string(path).ok()?;
            serde_json::from_str(&content).ok()
        }

        pub fn save(&self, path: &Path) -> std::io::Result<()> {
            let content = serde_json::to_string_pretty(self)?;
            let temp_path = path.with_extension("tmp");
            fs::write(&temp_path, &content)?;
            fs::rename(&temp_path, path)?;
            Ok(())
        }
    }

    /// Get the base directory for AirDB
    pub fn get_base_dir() -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            dirs::home_dir().map(|h| h.join(".local/share/airdb"))
        }
        
        #[cfg(target_os = "windows")]
        {
            dirs::data_local_dir().map(|d| d.join("AirDB"))
        }
        
        #[cfg(target_os = "macos")]
        {
            dirs::data_dir().map(|d| d.join("AirDB"))
        }
    }

    /// Get binary path for a version
    pub fn get_binary_path(base_dir: &Path, version: &str) -> PathBuf {
        let version_dir = base_dir.join("versions").join(version);
        
        #[cfg(target_os = "windows")]
        {
            version_dir.join("airdb.exe")
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            version_dir.join("airdb-desktop")
        }
    }

    /// Check if a version directory exists and has the binary
    pub fn is_version_valid(base_dir: &Path, version: &str) -> bool {
        get_binary_path(base_dir, version).exists()
    }
}

use bootstrap_core::*;

fn main() -> ExitCode {
    // Get base directory
    let Some(base_dir) = get_base_dir() else {
        eprintln!("Error: Could not determine AirDB directory");
        return ExitCode::FAILURE;
    };

    let state_path = base_dir.join("state.json");

    // Load or create state
    let mut state = BootState::load(&state_path).unwrap_or_default();

    // Determine which version to run
    let version_to_run = determine_version(&base_dir, &mut state);

    // Get binary path
    let binary_path = get_binary_path(&base_dir, &version_to_run);

    if !binary_path.exists() {
        eprintln!("Error: Binary not found at {:?}", binary_path);
        eprintln!("Please reinstall AirDB.");
        return ExitCode::FAILURE;
    }

    // Increment boot counter (will be reset by app on successful startup)
    state.failed_boot_count += 1;
    if let Err(e) = state.save(&state_path) {
        eprintln!("Warning: Could not save state: {}", e);
    }

    // Collect args (skip our own binary name)
    let args: Vec<String> = env::args().skip(1).collect();

    // Launch the actual binary
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new(&binary_path)
            .args(&args)
            .exec();
        eprintln!("Failed to exec: {}", err);
        ExitCode::FAILURE
    }

    #[cfg(windows)]
    {
        match Command::new(&binary_path)
            .args(&args)
            .status()
        {
            Ok(status) => {
                if status.success() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(status.code().unwrap_or(1) as u8)
                }
            }
            Err(e) => {
                eprintln!("Failed to launch: {}", e);
                ExitCode::FAILURE
            }
        }
    }
}

/// Determine which version to run, handling pending updates and rollbacks
fn determine_version(base_dir: &std::path::Path, state: &mut BootState) -> String {
    let state_path = base_dir.join("state.json");

    // Check if we need to rollback
    if state.failed_boot_count >= state.max_failed_boots {
        eprintln!("Warning: Too many failed boots, rolling back to {}", state.last_good_version);
        
        if is_version_valid(base_dir, &state.last_good_version) {
            state.current_version = state.last_good_version.clone();
            state.pending_version = None;
            state.failed_boot_count = 0;
            let _ = state.save(&state_path);
            return state.last_good_version.clone();
        }
    }

    // Check for pending version switch
    if let Some(pending) = &state.pending_version {
        if is_version_valid(base_dir, pending) {
            eprintln!("Switching to new version: {}", pending);
            state.last_good_version = state.current_version.clone();
            state.current_version = pending.clone();
            state.pending_version = None;
            state.failed_boot_count = 0;
            let _ = state.save(&state_path);
            return state.current_version.clone();
        } else {
            eprintln!("Warning: Pending version {} not found, staying on {}", 
                     pending, state.current_version);
            state.pending_version = None;
            let _ = state.save(&state_path);
        }
    }

    // Use current version if valid
    if is_version_valid(base_dir, &state.current_version) {
        return state.current_version.clone();
    }

    // Fall back to last good version
    if is_version_valid(base_dir, &state.last_good_version) {
        eprintln!("Warning: Current version invalid, falling back to {}", state.last_good_version);
        state.current_version = state.last_good_version.clone();
        let _ = state.save(&state_path);
        return state.last_good_version.clone();
    }

    // Last resort: use compiled-in version
    eprintln!("Warning: No valid version found, using built-in version");
    env!("CARGO_PKG_VERSION").to_string()
}
