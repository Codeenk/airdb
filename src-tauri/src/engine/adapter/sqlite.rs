//! SQLite Adapter
//!
//! Implements DatabaseAdapter for SQLite using rusqlite + r2d2 connection pooling.
//! This refactors the existing database.rs code into the adapter trait pattern.

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::{
    AdapterError, AdapterResult, ColumnMeta, ColumnSchema, DataPage, DatabaseAdapter,
    ExecResult, FilterSpec, ForeignKeyInfo, ForeignKeyRef, IndexInfo, QueryResult, SortSpec,
    SqlValue, TableSchema,
};
use super::dialect::SqlDialect;

type DbPool = Pool<SqliteConnectionManager>;
type DbConn = PooledConnection<SqliteConnectionManager>;

pub struct SqliteAdapter {
    pool: DbPool,
    db_path: PathBuf,
}

impl SqliteAdapter {
    pub fn new(db_path: &Path) -> AdapterResult<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        let adapter = Self {
            pool,
            db_path: db_path.to_path_buf(),
        };
        adapter.init_schema()?;
        Ok(adapter)
    }

    pub fn in_memory() -> AdapterResult<Self> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        let adapter = Self {
            pool,
            db_path: PathBuf::from(":memory:"),
        };
        adapter.init_schema()?;
        Ok(adapter)
    }

    fn init_schema(&self) -> AdapterResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS _airdb_migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL,
                checksum TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AdapterError::Schema(e.to_string()))?;

        let _: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|e| AdapterError::Schema(e.to_string()))?;
        conn.execute_batch("PRAGMA foreign_keys=ON")
            .map_err(|e| AdapterError::Schema(e.to_string()))?;

        Ok(())
    }

    fn get_conn(&self) -> AdapterResult<DbConn> {
        self.pool
            .get()
            .map_err(|e| AdapterError::Connection(e.to_string()))
    }

    /// Get the raw connection for legacy code compatibility
    pub fn get_connection(&self) -> AdapterResult<DbConn> {
        self.get_conn()
    }

    /// Migration support methods
    pub fn get_applied_migrations(&self) -> AdapterResult<Vec<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT name FROM _airdb_migrations ORDER BY id")
            .map_err(|e| AdapterError::Query(e.to_string()))?;
        let migrations = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| AdapterError::Query(e.to_string()))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| AdapterError::Query(e.to_string()))?;
        Ok(migrations)
    }

    pub fn record_migration(&self, name: &str, checksum: &str) -> AdapterResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO _airdb_migrations (name, applied_at, checksum) VALUES (?1, datetime('now'), ?2)",
            rusqlite::params![name, checksum],
        )
        .map_err(|e| AdapterError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn backup(&self, backup_path: &Path) -> AdapterResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            &format!("VACUUM INTO '{}'", backup_path.display()),
            [],
        )
        .map_err(|e| AdapterError::Query(e.to_string()))?;
        Ok(())
    }

    /// Convert a rusqlite ValueRef to serde_json Value
    fn value_ref_to_json(val: rusqlite::types::ValueRef<'_>) -> Value {
        match val {
            rusqlite::types::ValueRef::Null => Value::Null,
            rusqlite::types::ValueRef::Integer(i) => json!(i),
            rusqlite::types::ValueRef::Real(f) => json!(f),
            rusqlite::types::ValueRef::Text(t) => {
                json!(String::from_utf8_lossy(t).to_string())
            }
            rusqlite::types::ValueRef::Blob(b) => {
                json!(format!("BLOB({} bytes)", b.len()))
            }
        }
    }

}

