//! Data Browser Commands
//!
//! Tauri commands for browsing, editing, and managing table data.
//! Uses the DatabaseAdapter trait for database-agnostic operations.

use serde::Deserialize;
use tauri::State;

use crate::AppState;
use crate::engine::adapter::{FilterOp, FilterSpec, SortDirection, SortSpec};
use crate::engine::audit::{AuditLog, AuditEntry, AuditAction};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortParam {
    pub column: String,
    pub direction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterParam {
    pub column: String,
    pub operator: String,
    pub value: String,
}

impl FilterParam {
    fn to_filter_spec(&self) -> Result<FilterSpec, String> {
        let operator = match self.operator.as_str() {
            "eq" | "=" => FilterOp::Eq,
            "neq" | "!=" => FilterOp::Neq,
            "gt" | ">" => FilterOp::Gt,
            "lt" | "<" => FilterOp::Lt,
            "gte" | ">=" => FilterOp::Gte,
            "lte" | "<=" => FilterOp::Lte,
            "like" | "LIKE" => FilterOp::Like,
            "is_null" | "IS NULL" => FilterOp::IsNull,
            "is_not_null" | "IS NOT NULL" => FilterOp::IsNotNull,
            _ => return Err(format!("Unknown filter operator: {}", self.operator)),
        };

        Ok(FilterSpec {
            column: self.column.clone(),
            operator,
            value: self.value.clone(),
        })
    }
}

/// Query table data with pagination, sorting, and filtering
#[tauri::command]
pub fn query_table_data(
    state: State<AppState>,
    table: String,
    limit: Option<usize>,
    offset: Option<usize>,
    sort: Option<SortParam>,
    filters: Option<Vec<FilterParam>>,
) -> Result<serde_json::Value, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;

    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let sort_spec = sort.map(|s| SortSpec {
        column: s.column,
        direction: if s.direction.to_lowercase() == "desc" {
            SortDirection::Desc
        } else {
            SortDirection::Asc
        },
    });

    let filter_specs: Vec<FilterSpec> = filters
        .unwrap_or_default()
        .iter()
        .map(|f| f.to_filter_spec())
        .collect::<Result<Vec<_>, _>>()?;

    let page = adapter
        .query_rows(&table, limit, offset, sort_spec.as_ref(), &filter_specs)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "rows": page.rows,
        "totalCount": page.total_count,
        "columns": page.columns,
        "executionTimeMs": page.execution_time_ms,
    }))
}

/// Insert a new row into a table
#[tauri::command]
pub fn adapter_insert_row(
    state: State<AppState>,
    table: String,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;

    let obj = data.as_object().ok_or("Data must be a JSON object")?;
    let id = adapter.insert_row(&table, obj).map_err(|e| e.to_string())?;

    // Audit log
    if let Ok(project_dir) = state.project_dir.lock() {
        if let Some(dir) = project_dir.as_ref() {
            if let Ok(log) = AuditLog::new(dir) {
                let entry = AuditEntry::new(AuditAction::Insert, "table", &table)
                    .with_after(data.clone());
                let _ = log.append(&entry);
            }
        }
    }

    Ok(serde_json::json!({
        "id": id,
        "message": "Row inserted successfully"
    }))
}

/// Update an existing row by primary key
#[tauri::command]
pub fn adapter_update_row(
    state: State<AppState>,
    table: String,
    id: i64,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;

    let obj = data.as_object().ok_or("Data must be a JSON object")?;
    let affected = adapter.update_row(&table, id, obj).map_err(|e| e.to_string())?;

    // Audit log
    if let Ok(project_dir) = state.project_dir.lock() {
        if let Some(dir) = project_dir.as_ref() {
            if let Ok(log) = AuditLog::new(dir) {
                let entry = AuditEntry::new(AuditAction::Update, "table", &table)
                    .with_metadata(serde_json::json!({"id": id}));
                let _ = log.append(&entry);
            }
        }
    }

    Ok(serde_json::json!({
        "affected": affected,
        "message": "Row updated successfully"
    }))
}

/// Delete a row by primary key
#[tauri::command]
pub fn adapter_delete_row(
    state: State<AppState>,
    table: String,
    id: i64,
) -> Result<serde_json::Value, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;

    let affected = adapter.delete_row(&table, id).map_err(|e| e.to_string())?;

    // Audit log
    if let Ok(project_dir) = state.project_dir.lock() {
        if let Some(dir) = project_dir.as_ref() {
            if let Ok(log) = AuditLog::new(dir) {
                let entry = AuditEntry::new(AuditAction::Delete, "table", &table)
                    .with_metadata(serde_json::json!({"id": id}));
                let _ = log.append(&entry);
            }
        }
    }

    Ok(serde_json::json!({
        "affected": affected,
        "message": "Row deleted successfully"
    }))
}

/// Get full table schema via adapter
#[tauri::command]
pub fn adapter_get_table_schema(
    state: State<AppState>,
    table: String,
) -> Result<serde_json::Value, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;

    let schema = adapter.get_table_schema(&table).map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(schema).map_err(|e| e.to_string())?)
}

/// Get current dialect information
#[tauri::command]
pub fn get_dialect(state: State<AppState>) -> Result<String, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;
    Ok(adapter.dialect().to_string())
}

/// Get available data types for the current dialect
#[tauri::command]
pub fn get_dialect_types(state: State<AppState>) -> Result<Vec<String>, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;
    Ok(adapter.dialect().data_types().iter().map(|s| s.to_string()).collect())
}

/// Get database size
#[tauri::command]
pub fn get_database_size(state: State<AppState>) -> Result<u64, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;
    adapter.get_database_size().map_err(|e| e.to_string())
}

/// Get row count for a specific table
#[tauri::command]
pub fn get_table_row_count(
    state: State<AppState>,
    table: String,
) -> Result<u64, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;
    adapter.get_row_count(&table).map_err(|e| e.to_string())
}

/// Get entire schema graph for ER diagram (all tables, columns, FKs)
#[tauri::command]
pub fn get_schema_graph(state: State<AppState>) -> Result<serde_json::Value, String> {
    let adapter_lock = state.adapter.lock().map_err(|e| e.to_string())?;
    let adapter = adapter_lock.as_ref().ok_or("No database adapter initialized")?;

    let table_names = adapter.get_tables().map_err(|e| e.to_string())?;
    let mut tables = Vec::new();
    let mut edges = Vec::new();

    for tname in &table_names {
        let schema = adapter.get_table_schema(tname).map_err(|e| e.to_string())?;
        let row_count = adapter.get_row_count(tname).unwrap_or(0);
        let fks = adapter.get_foreign_keys(tname).map_err(|e| e.to_string())?;

        let mut columns = Vec::new();
        for col in &schema.columns {
            let fk_match = fks.iter().find(|fk| fk.from_column == col.name);
            columns.push(serde_json::json!({
                "name": col.name,
                "type": col.col_type,
                "isPk": col.is_primary_key,
                "isFk": fk_match.is_some(),
                "isNullable": col.nullable,
                "isUnique": col.is_unique,
                "defaultValue": col.default_value,
                "fkTable": fk_match.map(|fk| &fk.to_table),
                "fkColumn": fk_match.map(|fk| &fk.to_column),
            }));
        }

        for fk in &fks {
            edges.push(serde_json::json!({
                "from": tname,
                "fromColumn": fk.from_column,
                "to": fk.to_table,
                "toColumn": fk.to_column,
            }));
        }

        tables.push(serde_json::json!({
            "name": tname,
            "columns": columns,
            "rowCount": row_count,
        }));
    }

    Ok(serde_json::json!({ "tables": tables, "edges": edges }))
}
