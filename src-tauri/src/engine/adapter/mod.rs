//! Database Adapter Layer
//!
//! Provides a trait-based abstraction over multiple database backends.
//! Currently supports SQLite with PostgreSQL and MySQL planned.

pub mod sqlite;
pub mod dialect;

pub use dialect::{SqlDialect, DialectGenerator};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Universal result type for adapter operations
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Core database adapter trait — all backends implement this
pub trait DatabaseAdapter: Send + Sync {
    /// Which SQL dialect this adapter uses
    fn dialect(&self) -> SqlDialect;

    /// Execute a query that returns rows (SELECT, PRAGMA, etc.)
    fn query(&self, sql: &str, params: &[SqlValue]) -> AdapterResult<QueryResult>;

    /// Execute a statement that modifies data (INSERT, UPDATE, DELETE, CREATE, etc.)
    fn execute(&self, sql: &str, params: &[SqlValue]) -> AdapterResult<ExecResult>;

    /// Execute multiple statements as a batch
    fn execute_batch(&self, sql: &str) -> AdapterResult<()>;

    /// Get all user table names
    fn get_tables(&self) -> AdapterResult<Vec<String>>;

    /// Get detailed schema for a table
    fn get_table_schema(&self, table: &str) -> AdapterResult<TableSchema>;

    /// Get indexes for a table
    fn get_table_indexes(&self, table: &str) -> AdapterResult<Vec<IndexInfo>>;

    /// Get foreign key relationships for a table
    fn get_foreign_keys(&self, table: &str) -> AdapterResult<Vec<ForeignKeyInfo>>;

    /// Get row count for a table
    fn get_row_count(&self, table: &str) -> AdapterResult<u64>;

    /// Query rows with pagination, sorting, and filtering
    fn query_rows(
        &self,
        table: &str,
        limit: usize,
        offset: usize,
        sort: Option<&SortSpec>,
        filters: &[FilterSpec],
    ) -> AdapterResult<DataPage>;

    /// Insert a row and return the new ID
    fn insert_row(&self, table: &str, data: &serde_json::Map<String, serde_json::Value>) -> AdapterResult<i64>;

    /// Update a row by primary key
    fn update_row(&self, table: &str, id: i64, data: &serde_json::Map<String, serde_json::Value>) -> AdapterResult<u64>;

    /// Delete a row by primary key
    fn delete_row(&self, table: &str, id: i64) -> AdapterResult<u64>;

    /// Get database file size or connection info
    fn get_database_size(&self) -> AdapterResult<u64>;

    /// Test the connection is alive
    fn test_connection(&self) -> AdapterResult<()>;
}

/// SQL value for parameterized queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
    Blob(Vec<u8>),
}

/// Result from a SELECT-type query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub execution_time_ms: u64,
}

/// Metadata about a result column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
}

/// Result from an INSERT/UPDATE/DELETE-type statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub affected_rows: u64,
    pub last_insert_id: Option<i64>,
    pub execution_time_ms: u64,
}

/// Detailed table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
    pub indexes: Vec<IndexInfo>,
    pub row_count: Option<u64>,
}

/// Column schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub is_auto_increment: bool,
    pub foreign_key: Option<ForeignKeyRef>,
}

/// Foreign key reference on a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,
}

/// Index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// Foreign key relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    pub name: Option<String>,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub on_delete: String,
    pub on_update: String,
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub column: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "ASC"),
            SortDirection::Desc => write!(f, "DESC"),
        }
    }
}

/// Filter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSpec {
    pub column: String,
    pub operator: FilterOp,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    IsNull,
    IsNotNull,
}

impl FilterOp {
    pub fn to_sql(&self) -> &'static str {
        match self {
            FilterOp::Eq => "=",
            FilterOp::Neq => "!=",
            FilterOp::Gt => ">",
            FilterOp::Lt => "<",
            FilterOp::Gte => ">=",
            FilterOp::Lte => "<=",
            FilterOp::Like => "LIKE",
            FilterOp::IsNull => "IS NULL",
            FilterOp::IsNotNull => "IS NOT NULL",
        }
    }
    
    pub fn needs_value(&self) -> bool {
        !matches!(self, FilterOp::IsNull | FilterOp::IsNotNull)
    }
}

/// Paginated data result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPage {
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    pub total_count: u64,
    pub columns: Vec<ColumnMeta>,
    pub execution_time_ms: u64,
}

/// Database adapter errors
#[derive(Debug)]
pub enum AdapterError {
    Connection(String),
    Query(String),
    Schema(String),
    NotFound(String),
    Validation(String),
    Internal(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::Connection(e) => write!(f, "Connection error: {}", e),
            AdapterError::Query(e) => write!(f, "Query error: {}", e),
            AdapterError::Schema(e) => write!(f, "Schema error: {}", e),
            AdapterError::NotFound(e) => write!(f, "Not found: {}", e),
            AdapterError::Validation(e) => write!(f, "Validation error: {}", e),
            AdapterError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for AdapterError {}
