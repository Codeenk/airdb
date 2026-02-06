//! AirDB Keystore Module
//! Encrypted storage for API keys and GitHub tokens

use keyring::Entry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

const SERVICE_NAME: &str = "airdb";

#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("Keyring error: {0}")]
    KeyringError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Key not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub role: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStore {
    pub keys: Vec<ApiKey>,
}

impl Default for ApiKeyStore {
    fn default() -> Self {
        Self { keys: Vec::new() }
    }
}

pub struct Keystore {
    project_dir: PathBuf,
}

impl Keystore {
    pub fn new(project_dir: &Path) -> Self {
        Self {
            project_dir: project_dir.to_path_buf(),
        }
    }

    /// Store GitHub OAuth token in OS keyring
    pub fn store_github_token(&self, token: &str) -> Result<(), KeystoreError> {
        let entry = Entry::new(SERVICE_NAME, "github_token")
            .map_err(|e| KeystoreError::KeyringError(e.to_string()))?;
        entry
            .set_password(token)
            .map_err(|e| KeystoreError::KeyringError(e.to_string()))?;
        Ok(())
    }

    /// Retrieve GitHub OAuth token from OS keyring
    pub fn get_github_token(&self) -> Result<String, KeystoreError> {
        let entry = Entry::new(SERVICE_NAME, "github_token")
            .map_err(|e| KeystoreError::KeyringError(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| KeystoreError::KeyringError(e.to_string()))
    }

    /// Delete GitHub OAuth token from OS keyring
    pub fn delete_github_token(&self) -> Result<(), KeystoreError> {
        let entry = Entry::new(SERVICE_NAME, "github_token")
            .map_err(|e| KeystoreError::KeyringError(e.to_string()))?;
        entry
            .delete_credential()
            .map_err(|e| KeystoreError::KeyringError(e.to_string()))?;
        Ok(())
    }

    /// Generate a new API key and store it
    pub fn create_api_key(&self, name: &str, role: &str) -> Result<(String, ApiKey), KeystoreError> {
        let raw_key = format!("airdb_{}_{}", role, Uuid::new_v4());
        let key_hash = Self::hash_key(&raw_key);

        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            key_hash,
            role: role.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: None,
            last_used: None,
        };

        let mut store = self.load_api_keys()?;
        store.keys.push(api_key.clone());
        self.save_api_keys(&store)?;

        Ok((raw_key, api_key))
    }

    /// Validate an API key
    pub fn validate_api_key(&self, raw_key: &str) -> Result<Option<ApiKey>, KeystoreError> {
        let key_hash = Self::hash_key(raw_key);
        let store = self.load_api_keys()?;

        Ok(store.keys.into_iter().find(|k| k.key_hash == key_hash))
    }

    /// Revoke an API key by ID
    pub fn revoke_api_key(&self, key_id: &str) -> Result<bool, KeystoreError> {
        let mut store = self.load_api_keys()?;
        let original_len = store.keys.len();
        store.keys.retain(|k| k.id != key_id);

        if store.keys.len() < original_len {
            self.save_api_keys(&store)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List all API keys
    pub fn list_api_keys(&self) -> Result<Vec<ApiKey>, KeystoreError> {
        let store = self.load_api_keys()?;
        Ok(store.keys)
    }

    fn load_api_keys(&self) -> Result<ApiKeyStore, KeystoreError> {
        let keys_path = self.project_dir.join(".airdb").join("keys.json");
        
        if !keys_path.exists() {
            return Ok(ApiKeyStore::default());
        }

        let content = std::fs::read_to_string(&keys_path)?;
        let store: ApiKeyStore = serde_json::from_str(&content)?;
        Ok(store)
    }

    fn save_api_keys(&self, store: &ApiKeyStore) -> Result<(), KeystoreError> {
        let airdb_dir = self.project_dir.join(".airdb");
        std::fs::create_dir_all(&airdb_dir)?;

        let keys_path = airdb_dir.join("keys.json");
        let content = serde_json::to_string_pretty(store)?;
        std::fs::write(&keys_path, content)?;

        Ok(())
    }

    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
    }
}
