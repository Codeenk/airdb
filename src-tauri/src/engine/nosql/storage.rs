//! NoSQL Storage Engine
//! 
//! Main entry point for the NoSQL engine

use std::fs;
use std::path::{Path, PathBuf};

use super::collection::Collection;
use super::document::Document;
use super::error::{NoSqlError, Result};
use super::meta::Meta;
use super::query::Query;

/// The main NoSQL storage engine
pub struct NoSqlEngine {
    /// Base path for NoSQL storage
    base_path: PathBuf,
    
    /// Metadata for this store
    meta: Meta,
}

impl NoSqlEngine {
    /// Open an existing NoSQL store
    pub fn open(path: &Path) -> Result<Self> {
        let nosql_path = path.join("nosql");
        
        if !nosql_path.exists() {
            return Err(NoSqlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "NoSQL store not found",
            )));
        }
        
        let meta = Meta::load(&nosql_path)?;
        meta.check_app_compatibility()?;
        
        Ok(Self {
            base_path: nosql_path,
            meta,
        })
    }

    /// Create a new NoSQL store
    pub fn create(path: &Path) -> Result<Self> {
        let nosql_path = path.join("nosql");
        
        fs::create_dir_all(&nosql_path)?;
        
        let meta = Meta::new();
        meta.save(&nosql_path)?;
        
        Ok(Self {
            base_path: nosql_path,
            meta,
        })
    }

    /// Open or create a NoSQL store
    pub fn open_or_create(path: &Path) -> Result<Self> {
        let nosql_path = path.join("nosql");
        
        if nosql_path.exists() {
            Self::open(path)
        } else {
            Self::create(path)
        }
    }

    /// Get metadata
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Create a new collection
    pub fn create_collection(&self, name: &str) -> Result<Collection> {
        Collection::create(&self.base_path, name)
    }

    /// Open an existing collection
    pub fn collection(&self, name: &str) -> Result<Collection> {
        Collection::open(&self.base_path, name)
    }

    /// List all collections
    pub fn list_collections(&self) -> Result<Vec<String>> {
        let mut collections = Vec::new();
        
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    // Skip hidden directories
                    if !name_str.starts_with('.') && !name_str.starts_with('_') {
                        collections.push(name_str.to_string());
                    }
                }
            }
        }
        
        collections.sort();
        Ok(collections)
    }

    /// Drop a collection
    pub fn drop_collection(&self, name: &str) -> Result<()> {
        let collection = self.collection(name)?;
        collection.drop()
    }

    /// Check if a collection exists
    pub fn collection_exists(&self, name: &str) -> bool {
        self.base_path.join(name).exists()
    }

    // ========== Convenience Methods ==========

    /// Insert a document into a collection
    pub fn insert(&self, collection: &str, doc: Document) -> Result<String> {
        self.collection(collection)?.insert(doc)
    }

    /// Get a document by ID
    pub fn get(&self, collection: &str, id: &str) -> Result<Document> {
        self.collection(collection)?.get(id)
    }

    /// Query documents
    pub fn query(&self, collection: &str, query: Query) -> Result<Vec<Document>> {
        let col = self.collection(collection)?;
        let docs = col.all()?;
        Ok(query.execute(docs))
    }

    /// Count documents in a collection
    pub fn count(&self, collection: &str) -> Result<usize> {
        self.collection(collection)?.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_engine_lifecycle() {
        let dir = tempdir().unwrap();
        
        // Create engine
        let engine = NoSqlEngine::create(dir.path()).unwrap();
        assert!(engine.list_collections().unwrap().is_empty());
        
        // Create collection
        engine.create_collection("users").unwrap();
        assert!(engine.collection_exists("users"));
        
        // Insert document
        let doc = Document::new(json!({"name": "Alice", "age": 30}));
        let id = engine.insert("users", doc).unwrap();
        
        // Get document
        let retrieved = engine.get("users", &id).unwrap();
        assert_eq!(retrieved.get("name"), Some(&json!("Alice")));
        
        // Query
        let results = engine.query("users", Query::new()).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_engine_reopen() {
        let dir = tempdir().unwrap();
        
        // Create and insert
        {
            let engine = NoSqlEngine::create(dir.path()).unwrap();
            engine.create_collection("test").unwrap();
            engine.insert("test", Document::new(json!({"x": 1}))).unwrap();
        }
        
        // Reopen
        {
            let engine = NoSqlEngine::open(dir.path()).unwrap();
            assert_eq!(engine.count("test").unwrap(), 1);
        }
    }
}
