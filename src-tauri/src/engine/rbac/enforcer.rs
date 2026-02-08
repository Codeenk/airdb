//! RBAC Enforcement Layer
//!
//! Request-level, row-level, and field-level access control

use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::policy::{Policy, Permission, ResourceType};

/// Authorization request context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub api_key_id: Option<String>,
}

impl AuthContext {
    /// Create an anonymous context
    pub fn anonymous() -> Self {
        Self {
            user_id: None,
            roles: vec![],
            api_key_id: None,
        }
    }

    /// Create a context with roles
    pub fn with_roles(user_id: Option<String>, roles: Vec<String>) -> Self {
        Self {
            user_id,
            roles,
            api_key_id: None,
        }
    }

    /// Create a context from API key
    pub fn from_api_key(key_id: String, roles: Vec<String>) -> Self {
        Self {
            user_id: None,
            roles,
            api_key_id: Some(key_id),
        }
    }
}

/// Authorization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub filtered_fields: Vec<String>,
}

impl AuthResult {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            reason: None,
            filtered_fields: vec![],
        }
    }

    pub fn deny(reason: &str) -> Self {
        Self {
            allowed: false,
            reason: Some(reason.to_string()),
            filtered_fields: vec![],
        }
    }

    pub fn allow_with_filter(fields: Vec<String>) -> Self {
        Self {
            allowed: true,
            reason: None,
            filtered_fields: fields,
        }
    }
}

/// The enforcement engine
pub struct Enforcer<'a> {
    policy: &'a Policy,
}

impl<'a> Enforcer<'a> {
    pub fn new(policy: &'a Policy) -> Self {
        Self { policy }
    }

    /// Check request-level authorization
    pub fn authorize_request(
        &self,
        context: &AuthContext,
        resource_type: ResourceType,
        resource_name: &str,
        permission: Permission,
    ) -> AuthResult {
        // Use default role if no roles specified
        let roles_to_check: Vec<&str> = if context.roles.is_empty() {
            self.policy.default_role.as_deref().map(|r| vec![r]).unwrap_or_default()
        } else {
            context.roles.iter().map(|s| s.as_str()).collect()
        };

        // Check each role
        for role in roles_to_check {
            if self.policy.check_permission(role, resource_type.clone(), resource_name, permission) {
                return AuthResult::allow();
            }
        }

        AuthResult::deny("Insufficient permissions")
    }

    /// Filter document fields based on role
    pub fn filter_fields(
        &self,
        context: &AuthContext,
        _resource_type: ResourceType,
        resource_name: &str,
        document: &Value,
    ) -> (Value, Vec<String>) {
        let mut filtered_fields = Vec::new();
        
        if let Value::Object(obj) = document {
            let mut result = serde_json::Map::new();
            
            for (key, value) in obj {
                let field_resource = format!("{}.{}", resource_name, key);
                
                let can_read = context.roles.iter().any(|role| {
                    self.policy.check_permission(
                        role,
                        ResourceType::Field,
                        &field_resource,
                        Permission::Read,
                    )
                }) || !self.has_field_rules(resource_name);

                if can_read {
                    result.insert(key.clone(), value.clone());
                } else {
                    filtered_fields.push(key.clone());
                }
            }
            
            return (Value::Object(result), filtered_fields);
        }
        
        (document.clone(), filtered_fields)
    }

    /// Apply row-level security filter
    pub fn row_filter_expression(&self, context: &AuthContext, resource_name: &str) -> Option<String> {
        for rule in &self.policy.resource_rules {
            if rule.resource_pattern == resource_name || rule.resource_pattern == "*" {
                if context.roles.iter().any(|r| rule.allowed_roles.contains(r)) {
                    return rule.row_filter.clone();
                }
            }
        }
        None
    }

    /// Check if resource has field-level rules
    fn has_field_rules(&self, resource_name: &str) -> bool {
        self.policy.resource_rules.iter().any(|rule| {
            (rule.resource_pattern == resource_name || rule.resource_pattern == "*") 
                && !rule.field_rules.is_empty()
        })
    }

    /// Check write permission for specific fields
    pub fn authorize_field_write(
        &self,
        context: &AuthContext,
        resource_name: &str,
        fields: &[String],
    ) -> AuthResult {
        for field in fields {
            let field_resource = format!("{}.{}", resource_name, field);
            
            let can_write = context.roles.iter().any(|role| {
                self.policy.check_permission(
                    role,
                    ResourceType::Field,
                    &field_resource,
                    Permission::Write,
                )
            }) || !self.has_field_rules(resource_name);

            if !can_write {
                return AuthResult::deny(&format!("Cannot write to field: {}", field));
            }
        }
        
        AuthResult::allow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymous_context() {
        let ctx = AuthContext::anonymous();
        assert!(ctx.user_id.is_none());
        assert!(ctx.roles.is_empty());
    }

    #[test]
    fn test_authorization() {
        let policy = Policy::new("test");
        let enforcer = Enforcer::new(&policy);
        
        let admin_ctx = AuthContext::with_roles(Some("admin-user".to_string()), vec!["admin".to_string()]);
        let reader_ctx = AuthContext::with_roles(Some("reader-user".to_string()), vec!["reader".to_string()]);
        
        assert!(enforcer.authorize_request(&admin_ctx, ResourceType::Collection, "users", Permission::Write).allowed);
        assert!(!enforcer.authorize_request(&reader_ctx, ResourceType::Collection, "users", Permission::Write).allowed);
        assert!(enforcer.authorize_request(&reader_ctx, ResourceType::Collection, "users", Permission::Read).allowed);
    }
}
