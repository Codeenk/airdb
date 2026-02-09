//! Three-Way Merge for NoSQL Documents
//!
//! Conflict-safe merges with JSON diff and resolution

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Merge strategy options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    /// Accept all local changes
    Ours,
    /// Accept all remote changes
    Theirs,
    /// Prefer newer by timestamp
    Latest,
    /// Manual conflict resolution
    Manual,
    /// Auto-merge non-conflicting, mark conflicts
    Auto,
}

/// A single conflict in a merge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub path: String,
    pub base_value: Option<Value>,
    pub local_value: Option<Value>,
    pub remote_value: Option<Value>,
    pub resolution: Option<Value>,
}

/// Resolution for a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub path: String,
    pub resolved_value: Value,
}

/// Result of a three-way merge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub merged: Value,
    pub conflicts: Vec<Conflict>,
    pub auto_resolved_count: usize,
}

/// Three-way merge engine for JSON documents
pub struct ThreeWayMerge {
    strategy: MergeStrategy,
}

impl ThreeWayMerge {
    pub fn new(strategy: MergeStrategy) -> Self {
        Self { strategy }
    }

    /// Perform a three-way merge
    pub fn merge(&self, base: &Value, local: &Value, remote: &Value) -> MergeResult {
        let mut conflicts = Vec::new();
        let mut auto_resolved = 0;

        let merged = self.merge_value("", base, local, remote, &mut conflicts, &mut auto_resolved);

        MergeResult {
            merged,
            conflicts,
            auto_resolved_count: auto_resolved,
        }
    }

    fn merge_value(
        &self,
        path: &str,
        base: &Value,
        local: &Value,
        remote: &Value,
        conflicts: &mut Vec<Conflict>,
        auto_resolved: &mut usize,
    ) -> Value {
        // If local and remote are identical, no conflict
        if local == remote {
            return local.clone();
        }

        // If only local changed
        if base == remote && base != local {
            *auto_resolved += 1;
            return local.clone();
        }

        // If only remote changed
        if base == local && base != remote {
            *auto_resolved += 1;
            return remote.clone();
        }

        // Both changed - conflict based on type
        match (base, local, remote) {
            (Value::Object(b), Value::Object(l), Value::Object(r)) => {
                Value::Object(self.merge_objects(path, b, l, r, conflicts, auto_resolved))
            }
            (Value::Array(b), Value::Array(l), Value::Array(r)) => {
                Value::Array(self.merge_arrays(path, b, l, r, conflicts, auto_resolved))
            }
            _ => {
                // Scalar conflict
                self.resolve_conflict(path, base, local, remote, conflicts)
            }
        }
    }

    fn merge_objects(
        &self,
        path: &str,
        base: &Map<String, Value>,
        local: &Map<String, Value>,
        remote: &Map<String, Value>,
        conflicts: &mut Vec<Conflict>,
        auto_resolved: &mut usize,
    ) -> Map<String, Value> {
        let mut result = Map::new();
        
        // Get all keys from all three
        let mut all_keys: Vec<&String> = base.keys()
            .chain(local.keys())
            .chain(remote.keys())
            .collect();
        all_keys.sort();
        all_keys.dedup();

        for key in all_keys {
            let child_path = if path.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", path, key)
            };

            let b = base.get(key).unwrap_or(&Value::Null);
            let l = local.get(key).unwrap_or(&Value::Null);
            let r = remote.get(key).unwrap_or(&Value::Null);

            let merged = self.merge_value(&child_path, b, l, r, conflicts, auto_resolved);
            
            if merged != Value::Null {
                result.insert(key.clone(), merged);
            }
        }

        result
    }

    fn merge_arrays(
        &self,
        path: &str,
        base: &[Value],
        local: &[Value],
        remote: &[Value],
        conflicts: &mut Vec<Conflict>,
        auto_resolved: &mut usize,
    ) -> Vec<Value> {
        // Simple array merge: use local if only local changed, remote if only remote changed
        if base == remote {
            *auto_resolved += 1;
            return local.to_vec();
        }
        if base == local {
            *auto_resolved += 1;
            return remote.to_vec();
        }

        // Both changed - strategy-based resolution
        match self.strategy {
            MergeStrategy::Ours => local.to_vec(),
            MergeStrategy::Theirs => remote.to_vec(),
            _ => {
                // Mark as conflict
                conflicts.push(Conflict {
                    path: path.to_string(),
                    base_value: Some(Value::Array(base.to_vec())),
                    local_value: Some(Value::Array(local.to_vec())),
                    remote_value: Some(Value::Array(remote.to_vec())),
                    resolution: None,
                });
                local.to_vec() // Default to local
            }
        }
    }

    fn resolve_conflict(
        &self,
        path: &str,
        base: &Value,
        local: &Value,
        remote: &Value,
        conflicts: &mut Vec<Conflict>,
    ) -> Value {
        match self.strategy {
            MergeStrategy::Ours => local.clone(),
            MergeStrategy::Theirs => remote.clone(),
            MergeStrategy::Latest => {
                // Can't determine latest without timestamps, default to local
                local.clone()
            }
            MergeStrategy::Auto | MergeStrategy::Manual => {
                conflicts.push(Conflict {
                    path: path.to_string(),
                    base_value: Some(base.clone()),
                    local_value: Some(local.clone()),
                    remote_value: Some(remote.clone()),
                    resolution: None,
                });
                local.clone() // Placeholder
            }
        }
    }

    /// Apply resolutions to a merge result
    pub fn apply_resolutions(result: &mut MergeResult, resolutions: &[ConflictResolution]) {
        for resolution in resolutions {
            // Find and update the conflict
            if let Some(conflict) = result.conflicts.iter_mut().find(|c| c.path == resolution.path) {
                conflict.resolution = Some(resolution.resolved_value.clone());
            }
            
            // Apply to merged value
            set_value_at_path(&mut result.merged, &resolution.path, resolution.resolved_value.clone());
        }
    }
}

/// Set a value at a dot-separated path
fn set_value_at_path(root: &mut Value, path: &str, value: Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Value::Object(map) = current {
                map.insert(part.to_string(), value);
                return;
            }
        } else if let Value::Object(map) = current {
            current = map.entry(part.to_string()).or_insert(Value::Object(Map::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_conflict() {
        let merge = ThreeWayMerge::new(MergeStrategy::Auto);
        
        let base = json!({"a": 1, "b": 2});
        let local = json!({"a": 1, "b": 2});
        let remote = json!({"a": 1, "b": 2});
        
        let result = merge.merge(&base, &local, &remote);
        assert_eq!(result.merged, base);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_local_only_change() {
        let merge = ThreeWayMerge::new(MergeStrategy::Auto);
        
        let base = json!({"a": 1});
        let local = json!({"a": 2});
        let remote = json!({"a": 1});
        
        let result = merge.merge(&base, &local, &remote);
        assert_eq!(result.merged, json!({"a": 2}));
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_conflict() {
        let merge = ThreeWayMerge::new(MergeStrategy::Auto);
        
        let base = json!({"a": 1});
        let local = json!({"a": 2});
        let remote = json!({"a": 3});
        
        let result = merge.merge(&base, &local, &remote);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].path, "a");
    }
}
