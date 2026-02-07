//! Hybrid Query Executor
//!
//! Executes AirQL queries with cross-engine relation resolution

use serde_json::Value;
use std::path::Path;

use crate::engine::nosql::{NoSqlEngine, Query as NsQuery, Filter as NsFilter, Document};
use super::airql::{AirQuery, AirResult, AirFilter, AIRQL_VERSION};
use super::relations::{RelationsManifest, EngineType, Relation};

/// Query executor for hybrid operations
pub struct HybridExecutor<'a> {
    project_dir: &'a Path,
    relations: RelationsManifest,
}

impl<'a> HybridExecutor<'a> {
    /// Create a new executor
    pub fn new(project_dir: &'a Path) -> std::io::Result<Self> {
        let relations = RelationsManifest::load(project_dir)?;
        Ok(Self {
            project_dir,
            relations,
        })
    }

    /// Execute an AirQL query
    pub fn execute(&self, query: &AirQuery) -> Result<AirResult, Box<dyn std::error::Error>> {
        // Version compatibility check
        if query.airql_version > AIRQL_VERSION {
            return Err(format!(
                "Query requires AirQL v{}, but engine supports v{}",
                query.airql_version, AIRQL_VERSION
            ).into());
        }

        let mut result = match query.engine {
            EngineType::Nosql => self.execute_nosql(query)?,
            EngineType::Sql => self.execute_sql(query)?,
        };

        // Resolve includes (related data)
        for include_name in &query.include {
            if let Some(relation) = self.relations.find(include_name) {
                let included = self.resolve_include(&result, relation)?;
                result.included.insert(include_name.clone(), included);
            }
        }

        Ok(result)
    }

    /// Execute NoSQL query
    fn execute_nosql(&self, query: &AirQuery) -> Result<AirResult, Box<dyn std::error::Error>> {
        let engine = NoSqlEngine::open(self.project_dir)?;
        
        let mut ns_query = NsQuery::new();
        
        for filter in &query.filters {
            ns_query = ns_query.filter(self.convert_filter(filter));
        }
        
        if let Some(limit) = query.limit {
            ns_query = ns_query.limit(limit);
        }

        let docs = engine.query(&query.from, ns_query)?;
        let data: Vec<Value> = docs.iter().map(|d| d.to_json()).collect();

        Ok(AirResult::new(EngineType::Nosql, &query.from, data))
    }

    /// Execute SQL query (returns SQL string for now, actual exec would need DB connection)
    fn execute_sql(&self, query: &AirQuery) -> Result<AirResult, Box<dyn std::error::Error>> {
        // For now, just return the generated SQL
        // Full SQL execution would require database connection
        let _sql = query.to_sql();
        
        // Return empty result with SQL metadata
        let mut result = AirResult::new(EngineType::Sql, &query.from, Vec::new());
        result.included.insert("_sql".to_string(), vec![Value::String(query.to_sql())]);
        
        Ok(result)
    }

    /// Convert AirFilter to NoSQL filter
    fn convert_filter(&self, filter: &AirFilter) -> NsFilter {
        NsFilter::eq(&filter.field, filter.value.clone())
    }

    /// Resolve included relation data
    fn resolve_include(
        &self, 
        parent_result: &AirResult, 
        relation: &Relation
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        // Get all unique foreign key values from parent result
        let source_field = &relation.source.field;
        let mut fk_values: Vec<Value> = Vec::new();

        for doc in &parent_result.data {
            if let Some(val) = doc.get(source_field) {
                if !fk_values.contains(val) {
                    fk_values.push(val.clone());
                }
            }
        }

        // Query the target collection/table
        match relation.target.engine {
            EngineType::Nosql => {
                let engine = NoSqlEngine::open(self.project_dir)?;
                let target_field = &relation.target.field;
                
                let mut included = Vec::new();
                for fk in &fk_values {
                    let query = NsQuery::new()
                        .filter(NsFilter::eq(target_field, fk.clone()));
                    
                    let docs = engine.query(&relation.target.collection, query)?;
                    for doc in docs {
                        included.push(doc.to_json());
                    }
                }
                
                Ok(included)
            }
            EngineType::Sql => {
                // SQL includes would need database connection
                // Return placeholder for now
                Ok(Vec::new())
            }
        }
    }
}

/// Add to_json method to Document
impl Document {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "_id": self.id,
            "_schema_version": self.schema_version,
            "_created_at": self.created_at,
            "_modified_at": self.modified_at,
            "data": self.data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_executor_creation() {
        let dir = tempdir().unwrap();
        let executor = HybridExecutor::new(dir.path());
        assert!(executor.is_ok());
    }
}
