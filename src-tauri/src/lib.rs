//! AirDB - Local-first, GitHub-backed database platform
//! Tauri application library

pub mod engine;

use engine::config::Config;
use engine::database::Database;
use engine::migrations::MigrationRunner;
use engine::keystore::Keystore;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub project_dir: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<Database>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project_dir: Mutex::new(None),
            db: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let project_dir = state.project_dir.lock().unwrap();
    
    if let Some(dir) = project_dir.as_ref() {
        let config = Config::load(dir).map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "initialized": true,
            "project_name": config.project.name,
            "db_type": config.database.db_type,
            "api_port": config.api.port,
        }))
    } else {
        Ok(serde_json::json!({
            "initialized": false,
        }))
    }
}

#[tauri::command]
fn init_project(name: String, state: State<AppState>) -> Result<String, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let project_dir = home_dir.join("AirDB").join("projects").join(&name);
    
    std::fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
    
    let config = Config::default_for_project(&name);
    config.save(&project_dir).map_err(|e| e.to_string())?;
    
    // Create directory structure
    std::fs::create_dir_all(project_dir.join("sql").join("migrations")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_dir.join("access")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_dir.join("api")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_dir.join("data")).map_err(|e| e.to_string())?;
    
    *state.project_dir.lock().unwrap() = Some(project_dir.clone());
    
    Ok(project_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn open_project(path: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let project_dir = PathBuf::from(&path);
    let config = Config::load(&project_dir).map_err(|e| e.to_string())?;
    
    let db_path = project_dir.join(&config.database.path);
    let db = Database::new(&db_path).map_err(|e| e.to_string())?;
    
    *state.project_dir.lock().unwrap() = Some(project_dir);
    *state.db.lock().unwrap() = Some(db);
    
    Ok(serde_json::json!({
        "success": true,
        "project_name": config.project.name,
    }))
}

#[tauri::command]
fn create_migration(name: String, state: State<AppState>) -> Result<String, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let runner = MigrationRunner::new(project_dir);
    let path = runner.create(&name).map_err(|e| e.to_string())?;
    
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn run_migrations(state: State<AppState>) -> Result<Vec<String>, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;
    
    let runner = MigrationRunner::new(project_dir);
    runner.push(db).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_migration_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;
    
    let runner = MigrationRunner::new(project_dir);
    let status = runner.check(db).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "applied_count": status.applied_count,
        "pending_count": status.pending_count,
        "pending": status.pending_migrations,
    }))
}

#[tauri::command]
fn list_api_keys(state: State<AppState>) -> Result<Vec<engine::keystore::ApiKey>, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let keystore = Keystore::new(project_dir);
    keystore.list_api_keys().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_api_key(name: String, role: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let keystore = Keystore::new(project_dir);
    let (raw_key, key_info) = keystore.create_api_key(&name, &role).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "key": raw_key,
        "id": key_info.id,
        "name": key_info.name,
        "role": key_info.role,
    }))
}

#[tauri::command]
fn revoke_api_key(key_id: String, state: State<AppState>) -> Result<bool, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let keystore = Keystore::new(project_dir);
    keystore.revoke_api_key(&key_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_auth_status() -> Result<serde_json::Value, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_dir = home_dir.join(".airdb");
    let keystore = Keystore::new(&airdb_dir);
    
    match keystore.get_github_token() {
        Ok(_) => Ok(serde_json::json!({
            "authenticated": true
        })),
        Err(_) => Ok(serde_json::json!({
            "authenticated": false
        }))
    }
}

#[tauri::command]
async fn start_github_login() -> Result<serde_json::Value, String> {
    use engine::github::GitHubClient;
    
    let client = GitHubClient::new();
    let device_code = client.start_device_flow().await.map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "user_code": device_code.user_code,
        "verification_uri": device_code.verification_uri,
        "device_code": device_code.device_code,
        "expires_in": device_code.expires_in,
        "interval": device_code.interval
    }))
}

#[tauri::command]
async fn complete_github_login(device_code: String, interval: u64) -> Result<serde_json::Value, String> {
    use engine::github::{GitHubClient, DeviceCodeResponse};
    
    let mut client = GitHubClient::new();
    
    // Create a minimal device code response for polling
    let dc = DeviceCodeResponse {
        device_code,
        user_code: String::new(),
        verification_uri: String::new(),
        expires_in: 900,
        interval,
    };
    
    let token = client.complete_device_flow(&dc).await.map_err(|e| e.to_string())?;
    let user = client.get_user().await.map_err(|e| e.to_string())?;
    
    // Store token
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_dir = home_dir.join(".airdb");
    std::fs::create_dir_all(&airdb_dir).map_err(|e| e.to_string())?;
    let keystore = Keystore::new(&airdb_dir);
    keystore.store_github_token(&token).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true,
        "username": user.login,
        "name": user.name
    }))
}

#[tauri::command]
fn github_logout() -> Result<bool, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_dir = home_dir.join(".airdb");
    let keystore = Keystore::new(&airdb_dir);
    keystore.delete_github_token().map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn list_projects() -> Result<Vec<serde_json::Value>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let projects_dir = home_dir.join("AirDB").join("projects");
    
    if !projects_dir.exists() {
        return Ok(vec![]);
    }
    
    let mut projects = Vec::new();
    for entry in std::fs::read_dir(&projects_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let config_path = path.join("airdb.config.json");
                let has_config = config_path.exists();
                projects.push(serde_json::json!({
                    "name": name,
                    "path": path.to_string_lossy(),
                    "configured": has_config
                }));
            }
        }
    }
    
    Ok(projects)
}

#[tauri::command]
fn list_conflicts(state: State<AppState>) -> Result<Vec<String>, String> {
    use engine::github::GitSync;
    
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_dir = home_dir.join(".airdb");
    let keystore = Keystore::new(&airdb_dir);
    let token = keystore.get_github_token().map_err(|_| "Not authenticated")?;
    
    let sync = GitSync::new(project_dir, &token);
    sync.list_conflicts().map_err(|e| e.to_string())
}

#[tauri::command]
fn resolve_conflict(file: String, strategy: String, state: State<AppState>) -> Result<(), String> {
    use engine::github::GitSync;
    
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_dir = home_dir.join(".airdb");
    let keystore = Keystore::new(&airdb_dir);
    let token = keystore.get_github_token().map_err(|_| "Not authenticated")?;
    
    let sync = GitSync::new(project_dir, &token);
    sync.resolve_conflict(&file, &strategy).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_status,
            init_project,
            open_project,
            create_migration,
            run_migrations,
            get_migration_status,
            list_api_keys,
            create_api_key,
            revoke_api_key,
            get_auth_status,
            start_github_login,
            complete_github_login,
            github_logout,
            list_projects,
            list_conflicts,
            resolve_conflict,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

