//! Audit and Health Tauri Commands

use crate::engine::audit::{AuditLog, AuditEntry, AuditAction};
use crate::engine::observability::dashboard::HealthDashboardGenerator;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_audit_log(limit: Option<usize>, state: State<AppState>) -> Result<serde_json::Value, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let log = AuditLog::new(project_dir).map_err(|e| e.to_string())?;
    let entries = log.query_all().map_err(|e| e.to_string())?;
    
    let count = entries.len();
    let limited: Vec<&AuditEntry> = match limit {
        Some(n) => entries.iter().rev().take(n).collect(),
        None => entries.iter().rev().take(100).collect(),
    };
    
    Ok(serde_json::json!({
        "total": count,
        "entries": limited,
    }))
}

#[tauri::command]
pub fn get_audit_count(state: State<AppState>) -> Result<usize, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let log = AuditLog::new(project_dir).map_err(|e| e.to_string())?;
    log.count().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn append_audit_entry(
    action: String,
    resource_type: String,
    resource_name: String,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let audit_action = match action.as_str() {
        "insert" => AuditAction::Insert,
        "update" => AuditAction::Update,
        "delete" => AuditAction::Delete,
        "schema_create" => AuditAction::SchemaCreate,
        "schema_update" => AuditAction::SchemaUpdate,
        "schema_migrate" => AuditAction::SchemaMigrate,
        "collection_create" => AuditAction::CollectionCreate,
        "collection_drop" => AuditAction::CollectionDrop,
        "backup" => AuditAction::Backup,
        other => AuditAction::Custom(other.to_string()),
    };
    
    let entry = AuditEntry::new(audit_action, &resource_type, &resource_name);
    
    let log = AuditLog::new(project_dir).map_err(|e| e.to_string())?;
    log.append(&entry).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_health_dashboard(state: State<AppState>) -> Result<serde_json::Value, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = project_dir.as_ref().ok_or("No project open")?;
    
    let generator = HealthDashboardGenerator::new();
    let dashboard = generator.generate(project_dir).map_err(|e| e.to_string())?;
    
    serde_json::to_value(&dashboard).map_err(|e| e.to_string())
}
