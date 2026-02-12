//! AirDB Database Module
//! SQLite adapter with connection pooling

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::path::Path;
use thiserror::Error;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to create database pool: {0}")]
    PoolError(#[from] r2d2::Error),
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("Database file not found: {0}")]
    NotFound(String),
}

#[derive(Clone)]
pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, DatabaseError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)?;

        let db = Self { pool };
        db.init_schema()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self, DatabaseError> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)?;
        let db = Self { pool };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        
        // Create migrations journal table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _airdb_migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL,
                checksum TEXT NOT NULL
            )",
            [],
        )?;

        // Enable WAL mode for better concurrency (PRAGMA returns result, use query)
        let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        // Foreign keys PRAGMA doesn't return value in same way
        conn.execute_batch("PRAGMA foreign_keys=ON")?;

        Ok(())
    }

    pub fn get_connection(&self) -> Result<DbConnection, DatabaseError> {
        Ok(self.pool.get()?)
    }

    pub fn get_applied_migrations(&self) -> Result<Vec<String>, DatabaseError> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT name FROM _airdb_migrations ORDER BY id")?;
        let migrations = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(migrations)
    }

    pub fn record_migration(&self, name: &str, checksum: &str) -> Result<(), DatabaseError> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO _airdb_migrations (name, applied_at, checksum) VALUES (?1, datetime('now'), ?2)",
            params![name, checksum],
        )?;
        Ok(())
    }

    pub fn get_tables(&self) -> Result<Vec<String>, DatabaseError> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_airdb_%' AND name NOT LIKE 'sqlite_%'"
        )?;
        let tables = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(tables)
    }

    pub fn get_table_schema(&self, table_name: &str) -> Result<Vec<ColumnInfo>, DatabaseError> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnInfo {
                    cid: row.get(0)?,
                    name: row.get(1)?,
                    col_type: row.get(2)?,
                    notnull: row.get(3)?,
                    dflt_value: row.get(4)?,
                    pk: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<ColumnInfo>, _>>()?;
        Ok(columns)
    }

    pub fn backup(&self, backup_path: &Path) -> Result<(), DatabaseError> {
        let conn = self.get_connection()?;
        conn.execute(&format!("VACUUM INTO '{}'", backup_path.display()), [])?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub col_type: String,
    pub notnull: bool,
    pub dflt_value: Option<String>,
    pub pk: bool,
}
