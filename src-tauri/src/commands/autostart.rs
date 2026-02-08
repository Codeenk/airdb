//! Autostart Tauri Commands
//!
//! Commands for managing system autostart

use tauri::State;
use crate::AppState;
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
    // Get the bootstrapper path based on current executable
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "airdb-bootstrap".to_string());
    
    AutostartManager::new("AirDB", &exe_path)
}
