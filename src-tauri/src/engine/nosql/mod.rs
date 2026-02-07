//! AirDB NoSQL Engine
//! 
//! A versioned, document-based JSON storage engine with:
//! - Format versioning for safe updates
//! - Migration-based schema changes
//! - ULID-based document IDs
//! - Update-safe design

pub mod storage;
pub mod meta;
pub mod schema;
pub mod collection;
pub mod document;
pub mod query;
pub mod error;
pub mod migration;

pub use storage::NoSqlEngine;
pub use meta::Meta;
pub use schema::Schema;
pub use collection::Collection;
pub use document::Document;
pub use query::{Filter, Query};
pub use error::NoSqlError;
pub use migration::{Migration, MigrationOp, MigrationRunner};

