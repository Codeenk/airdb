//! Role-Based Access Control (RBAC) Policy Engine
//!
//! Version-safe policy format with role definitions and permission matrix

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Current policy schema version
pub const POLICY_VERSION: u32 = 1;

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    Execute,
}

/// Resource types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Collection,
    Table,
    Document,
    Row,
    Field,
    Api,
    Relation,
    Schema,
}

/// A role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: Vec<RolePermission>,
    pub inherits: Vec<String>,
}

/// Permission attached to a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    pub resource_type: ResourceType,
    #[serde(default)]
    pub resource_pattern: Option<String>,
    pub permissions: Vec<Permission>,
    #[serde(default)]
    pub conditions: HashMap<String, serde_json::Value>,
}

/// Resource-level rule for fine-grained access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRule {
    pub resource_type: ResourceType,
    pub resource_pattern: String,
    pub allowed_roles: Vec<String>,
    pub denied_roles: Vec<String>,
    #[serde(default)]
    pub field_rules: Vec<FieldRule>,
    #[serde(default)]
    pub row_filter: Option<String>,
}

/// Field-level access rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRule {
    pub field: String,
    pub readable_by: Vec<String>,
    pub writable_by: Vec<String>,
}

/// The main policy document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Schema version for compatibility
    pub version: u32,
    
    /// Policy name
    pub name: String,
    
    /// Role definitions
    pub roles: Vec<Role>,
    
    /// Resource-level rules
    pub resource_rules: Vec<ResourceRule>,
    
    /// Default role for unauthenticated access
    #[serde(default)]
    pub default_role: Option<String>,
    
    /// Whether to deny by default
    #[serde(default = "default_true")]
    pub deny_by_default: bool,
}

fn default_true() -> bool { true }

impl Policy {
    /// Create a new empty policy
    pub fn new(name: &str) -> Self {
        Self {
            version: POLICY_VERSION,
            name: name.to_string(),
            roles: vec![
                Role {
                    name: "admin".to_string(),
                    description: "Full system access".to_string(),
                    permissions: vec![RolePermission {
                        resource_type: ResourceType::Collection,
                        resource_pattern: Some("*".to_string()),
                        permissions: vec![Permission::Read, Permission::Write, Permission::Delete, Permission::Admin],
                        conditions: HashMap::new(),
                    }],
                    inherits: vec![],
                },
                Role {
                    name: "reader".to_string(),
                    description: "Read-only access".to_string(),
                    permissions: vec![RolePermission {
                        resource_type: ResourceType::Collection,
                        resource_pattern: Some("*".to_string()),
                        permissions: vec![Permission::Read],
                        conditions: HashMap::new(),
                    }],
                    inherits: vec![],
                },
            ],
            resource_rules: vec![],
            default_role: None,
            deny_by_default: true,
        }
    }

    /// Load policy from file
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let policy_path = path.join("policy.json");
        
        if !policy_path.exists() {
            return Ok(Self::new("default"));
        }
        
        let content = fs::read_to_string(&policy_path)?;
        let policy: Policy = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        // Version compatibility check
        if policy.version > POLICY_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Policy version {} is newer than supported {}", policy.version, POLICY_VERSION)
            ));
        }
        
        Ok(policy)
    }

    /// Save policy to file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let policy_path = path.join("policy.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&policy_path, content)
    }

    /// Add a role
    pub fn add_role(&mut self, role: Role) {
        self.roles.retain(|r| r.name != role.name);
        self.roles.push(role);
    }

    /// Get a role by name (including inherited permissions)
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.iter().find(|r| r.name == name)
    }

    /// Add a resource rule
    pub fn add_resource_rule(&mut self, rule: ResourceRule) {
        self.resource_rules.push(rule);
    }

    /// Check if a role has permission on a resource
    pub fn check_permission(
        &self,
        role_name: &str,
        resource_type: ResourceType,
        resource_name: &str,
        permission: Permission,
    ) -> bool {
        if let Some(role) = self.get_role(role_name) {
            // Check direct permissions
            for perm in &role.permissions {
                if perm.resource_type == resource_type {
                    if let Some(pattern) = &perm.resource_pattern {
                        if matches_pattern(pattern, resource_name) && perm.permissions.contains(&permission) {
                            return true;
                        }
                    } else if perm.permissions.contains(&permission) {
                        return true;
                    }
                }
            }

            // Check inherited roles
            for inherited in &role.inherits {
                if self.check_permission(inherited, resource_type.clone(), resource_name, permission) {
                    return true;
                }
            }
        }

        // Check resource rules
        for rule in &self.resource_rules {
            if rule.resource_type == resource_type && matches_pattern(&rule.resource_pattern, resource_name) {
                if rule.denied_roles.contains(&role_name.to_string()) {
                    return false;
                }
                if rule.allowed_roles.contains(&role_name.to_string()) {
                    return true;
                }
            }
        }

        !self.deny_by_default
    }
}

/// Simple pattern matching (supports * wildcard)
fn matches_pattern(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }
    
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return name.ends_with(suffix);
    }
    
    pattern == name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new("test");
        assert_eq!(policy.version, POLICY_VERSION);
        assert_eq!(policy.roles.len(), 2);
    }

    #[test]
    fn test_permission_check() {
        let policy = Policy::new("test");
        
        assert!(policy.check_permission("admin", ResourceType::Collection, "users", Permission::Write));
        assert!(policy.check_permission("reader", ResourceType::Collection, "users", Permission::Read));
        assert!(!policy.check_permission("reader", ResourceType::Collection, "users", Permission::Write));
    }

    #[test]
    fn test_pattern_matching() {
        assert!(matches_pattern("*", "anything"));
        assert!(matches_pattern("users*", "users_archive"));
        assert!(!matches_pattern("users*", "documents"));
        assert!(matches_pattern("*_log", "audit_log"));
    }
}
