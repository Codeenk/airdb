//! Installer Commands
//!
//! Commands for installation tasks

use crate::engine::installer::{PathSetup, PathStatus};

#[tauri::command]
pub fn check_path_status() -> PathStatus {
    PathSetup::check()
}

#[tauri::command]
pub fn add_to_path() -> Result<(), String> {
    PathSetup::add_to_path()
}
