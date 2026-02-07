//! Tauri Commands for NoSQL and Hybrid Operations

use tauri::State;
use serde_json::Value;
use std::sync::Mutex;
use std::path::PathBuf;

use crate::engine::nosql::{NoSqlEngine, Document, Query, Filter};
use crate::engine::hybrid::{HybridExecutor, AirQuery, RelationsManifest, Relation, FieldRef, RelationType};

/// App state for NoSQL operations
pub struct NoSqlState {
    pub project_dir: Mutex<Option<PathBuf>>,
}

/// Initialize NoSQL storage
#[tauri::command]
pub async fn nosql_init(
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    NoSqlEngine::open_or_create(&project_dir)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "status": "initialized",
        "path": project_dir.to_string_lossy()
    }))
}

/// Create a collection
#[tauri::command]
pub async fn nosql_create_collection(
    name: String,
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let engine = NoSqlEngine::open(&project_dir)
        .map_err(|e| e.to_string())?;

    engine.create_collection(&name)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "status": "created",
        "collection": name
    }))
}

/// List all collections
#[tauri::command]
pub async fn nosql_list_collections(
    state: State<'_, NoSqlState>
) -> Result<Vec<String>, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let engine = NoSqlEngine::open(&project_dir)
        .map_err(|e| e.to_string())?;

    engine.list_collections()
        .map_err(|e| e.to_string())
}

/// Insert a document
#[tauri::command]
pub async fn nosql_insert(
    collection: String,
    data: Value,
    state: State<'_, NoSqlState>
) -> Result<String, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let engine = NoSqlEngine::open(&project_dir)
        .map_err(|e| e.to_string())?;

    let doc = Document::new(data);
    engine.insert(&collection, doc)
        .map_err(|e| e.to_string())
}

/// Get a document by ID
#[tauri::command]
pub async fn nosql_get(
    collection: String,
    id: String,
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let engine = NoSqlEngine::open(&project_dir)
        .map_err(|e| e.to_string())?;

    let doc = engine.get(&collection, &id)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(doc).map_err(|e| e.to_string())?)
}

/// Query documents
#[tauri::command]
pub async fn nosql_query(
    collection: String,
    filters: Vec<(String, Value)>,
    limit: Option<usize>,
    state: State<'_, NoSqlState>
) -> Result<Vec<Value>, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let engine = NoSqlEngine::open(&project_dir)
        .map_err(|e| e.to_string())?;

    let mut query = Query::new();
    
    for (field, value) in filters {
        query = query.filter(Filter::eq(&field, value));
    }
    
    if let Some(n) = limit {
        query = query.limit(n);
    }

    let docs = engine.query(&collection, query)
        .map_err(|e| e.to_string())?;

    Ok(docs.iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect())
}

/// Delete a document
#[tauri::command]
pub async fn nosql_delete(
    collection: String,
    id: String,
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let engine = NoSqlEngine::open(&project_dir)
        .map_err(|e| e.to_string())?;

    let col = engine.collection(&collection)
        .map_err(|e| e.to_string())?;
    
    col.delete(&id)
        .map_err(|e: crate::engine::nosql::NoSqlError| e.to_string())?;

    Ok(serde_json::json!({
        "status": "deleted",
        "id": id
    }))
}

/// Create a relation
#[tauri::command]
pub async fn hybrid_create_relation(
    name: String,
    source: String,
    target: String,
    relation_type: String,
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let source_ref = FieldRef::parse(&source)
        .ok_or_else(|| format!("Invalid source format: {}", source))?;
    let target_ref = FieldRef::parse(&target)
        .ok_or_else(|| format!("Invalid target format: {}", target))?;

    let rel_type = match relation_type.to_lowercase().as_str() {
        "one-to-one" => RelationType::OneToOne,
        "one-to-many" => RelationType::OneToMany,
        "many-to-one" => RelationType::ManyToOne,
        "many-to-many" => RelationType::ManyToMany,
        _ => RelationType::ManyToOne,
    };

    let relation = Relation::new(&name, source_ref, target_ref, rel_type);
    
    let mut manifest = RelationsManifest::load(&project_dir)
        .map_err(|e| e.to_string())?;
    manifest.add(relation);
    manifest.save(&project_dir)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "status": "created",
        "name": name
    }))
}

/// List all relations
#[tauri::command]
pub async fn hybrid_list_relations(
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let manifest = RelationsManifest::load(&project_dir)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(manifest).map_err(|e| e.to_string())?)
}

/// Execute an AirQL query
#[tauri::command]
pub async fn hybrid_query(
    query_json: String,
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let project_dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No project directory set")?;

    let query: AirQuery = serde_json::from_str(&query_json)
        .map_err(|e| e.to_string())?;

    let executor = HybridExecutor::new(&project_dir)
        .map_err(|e| e.to_string())?;

    let result = executor.execute(&query)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// Set the project directory
#[tauri::command]
pub async fn set_project_dir(
    path: String,
    state: State<'_, NoSqlState>
) -> Result<Value, String> {
    let mut dir = state.project_dir.lock()
        .map_err(|e| e.to_string())?;
    
    *dir = Some(PathBuf::from(path.clone()));

    Ok(serde_json::json!({
        "status": "set",
        "path": path
    }))
}
