//! AirQL - Unified Query Language
//!
//! Version-aware query language for both SQL and NoSQL

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::relations::EngineType;

/// AirQL version
pub const AIRQL_VERSION: u32 = 1;

/// Filter operator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

/// A single filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirFilter {
    pub field: String,
    pub op: FilterOp,
    pub value: Value,
}

impl AirFilter {
    pub fn eq(field: &str, value: impl Into<Value>) -> Self {
        Self {
            field: field.to_string(),
            op: FilterOp::Eq,
            value: value.into(),
        }
    }

    pub fn contains(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            op: FilterOp::Contains,
            value: Value::String(value.to_string()),
        }
    }

    /// Convert to SQL WHERE clause fragment
    pub fn to_sql(&self) -> (String, Value) {
        let op_str = match self.op {
            FilterOp::Eq => "=",
            FilterOp::Ne => "!=",
            FilterOp::Gt => ">",
            FilterOp::Gte => ">=",
            FilterOp::Lt => "<",
            FilterOp::Lte => "<=",
            FilterOp::Contains => "LIKE",
            FilterOp::StartsWith => "LIKE",
            FilterOp::EndsWith => "LIKE",
            FilterOp::In => "IN",
            FilterOp::NotIn => "NOT IN",
            FilterOp::IsNull => "IS NULL",
            FilterOp::IsNotNull => "IS NOT NULL",
        };

        let value = match self.op {
            FilterOp::Contains => Value::String(format!("%{}%", self.value.as_str().unwrap_or(""))),
            FilterOp::StartsWith => Value::String(format!("{}%", self.value.as_str().unwrap_or(""))),
            FilterOp::EndsWith => Value::String(format!("%{}", self.value.as_str().unwrap_or(""))),
            _ => self.value.clone(),
        };

        (format!("{} {} ?", self.field, op_str), value)
    }
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    #[serde(default)]
    pub dir: SortDir,
}

/// An AirQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirQuery {
    /// AirQL version for compatibility
    #[serde(default = "default_version")]
    pub airql_version: u32,

    /// Target engine
    pub engine: EngineType,

    /// Target collection/table
    pub from: String,

    /// Fields to select (empty = all)
    #[serde(default)]
    pub select: Vec<String>,

    /// Filter conditions (AND)
    #[serde(default)]
    pub filters: Vec<AirFilter>,

    /// Sort specifications
    #[serde(default)]
    pub sort: Vec<SortSpec>,

    /// Limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Offset
    #[serde(default)]
    pub offset: usize,

    /// Include related data
    #[serde(default)]
    pub include: Vec<String>,
}

fn default_version() -> u32 {
    AIRQL_VERSION
}

impl AirQuery {
    /// Create a new SQL query
    pub fn sql(table: &str) -> Self {
        Self {
            airql_version: AIRQL_VERSION,
            engine: EngineType::Sql,
            from: table.to_string(),
            select: Vec::new(),
            filters: Vec::new(),
            sort: Vec::new(),
            limit: None,
            offset: 0,
            include: Vec::new(),
        }
    }

    /// Create a new NoSQL query
    pub fn nosql(collection: &str) -> Self {
        Self {
            airql_version: AIRQL_VERSION,
            engine: EngineType::Nosql,
            from: collection.to_string(),
            select: Vec::new(),
            filters: Vec::new(),
            sort: Vec::new(),
            limit: None,
            offset: 0,
            include: Vec::new(),
        }
    }

    /// Add a filter
    pub fn filter(mut self, f: AirFilter) -> Self {
        self.filters.push(f);
        self
    }

    /// Add sort
    pub fn sort_by(mut self, field: &str, desc: bool) -> Self {
        self.sort.push(SortSpec {
            field: field.to_string(),
            dir: if desc { SortDir::Desc } else { SortDir::Asc },
        });
        self
    }

    /// Set limit
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Set offset
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = n;
        self
    }

    /// Include related data
    pub fn include(mut self, relation: &str) -> Self {
        self.include.push(relation.to_string());
        self
    }

    /// Check if version is compatible
    pub fn is_compatible(&self) -> bool {
        self.airql_version <= AIRQL_VERSION
    }

    /// Generate SQL query string
    pub fn to_sql(&self) -> String {
        let select = if self.select.is_empty() {
            "*".to_string()
        } else {
            self.select.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", select, self.from);

        if !self.filters.is_empty() {
            let conditions: Vec<String> = self.filters.iter().map(|f| f.to_sql().0).collect();
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        if !self.sort.is_empty() {
            let orders: Vec<String> = self
                .sort
                .iter()
                .map(|s| {
                    format!(
                        "{} {}",
                        s.field,
                        match s.dir {
                            SortDir::Asc => "ASC",
                            SortDir::Desc => "DESC",
                        }
                    )
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", orders.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if self.offset > 0 {
            sql.push_str(&format!(" OFFSET {}", self.offset));
        }

        sql
    }
}

/// Query result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirResult {
    /// Query version
    pub airql_version: u32,

    /// Source engine
    pub engine: EngineType,

    /// Source collection/table
    pub from: String,

    /// Result count
    pub count: usize,

    /// Result data
    pub data: Vec<Value>,

    /// Related data (by relation name)
    #[serde(default)]
    pub included: std::collections::HashMap<String, Vec<Value>>,
}

impl AirResult {
    pub fn new(engine: EngineType, from: &str, data: Vec<Value>) -> Self {
        Self {
            airql_version: AIRQL_VERSION,
            engine,
            from: from.to_string(),
            count: data.len(),
            data,
            included: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airql_sql_generation() {
        let query = AirQuery::sql("users")
            .filter(AirFilter::eq("status", "active"))
            .sort_by("created_at", true)
            .limit(10);

        let sql = query.to_sql();
        assert!(sql.contains("SELECT * FROM users"));
        assert!(sql.contains("status = ?"));
        assert!(sql.contains("ORDER BY created_at DESC"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_airql_nosql() {
        let query = AirQuery::nosql("posts")
            .filter(AirFilter::contains("title", "rust"))
            .limit(5);

        assert_eq!(query.engine, EngineType::Nosql);
        assert_eq!(query.from, "posts");
    }
}
