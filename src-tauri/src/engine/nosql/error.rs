//! NoSQL Error Types

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoSqlError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Collection already exists: {0}")]
    CollectionAlreadyExists(String),

    #[error("Invalid collection name: {0}")]
    InvalidCollectionName(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Format version {found} not supported (min: {min}, max: {max})")]
    UnsupportedFormatVersion {
        found: u32,
        min: u32,
        max: u32,
    },

    #[error("App version {app} too old for format (requires: {required})")]
    AppVersionTooOld {
        app: String,
        required: String,
    },

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Document ID already exists: {0}")]
    DuplicateId(String),
}

pub type Result<T> = std::result::Result<T, NoSqlError>;
