//! Connection Manager
//!
//! Manages database connections, stores connection configurations,
//! and provides the active adapter for the current session.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::adapter::DatabaseAdapter;
use super::adapter::dialect::SqlDialect;

/// Persisted connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub dialect: SqlDialect,
    pub config: AdapterConfig,
    pub color: Option<String>,
    pub is_default: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Backend-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AdapterConfig {
    Sqlite {
        path: PathBuf,
    },
    Postgres {
        host: String,
        port: u16,
        database: String,
        username: String,
        #[serde(skip_serializing)]
        password: String,
        #[serde(default = "default_ssl_mode")]
        ssl_mode: SslMode,
    },
    Mysql {
        host: String,
        port: u16,
        database: String,
        username: String,
        #[serde(skip_serializing)]
        password: String,
        #[serde(default)]
        ssl: bool,
    },
}

fn default_ssl_mode() -> SslMode {
    SslMode::Prefer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl AdapterConfig {
    /// Get a display-friendly connection string (no password)
    pub fn display_string(&self) -> String {
        match self {
            AdapterConfig::Sqlite { path } => {
                format!("sqlite://{}", path.display())
            }
            AdapterConfig::Postgres {
                host,
                port,
                database,
                username,
                ..
            } => {
                format!("postgres://{}@{}:{}/{}", username, host, port, database)
            }
            AdapterConfig::Mysql {
                host,
                port,
                database,
                username,
                ..
            } => {
                format!("mysql://{}@{}:{}/{}", username, host, port, database)
            }
        }
    }
}

/// Manages saved connections stored in ~/.airdb/connections.json
pub struct ConnectionManager {
    config_path: PathBuf,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            config_path: home.join(".airdb").join("connections.json"),
        }
    }

    pub fn from_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Load all saved connections
    pub fn list_connections(&self) -> Result<Vec<ConnectionConfig>, String> {
        if !self.config_path.exists() {
            return Ok(vec![]);
        }

        let data = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read connections: {}", e))?;
        let connections: Vec<ConnectionConfig> =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse connections: {}", e))?;
        Ok(connections)
    }

    /// Save all connections
    fn save_connections(&self, connections: &[ConnectionConfig]) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let data = serde_json::to_string_pretty(connections)
            .map_err(|e| format!("Failed to serialize connections: {}", e))?;
        std::fs::write(&self.config_path, data)
            .map_err(|e| format!("Failed to write connections: {}", e))?;
        Ok(())
    }

    /// Add a new connection
    pub fn add_connection(&self, config: ConnectionConfig) -> Result<(), String> {
        let mut connections = self.list_connections()?;

        // If it's set as default, unset others
        if config.is_default {
            for conn in &mut connections {
                conn.is_default = false;
            }
        }

        connections.push(config);
        self.save_connections(&connections)
    }

    /// Update an existing connection
    pub fn update_connection(&self, config: ConnectionConfig) -> Result<(), String> {
        let mut connections = self.list_connections()?;

        if config.is_default {
            for conn in &mut connections {
                conn.is_default = false;
            }
        }

        if let Some(existing) = connections.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
        } else {
            return Err(format!("Connection {} not found", config.id));
        }

        self.save_connections(&connections)
    }

    /// Remove a connection by ID
    pub fn remove_connection(&self, id: &str) -> Result<(), String> {
        let mut connections = self.list_connections()?;
        let original_len = connections.len();
        connections.retain(|c| c.id != id);

        if connections.len() == original_len {
            return Err(format!("Connection {} not found", id));
        }

        self.save_connections(&connections)
    }

    /// Get the default connection, or None
    pub fn get_default_connection(&self) -> Result<Option<ConnectionConfig>, String> {
        let connections = self.list_connections()?;
        Ok(connections.into_iter().find(|c| c.is_default))
    }

    /// Get a connection by ID
    pub fn get_connection(&self, id: &str) -> Result<ConnectionConfig, String> {
        let connections = self.list_connections()?;
        connections
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("Connection {} not found", id))
    }

    /// Test a connection configuration without saving it
    pub fn test_connection(config: &AdapterConfig) -> Result<String, String> {
        match config {
            AdapterConfig::Sqlite { path } => {
                use super::adapter::sqlite::SqliteAdapter;
                let adapter = SqliteAdapter::new(path).map_err(|e| e.to_string())?;
                adapter.test_connection().map_err(|e| e.to_string())?;
                Ok("SQLite connection successful".to_string())
            }
            AdapterConfig::Postgres { host, port, database, .. } => {
                // Placeholder — requires sqlx dependency
                Ok(format!(
                    "PostgreSQL connection test to {}:{}/{} — driver not yet installed",
                    host, port, database
                ))
            }
            AdapterConfig::Mysql { host, port, database, .. } => {
                // Placeholder — requires sqlx dependency
                Ok(format!(
                    "MySQL connection test to {}:{}/{} — driver not yet installed",
                    host, port, database
                ))
            }
        }
    }

    /// Create a database adapter from a connection config
    pub fn create_adapter(
        config: &AdapterConfig,
    ) -> Result<Box<dyn super::adapter::DatabaseAdapter>, String> {
        match config {
            AdapterConfig::Sqlite { path } => {
                let adapter = super::adapter::sqlite::SqliteAdapter::new(path)
                    .map_err(|e| e.to_string())?;
                Ok(Box::new(adapter))
            }
            AdapterConfig::Postgres { .. } => {
                Err("PostgreSQL adapter not yet implemented — add sqlx dependency".to_string())
            }
            AdapterConfig::Mysql { .. } => {
                Err("MySQL adapter not yet implemented — add sqlx dependency".to_string())
            }
        }
    }
}
