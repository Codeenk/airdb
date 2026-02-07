//! Query engine for NoSQL documents

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::document::Document;

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    /// Equality
    Eq(Value),
    /// Not equal
    Ne(Value),
    /// Greater than
    Gt(Value),
    /// Greater than or equal
    Gte(Value),
    /// Less than
    Lt(Value),
    /// Less than or equal
    Lte(Value),
    /// String contains
    Contains(String),
    /// String starts with
    StartsWith(String),
    /// String ends with
    EndsWith(String),
    /// In array of values
    In(Vec<Value>),
    /// Not in array of values
    NotIn(Vec<Value>),
    /// Field exists
    Exists(bool),
}

/// A single filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub op: FilterOp,
}

impl Filter {
    pub fn eq(field: &str, value: impl Into<Value>) -> Self {
        Self { field: field.to_string(), op: FilterOp::Eq(value.into()) }
    }

    pub fn ne(field: &str, value: impl Into<Value>) -> Self {
        Self { field: field.to_string(), op: FilterOp::Ne(value.into()) }
    }

    pub fn contains(field: &str, value: &str) -> Self {
        Self { field: field.to_string(), op: FilterOp::Contains(value.to_string()) }
    }

    /// Check if a document matches this filter
    pub fn matches(&self, doc: &Document) -> bool {
        let value = match self.field.as_str() {
            "_id" => Some(&Value::String(doc.id.clone())),
            _ => doc.data.get(&self.field),
        };

        match (&self.op, value) {
            (FilterOp::Exists(should_exist), val) => {
                val.is_some() == *should_exist
            }
            (_, None) => false,
            (FilterOp::Eq(expected), Some(actual)) => actual == expected,
            (FilterOp::Ne(expected), Some(actual)) => actual != expected,
            (FilterOp::Gt(expected), Some(actual)) => {
                compare_values(actual, expected) == Some(std::cmp::Ordering::Greater)
            }
            (FilterOp::Gte(expected), Some(actual)) => {
                matches!(compare_values(actual, expected), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal))
            }
            (FilterOp::Lt(expected), Some(actual)) => {
                compare_values(actual, expected) == Some(std::cmp::Ordering::Less)
            }
            (FilterOp::Lte(expected), Some(actual)) => {
                matches!(compare_values(actual, expected), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal))
            }
            (FilterOp::Contains(substr), Some(Value::String(s))) => {
                s.contains(substr)
            }
            (FilterOp::StartsWith(prefix), Some(Value::String(s))) => {
                s.starts_with(prefix)
            }
            (FilterOp::EndsWith(suffix), Some(Value::String(s))) => {
                s.ends_with(suffix)
            }
            (FilterOp::In(values), Some(actual)) => {
                values.contains(actual)
            }
            (FilterOp::NotIn(values), Some(actual)) => {
                !values.contains(actual)
            }
            _ => false,
        }
    }
}

/// Compare two JSON values
fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let a = a.as_f64()?;
            let b = b.as_f64()?;
            a.partial_cmp(&b)
        }
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

/// Query with multiple filters and options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Query {
    /// AirQL version for compatibility
    #[serde(default = "default_version")]
    pub airql_version: u32,
    
    /// All filters must match (AND)
    #[serde(default)]
    pub filters: Vec<Filter>,
    
    /// Sort by field (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    
    /// Sort descending
    #[serde(default)]
    pub sort_desc: bool,
    
    /// Limit results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    
    /// Skip results (for pagination)
    #[serde(default)]
    pub skip: usize,
}

fn default_version() -> u32 { 1 }

impl Query {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn sort(mut self, field: &str, desc: bool) -> Self {
        self.sort_by = Some(field.to_string());
        self.sort_desc = desc;
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn skip(mut self, n: usize) -> Self {
        self.skip = n;
        self
    }

    /// Execute query on a list of documents
    pub fn execute(&self, docs: Vec<Document>) -> Vec<Document> {
        let mut results: Vec<Document> = docs
            .into_iter()
            .filter(|doc| self.matches(doc))
            .collect();

        // Sort
        if let Some(ref field) = self.sort_by {
            results.sort_by(|a, b| {
                let a_val = a.data.get(field);
                let b_val = b.data.get(field);
                
                let ordering = match (a_val, b_val) {
                    (Some(a), Some(b)) => compare_values(a, b).unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                };
                
                if self.sort_desc { ordering.reverse() } else { ordering }
            });
        }

        // Skip and limit
        let results: Vec<Document> = results.into_iter().skip(self.skip).collect();
        match self.limit {
            Some(n) => results.into_iter().take(n).collect(),
            None => results,
        }
    }

    fn matches(&self, doc: &Document) -> bool {
        self.filters.iter().all(|f| f.matches(doc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_eq() {
        let doc = Document::new(json!({"name": "Alice", "age": 30}));
        
        assert!(Filter::eq("name", "Alice").matches(&doc));
        assert!(!Filter::eq("name", "Bob").matches(&doc));
        assert!(Filter::eq("age", 30).matches(&doc));
    }

    #[test]
    fn test_query_execution() {
        let docs = vec![
            Document::new(json!({"name": "Alice", "age": 30})),
            Document::new(json!({"name": "Bob", "age": 25})),
            Document::new(json!({"name": "Charlie", "age": 35})),
        ];

        let results = Query::new()
            .filter(Filter { field: "age".to_string(), op: FilterOp::Gte(json!(30)) })
            .execute(docs);

        assert_eq!(results.len(), 2);
    }
}
