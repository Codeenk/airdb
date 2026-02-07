//! Hybrid SQL/NoSQL Relations
//!
//! Cross-engine relation definitions with version constraints

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Relation type between entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RelationType {
    /// One-to-one relationship
    OneToOne,
    /// One-to-many relationship
    OneToMany,
    /// Many-to-one relationship
    ManyToOne,
    /// Many-to-many relationship
    ManyToMany,
}

/// Source engine type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    Sql,
    Nosql,
}

/// A field reference (engine.collection.field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRef {
    pub engine: EngineType,
    pub collection: String,
    pub field: String,
}

impl FieldRef {
    pub fn sql(table: &str, column: &str) -> Self {
        Self {
            engine: EngineType::Sql,
            collection: table.to_string(),
            field: column.to_string(),
        }
    }

    pub fn nosql(collection: &str, field: &str) -> Self {
        Self {
            engine: EngineType::Nosql,
            collection: collection.to_string(),
            field: field.to_string(),
        }
    }

    /// Parse from string format: "sql.users.id" or "nosql.sessions.user_id"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let engine = match parts[0] {
            "sql" => EngineType::Sql,
            "nosql" => EngineType::Nosql,
            _ => return None,
        };

        Some(Self {
            engine,
            collection: parts[1].to_string(),
            field: parts[2].to_string(),
        })
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        let engine = match self.engine {
            EngineType::Sql => "sql",
            EngineType::Nosql => "nosql",
        };
        format!("{}.{}.{}", engine, self.collection, self.field)
    }
}

/// Cascade behavior on delete/update
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CascadeBehavior {
    /// No action (default)
    #[default]
    None,
    /// Cascade delete/update to related records
    Cascade,
    /// Set to null
    SetNull,
    /// Restrict operation
    Restrict,
}

/// A cross-engine relation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Unique name for this relation
    pub name: String,

    /// Source field reference
    pub source: FieldRef,

    /// Target field reference
    pub target: FieldRef,

    /// Relation type
    #[serde(rename = "type")]
    pub relation_type: RelationType,

    /// App version that introduced this relation
    pub introduced_in: String,

    /// On delete behavior
    #[serde(default)]
    pub on_delete: CascadeBehavior,

    /// On update behavior
    #[serde(default)]
    pub on_update: CascadeBehavior,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Relation {
    /// Create a new relation
    pub fn new(name: &str, source: FieldRef, target: FieldRef, rel_type: RelationType) -> Self {
        Self {
            name: name.to_string(),
            source,
            target,
            relation_type: rel_type,
            introduced_in: env!("CARGO_PKG_VERSION").to_string(),
            on_delete: CascadeBehavior::None,
            on_update: CascadeBehavior::None,
            description: None,
        }
    }

    /// Check if this relation is compatible with the given app version
    pub fn is_compatible(&self, app_version: &str) -> bool {
        // Simple version comparison (for production, use semver crate)
        app_version >= self.introduced_in.as_str()
    }
}

/// Relations manifest file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationsManifest {
    /// Format version
    pub version: u32,

    /// All defined relations
    pub relations: Vec<Relation>,
}

impl RelationsManifest {
    pub fn new() -> Self {
        Self {
            version: 1,
            relations: Vec::new(),
        }
    }

    /// Load from project directory
    pub fn load(project_dir: &Path) -> std::io::Result<Self> {
        let path = project_dir.join("relations.json");
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    /// Save to project directory
    pub fn save(&self, project_dir: &Path) -> std::io::Result<()> {
        let path = project_dir.join("relations.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Add a relation
    pub fn add(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    /// Find relation by name
    pub fn find(&self, name: &str) -> Option<&Relation> {
        self.relations.iter().find(|r| r.name == name)
    }

    /// Get relations involving a specific collection
    pub fn for_collection(&self, engine: &EngineType, collection: &str) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| {
                (r.source.engine == *engine && r.source.collection == collection)
                    || (r.target.engine == *engine && r.target.collection == collection)
            })
            .collect()
    }

    /// Get compatible relations for an app version
    pub fn compatible(&self, app_version: &str) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| r.is_compatible(app_version))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_ref_parse() {
        let ref1 = FieldRef::parse("sql.users.id").unwrap();
        assert_eq!(ref1.engine, EngineType::Sql);
        assert_eq!(ref1.collection, "users");
        assert_eq!(ref1.field, "id");

        let ref2 = FieldRef::parse("nosql.sessions.user_id").unwrap();
        assert_eq!(ref2.engine, EngineType::Nosql);
    }

    #[test]
    fn test_relation_creation() {
        let rel = Relation::new(
            "user_sessions",
            FieldRef::nosql("sessions", "user_id"),
            FieldRef::sql("users", "id"),
            RelationType::ManyToOne,
        );

        assert_eq!(rel.name, "user_sessions");
        assert_eq!(rel.source.collection, "sessions");
        assert_eq!(rel.target.collection, "users");
    }
}
