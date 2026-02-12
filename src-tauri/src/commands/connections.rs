//! Connection Management Commands
//!
//! Tauri commands for managing database connections.

use tauri::State;

use crate::AppState;
use crate::engine::connections::{AdapterConfig, ConnectionConfig, ConnectionManager};

/// List all saved connections
#[tauri::command]
pub fn list_connections() -> Result<Vec<ConnectionConfig>, String> {
    let manager = ConnectionManager::new();
    manager.list_connections()
}

/// Add a new connection
#[tauri::command]
pub fn add_connection(config: ConnectionConfig) -> Result<(), String> {
    let manager = ConnectionManager::new();
    manager.add_connection(config)
}

/// Update an existing connection
#[tauri::command]
pub fn update_connection(config: ConnectionConfig) -> Result<(), String> {
    let manager = ConnectionManager::new();
    manager.update_connection(config)
}

/// Remove a connection by ID
#[tauri::command]
pub fn remove_connection(id: String) -> Result<(), String> {
    let manager = ConnectionManager::new();
    manager.remove_connection(&id)
}

/// Test a connection configuration
#[tauri::command]
pub fn test_connection(config: AdapterConfig) -> Result<String, String> {
    ConnectionManager::test_connection(&config)
}

/// Connect to a saved connection (set as active adapter)
#[tauri::command]
pub fn connect_to_database(
    connection_id: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let manager = ConnectionManager::new();
    let config = manager.get_connection(&connection_id)?;

    let adapter = ConnectionManager::create_adapter(&config.config)?;
    adapter.test_connection().map_err(|e| e.to_string())?;

    let dialect = adapter.dialect().to_string();
    *state.adapter.lock().unwrap() = Some(adapter);

    Ok(serde_json::json!({
        "connected": true,
        "name": config.name,
        "dialect": dialect,
    }))
}
