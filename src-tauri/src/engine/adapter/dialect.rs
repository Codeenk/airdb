//! SQL Dialect Generator
//!
//! Generates database-specific SQL from universal type definitions.
//! Supports SQLite, PostgreSQL, and MySQL dialects.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SqlDialect {
    Sqlite,
    Postgres,
    Mysql,
}

impl fmt::Display for SqlDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlDialect::Sqlite => write!(f, "sqlite"),
            SqlDialect::Postgres => write!(f, "postgres"),
            SqlDialect::Mysql => write!(f, "mysql"),
        }
    }
}

impl SqlDialect {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => Some(SqlDialect::Sqlite),
            "postgres" | "postgresql" | "pg" => Some(SqlDialect::Postgres),
            "mysql" | "mariadb" => Some(SqlDialect::Mysql),
            _ => None,
        }
    }

    /// Get the data types available for this dialect
    pub fn data_types(&self) -> Vec<&'static str> {
        match self {
            SqlDialect::Sqlite => vec![
                "INTEGER", "REAL", "TEXT", "BLOB", "NUMERIC",
            ],
            SqlDialect::Postgres => vec![
                "SERIAL", "BIGSERIAL", "SMALLINT", "INTEGER", "BIGINT",
                "REAL", "DOUBLE PRECISION", "NUMERIC", "DECIMAL",
                "TEXT", "VARCHAR", "CHAR", "BOOLEAN", "UUID",
                "DATE", "TIME", "TIMESTAMP", "TIMESTAMPTZ",
                "JSON", "JSONB", "BYTEA", "INET", "CIDR",
                "MACADDR", "POINT", "LINE", "INTERVAL",
                "INT[]", "TEXT[]",
            ],
            SqlDialect::Mysql => vec![
                "TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT",
                "FLOAT", "DOUBLE", "DECIMAL",
                "CHAR", "VARCHAR", "TINYTEXT", "TEXT", "MEDIUMTEXT", "LONGTEXT",
                "BOOLEAN", "DATE", "TIME", "DATETIME", "TIMESTAMP",
                "JSON", "BLOB", "MEDIUMBLOB", "LONGBLOB",
                "ENUM", "SET",
            ],
        }
    }
}

/// SQL dialect-aware code generator
pub struct DialectGenerator {
    pub dialect: SqlDialect,
}

impl DialectGenerator {
    pub fn new(dialect: SqlDialect) -> Self {
        Self { dialect }
    }

