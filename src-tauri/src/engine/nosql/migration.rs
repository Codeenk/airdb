//! NoSQL Migration System
//! 
//! Migration-based schema changes for safe updates

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

use super::error::Result;
use super::schema::{Schema, FieldDef, FieldType};

/// A single migration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum MigrationOp {
    /// Add a new field
    AddField {
        name: String,
        field_type: FieldType,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<serde_json::Value>,
    },
    
    /// Remove a field
    RemoveField {
        name: String,
    },
    
    /// Rename a field
    RenameField {
        from: String,
        to: String,
    },
    
    /// Change field type
    ChangeType {
        name: String,
        new_type: FieldType,
    },
    
    /// Make field required
    MakeRequired {
        name: String,
    },
    
    /// Make field optional
    MakeOptional {
        name: String,
    },
    
    /// Set allow_additional property
    SetAllowAdditional {
        value: bool,
    },
}

/// A migration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Migration version (sequential number)
    pub version: u32,
    
    /// Human-readable name
    pub name: String,
    
    /// When this migration was created
    pub created_at: DateTime<Utc>,
    
    /// App version that created this migration
    pub app_version: String,
    
    /// Operations to apply
    pub operations: Vec<MigrationOp>,
    
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Migration {
    /// Create a new migration
    pub fn new(version: u32, name: &str) -> Self {
        Self {
            version,
            name: name.to_string(),
            created_at: Utc::now(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            operations: Vec::new(),
            description: None,
        }
    }

    /// Add an operation
    pub fn add_op(mut self, op: MigrationOp) -> Self {
        self.operations.push(op);
        self
    }

    /// Save migration to migrations directory
    pub fn save(&self, migrations_dir: &Path) -> Result<()> {
        let filename = format!("{:03}_{}.json", self.version, self.name);
        let path = migrations_dir.join(filename);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load a migration from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

/// Migration runner
pub struct MigrationRunner {
    collection_path: std::path::PathBuf,
    migrations_dir: std::path::PathBuf,
}

impl MigrationRunner {
    /// Create a new migration runner for a collection
    pub fn new(collection_path: &Path) -> Self {
        Self {
            collection_path: collection_path.to_path_buf(),
            migrations_dir: collection_path.join("migrations"),
        }
    }

    /// Get all migrations in order
    pub fn list_migrations(&self) -> Result<Vec<Migration>> {
        if !self.migrations_dir.exists() {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();
        
        for entry in fs::read_dir(&self.migrations_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(migration) = Migration::load(&path) {
                    migrations.push(migration);
                }
            }
        }
        
        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }

    /// Get next migration version number
    pub fn next_version(&self) -> Result<u32> {
        let migrations = self.list_migrations()?;
        Ok(migrations.last().map(|m| m.version + 1).unwrap_or(1))
    }

    /// Create a new migration
    pub fn create_migration(&self, name: &str) -> Result<Migration> {
        fs::create_dir_all(&self.migrations_dir)?;
        
        let version = self.next_version()?;
        let migration = Migration::new(version, name);
        
        Ok(migration)
    }

    /// Apply all pending migrations to build current schema
    pub fn build_schema(&self) -> Result<Schema> {
        let migrations = self.list_migrations()?;
        
        // Get collection name from path
        let collection_name = self.collection_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let mut schema = Schema::new(&collection_name);
        
        for migration in migrations {
            schema = self.apply_migration(schema, &migration)?;
        }
        
        Ok(schema)
    }

    /// Apply a single migration to a schema
    fn apply_migration(&self, mut schema: Schema, migration: &Migration) -> Result<Schema> {
        for op in &migration.operations {
            match op {
                MigrationOp::AddField { name, field_type, required, default } => {
                    schema.fields.insert(name.clone(), FieldDef {
                        field_type: field_type.clone(),
                        required: *required,
                        default: default.clone(),
                        description: None,
                    });
                }
                
                MigrationOp::RemoveField { name } => {
                    schema.fields.remove(name);
                }
                
                MigrationOp::RenameField { from, to } => {
                    if let Some(field) = schema.fields.remove(from) {
                        schema.fields.insert(to.clone(), field);
                    }
                }
                
                MigrationOp::ChangeType { name, new_type } => {
                    if let Some(field) = schema.fields.get_mut(name) {
                        field.field_type = new_type.clone();
                    }
                }
                
                MigrationOp::MakeRequired { name } => {
                    if let Some(field) = schema.fields.get_mut(name) {
                        field.required = true;
                    }
                }
                
                MigrationOp::MakeOptional { name } => {
                    if let Some(field) = schema.fields.get_mut(name) {
                        field.required = false;
                    }
                }
                
                MigrationOp::SetAllowAdditional { value } => {
                    schema.allow_additional = *value;
                }
            }
        }
        
        schema.version = migration.version;
        Ok(schema)
    }

    /// Save current schema as versioned file
    pub fn save_schema(&self, schema: &Schema) -> Result<()> {
        schema.save(&self.collection_path)
    }

    /// Apply migrations and save schema
    pub fn run(&self) -> Result<Schema> {
        let schema = self.build_schema()?;
        self.save_schema(&schema)?;
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_migration_creation() {
        let dir = tempdir().unwrap();
        let migrations_dir = dir.path().join("migrations");
        fs::create_dir_all(&migrations_dir).unwrap();
        
        let runner = MigrationRunner::new(dir.path());
        
        let migration = runner.create_migration("add_email")
            .unwrap()
            .add_op(MigrationOp::AddField {
                name: "email".to_string(),
                field_type: FieldType::String,
                required: true,
                default: None,
            });
        
        migration.save(&migrations_dir).unwrap();
        
        let migrations = runner.list_migrations().unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].name, "add_email");
    }

    #[test]
    fn test_schema_building() {
        let dir = tempdir().unwrap();
        let migrations_dir = dir.path().join("migrations");
        fs::create_dir_all(&migrations_dir).unwrap();
        
        let runner = MigrationRunner::new(dir.path());
        
        // Create first migration
        let m1 = Migration::new(1, "init")
            .add_op(MigrationOp::AddField {
                name: "name".to_string(),
                field_type: FieldType::String,
                required: true,
                default: None,
            });
        m1.save(&migrations_dir).unwrap();
        
        // Create second migration
        let m2 = Migration::new(2, "add_age")
            .add_op(MigrationOp::AddField {
                name: "age".to_string(),
                field_type: FieldType::Number,
                required: false,
                default: None,
            });
        m2.save(&migrations_dir).unwrap();
        
        // Build schema
        let schema = runner.build_schema().unwrap();
        
        assert_eq!(schema.version, 2);
        assert!(schema.fields.contains_key("name"));
        assert!(schema.fields.contains_key("age"));
    }
}
