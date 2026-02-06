//! AirDB Migrations Module
//! Handles migration file generation, execution, and rollback

use crate::engine::database::{Database, DatabaseError};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Migration already applied: {0}")]
    AlreadyApplied(String),
    #[error("Migration not found: {0}")]
    NotFound(String),
    #[error("Schema drift detected: {0}")]
    SchemaDrift(String),
    #[error("SQL execution error: {0}")]
    SqlError(#[from] rusqlite::Error),
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub name: String,
    pub path: PathBuf,
    pub sql: String,
    pub checksum: String,
}

pub struct MigrationRunner {
    migrations_dir: PathBuf,
}

impl MigrationRunner {
    pub fn new(project_dir: &Path) -> Self {
        Self {
            migrations_dir: project_dir.join("sql").join("migrations"),
        }
    }

    pub fn create(&self, name: &str) -> Result<PathBuf, MigrationError> {
        fs::create_dir_all(&self.migrations_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let safe_name = name.replace(' ', "_").to_lowercase();
        let filename = format!("{}_{}.sql", timestamp, safe_name);
        let path = self.migrations_dir.join(&filename);

        let template = format!(
            "-- Migration: {}\n-- Created: {}\n\n-- Write your SQL here\n",
            name,
            Utc::now().to_rfc3339()
        );

        fs::write(&path, template)?;
        Ok(path)
    }

    pub fn list_pending(&self, db: &Database) -> Result<Vec<Migration>, MigrationError> {
        let applied = db.get_applied_migrations()?;
        let mut pending = Vec::new();

        if !self.migrations_dir.exists() {
            return Ok(pending);
        }

        let mut entries: Vec<_> = fs::read_dir(&self.migrations_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if !applied.contains(&name) {
                let sql = fs::read_to_string(entry.path())?;
                let checksum = Self::compute_checksum(&sql);
                pending.push(Migration {
                    name,
                    path: entry.path(),
                    sql,
                    checksum,
                });
            }
        }

        Ok(pending)
    }

    pub fn apply(&self, db: &Database, migration: &Migration) -> Result<(), MigrationError> {
        let conn = db.get_connection()?;

        // Execute in a transaction
        conn.execute("BEGIN TRANSACTION", [])?;

        match conn.execute_batch(&migration.sql) {
            Ok(_) => {
                db.record_migration(&migration.name, &migration.checksum)?;
                conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK", [])?;
                Err(MigrationError::SqlError(e))
            }
        }
    }

    pub fn push(&self, db: &Database) -> Result<Vec<String>, MigrationError> {
        let pending = self.list_pending(db)?;
        let mut applied = Vec::new();

        for migration in pending {
            self.apply(db, &migration)?;
            applied.push(migration.name);
        }

        Ok(applied)
    }

    pub fn check(&self, db: &Database) -> Result<MigrationStatus, MigrationError> {
        let applied = db.get_applied_migrations()?;
        let pending = self.list_pending(db)?;

        Ok(MigrationStatus {
            applied_count: applied.len(),
            pending_count: pending.len(),
            pending_migrations: pending.iter().map(|m| m.name.clone()).collect(),
        })
    }

    pub fn generate_schema_snapshot(&self, db: &Database, project_dir: &Path) -> Result<PathBuf, MigrationError> {
        let tables = db.get_tables()?;
        let mut schema = String::new();

        schema.push_str("-- AirDB Schema Snapshot\n");
        schema.push_str(&format!("-- Generated: {}\n\n", Utc::now().to_rfc3339()));

        for table in tables {
            let columns = db.get_table_schema(&table)?;
            schema.push_str(&format!("CREATE TABLE {} (\n", table));

            for (i, col) in columns.iter().enumerate() {
                let null_str = if col.notnull { " NOT NULL" } else { "" };
                let pk_str = if col.pk { " PRIMARY KEY" } else { "" };
                let default_str = col.dflt_value.as_ref()
                    .map(|d| format!(" DEFAULT {}", d))
                    .unwrap_or_default();
                let comma = if i < columns.len() - 1 { "," } else { "" };
                
                schema.push_str(&format!(
                    "  {} {}{}{}{}{}\n",
                    col.name, col.col_type, null_str, pk_str, default_str, comma
                ));
            }

            schema.push_str(");\n\n");
        }

        let schema_path = project_dir.join("sql").join("schema.sql");
        fs::create_dir_all(schema_path.parent().unwrap())?;
        fs::write(&schema_path, schema)?;

        Ok(schema_path)
    }

    fn compute_checksum(sql: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sql.as_bytes());
        let result = hasher.finalize();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
    }
}

#[derive(Debug)]
pub struct MigrationStatus {
    pub applied_count: usize,
    pub pending_count: usize,
    pub pending_migrations: Vec<String>,
}