    /// Map a universal/generic type to the dialect-specific type
    pub fn map_type(&self, universal_type: &str) -> String {
        let upper = universal_type.to_uppercase();
        match (self.dialect, upper.as_str()) {
            // Auto-increment primary key
            (SqlDialect::Sqlite, "SERIAL") => "INTEGER".into(),
            (SqlDialect::Postgres, "SERIAL") => "SERIAL".into(),
            (SqlDialect::Mysql, "SERIAL") => "INT AUTO_INCREMENT".into(),

            // UUID
            (SqlDialect::Sqlite, "UUID") => "TEXT".into(),
            (SqlDialect::Postgres, "UUID") => "UUID DEFAULT gen_random_uuid()".into(),
            (SqlDialect::Mysql, "UUID") => "CHAR(36)".into(),

            // Booleans
            (SqlDialect::Sqlite, "BOOLEAN") => "INTEGER".into(),
            (SqlDialect::Postgres, "BOOLEAN") => "BOOLEAN".into(),
            (SqlDialect::Mysql, "BOOLEAN") => "TINYINT(1)".into(),

            // Date/Time
            (SqlDialect::Sqlite, "DATETIME") | (SqlDialect::Sqlite, "TIMESTAMP") => "TEXT".into(),
            (SqlDialect::Postgres, "DATETIME") | (SqlDialect::Postgres, "TIMESTAMP") => {
                "TIMESTAMPTZ".into()
            }
            (SqlDialect::Mysql, "DATETIME") => "DATETIME".into(),
            (SqlDialect::Mysql, "TIMESTAMP") => "TIMESTAMP".into(),

            (SqlDialect::Sqlite, "DATE") => "TEXT".into(),
            (SqlDialect::Postgres, "DATE") => "DATE".into(),
            (SqlDialect::Mysql, "DATE") => "DATE".into(),

            (SqlDialect::Sqlite, "TIME") => "TEXT".into(),
            (SqlDialect::Postgres, "TIME") => "TIME".into(),
            (SqlDialect::Mysql, "TIME") => "TIME".into(),

            // JSON
            (SqlDialect::Sqlite, "JSON") | (SqlDialect::Sqlite, "JSONB") => "TEXT".into(),
            (SqlDialect::Postgres, "JSON") => "JSON".into(),
            (SqlDialect::Postgres, "JSONB") => "JSONB".into(),
            (SqlDialect::Mysql, "JSON") | (SqlDialect::Mysql, "JSONB") => "JSON".into(),

            // Binary
            (SqlDialect::Sqlite, "BYTEA") => "BLOB".into(),
            (SqlDialect::Postgres, "BLOB") => "BYTEA".into(),
            (SqlDialect::Mysql, "BYTEA") => "LONGBLOB".into(),

            // Big integer
            (SqlDialect::Sqlite, "BIGINT") => "INTEGER".into(),
            (SqlDialect::Postgres, "BIGINT") => "BIGINT".into(),
            (SqlDialect::Mysql, "BIGINT") => "BIGINT".into(),

            // Float / Double
            (SqlDialect::Sqlite, "FLOAT") | (SqlDialect::Sqlite, "DOUBLE") | (SqlDialect::Sqlite, "DOUBLE PRECISION") => "REAL".into(),
            (SqlDialect::Postgres, "FLOAT") | (SqlDialect::Postgres, "DOUBLE") => "DOUBLE PRECISION".into(),
            (SqlDialect::Mysql, "FLOAT") => "FLOAT".into(),
            (SqlDialect::Mysql, "DOUBLE") | (SqlDialect::Mysql, "DOUBLE PRECISION") => "DOUBLE".into(),

            // VARCHAR with size
            _ if upper.starts_with("VARCHAR") => {
                match self.dialect {
                    SqlDialect::Sqlite => "TEXT".into(),
                    _ => universal_type.to_string(),
                }
            }

            // DECIMAL with precision
            _ if upper.starts_with("DECIMAL") || upper.starts_with("NUMERIC") => {
                match self.dialect {
                    SqlDialect::Sqlite => "REAL".into(),
                    _ => universal_type.to_string(),
                }
            }

            // Pass through unchanged
            _ => universal_type.to_string(),
        }
    }

    /// Get the auto-increment syntax for a primary key column
    pub fn auto_increment_pk(&self, col_name: &str) -> String {
        match self.dialect {
            SqlDialect::Sqlite => {
                format!("{} INTEGER PRIMARY KEY AUTOINCREMENT", col_name)
            }
            SqlDialect::Postgres => {
                format!("{} SERIAL PRIMARY KEY", col_name)
            }
            SqlDialect::Mysql => {
                format!("{} INT AUTO_INCREMENT PRIMARY KEY", col_name)
            }
        }
    }

    /// Quote an identifier (table or column name)
    pub fn quote_ident(&self, name: &str) -> String {
        match self.dialect {
            SqlDialect::Sqlite | SqlDialect::Postgres => format!("\"{}\"", name),
            SqlDialect::Mysql => format!("`{}`", name),
        }
    }

    /// Generate CREATE TABLE SQL
    pub fn create_table(&self, table: &str, columns: &[ColumnDef]) -> String {
        let mut col_defs = Vec::new();

        for col in columns {
            let mut def = if col.is_auto_increment && col.is_primary_key {
                self.auto_increment_pk(&col.name)
            } else {
                let mapped_type = self.map_type(&col.col_type);
                format!("{} {}", self.quote_ident(&col.name), mapped_type)
            };

            if !col.is_auto_increment || !col.is_primary_key {
                if col.is_primary_key {
                    def.push_str(" PRIMARY KEY");
                }
                if !col.nullable && !col.is_primary_key {
                    def.push_str(" NOT NULL");
                }
                if col.is_unique && !col.is_primary_key {
                    def.push_str(" UNIQUE");
                }
                if let Some(ref default) = col.default_value {
                    if !default.is_empty() {
                        def.push_str(&format!(" DEFAULT {}", default));
                    }
                }
                if let Some(ref fk) = col.foreign_key {
                    def.push_str(&format!(
                        " REFERENCES {}({})",
                        self.quote_ident(&fk.table),
                        self.quote_ident(&fk.column)
                    ));
                }
            }

            col_defs.push(def);
        }

        format!(
            "CREATE TABLE {} (\n  {}\n);",
            self.quote_ident(table),
            col_defs.join(",\n  ")
        )
    }

