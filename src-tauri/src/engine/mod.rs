// AirDB Engine - Core module structure
pub mod config;
pub mod database;
pub mod migrations;
pub mod api;
pub mod github;
pub mod keystore;
pub mod cli;
pub mod updater;
pub mod nosql;
pub mod hybrid;
pub mod rbac;
pub mod team;
pub mod audit;

pub use config::Config;
pub use database::Database;
pub use nosql::NoSqlEngine;
pub use hybrid::{Relation, RelationType, AirQuery, AirResult};
pub use rbac::{Policy, Enforcer, AuthContext};
pub use team::{BranchContext, BranchLock, ThreeWayMerge, MergeStrategy};
pub use audit::{AuditLog, AuditEntry, AuditAction, BackupManager, Backup};

