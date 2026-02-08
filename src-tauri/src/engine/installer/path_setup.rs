//! CLI PATH Setup
//!
//! Helper to add the CLI binary to the system PATH.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStatus {
    pub in_path: bool,
    pub path_dirs: Vec<String>,
    pub binary_path: Option<String>,
    pub error: Option<String>,
}

pub struct PathSetup;

impl PathSetup {
    /// check if airdb is in PATH
    pub fn check() -> PathStatus {
        let path_var = env::var("PATH").unwrap_or_default();
        let path_dirs: Vec<String> = env::split_paths(&path_var)
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        
        let binary_name = if cfg!(windows) { "airdb.exe" } else { "airdb" };
        
        let in_path = which::which(binary_name).is_ok();
        let binary_path = which::which(binary_name)
            .ok()
            .map(|p: PathBuf| p.to_string_lossy().to_string());

        PathStatus {
            in_path,
            path_dirs,
            binary_path,
            error: None,
        }
    }

    /// Add CLI to PATH (Best effort without elevation)
    pub fn add_to_path() -> Result<(), String> {
        #[cfg(windows)]
        return Self::add_windows();
        
        #[cfg(unix)]
        return Self::add_unix();
    }

    #[cfg(windows)]
    fn add_windows() -> Result<(), String> {
        use std::process::Command;
        use winreg::enums::*;
        use winreg::RegKey;

        // Get current executable directory
        let exe_path = env::current_exe().map_err(|e| e.to_string())?;
        let bin_dir = exe_path.parent()
            .ok_or("Could not find binary directory")?
            .to_string_lossy()
            .to_string();

        // Open HKCU Environment key
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| format!("Failed to open registry: {}", e))?;

        // Get current PATH
        let current_path: String = env_key.get_value("Path")
            .unwrap_or_default();

        if !current_path.contains(&bin_dir) {
            let new_path = if current_path.is_empty() {
                bin_dir
            } else {
                format!("{};{}", current_path, bin_dir)
            };

            env_key.set_value("Path", &new_path)
                .map_err(|e| format!("Failed to update registry: {}", e))?;
            
            // Broadcast environment change
            unsafe {
                use winapi::um::winuser::{SendMessageTimeoutA, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG};
                let env_str = std::ffi::CString::new("Environment").unwrap();
                SendMessageTimeoutA(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0,
                    env_str.as_ptr() as _,
                    SMTO_ABORTIFHUNG,
                    5000,
                    std::ptr::null_mut(),
                );
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn add_unix() -> Result<(), String> {
        use std::os::unix::fs::symlink;
        use std::fs;
        use std::path::Path;

        let exe_path = env::current_exe().map_err(|e| e.to_string())?;
        
        // Try ~/.local/bin first (no sudo needed usually)
        let home = dirs::home_dir().ok_or("No home directory")?;
        let local_bin = home.join(".local/bin");
        
        if !local_bin.exists() {
            fs::create_dir_all(&local_bin).map_err(|e| e.to_string())?;
        }

        let target = local_bin.join("airdb");
        
        if target.exists() {
            fs::remove_file(&target).map_err(|e| e.to_string())?;
        }

        symlink(&exe_path, &target).map_err(|e| e.to_string())?;

        // Check if ~/.local/bin is in PATH
        let path_var = env::var("PATH").unwrap_or_default();
        if !path_var.contains(".local/bin") {
            // Need to guide user to update shell rc
            return Err("Added symlink to ~/.local/bin/airdb, but ~/.local/bin is not in PATH. Please add it to your shell configuration.".to_string());
        }

        Ok(())
    }
}