    /// Generate DROP TABLE SQL
    pub fn drop_table(&self, table: &str) -> String {
        format!("DROP TABLE {};", self.quote_ident(table))
    }

    /// Generate ALTER TABLE ADD COLUMN SQL
    pub fn add_column(&self, table: &str, col: &ColumnDef) -> String {
        let mapped_type = self.map_type(&col.col_type);
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.quote_ident(table),
            self.quote_ident(&col.name),
            mapped_type
        );

        if !col.nullable {
            sql.push_str(" NOT NULL");
        }

        if let Some(ref default) = col.default_value {
            if !default.is_empty() {
                sql.push_str(&format!(" DEFAULT {}", default));
            }
        }

        sql.push(';');
        sql
    }

    /// Generate ALTER TABLE DROP COLUMN SQL (not supported in old SQLite)
    pub fn drop_column(&self, table: &str, column: &str) -> String {
        match self.dialect {
            SqlDialect::Sqlite => {
                format!(
                    "-- SQLite: DROP COLUMN requires table rebuild\n-- ALTER TABLE {} DROP COLUMN {};",
                    self.quote_ident(table),
                    self.quote_ident(column)
                )
            }
            _ => format!(
                "ALTER TABLE {} DROP COLUMN {};",
                self.quote_ident(table),
                self.quote_ident(column)
            ),
        }
    }

    /// Generate ALTER TABLE RENAME COLUMN SQL
    pub fn rename_column(&self, table: &str, old_name: &str, new_name: &str) -> String {
        format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {};",
            self.quote_ident(table),
            self.quote_ident(old_name),
            self.quote_ident(new_name)
        )
    }

    /// Generate CREATE INDEX SQL
    pub fn create_index(&self, table: &str, index_name: &str, columns: &[&str], unique: bool) -> String {
        let unique_str = if unique { "UNIQUE " } else { "" };
        let col_list: Vec<String> = columns.iter().map(|c| self.quote_ident(c)).collect();
        format!(
            "CREATE {}INDEX {} ON {} ({});",
            unique_str,
            self.quote_ident(index_name),
            self.quote_ident(table),
            col_list.join(", ")
        )
    }

    /// Generate DROP INDEX SQL
    pub fn drop_index(&self, index_name: &str) -> String {
        match self.dialect {
            SqlDialect::Mysql => {
                // MySQL requires table name for DROP INDEX, but we generate basic form
                format!("DROP INDEX {};", self.quote_ident(index_name))
            }
            _ => format!("DROP INDEX {};", self.quote_ident(index_name)),
        }
    }

    /// Generate SELECT with LIMIT/OFFSET
    pub fn select_paginated(&self, table: &str, limit: usize, offset: usize) -> String {
        format!(
            "SELECT * FROM {} LIMIT {} OFFSET {}",
            self.quote_ident(table),
            limit,
            offset
        )
    }

    /// Get the current timestamp expression
    pub fn now_expr(&self) -> &'static str {
        match self.dialect {
            SqlDialect::Sqlite => "datetime('now')",
            SqlDialect::Postgres => "NOW()",
            SqlDialect::Mysql => "NOW()",
        }
    }
}

/// Column definition used by the dialect generator
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub is_auto_increment: bool,
    pub foreign_key: Option<ForeignKeyDef>,
}

/// Foreign key definition for column defs
#[derive(Debug, Clone)]
pub struct ForeignKeyDef {
    pub table: String,
    pub column: String,
}
