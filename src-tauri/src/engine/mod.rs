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

pub use config::Config;
pub use database::Database;
pub use nosql::NoSqlEngine;
