//! AirDB Configuration Module
//! Handles loading and validating airdb.config.json

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),
    #[error("Failed to read config: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Invalid config format: {0}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub project: ProjectConfig,
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    #[serde(default)]
    pub github: Option<GitHubConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub repo: String,
    pub visibility: String,
}

fn default_port() -> u16 {
    54321
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

impl Config {
    pub fn load(project_dir: &Path) -> Result<Self, ConfigError> {
        let config_path = project_dir.join("airdb.config.json");
        if !config_path.exists() {
            return Err(ConfigError::NotFound(config_path));
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, project_dir: &Path) -> Result<(), ConfigError> {
        let config_path = project_dir.join("airdb.config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn default_for_project(name: &str) -> Self {
        Self {
            version: "0.1.0".to_string(),
            project: ProjectConfig {
                name: name.to_string(),
                id: format!("airdb-{}", name),
            },
            database: DatabaseConfig {
                db_type: "sqlite".to_string(),
                path: PathBuf::from("./data/airdb.db"),
            },
            api: ApiConfig {
                port: default_port(),
                host: default_host(),
            },
            github: None,
        }
    }
}
