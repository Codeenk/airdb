//! Schema management for NoSQL collections
//! 
//! Schemas are versioned and migration-based

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

use super::error::{NoSqlError, Result};

/// Field type definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Any,
}

/// Field definition in schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    
    #[serde(default)]
    pub required: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema version (increments with each migration)
    pub version: u32,
    
    /// Collection name this schema belongs to
    pub collection: String,
    
    /// Field definitions
    pub fields: std::collections::HashMap<String, FieldDef>,
    
    /// Whether to allow additional fields not in schema
    #[serde(default = "default_true")]
    pub allow_additional: bool,
}

fn default_true() -> bool { true }

impl Schema {
    /// Create a new empty schema
    pub fn new(collection: &str) -> Self {
        Self {
            version: 1,
            collection: collection.to_string(),
            fields: std::collections::HashMap::new(),
            allow_additional: true,
        }
    }

    /// Load the latest schema version for a collection
    pub fn load_latest(collection_path: &Path) -> Result<Self> {
        // Find highest version schema file
        let mut latest_version = 0u32;
        let mut latest_path = None;
        
        for entry in fs::read_dir(collection_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with("_schema.v") && name_str.ends_with(".json") {
                // Parse version number
                if let Some(version_str) = name_str
                    .strip_prefix("_schema.v")
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    if let Ok(version) = version_str.parse::<u32>() {
                        if version > latest_version {
                            latest_version = version;
                            latest_path = Some(entry.path());
                        }
                    }
                }
            }
        }
        
        match latest_path {
            Some(path) => {
                let content = fs::read_to_string(path)?;
                Ok(serde_json::from_str(&content)?)
            }
            None => Err(NoSqlError::SchemaValidation("No schema found".to_string())),
        }
    }

    /// Save schema to collection directory
    pub fn save(&self, collection_path: &Path) -> Result<()> {
        let schema_path = collection_path.join(format!("_schema.v{}.json", self.version));
        let content = serde_json::to_string_pretty(self)?;
        fs::write(schema_path, content)?;
        Ok(())
    }

    /// Validate a document against this schema
    pub fn validate(&self, data: &Value) -> Result<()> {
        let obj = data.as_object().ok_or_else(|| {
            NoSqlError::SchemaValidation("Document must be an object".to_string())
        })?;
        
        // Check required fields
        for (field_name, field_def) in &self.fields {
            if field_def.required && !obj.contains_key(field_name) {
                return Err(NoSqlError::SchemaValidation(
                    format!("Missing required field: {}", field_name)
                ));
            }
            
            // Validate type if field exists
            if let Some(value) = obj.get(field_name) {
                self.validate_type(field_name, value, &field_def.field_type)?;
            }
        }
        
        // Check for unknown fields if not allowed
        if !self.allow_additional {
            for key in obj.keys() {
                if !self.fields.contains_key(key) && !key.starts_with('_') {
                    return Err(NoSqlError::SchemaValidation(
                        format!("Unknown field: {}", key)
                    ));
                }
            }
        }
        
        Ok(())
    }

    fn validate_type(&self, field: &str, value: &Value, expected: &FieldType) -> Result<()> {
        let valid = match expected {
            FieldType::String => value.is_string(),
            FieldType::Number => value.is_number(),
            FieldType::Boolean => value.is_boolean(),
            FieldType::Array => value.is_array(),
            FieldType::Object => value.is_object(),
            FieldType::Any => true,
        };
        
        if !valid {
            return Err(NoSqlError::SchemaValidation(
                format!("Field '{}' has wrong type, expected {:?}", field, expected)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validation() {
        let mut schema = Schema::new("users");
        schema.fields.insert("name".to_string(), FieldDef {
            field_type: FieldType::String,
            required: true,
            default: None,
            description: None,
        });
        schema.fields.insert("age".to_string(), FieldDef {
            field_type: FieldType::Number,
            required: false,
            default: None,
            description: None,
        });
        
        // Valid document
        assert!(schema.validate(&json!({"name": "Alice", "age": 30})).is_ok());
        
        // Missing required field
        assert!(schema.validate(&json!({"age": 30})).is_err());
        
        // Wrong type
        assert!(schema.validate(&json!({"name": "Alice", "age": "thirty"})).is_err());
    }
}
