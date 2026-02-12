//! Schema Editor Commands
//!
//! Backend support for visual table editing

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    #[serde(alias = "type", alias = "column_type", rename(serialize = "type"))]
    pub column_type: String,
    #[serde(alias = "is_nullable", default)]
    pub nullable: bool,
    #[serde(alias = "defaultValue", alias = "default_value", rename(serialize = "default_value"))]
    pub default_value: Option<String>,
    #[serde(alias = "is_pk", alias = "isPrimaryKey", rename(serialize = "is_pk"))]
    pub is_primary_key: bool,
    #[serde(alias = "is_unique", alias = "isUnique", rename(serialize = "is_unique"))]
    pub is_unique: bool,
    #[serde(alias = "foreignKey", alias = "foreign_key", rename(serialize = "foreign_key"), default)]
    pub foreign_key: Option<ForeignKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPreview {
    #[serde(rename = "upSql")]
    pub up_sql: String,
    #[serde(rename = "downSql")]
    pub down_sql: String,
    pub version: u32,
    pub name: String,
}

/// Get list of all user tables
#[tauri::command]
pub fn get_tables(state: State<AppState>) -> Result<Vec<String>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_airdb_%' ORDER BY name")
        .map_err(|e| e.to_string())?;
    
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    Ok(tables)
}

/// Get table indexes
#[tauri::command]
pub fn get_table_indexes(
    state: State<AppState>,
    table_name: String,
) -> Result<Vec<Index>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    
    let mut idx_stmt = conn
        .prepare(&format!("PRAGMA index_list('{}')", table_name))
        .map_err(|e| e.to_string())?;
    
    let indexes: Vec<Index> = idx_stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let unique: i32 = row.get(2)?;
            Ok((name, unique > 0))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|(name, _)| !name.starts_with("sqlite_"))
        .map(|(name, unique)| {
            let cols = get_index_columns(&conn, &name).unwrap_or_default();
            Index {
                name,
                columns: cols,
                unique,
            }
        })
        .collect();
    
    Ok(indexes)
}

/// Get table schema
#[tauri::command]
pub fn get_table_schema(
    state: State<AppState>,
    table_name: String,
) -> Result<TableSchema, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    
    // Get column info using PRAGMA
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info('{}')", table_name))
        .map_err(|e| e.to_string())?;
    
    let columns: Vec<Column> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            let not_null: i32 = row.get(3)?;
            let default: Option<String> = row.get(4)?;
            let pk: i32 = row.get(5)?;
            
            Ok(Column {
                name,
                column_type: col_type,
                nullable: not_null == 0,
                default_value: default,
                is_primary_key: pk > 0,
                is_unique: false, // Will be updated from index check
                foreign_key: None,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    // Get indexes
    let mut idx_stmt = conn
        .prepare(&format!("PRAGMA index_list('{}')", table_name))
        .map_err(|e| e.to_string())?;
    
    let indexes: Vec<Index> = idx_stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let unique: i32 = row.get(2)?;
            Ok((name, unique > 0))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|(name, _)| !name.starts_with("sqlite_"))
        .map(|(name, unique)| {
            // Get columns for each index
            let cols = get_index_columns(&conn, &name).unwrap_or_default();
            Index {
                name,
                columns: cols,
                unique,
            }
        })
        .collect();
    
    Ok(TableSchema {
        name: table_name,
        columns,
        indexes,
    })
}

