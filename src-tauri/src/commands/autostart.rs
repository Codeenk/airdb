//! Autostart Tauri Commands
//!
//! Commands for managing system autostart

use crate::engine::autostart::{AutostartManager, AutostartStatus};

/// Get autostart status
#[tauri::command]
pub fn get_autostart_status() -> AutostartStatus {
    let manager = get_autostart_manager();
    AutostartStatus {
        enabled: manager.is_enabled(),
        last_start_success: None,
        last_start_time: None,
        error_message: None,
    }
}

/// Enable autostart
#[tauri::command]
pub fn enable_autostart() -> Result<AutostartStatus, String> {
    let manager = get_autostart_manager();
    manager.enable().map_err(|e| e.to_string())?;
    
    Ok(AutostartStatus {
        enabled: true,
        last_start_success: None,
        last_start_time: None,
        error_message: None,
    })
}

/// Disable autostart
#[tauri::command]
pub fn disable_autostart() -> Result<AutostartStatus, String> {
    let manager = get_autostart_manager();
    manager.disable().map_err(|e| e.to_string())?;
    
    Ok(AutostartStatus {
        enabled: false,
        last_start_success: None,
        last_start_time: None,
        error_message: None,
    })
}

fn get_autostart_manager() -> AutostartManager {
    // Always point to the bootstrapper, not the current executable
    let bootstrapper_path = get_bootstrapper_path();
    AutostartManager::new("AirDB", &bootstrapper_path)
}

/// Get the path to the bootstrapper binary
fn get_bootstrapper_path() -> String {
    #[cfg(target_os = "linux")]
    {
        // Check if installed in system bin
        if std::path::Path::new("/usr/local/bin/airdb-bootstrap").exists() {
            return "/usr/local/bin/airdb-bootstrap".to_string();
        }
        // Check user local bin
        if let Some(home) = dirs::home_dir() {
            let user_bin = home.join(".local/bin/airdb-bootstrap");
            if user_bin.exists() {
                return user_bin.to_string_lossy().to_string();
            }
            // Fallback to app directory
            let app_path = home.join(".local/share/airdb/current/airdb-bootstrap");
            if app_path.exists() {
                return app_path.to_string_lossy().to_string();
            }
        }
        "airdb-bootstrap".to_string()
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = dirs::data_local_dir() {
            let bootstrapper = local_app_data.join("AirDB").join("bin").join("airdb-bootstrap.exe");
            if bootstrapper.exists() {
                return bootstrapper.to_string_lossy().to_string();
            }
        }
        "airdb-bootstrap.exe".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let bootstrapper = home.join("Library/Application Support/AirDB/bin/airdb-bootstrap");
            if bootstrapper.exists() {
                return bootstrapper.to_string_lossy().to_string();
            }
        }
        "airdb-bootstrap".to_string()
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        "airdb-bootstrap".to_string()
    }
}
