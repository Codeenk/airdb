//! Document representation for NoSQL storage

use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};

/// A NoSQL document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document ID (ULID format)
    #[serde(rename = "_id")]
    pub id: String,
    
    /// Schema version this document was created with
    #[serde(rename = "_schema_version", skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    
    /// Creation timestamp
    #[serde(rename = "_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Last modified timestamp
    #[serde(rename = "_modified_at")]
    pub modified_at: DateTime<Utc>,
    
    /// The actual document data
    #[serde(flatten)]
    pub data: Value,
}

impl Document {
    /// Create a new document with generated ID
    pub fn new(data: Value) -> Self {
        let now = Utc::now();
        Self {
            id: generate_ulid(),
            schema_version: None,
            created_at: now,
            modified_at: now,
            data,
        }
    }

    /// Create a document with a specific ID
    pub fn with_id(id: String, data: Value) -> Self {
        let now = Utc::now();
        Self {
            id,
            schema_version: None,
            created_at: now,
            modified_at: now,
            data,
        }
    }

    /// Update the document data
    pub fn update(&mut self, data: Value) {
        self.data = data;
        self.modified_at = Utc::now();
    }

    /// Get a field from the document
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.data.get(field)
    }

    /// Set a field in the document
    pub fn set(&mut self, field: &str, value: Value) {
        if let Value::Object(ref mut map) = self.data {
            map.insert(field.to_string(), value);
            self.modified_at = Utc::now();
        }
    }
}

/// Generate a ULID (Universally Unique Lexicographically Sortable Identifier)
fn generate_ulid() -> String {
    // Simplified ULID: timestamp + random
    // Format: TTTTTTTTTTRRRRRRRRRRRRRRR (26 chars)
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let random: u64 = rand_simple();
    
    // Crockford's base32 encoding
    let chars: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let mut result = String::with_capacity(26);
    
    // Encode timestamp (10 chars)
    let mut ts = timestamp;
    for _ in 0..10 {
        result.push(chars[(ts & 0x1F) as usize] as char);
        ts >>= 5;
    }
    result = result.chars().rev().collect();
    
    // Encode random (16 chars)
    let mut rnd = random;
    let mut random_part = String::with_capacity(16);
    for _ in 0..16 {
        random_part.push(chars[(rnd & 0x1F) as usize] as char);
        rnd >>= 5;
    }
    result.push_str(&random_part.chars().rev().collect::<String>());
    
    result
}

/// Simple random number generator (no external deps)
fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_creation() {
        let doc = Document::new(json!({"name": "test", "value": 42}));
        assert!(!doc.id.is_empty());
        assert_eq!(doc.get("name"), Some(&json!("test")));
    }

    #[test]
    fn test_ulid_uniqueness() {
        let id1 = generate_ulid();
        let id2 = generate_ulid();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 26);
    }
}
