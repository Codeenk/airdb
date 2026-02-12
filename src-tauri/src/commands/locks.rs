use crate::engine::locks::{OperationLock, LockType};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn is_update_blocked(state: State<AppState>) -> Result<bool, String> {
    let project_dir = state.project_dir.lock().unwrap();
    
    // If no project is open, we can't check project-specific locks, 
    // but updates might still be blocked by global locks if we had them.
    // For now, if no project, assume not blocked (or maybe we shouldn't update without project context?)
    // usage in UpdateSettings implies we might be in settings view.
    // If project_dir is None, we return false (not blocked).
    let project_dir = match project_dir.as_ref() {
        Some(p) => p,
        None => return Ok(false),
    };

    let locks = OperationLock::new(project_dir);
    Ok(locks.is_update_blocked().is_some())
}

#[tauri::command]
pub fn get_active_locks(state: State<AppState>) -> Result<Vec<String>, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = match project_dir.as_ref() {
        Some(p) => p,
        None => return Ok(vec![]),
    };

    let locks = OperationLock::new(project_dir);
    let active = locks.get_active_locks();
    
    Ok(active.into_iter().map(|l| l.description).collect())
}

// Check if a specific operation can proceed
#[tauri::command]
pub fn check_lock(operation: String, state: State<AppState>) -> Result<bool, String> {
    let project_dir = state.project_dir.lock().unwrap();
    let project_dir = match project_dir.as_ref() {
        Some(p) => p,
        None => return Ok(true), // Safe default if no project
    };

    let lock_type = match operation.as_str() {
        "migration" => LockType::Migration,
        "backup" => LockType::Backup,
        "serve" => LockType::Serve,
        "update" => LockType::Update,
        "branch_preview" => LockType::BranchPreview,
        _ => return Err(format!("Invalid operation type: {}", operation)),
    };

    let locks = OperationLock::new(project_dir);
    
    // Check if any blocking locks prevent this operation
    match locks.acquire(lock_type) {
        Ok(guard) => {
            // We could acquire it, so it's not blocked — immediately drop the guard
            drop(guard);
            Ok(true)
        }
        Err(_) => Ok(false), // Blocked by another operation
    }
}