impl DatabaseAdapter for SqliteAdapter {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Sqlite
    }

    fn query(&self, sql: &str, params: &[SqlValue]) -> AdapterResult<QueryResult> {
        let conn = self.get_conn()?;
        let start = Instant::now();

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let columns: Vec<ColumnMeta> = column_names
            .iter()
            .map(|name| ColumnMeta {
                name: name.clone(),
                col_type: String::from("TEXT"), // SQLite is dynamically typed
            })
            .collect();

        // Build params for rusqlite
        let param_values: Vec<Box<dyn rusqlite::types::ToSql>> = params
            .iter()
            .map(|p| -> Box<dyn rusqlite::types::ToSql> {
                match p {
                    SqlValue::Null => Box::new(rusqlite::types::Null),
                    SqlValue::Integer(i) => Box::new(*i),
                    SqlValue::Real(f) => Box::new(*f),
                    SqlValue::Text(s) => Box::new(s.clone()),
                    SqlValue::Bool(b) => Box::new(*b as i64),
                    SqlValue::Blob(b) => Box::new(b.clone()),
                }
            })
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows: Vec<Vec<Value>> = stmt
            .query_map(param_refs.as_slice(), |row| {
                let mut vals = Vec::with_capacity(column_names.len());
                for i in 0..column_names.len() {
                    let val = row.get_ref(i).unwrap_or(rusqlite::types::ValueRef::Null);
                    vals.push(Self::value_ref_to_json(val));
                }
                Ok(vals)
            })
            .map_err(|e| AdapterError::Query(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(QueryResult {
            columns,
            rows,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn execute(&self, sql: &str, params: &[SqlValue]) -> AdapterResult<ExecResult> {
        let conn = self.get_conn()?;
        let start = Instant::now();

        let param_values: Vec<Box<dyn rusqlite::types::ToSql>> = params
            .iter()
            .map(|p| -> Box<dyn rusqlite::types::ToSql> {
                match p {
                    SqlValue::Null => Box::new(rusqlite::types::Null),
                    SqlValue::Integer(i) => Box::new(*i),
                    SqlValue::Real(f) => Box::new(*f),
                    SqlValue::Text(s) => Box::new(s.clone()),
                    SqlValue::Bool(b) => Box::new(*b as i64),
                    SqlValue::Blob(b) => Box::new(b.clone()),
                }
            })
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let affected = conn
            .execute(sql, param_refs.as_slice())
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        let last_id = conn.last_insert_rowid();

        Ok(ExecResult {
            affected_rows: affected as u64,
            last_insert_id: Some(last_id),
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn execute_batch(&self, sql: &str) -> AdapterResult<()> {
        let conn = self.get_conn()?;
        conn.execute_batch(sql)
            .map_err(|e| AdapterError::Query(e.to_string()))
    }

    fn get_tables(&self) -> AdapterResult<Vec<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_airdb_%' ORDER BY name")
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        let tables = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| AdapterError::Query(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tables)
    }

    fn get_table_schema(&self, table: &str) -> AdapterResult<TableSchema> {
        let conn = self.get_conn()?;

        // Get columns via PRAGMA table_info
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info('{}')", table))
            .map_err(|e| AdapterError::Schema(e.to_string()))?;

        let columns: Vec<ColumnSchema> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let col_type: String = row.get(2)?;
                let not_null: i32 = row.get(3)?;
                let default: Option<String> = row.get(4)?;
                let pk: i32 = row.get(5)?;

                let is_auto = pk > 0 && col_type.to_uppercase() == "INTEGER";
                Ok(ColumnSchema {
                    name,
                    col_type,
                    nullable: not_null == 0,
                    default_value: default,
                    is_primary_key: pk > 0,
                    is_unique: false,
                    is_auto_increment: is_auto,
                    foreign_key: None,
                })
            })
            .map_err(|e| AdapterError::Schema(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        // Get foreign keys
        let fks = self.get_foreign_keys(table)?;
        let mut columns = columns;
        for fk in &fks {
            if let Some(col) = columns.iter_mut().find(|c| c.name == fk.from_column) {
                col.foreign_key = Some(ForeignKeyRef {
                    table: fk.to_table.clone(),
                    column: fk.to_column.clone(),
                });
            }
        }

        // Get indexes
        let indexes = self.get_table_indexes(table)?;

        // Mark unique columns from indexes
        for idx in &indexes {
            if idx.unique && idx.columns.len() == 1 {
                if let Some(col) = columns.iter_mut().find(|c| c.name == idx.columns[0]) {
                    col.is_unique = true;
                }
            }
        }

        // Get row count
        let row_count = self.get_row_count(table).ok();

        Ok(TableSchema {
            name: table.to_string(),
            columns,
            indexes,
            row_count,
        })
    }

    fn get_table_indexes(&self, table: &str) -> AdapterResult<Vec<IndexInfo>> {
        let conn = self.get_conn()?;

        let mut idx_stmt = conn
            .prepare(&format!("PRAGMA index_list('{}')", table))
            .map_err(|e| AdapterError::Schema(e.to_string()))?;

        let indexes: Vec<IndexInfo> = idx_stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let unique: i32 = row.get(2)?;
                Ok((name, unique > 0))
            })
            .map_err(|e| AdapterError::Schema(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter(|(name, _)| !name.starts_with("sqlite_"))
            .map(|(name, unique)| {
                let columns = self.get_index_columns_inner(&conn, &name);
                IndexInfo {
                    name,
                    columns,
                    unique,
                }
            })
            .collect();

        Ok(indexes)
    }

    fn get_foreign_keys(&self, table: &str) -> AdapterResult<Vec<ForeignKeyInfo>> {
        let conn = self.get_conn()?;

        let mut stmt = conn
            .prepare(&format!("PRAGMA foreign_key_list('{}')", table))
            .map_err(|e| AdapterError::Schema(e.to_string()))?;

        let fks: Vec<ForeignKeyInfo> = stmt
            .query_map([], |row| {
                let _id: i32 = row.get(0)?;
                let _seq: i32 = row.get(1)?;
                let to_table: String = row.get(2)?;
                let from_column: String = row.get(3)?;
                let to_column: String = row.get(4)?;
                let on_update: String = row.get(5)?;
                let on_delete: String = row.get(6)?;

                Ok(ForeignKeyInfo {
                    name: None,
                    from_table: table.to_string(),
                    from_column,
                    to_table,
                    to_column,
                    on_delete,
                    on_update,
                })
            })
            .map_err(|e| AdapterError::Schema(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(fks)
    }

    fn get_row_count(&self, table: &str) -> AdapterResult<u64> {
        let conn = self.get_conn()?;
        let count: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM \"{}\"", table),
                [],
                |row| row.get(0),
            )
            .map_err(|e| AdapterError::Query(e.to_string()))?;
        Ok(count as u64)
    }

    fn query_rows(
        &self,
        table: &str,
        limit: usize,
        offset: usize,
        sort: Option<&SortSpec>,
        filters: &[FilterSpec],
    ) -> AdapterResult<DataPage> {
        let conn = self.get_conn()?;
        let start = Instant::now();

        // Build WHERE clause
        let mut where_parts = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        for filter in filters {
            if filter.operator.needs_value() {
                where_parts.push(format!(
                    "\"{}\" {} ?{}",
                    filter.column,
                    filter.operator.to_sql(),
                    param_idx
                ));
                param_values.push(Box::new(filter.value.clone()));
                param_idx += 1;
            } else {
                where_parts.push(format!(
                    "\"{}\" {}",
                    filter.column,
                    filter.operator.to_sql()
                ));
            }
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };

        // Build ORDER BY
        let order_clause = match sort {
            Some(s) => format!(" ORDER BY \"{}\" {}", s.column, s.direction),
            None => String::new(),
        };

        // Get total count
        let count_sql = format!("SELECT COUNT(*) FROM \"{}\"{}", table, where_clause);
        let count_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let total_count: i64 = conn
            .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        // Build data query
        let data_sql = format!(
            "SELECT * FROM \"{}\"{}{}  LIMIT ?{} OFFSET ?{}",
            table, where_clause, order_clause, param_idx, param_idx + 1
        );

        param_values.push(Box::new(limit as i64));
        param_values.push(Box::new(offset as i64));

        let data_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(&data_sql)
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let columns: Vec<ColumnMeta> = column_names
            .iter()
            .map(|name| ColumnMeta {
                name: name.clone(),
                col_type: String::from("TEXT"),
            })
            .collect();

        let rows: Vec<serde_json::Map<String, Value>> = stmt
            .query_map(data_refs.as_slice(), |row| {
                let mut map = serde_json::Map::new();
                for (i, col_name) in column_names.iter().enumerate() {
                    let val = row.get_ref(i).unwrap_or(rusqlite::types::ValueRef::Null);
                    map.insert(col_name.clone(), Self::value_ref_to_json(val));
                }
                Ok(map)
            })
            .map_err(|e| AdapterError::Query(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(DataPage {
            rows,
            total_count: total_count as u64,
            columns,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn insert_row(
        &self,
        table: &str,
        data: &serde_json::Map<String, Value>,
    ) -> AdapterResult<i64> {
        let conn = self.get_conn()?;

        let columns: Vec<&str> = data.keys().map(|k| k.as_str()).collect();
        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("?{}", i)).collect();
        let col_names: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();

        let sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table,
            col_names.join(", "),
            placeholders.join(", ")
        );

        let values: Vec<String> = data
            .values()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                _ => v.to_string(),
            })
            .collect();

        let params: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

        conn.execute(&sql, params.as_slice())
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        Ok(conn.last_insert_rowid())
    }

    fn update_row(
        &self,
        table: &str,
        id: i64,
        data: &serde_json::Map<String, Value>,
    ) -> AdapterResult<u64> {
        let conn = self.get_conn()?;

        let set_clauses: Vec<String> = data
            .keys()
            .enumerate()
            .map(|(i, k)| format!("\"{}\" = ?{}", k, i + 1))
            .collect();

        let sql = format!(
            "UPDATE \"{}\" SET {} WHERE id = ?{}",
            table,
            set_clauses.join(", "),
            data.len() + 1
        );

        let mut values: Vec<String> = data
            .values()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            })
            .collect();
        values.push(id.to_string());

        let params: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

        let affected = conn
            .execute(&sql, params.as_slice())
            .map_err(|e| AdapterError::Query(e.to_string()))?;

        Ok(affected as u64)
    }

    fn delete_row(&self, table: &str, id: i64) -> AdapterResult<u64> {
        let conn = self.get_conn()?;
        let affected = conn
            .execute(
                &format!("DELETE FROM \"{}\" WHERE id = ?1", table),
                [id],
            )
            .map_err(|e| AdapterError::Query(e.to_string()))?;
        Ok(affected as u64)
    }

    fn get_database_size(&self) -> AdapterResult<u64> {
        if self.db_path.to_str() == Some(":memory:") {
            return Ok(0);
        }
        let metadata = std::fs::metadata(&self.db_path)
            .map_err(|e| AdapterError::Internal(e.to_string()))?;
        Ok(metadata.len())
    }

    fn test_connection(&self) -> AdapterResult<()> {
        let conn = self.get_conn()?;
        conn.execute_batch("SELECT 1")
            .map_err(|e| AdapterError::Connection(e.to_string()))
    }
}

impl SqliteAdapter {
    fn get_index_columns_inner(&self, conn: &rusqlite::Connection, index_name: &str) -> Vec<String> {
        let mut stmt = match conn.prepare(&format!("PRAGMA index_info('{}')", index_name)) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let result: Vec<String> = match stmt.query_map([], |row| {
            let col_name: String = row.get(2)?;
            Ok(col_name)
        }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        };
        result
    }
}
