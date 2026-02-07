//! Collection management for NoSQL storage

use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

use super::document::Document;
use super::error::{NoSqlError, Result};
use super::schema::Schema;
use super::migration::MigrationRunner;

/// A NoSQL collection (like a table in SQL)
pub struct Collection {
    /// Collection name
    pub name: String,
    
    /// Path to collection directory
    path: PathBuf,
    
    /// Current schema (if any)
    schema: Option<Schema>,
}

impl Collection {
    /// Open an existing collection
    pub fn open(base_path: &Path, name: &str) -> Result<Self> {
        let path = base_path.join(name);
        
        if !path.exists() {
            return Err(NoSqlError::CollectionNotFound(name.to_string()));
        }
        
        // Build schema from migrations
        let runner = MigrationRunner::new(&path);
        let schema = match runner.build_schema() {
            Ok(s) if !s.fields.is_empty() => Some(s),
            _ => None,
        };
        
        Ok(Self {
            name: name.to_string(),
            path,
            schema,
        })
    }

    /// Create a new collection
    pub fn create(base_path: &Path, name: &str) -> Result<Self> {
        validate_collection_name(name)?;
        
        let path = base_path.join(name);
        
        if path.exists() {
            return Err(NoSqlError::CollectionAlreadyExists(name.to_string()));
        }
        
        fs::create_dir_all(&path)?;
        fs::create_dir_all(path.join("migrations"))?;
        
        Ok(Self {
            name: name.to_string(),
            path,
            schema: None,
        })
    }

    /// Insert a document
    pub fn insert(&self, doc: Document) -> Result<String> {
        // Validate against schema if exists
        if let Some(ref schema) = self.schema {
            schema.validate(&doc.data)?;
        }
        
        let doc_path = self.path.join(format!("{}.json", doc.id));
        
        if doc_path.exists() {
            return Err(NoSqlError::DuplicateId(doc.id));
        }
        
        let content = serde_json::to_string_pretty(&doc)?;
        fs::write(doc_path, content)?;
        
        Ok(doc.id)
    }

    /// Get a document by ID
    pub fn get(&self, id: &str) -> Result<Document> {
        let doc_path = self.path.join(format!("{}.json", id));
        
        if !doc_path.exists() {
            return Err(NoSqlError::DocumentNotFound(id.to_string()));
        }
        
        let content = fs::read_to_string(doc_path)?;
        let doc: Document = serde_json::from_str(&content)?;
        
        Ok(doc)
    }

    /// Update a document
    pub fn update(&self, id: &str, data: Value) -> Result<Document> {
        let mut doc = self.get(id)?;
        
        // Validate against schema if exists
        if let Some(ref schema) = self.schema {
            schema.validate(&data)?;
        }
        
        doc.update(data);
        
        let doc_path = self.path.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&doc)?;
        fs::write(doc_path, content)?;
        
        Ok(doc)
    }

    /// Delete a document
    pub fn delete(&self, id: &str) -> Result<()> {
        let doc_path = self.path.join(format!("{}.json", id));
        
        if !doc_path.exists() {
            return Err(NoSqlError::DocumentNotFound(id.to_string()));
        }
        
        fs::remove_file(doc_path)?;
        Ok(())
    }

    /// List all document IDs
    pub fn list_ids(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy();
                    // Skip system files
                    if !name.starts_with('_') {
                        ids.push(name.to_string());
                    }
                }
            }
        }
        
        ids.sort();
        Ok(ids)
    }

    /// Count documents
    pub fn count(&self) -> Result<usize> {
        Ok(self.list_ids()?.len())
    }

    /// Get all documents
    pub fn all(&self) -> Result<Vec<Document>> {
        let ids = self.list_ids()?;
        let mut docs = Vec::with_capacity(ids.len());
        
        for id in ids {
            docs.push(self.get(&id)?);
        }
        
        Ok(docs)
    }

    /// Drop this collection
    pub fn drop(self) -> Result<()> {
        fs::remove_dir_all(&self.path)?;
        Ok(())
    }
}

/// Validate collection name
fn validate_collection_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(NoSqlError::InvalidCollectionName("name cannot be empty".to_string()));
    }
    
    if name.starts_with('_') {
        return Err(NoSqlError::InvalidCollectionName("name cannot start with underscore".to_string()));
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(NoSqlError::InvalidCollectionName("name must be alphanumeric".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde_json::json;

    #[test]
    fn test_collection_crud() {
        let dir = tempdir().unwrap();
        
        // Create collection
        let col = Collection::create(dir.path(), "users").unwrap();
        
        // Insert
        let doc = Document::new(json!({"name": "Alice", "age": 30}));
        let id = col.insert(doc).unwrap();
        
        // Get
        let retrieved = col.get(&id).unwrap();
        assert_eq!(retrieved.get("name"), Some(&json!("Alice")));
        
        // Update
        col.update(&id, json!({"name": "Alice", "age": 31})).unwrap();
        let updated = col.get(&id).unwrap();
        assert_eq!(updated.get("age"), Some(&json!(31)));
        
        // Delete
        col.delete(&id).unwrap();
        assert!(col.get(&id).is_err());
    }
}
