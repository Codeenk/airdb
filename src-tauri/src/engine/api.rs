//! AirDB API Module
//! Auto-generated REST API server with OpenAPI documentation

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

use crate::engine::database::Database;

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Database>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_tables,
        get_table_rows,
        insert_row,
        update_row,
        delete_row,
    ),
    tags(
        (name = "tables", description = "Table operations"),
        (name = "rows", description = "Row CRUD operations"),
    )
)]
pub struct ApiDoc;

pub fn create_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/tables", get(list_tables))
        .route("/api/tables/:table", get(get_table_rows))
        .route("/api/tables/:table", post(insert_row))
        .route("/api/tables/:table/:id", put(update_row))
        .route("/api/tables/:table/:id", delete(delete_row))
        .route("/api/health", get(health_check))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/api/tables",
    responses(
        (status = 200, description = "List of tables", body = Vec<String>)
    ),
    tag = "tables"
)]
async fn list_tables(State(state): State<ApiState>) -> Result<Json<Vec<String>>, StatusCode> {
    state
        .db
        .get_tables()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    100
}

#[utoipa::path(
    get,
    path = "/api/tables/{table}",
    params(
        ("table" = String, Path, description = "Table name"),
    ),
    responses(
        (status = 200, description = "Table rows", body = Value)
    ),
    tag = "rows"
)]
async fn get_table_rows(
    State(state): State<ApiState>,
    Path(table): Path<String>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Value>, StatusCode> {
    let conn = state.db.get_connection().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let query = format!(
        "SELECT * FROM {} LIMIT {} OFFSET {}",
        table, params.limit, params.offset
    );

    let mut stmt = conn.prepare(&query).map_err(|_| StatusCode::BAD_REQUEST)?;
    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    
    let rows: Vec<Value> = stmt
        .query_map([], |row| {
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value: Value = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(i)) => json!(i),
                    Ok(rusqlite::types::ValueRef::Real(f)) => json!(f),
                    Ok(rusqlite::types::ValueRef::Text(t)) => {
                        json!(String::from_utf8_lossy(t).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                        json!(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b))
                    }
                    Err(_) => Value::Null,
                };
                obj.insert(col_name.clone(), value);
            }
            Ok(Value::Object(obj))
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(json!({
        "data": rows,
        "count": rows.len(),
        "limit": params.limit,
        "offset": params.offset
    })))
}

#[utoipa::path(
    post,
    path = "/api/tables/{table}",
    params(
        ("table" = String, Path, description = "Table name"),
    ),
    request_body = Value,
    responses(
        (status = 201, description = "Row created", body = Value)
    ),
    tag = "rows"
)]
async fn insert_row(
    State(state): State<ApiState>,
    Path(table): Path<String>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let obj = body.as_object().ok_or(StatusCode::BAD_REQUEST)?;
    
    let columns: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
    let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("?{}", i)).collect();
    
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        placeholders.join(", ")
    );

    let conn = state.db.get_connection().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let values: Vec<String> = obj.values().map(|v| {
        match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        }
    }).collect();

    let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    
    conn.execute(&query, params.as_slice())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let last_id = conn.last_insert_rowid();

    Ok((StatusCode::CREATED, Json(json!({
        "id": last_id,
        "message": "Row created successfully"
    }))))
}

#[utoipa::path(
    put,
    path = "/api/tables/{table}/{id}",
    params(
        ("table" = String, Path, description = "Table name"),
        ("id" = i64, Path, description = "Row ID"),
    ),
    request_body = Value,
    responses(
        (status = 200, description = "Row updated", body = Value)
    ),
    tag = "rows"
)]
async fn update_row(
    State(state): State<ApiState>,
    Path((table, id)): Path<(String, i64)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let obj = body.as_object().ok_or(StatusCode::BAD_REQUEST)?;
    
    let set_clauses: Vec<String> = obj
        .keys()
        .enumerate()
        .map(|(i, k)| format!("{} = ?{}", k, i + 1))
        .collect();
    
    let query = format!(
        "UPDATE {} SET {} WHERE id = ?{}",
        table,
        set_clauses.join(", "),
        obj.len() + 1
    );

    let conn = state.db.get_connection().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut values: Vec<String> = obj.values().map(|v| {
        match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        }
    }).collect();
    values.push(id.to_string());

    let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

    let affected = conn.execute(&query, params.as_slice())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(json!({
        "affected": affected,
        "message": "Row updated successfully"
    })))
}

#[utoipa::path(
    delete,
    path = "/api/tables/{table}/{id}",
    params(
        ("table" = String, Path, description = "Table name"),
        ("id" = i64, Path, description = "Row ID"),
    ),
    responses(
        (status = 200, description = "Row deleted", body = Value)
    ),
    tag = "rows"
)]
async fn delete_row(
    State(state): State<ApiState>,
    Path((table, id)): Path<(String, i64)>,
) -> Result<Json<Value>, StatusCode> {
    let conn = state.db.get_connection().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let query = format!("DELETE FROM {} WHERE id = ?1", table);
    let affected = conn.execute(&query, [id])
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(json!({
        "affected": affected,
        "message": "Row deleted successfully"
    })))
}