fn get_index_columns(conn: &rusqlite::Connection, index_name: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA index_info('{}')", index_name))
        .map_err(|e| e.to_string())?;
    
    let cols: Vec<String> = stmt
        .query_map([], |row| {
            let col_name: String = row.get(2)?;
            Ok(col_name)
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    Ok(cols)
}

/// Generate migration from visual edits
#[tauri::command]
pub fn generate_table_migration(
    table_name: String,
    columns: Vec<Column>,
    is_new: bool,
    state: State<AppState>,
) -> Result<MigrationPreview, String> {
    if is_new {
        generate_create_table_migration(&table_name, &columns)
    } else {
        // Load original columns from the database instead of requiring frontend to send them
        let original_columns = load_original_columns(&state, &table_name)?;
        generate_alter_table_migration(&table_name, &columns, &original_columns)
    }
}

/// Load original column definitions from the database for diff-based migration generation
fn load_original_columns(state: &State<AppState>, table_name: &str) -> Result<Vec<Column>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info('{}')", table_name))
        .map_err(|e| e.to_string())?;

    let columns: Vec<Column> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            let not_null: i32 = row.get(3)?;
            let default: Option<String> = row.get(4)?;
            let pk: i32 = row.get(5)?;

            Ok(Column {
                name,
                column_type: col_type,
                nullable: not_null == 0,
                default_value: default,
                is_primary_key: pk > 0,
                is_unique: false,
                foreign_key: None,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(columns)
}

fn generate_create_table_migration(
    table_name: &str,
    columns: &[Column],
) -> Result<MigrationPreview, String> {
    let mut col_defs = Vec::new();
    let mut pk_cols = Vec::new();
    
    for col in columns {
        let mut def = format!("{} {}", col.name, col.column_type);
        
        if col.is_primary_key {
            pk_cols.push(col.name.clone());
            if col.column_type == "INTEGER" && pk_cols.len() == 1 {
                def.push_str(" PRIMARY KEY AUTOINCREMENT");
            }
        }
        
        if !col.nullable && !col.is_primary_key {
            def.push_str(" NOT NULL");
        }
        
        if col.is_unique && !col.is_primary_key {
            def.push_str(" UNIQUE");
        }
        
        if let Some(default) = &col.default_value {
            if !default.is_empty() {
                def.push_str(&format!(" DEFAULT {}", default));
            }
        }
        
        if let Some(fk) = &col.foreign_key {
            def.push_str(&format!(" REFERENCES {}({})", fk.table, fk.column));
        }
        
        col_defs.push(def);
    }
    
    let up_sql = format!(
        "CREATE TABLE {} (\n  {}\n);",
        table_name,
        col_defs.join(",\n  ")
    );
    
    let down_sql = format!("DROP TABLE {};", table_name);
    
    let name = format!("create_{}", table_name);
    
    Ok(MigrationPreview {
        up_sql,
        down_sql,
        version: get_next_migration_version()?,
        name,
    })
}

fn generate_alter_table_migration(
    table_name: &str,
    new_columns: &[Column],
    original_columns: &[Column],
) -> Result<MigrationPreview, String> {
    let mut up_statements = Vec::new();
    let mut down_statements = Vec::new();
    
    // Find added columns
    for col in new_columns {
        if !original_columns.iter().any(|c| c.name == col.name) {
            let mut def = format!("ALTER TABLE {} ADD COLUMN {} {}", 
                table_name, col.name, col.column_type);
            
            if !col.nullable {
                def.push_str(" NOT NULL");
            }
            
            if let Some(default) = &col.default_value {
                if !default.is_empty() {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
            }
            
            def.push(';');
            up_statements.push(def);
            
            // SQLite doesn't support DROP COLUMN easily, note in down
            down_statements.push(format!("-- Cannot easily drop column {} in SQLite", col.name));
        }
    }
    
    // Find removed columns (note: SQLite doesn't support ALTER TABLE DROP COLUMN easily)
    for col in original_columns {
        if !new_columns.iter().any(|c| c.name == col.name) {
            up_statements.push(format!(
                "-- Column {} removed (requires table rebuild in SQLite)",
                col.name
            ));
            
            let mut def = format!("ALTER TABLE {} ADD COLUMN {} {}",
                table_name, col.name, col.column_type);
            if !col.nullable {
                def.push_str(" NOT NULL");
            }
            def.push(';');
            down_statements.push(def);
        }
    }
    
    if up_statements.is_empty() {
        return Err("No changes detected".to_string());
    }
    
    let up_sql = up_statements.join("\n");
    let down_sql = if down_statements.is_empty() {
        "-- No automatic rollback available".to_string()
    } else {
        down_statements.join("\n")
    };
    
    let name = format!("alter_{}", table_name);
    
    Ok(MigrationPreview {
        up_sql,
        down_sql,
        version: get_next_migration_version()?,
        name,
    })
}

fn get_next_migration_version() -> Result<u32, String> {
    // In a real implementation, scan migrations directory
    // For now, use timestamp-based version
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok((duration.as_secs() % 1000000) as u32)
}

/// Apply a generated migration
#[tauri::command]
pub fn apply_generated_migration(
    state: State<AppState>,
    name: String,
    up_sql: String,
    down_sql: String,
) -> Result<(), String> {
    use std::fs;
    
    let project_dir_lock = state.project_dir.lock().map_err(|e| e.to_string())?;
    let project_dir = project_dir_lock.as_ref().ok_or("Project directory not set")?;
    let migrations_dir = project_dir.join("migrations");
    
    // Create migrations directory if needed
    fs::create_dir_all(&migrations_dir).map_err(|e| e.to_string())?;
    
    // Find next migration number
    let existing: Vec<_> = fs::read_dir(&migrations_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "sql").unwrap_or(false))
        .collect();
    
    let next_num = existing.len() + 1;
    let filename = format!("{:03}_{}.sql", next_num, name);
    let filepath = migrations_dir.join(&filename);
    
    // Write migration file
    let content = format!("-- Up\n{}\n\n-- Down\n{}", up_sql, down_sql);
    fs::write(&filepath, content).map_err(|e| e.to_string())?;
    
    // Apply migration
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    conn.execute_batch(&up_sql).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Execute raw SQL query
#[tauri::command]
pub fn execute_raw_sql(
    state: State<AppState>,
    sql: String,
) -> Result<serde_json::Value, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    
    // Check if it's a SELECT query
    let is_select = sql.trim().to_uppercase().starts_with("SELECT");
    
    if is_select {
        // Execute SELECT query and return results
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let column_count = stmt.column_count();
        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        
        let rows: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                let mut map = serde_json::Map::new();
                for i in 0..column_count {
                    let col_name = &column_names[i];
                    let value: rusqlite::types::Value = row.get(i)?;
                    let json_value = match value {
                        rusqlite::types::Value::Null => serde_json::Value::Null,
                        rusqlite::types::Value::Integer(n) => serde_json::json!(n),
                        rusqlite::types::Value::Real(f) => serde_json::json!(f),
                        rusqlite::types::Value::Text(s) => serde_json::json!(s),
                        rusqlite::types::Value::Blob(b) => {
                            serde_json::json!(format!("BLOB({} bytes)", b.len()))
                        }
                    };
                    map.insert(col_name.clone(), json_value);
                }
                Ok(serde_json::Value::Object(map))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(serde_json::json!({
            "rows": rows,
            "rowCount": rows.len()
        }))
    } else {
        // Execute non-SELECT query (INSERT, UPDATE, DELETE, etc.)
        let changes = conn.execute(&sql, []).map_err(|e| e.to_string())?;
        
        Ok(serde_json::json!({
            "affectedRows": changes,
            "message": format!("Query executed successfully. {} row(s) affected.", changes)
        }))
    }
}

/// Get project type (sql or nosql)
#[tauri::command]
pub fn get_project_type(state: State<AppState>) -> Result<String, String> {
    use crate::engine::config::Config;
    
    let project_dir_lock = state.project_dir.lock().map_err(|e| e.to_string())?;
    let project_dir = project_dir_lock
        .as_ref()
        .ok_or("Project directory not set")?;
    
    let config = Config::load(project_dir).map_err(|e| e.to_string())?;
    Ok(config.database.db_type)
}

/// Set project type (sql or nosql)
#[tauri::command]
pub fn set_project_type(
    state: State<AppState>,
    project_type: String,
) -> Result<(), String> {
    use crate::engine::config::Config;
    
    if project_type != "sql" && project_type != "nosql" && project_type != "hybrid" {
        return Err("Invalid project type. Must be 'sql', 'nosql', or 'hybrid'".to_string());
    }
    
    let project_dir_lock = state.project_dir.lock().map_err(|e| e.to_string())?;
    let project_dir = project_dir_lock
        .as_ref()
        .ok_or("Project directory not set")?;
    
    let mut config = Config::load(project_dir).map_err(|e| e.to_string())?;
    config.database.db_type = project_type;
    config.save(project_dir).map_err(|e| e.to_string())?;
    
    Ok(())
}
