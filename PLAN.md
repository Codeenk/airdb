# AirDB Revolution Plan

> **Date**: 2026-02-12
> **Version**: 0.2.6 → 1.0.0
> **Mission**: Transform AirDB from a local SQLite manager into the world's most intuitive, powerful, local-first database platform — supporting SQLite, PostgreSQL, and MySQL with hybrid SQL/NoSQL, AI assistance, team collaboration, and auto-generated APIs.

---

## Part 0: Codebase Audit — Brutal Honest Assessment

### What We Have (The Good)

| Layer | Component | Status | Lines |
|-------|-----------|--------|-------|
| Frontend | App shell, routing, auth, toast system | Working | 873 |
| Frontend | TableEditor (Visual + Raw SQL) | Working | 435 |
| Frontend | NoSqlBrowser (3-pane inspector) | Working | 375 |
| Frontend | Settings page | Barely exists | 98 |
| Frontend | Logo, ConfirmDialog, UpdateBanner | Working | ~170 |
| Backend | SQLite adapter (r2d2 pool) | Working | 141 |
| Backend | Migration runner (apply, rollback, checksum) | Working | 185 |
| Backend | GitHub OAuth device flow | Working | 688 |
| Backend | Keystore (OS keyring + API keys) | Working | 167 |
| Backend | Axum REST API server | Dead code | 250 |
| Backend | NoSQL JSON document engine | Working | ~800 |
| Backend | Hybrid SQL/NoSQL bridge (AirQL) | Working | ~300 |
| Backend | RBAC module | Exists, never used | ~200 |
| Backend | Team branch/merge system | Exists, never used | ~200 |
| Backend | Audit log + backup | Exists, never used | ~200 |
| Backend | Observability/metrics | Exists, never used | ~200 |
| Backend | Operation locks | Working | 300 |
| Backend | Auto-start (Win/Mac/Linux) | Working | 200 |
| Backend | Self-update system | Partial | ~400 |
| Backend | Cross-platform installer | Working | 250 |
| Backend | CLI (clap definitions) | Defined, not wired | 350 |
| Design | Void Cyan design system | Polished | 100 |

### What's Broken or Missing (The Bad)

#### Critical Bugs
1. **Frontend/Backend Column type mismatch**: Frontend `Column` has `{name, type, is_pk, is_nullable, is_unique}` but backend `Column` deserializes as `{name, column_type, nullable, is_primary_key, is_unique}` with different serde renames. `generate_table_migration` expects `column_type` but receives `type`. This means **visual schema editing is broken** — the type field won't deserialize correctly.

2. **`generate_table_migration` signature mismatch**: Backend expects 4 params `(table_name, columns, is_new, original_columns)` but frontend sends 3 `(tableName, columns, isNew)` — missing `original_columns`. ALTER TABLE migrations **crash silently**.

3. **NoSQL `handleCreateCollection` is fake**: It just pushes to local React state — never calls `nosql_create_collection` backend command. Collections vanish on refresh.

4. **`handleDeleteDocument` uses browser `confirm()`**: Not the ConfirmDialog component. Breaks desktop UX.

5. **CSS variable mismatch**: TableEditor.css and NoSqlBrowser.css reference `var(--border)`, `var(--bg-primary)`, `var(--text-sm)`, `var(--text-lg)` etc. that are **NOT defined** in theme.css. The theme defines `--surface-3`, `--void`, but not those aliases. Entire sections may render invisible or broken.

6. **`check_lock` returns hardcoded `Ok(true)`**: With a `// TODO` comment. Lock checking is non-functional.

7. **SQL injection in api.rs**: Uses `format!()` with user input directly in SQL queries. Table names, column names — all injectable.

8. **`execute_raw_sql` is a double-edged sword**: No SQL sanitization, no read-only mode, executes anything including `DROP TABLE`.

#### Dead Code / Phantom Features
9. Settings page: Only theme toggle (disabled "Light" button) and About. No actual settings for database, sync, updates, or API.
10. Migrations page: Literal "Migration history coming soon..." placeholder text.
11. Dashboard: Only 3 stat cards. No data preview, no activity feed, no recent changes.
12. REST API (Axum): Fully built but **never started** from UI or CLI. Complete dead code.
13. RBAC, Team, Audit, Observability modules: All exist in engine but have **zero** integration with commands or UI.
14. `loadProjectType` is commented out in App.tsx (lines 148-155).
15. `updateStatus` state is created but commented out (line 56).

#### Architecture Gaps
16. **SQLite-only**: `database.rs` hardcodes rusqlite. No PostgreSQL/MySQL support at all.
17. **Migration SQL is SQLite dialect**: `AUTOINCREMENT`, `PRAGMA table_info()`, `sqlite_master` — all SQLite-specific.
18. **No data browsing**: TableEditor shows schema only. You cannot view, search, or edit actual row data.
19. **No syntax highlighting**: SQL textarea is plain `<textarea>` — no CodeMirror/Monaco.
20. **Single-tab interface**: Can only view one table at a time.
21. **No schema visualization**: No ER diagrams, no relationship view.
22. **NoSqlState is separate from AppState**: Two state atoms in Tauri, inconsistent.

---

## Part 1: Foundation — Fix What's Broken

> **Goal**: Make everything that exists actually work correctly.
> **Timeline**: Sprint 1-2 (Weeks 1-4)

### 1.1 Fix Critical Frontend/Backend Contract

**Problem**: The frontend `Column` type and backend `Column` struct use different field names.

**Frontend (types/index.ts)**:
```typescript
interface Column {
  name: string;
  type: string;       // ← field name
  is_pk: boolean;     // ← field name
  is_nullable: boolean;
  is_unique: boolean;
  default_value?: string;
}
```

**Backend (schema_editor.rs)**:
```rust
struct Column {
    name: String,
    #[serde(rename = "type")]
    column_type: String,    // serde renames to "type" ✓
    nullable: bool,         // ← WRONG: frontend sends "is_nullable"
    #[serde(rename = "isPrimaryKey")]
    is_primary_key: bool,   // ← WRONG: frontend sends "is_pk"
    #[serde(rename = "isUnique")]
    is_unique: bool,        // ← this one matches via serde
    #[serde(rename = "foreignKey")]
    foreign_key: Option<ForeignKey>,
}
```

**Fix**: Align serde renames to match what the frontend sends:
```rust
#[serde(rename = "is_pk")]
is_primary_key: bool,
#[serde(rename = "is_nullable")]
nullable: bool,
```

Or better: create a proper DTO layer for Tauri commands separate from internal types.

### 1.2 Fix generate_table_migration Signature

The frontend calls:
```typescript
invoke('generate_table_migration', { tableName, columns, isNew: isNewTable })
```

But backend expects:
```rust
fn generate_table_migration(table_name, columns, is_new, original_columns) -> ...
```

**Fix**: Make `original_columns` optional. When editing existing table, load original columns from database on the backend side:
```rust
fn generate_table_migration(
    table_name: String,
    columns: Vec<Column>,
    is_new: bool,
    state: State<AppState>,  // Add state to load originals
) -> Result<MigrationPreview, String>
```

### 1.3 Fix CSS Variable References

Create CSS variable aliases in theme.css:
```css
:root {
  /* Aliases for component compatibility */
  --border: var(--surface-3);
  --bg-primary: var(--void);
  --bg-secondary: var(--surface-0);
  --text-sm: 13px;
  --text-lg: 16px;
  --error: var(--danger);
}
```

### 1.4 Fix NoSQL Collection Creation

```typescript
// BEFORE (broken):
async function handleCreateCollection() {
    setSelectedCollection(newCollectionName);
    setCollections([...collections, { name: newCollectionName, count: 0, size_bytes: 0 }]);
}

// AFTER:
async function handleCreateCollection() {
    await invoke('nosql_create_collection', { name: newCollectionName });
    await loadCollections(); // Reload from backend
    setSelectedCollection(newCollectionName);
}
```

### 1.5 Replace All browser confirm()/alert() with ConfirmDialog

Audit all components for `confirm()` and `alert()` calls:
- NoSqlBrowser.tsx line: `confirm('Delete this document?')` → use ConfirmDialog
- TableEditor.tsx: `// alert('Changes applied successfully')` → already silenced but needs toast

### 1.6 Implement check_lock Properly

```rust
#[tauri::command]
pub fn check_lock(operation: String, state: State<AppState>) -> Result<bool, String> {
    let lock_type = match operation.as_str() { ... };
    let locks = OperationLock::new(project_dir);
    
    // Check if any blocking locks exist
    let blocking_types = locks.get_blocking_lock_types(lock_type);
    for bt in blocking_types {
        if let Some(info) = locks.read_lock_for_type(bt) {
            if info.is_process_alive() && !info.is_expired() {
                return Ok(false); // Cannot proceed
            }
        }
    }
    Ok(true) // Can proceed
}
```

### 1.7 Security: Parameterize SQL in api.rs

Replace ALL `format!()` SQL with parameterized queries:
```rust
// BEFORE:
let query = format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, limit, offset);

// AFTER:
// Whitelist table name (can't parameterize identifiers)
fn validate_table_name(name: &str) -> Result<&str, StatusCode> {
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(name)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
```

---

## Part 2: Multi-Database Engine — The Core Revolution

> **Goal**: Support SQLite + PostgreSQL + MySQL dynamically, with dialect-aware SQL generation.
> **Timeline**: Sprint 3-6 (Weeks 5-12)

### 2.1 Database Adapter Trait

```rust
// engine/adapter/mod.rs
pub trait DatabaseAdapter: Send + Sync {
    fn dialect(&self) -> SqlDialect;
    fn execute(&self, sql: &str, params: &[Value]) -> Result<QueryResult>;
    fn execute_batch(&self, sql: &str) -> Result<()>;
    fn get_tables(&self) -> Result<Vec<String>>;
    fn get_table_schema(&self, table: &str) -> Result<TableSchema>;
    fn get_table_indexes(&self, table: &str) -> Result<Vec<Index>>;
    fn get_foreign_keys(&self, table: &str) -> Result<Vec<ForeignKeyInfo>>;
    fn get_row_count(&self, table: &str) -> Result<u64>;
    fn query_rows(&self, table: &str, limit: usize, offset: usize,
                  sort: Option<SortSpec>, filter: Option<FilterSpec>) -> Result<DataPage>;
}

#[derive(Debug, Clone, Copy)]
pub enum SqlDialect {
    Sqlite,
    Postgres,
    Mysql,
}

pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: u64,
    pub execution_time_ms: u64,
}
```

### 2.2 SQLite Adapter (Refactor Existing)

```rust
// engine/adapter/sqlite.rs
pub struct SqliteAdapter {
    pool: Pool<SqliteConnectionManager>,
}

impl DatabaseAdapter for SqliteAdapter {
    fn dialect(&self) -> SqlDialect { SqlDialect::Sqlite }
    
    fn get_tables(&self) -> Result<Vec<String>> {
        // Existing PRAGMA-based code
    }
    
    fn get_table_schema(&self, table: &str) -> Result<TableSchema> {
        // PRAGMA table_info() — existing code
    }
}
```

### 2.3 PostgreSQL Adapter (NEW)

New dependency: `tokio-postgres` or `sqlx` with PostgreSQL feature.

```rust
// engine/adapter/postgres.rs
pub struct PostgresAdapter {
    pool: PgPool,
    config: PostgresConfig,
}

impl DatabaseAdapter for PostgresAdapter {
    fn dialect(&self) -> SqlDialect { SqlDialect::Postgres }
    
    fn get_tables(&self) -> Result<Vec<String>> {
        // SELECT tablename FROM pg_tables WHERE schemaname = 'public'
    }
    
    fn get_table_schema(&self, table: &str) -> Result<TableSchema> {
        // SELECT column_name, data_type, is_nullable, column_default
        // FROM information_schema.columns WHERE table_name = $1
    }
}
```

### 2.4 MySQL Adapter (NEW)

```rust
// engine/adapter/mysql.rs
pub struct MysqlAdapter {
    pool: mysql_async::Pool,
}

impl DatabaseAdapter for MysqlAdapter {
    fn dialect(&self) -> SqlDialect { SqlDialect::Mysql }
    
    fn get_tables(&self) -> Result<Vec<String>> {
        // SHOW TABLES
    }
}
```

### 2.5 SQL Dialect Generator

The migration generator must produce correct SQL for each database:

```rust
// engine/dialect/mod.rs
pub struct DialectGenerator {
    dialect: SqlDialect,
}

impl DialectGenerator {
    pub fn create_table(&self, table: &str, columns: &[Column]) -> String {
        match self.dialect {
            SqlDialect::Sqlite => self.create_table_sqlite(table, columns),
            SqlDialect::Postgres => self.create_table_postgres(table, columns),
            SqlDialect::Mysql => self.create_table_mysql(table, columns),
        }
    }
    
    fn auto_increment_syntax(&self) -> &str {
        match self.dialect {
            SqlDialect::Sqlite => "AUTOINCREMENT",
            SqlDialect::Postgres => "", // Use SERIAL type instead
            SqlDialect::Mysql => "AUTO_INCREMENT",
        }
    }
    
    fn map_type(&self, generic_type: &str) -> &str {
        match (self.dialect, generic_type) {
            (SqlDialect::Sqlite, "SERIAL") => "INTEGER",
            (SqlDialect::Postgres, "INTEGER PRIMARY KEY AUTOINCREMENT") => "SERIAL PRIMARY KEY",
            (SqlDialect::Postgres, "BOOLEAN") => "BOOLEAN",
            (SqlDialect::Sqlite, "BOOLEAN") => "INTEGER",
            (SqlDialect::Postgres, "DATETIME") => "TIMESTAMPTZ",
            (SqlDialect::Mysql, "DATETIME") => "DATETIME",
            (SqlDialect::Postgres, "TEXT") => "TEXT",
            (SqlDialect::Postgres, "BLOB") => "BYTEA",
            (SqlDialect::Mysql, "BLOB") => "BLOB",
            _ => generic_type,
        }
    }
}
```

### 2.6 Type System — Unified Column Types

Define a universal type system that maps to each dialect:

| Universal Type | SQLite | PostgreSQL | MySQL |
|---------------|--------|------------|-------|
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | `SERIAL PRIMARY KEY` | `INT AUTO_INCREMENT PRIMARY KEY` |
| `uuid` | `TEXT` | `UUID DEFAULT gen_random_uuid()` | `CHAR(36)` |
| `string` | `TEXT` | `TEXT` | `VARCHAR(255)` |
| `string(N)` | `TEXT` | `VARCHAR(N)` | `VARCHAR(N)` |
| `text` | `TEXT` | `TEXT` | `LONGTEXT` |
| `integer` | `INTEGER` | `INTEGER` | `INT` |
| `bigint` | `INTEGER` | `BIGINT` | `BIGINT` |
| `float` | `REAL` | `DOUBLE PRECISION` | `DOUBLE` |
| `decimal(P,S)` | `REAL` | `DECIMAL(P,S)` | `DECIMAL(P,S)` |
| `boolean` | `INTEGER` | `BOOLEAN` | `TINYINT(1)` |
| `date` | `TEXT` | `DATE` | `DATE` |
| `datetime` | `TEXT` | `TIMESTAMPTZ` | `DATETIME` |
| `time` | `TEXT` | `TIME` | `TIME` |
| `json` | `TEXT` | `JSONB` | `JSON` |
| `blob` | `BLOB` | `BYTEA` | `LONGBLOB` |
| `enum(...)` | `TEXT CHECK(...)` | `TYPE_ENUM` | `ENUM(...)` |
| `array` | N/A | `type[]` | N/A (use JSON) |

### 2.7 Connection Manager

```rust
// engine/connections.rs
#[derive(Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub adapter: AdapterType,
    pub config: AdapterConfig,
    pub color: Option<String>,  // UI color coding
    pub is_default: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdapterConfig {
    Sqlite {
        path: PathBuf,
    },
    Postgres {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String, // Stored encrypted via keyring
        ssl_mode: SslMode,
    },
    Mysql {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        ssl: bool,
    },
}
```

### 2.8 Frontend Connection UI

New Settings section: **Connections**
- Add/edit/remove database connections
- Connection string builder (or paste URI)
- Test connection button
- Color-coded connection indicator in sidebar
- Quick-switch dropdown in topbar

---

## Part 3: The Data Browser — See Your Data

> **Goal**: Browse, search, edit, and export actual row data. Not just schema.
> **Timeline**: Sprint 4-6 (Weeks 7-12)

### 3.1 DataGrid Component

A high-performance, virtualized data grid:

```
┌─────────────────────────────────────────────────────────────┐
│ users (2,847 rows)    [Filter] [Sort] [Export▾] [+ Insert]  │
├─────────┬──────────────┬────────────────┬──────────┬────────┤
│ id (PK) │ name         │ email          │ role     │ active │
├─────────┼──────────────┼────────────────┼──────────┼────────┤
│ 1       │ John Doe     │ john@test.com  │ admin    │ ✓      │
│ 2       │ Jane Smith   │ jane@test.com  │ user     │ ✓      │
│ 3       │ Bob Wilson   │ bob@test.com   │ user     │ ✗      │
│ ...     │ ...          │ ...            │ ...      │ ...    │
├─────────┴──────────────┴────────────────┴──────────┴────────┤
│ Showing 1-100 of 2,847  │  ◁ 1 2 3 ... 29 ▷  │  100/page  │
└─────────────────────────────────────────────────────────────┘
```

**Features**:
- Virtual scrolling (render only visible rows)
- Click-to-edit cells (inline editing)
- Column resizing and reordering
- Multi-row selection
- Sort by any column (ASC/DESC)
- Filter with operators (=, !=, LIKE, >, <, IS NULL, IS NOT NULL)
- Pagination with configurable page size
- Copy cell / copy row as JSON / copy as INSERT statement
- Export selection as CSV, JSON, SQL INSERT
- FK cells show preview of related row on hover
- NULL values styled distinctly
- Column type icons (🔑 PK, 🔗 FK, # integer, "a" text, etc.)

**Technical approach**:
```typescript
// New Tauri command
invoke<DataPage>('query_table_data', {
  table: 'users',
  limit: 100,
  offset: 0,
  sort: { column: 'id', direction: 'asc' },
  filters: [
    { column: 'role', operator: 'eq', value: 'admin' }
  ]
})

interface DataPage {
  rows: Record<string, any>[];
  total_count: number;
  columns: ColumnMeta[];
  execution_time_ms: number;
}
```

### 3.2 Inline Cell Editor

Click a cell → it becomes an editor matching the column type:
- **TEXT/VARCHAR**: Input field
- **INTEGER**: Number input with up/down arrows
- **BOOLEAN**: Toggle switch
- **DATETIME**: Date/time picker
- **JSON/JSONB**: JSON editor popup
- **ENUM**: Dropdown select
- **BLOB**: "View binary" link
- **NULL**: "Set NULL" button

On blur/Enter: Generate and execute UPDATE statement.
On Escape: Cancel edit.

### 3.3 Row Inspector Panel

When a row is selected, show a right-side panel:
```
┌────────────────────┐
│ Row #42            │
│ ─────────────────  │
│ id: 42             │
│ name: "John Doe"   │
│ email: "j@t.com"   │
│ created: 2026-01-  │
│                    │
│ ─── Relations ───  │
│ → orders (12)      │
│ → sessions (3)     │
│ ← team.members (1) │
│                    │
│ [Edit] [Delete]    │
│ [Copy JSON]        │
│ [Copy INSERT]      │
└────────────────────┘
```

### 3.4 Insert Row Dialog

Modal with form fields auto-generated from table schema:
- Required fields marked with *
- Default values pre-filled
- FK fields get a searchable dropdown of related table values
- JSON fields get a proper editor
- Validation before submit

### 3.5 Bulk Operations

- **Multi-select rows** → Delete selected, Export selected
- **Import data** → CSV file upload → Map columns → Preview → Insert
- **Truncate table** → With ConfirmDialog (danger variant)

---

## Part 4: SQL Editor — From Textarea to IDE

> **Goal**: Replace the plain textarea with a proper code editor experience.
> **Timeline**: Sprint 5-7 (Weeks 9-14)

### 4.1 CodeMirror 6 Integration

Replace `<textarea>` with CodeMirror 6:
- SQL syntax highlighting (dialect-aware)
- Autocomplete (table names, column names, SQL keywords)
- Error highlighting (underline syntax errors)
- Multi-cursor editing
- Line numbers
- Code folding for CTEs and subqueries
- Format SQL button (auto-indent)
- Dark theme matching Void Cyan

Dependencies: `@codemirror/lang-sql`, `@codemirror/autocomplete`, `@codemirror/theme-one-dark`

### 4.2 Query Results Grid

Replace the plain `<pre>` output with a proper results table:
```
┌──────────────────────────────────────────────────┐
│ Results (142 rows, 3.2ms)          [Export] [Copy]│
├───────┬──────────────┬──────────────────┬────────┤
│ id    │ name         │ email            │ total  │
│ INT   │ TEXT         │ TEXT             │ REAL   │
├───────┼──────────────┼──────────────────┼────────┤
│ 1     │ John Doe     │ john@test.com    │ 142.50 │
│ 2     │ Jane Smith   │ jane@test.com    │ 89.00  │
└───────┴──────────────┴──────────────────┴────────┘
```

### 4.3 Multi-Tab Query Interface

```
┌─────────────────────────────────────────────┐
│ [Query 1] [Query 2 ●] [Query 3] [+]        │
│ ────────────────────────────────────────     │
│ SELECT u.name, COUNT(o.id)                  │
│ FROM users u                                │
│ LEFT JOIN orders o ON o.user_id = u.id      │
│ GROUP BY u.name                             │
│ ─────────────────────── [▶ Run] [Explain]   │
│ ┌─ Results ─────────────────────────────┐   │
│ │ ...                                   │   │
│ └───────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

Features:
- Named tabs (rename by double-click)
- Unsaved indicator (● dot)
- Save/load queries (stored per project)
- Query history (last 100 queries with timestamps)
- Explain plan visualization

### 4.4 Query Explain Visualizer

For PostgreSQL: Parse `EXPLAIN ANALYZE` output into a tree view:
```
Seq Scan on users (cost=0.00..35.50 rows=2550)
  Filter: (role = 'admin')
  → Rows: 42 (estimated 50)
  → Time: 0.8ms
  
⚠ Suggestion: Add index on users(role)
```

### 4.5 Saved Queries & Snippets

```typescript
interface SavedQuery {
  id: string;
  name: string;
  sql: string;
  connection_id: string;
  created_at: string;
  last_run_at?: string;
  tags: string[];
}
```

Sidebar section: "Saved Queries" with search and folders.

---

## Part 5: Schema Visualization — See the Big Picture

> **Goal**: Interactive ER diagram, relationship visualization, schema diffing.
> **Timeline**: Sprint 6-8 (Weeks 11-16)

### 5.1 ER Diagram View

New page: **Schema Map**

Interactive canvas showing:
```
┌──────────┐     1:N      ┌──────────────┐
│  users   │──────────────│   orders     │
│──────────│              │──────────────│
│ 🔑 id    │              │ 🔑 id        │
│ name     │    N:1       │ 🔗 user_id   │
│ email    │◄─────────────│ total        │
│ role     │              │ status       │
└──────────┘              └──────────────┘
                               │ 1:N
                               ▼
                          ┌──────────────┐
                          │ order_items  │
                          │──────────────│
                          │ 🔑 id        │
                          │ 🔗 order_id  │
                          │ 🔗 product_id│
                          │ quantity     │
                          └──────────────┘
```

**Technology**: React Flow or D3.js for interactive canvas.

**Features**:
- Auto-layout (dagre algorithm)
- Drag to rearrange tables
- Click table → navigate to TableEditor
- Click relationship → see FK details
- Color-code by: table group, data type distribution, row count heat map
- Zoom/pan controls
- Export as PNG/SVG
- Show/hide columns on cards
- Filter tables by name

### 5.2 Schema Diff / Compare

Compare two schemas side-by-side:
```
┌─ Before ────────────────┬─ After ─────────────────┐
│ users                   │ users                    │
│   id INTEGER PK         │   id INTEGER PK          │
│ - name TEXT NOT NULL     │ + full_name TEXT NOT NULL │
│   email TEXT UNIQUE      │   email TEXT UNIQUE       │
│                         │ + avatar_url TEXT          │
│                         │ + last_login DATETIME      │
│─────────────────────────│──────────────────────────│
│                         │ + user_preferences (NEW)  │
│                         │   id INTEGER PK           │
│                         │   user_id INTEGER FK      │
│                         │   theme TEXT               │
└─────────────────────────┴──────────────────────────┘
```

Use cases:
- Compare current schema vs last migration
- Compare local schema vs GitHub remote
- Compare dev vs production

### 5.3 Schema Templates (Starters)

Pre-built schema templates for common applications:

| Template | Tables | Description |
|----------|--------|-------------|
| **Blog** | users, posts, comments, tags, post_tags | Standard blog with tagging |
| **E-Commerce** | users, products, categories, orders, order_items, payments | Full shop |
| **SaaS** | organizations, users, teams, team_members, api_keys, subscriptions | Multi-tenant |
| **Social** | users, posts, follows, likes, messages, conversations | Social network |
| **CMS** | pages, sections, media, navigation, settings | Content management |
| **Analytics** | events, sessions, users, funnels, metrics | Event tracking |
| **Chat** | users, rooms, messages, attachments, reactions | Real-time chat |
| **IoT** | devices, readings, alerts, dashboards, widgets | IoT platform |

Templates are JSON files that define tables + columns + relations.
"New Project" dialog gets a template selector.

---

## Part 6: AI Query Assistant

> **Goal**: Natural language to SQL, query explanation, error fixing, schema suggestions.
> **Timeline**: Sprint 7-9 (Weeks 13-18)

### 6.1 Natural Language Query Bar

```
┌──────────────────────────────────────────────────┐
│ 🔮 Ask: "Show me users who signed up this month  │
│         and have made more than 3 orders"         │
│ ─────────────────────────────────────────────     │
│ Generated SQL:                                    │
│ SELECT u.* FROM users u                           │
│ WHERE u.created_at >= DATE_TRUNC('month', NOW())  │
│ AND (SELECT COUNT(*) FROM orders o                │
│      WHERE o.user_id = u.id) > 3;                │
│                                                   │
│ [▶ Execute] [Edit SQL] [Explain] [Save]           │
└──────────────────────────────────────────────────┘
```

### 6.2 Implementation Strategy

**Option A**: Local LLM (llama.cpp sidecar) — works offline, slower
**Option B**: API-based (OpenAI/Anthropic) — fast, requires key
**Option C**: Hybrid — local for simple, API for complex (recommended)

The AI needs schema context:
```json
{
  "tables": [
    {"name": "users", "columns": [{"name": "id", "type": "INTEGER"}, ...]},
    {"name": "orders", "columns": [...]}
  ],
  "relationships": [
    {"from": "orders.user_id", "to": "users.id", "type": "many-to-one"}
  ],
  "dialect": "postgres"
}
```

### 6.3 AI Features Matrix

| Feature | Input | Output |
|---------|-------|--------|
| **NL → SQL** | "Get top 10 customers by revenue" | `SELECT ... ORDER BY ... LIMIT 10` |
| **Explain Query** | Complex SQL | Plain English explanation |
| **Fix Error** | SQL + error message | Corrected SQL |
| **Suggest Index** | Slow query + EXPLAIN | `CREATE INDEX ...` suggestion |
| **Generate Seed Data** | Table schema | Realistic INSERT statements |
| **Schema Review** | Full schema | Optimization suggestions |
| **Describe Table** | Table name | Column descriptions generated |

### 6.4 Settings: AI Configuration

```
┌─ AI Assistant ──────────────────┐
│                                 │
│ Provider: [Local ▾]             │
│   Model: llama-3.1-8b           │
│                                 │
│ - OR -                          │
│                                 │
│ Provider: [OpenAI ▾]            │
│   API Key: sk-***...            │
│   Model: gpt-4o                 │
│                                 │
│ [✓] Include schema context      │
│ [✓] Auto-suggest on errors      │
│ [ ] Send query history          │
│                                 │
└─────────────────────────────────┘
```

---

## Part 7: Complete the Settings Page

> **Goal**: Make Settings a real, comprehensive configuration center.
> **Timeline**: Sprint 3-4 (Weeks 5-8) alongside other work

### 7.1 Settings Architecture

The Settings page should have these sections:

```
┌── Settings ─────────────────────────────────┐
│                                             │
│ ┌─ Tabs ────────────────────────────────┐   │
│ │ General │ Connections │ Editor │ Sync  │   │
│ │ Updates │ API │ Security │ About      │   │
│ └───────────────────────────────────────┘   │
│                                             │
│ [tab content here]                          │
│                                             │
└─────────────────────────────────────────────┘
```

### 7.2 Tab: General
- Theme selector (Void Cyan / Light / Solarized / Nord)
- Font size slider (12-20px)
- Font family (Inter / JetBrains Mono / System)
- Language/locale
- Sidebar position (left/right)
- Startup behavior (last project / home / specific project)
- Keyboard shortcut customization

### 7.3 Tab: Connections
- List of saved database connections
- Add new connection (SQLite file / PostgreSQL / MySQL)
- Test connection
- Set default connection
- Import/export connection configs
- Connection color coding

### 7.4 Tab: Editor
- Auto-save interval
- Tab size (2/4 spaces)
- Word wrap on/off
- Show line numbers
- Show minimap
- Auto-format on save
- Default SQL limit (100/500/1000/5000)

### 7.5 Tab: Sync (GitHub)
- Connected GitHub account info
- Repository selection
- Auto-sync on/off
- Sync interval
- Branch management
- Conflict resolution strategy (ours/theirs/manual)

### 7.6 Tab: Updates
- Current version display
- Update channel (Stable/Beta/Nightly)
- Auto-check for updates toggle
- Check now button
- Update history
- Rollback button

### 7.7 Tab: API
- API server on/off toggle
- Port configuration
- CORS settings
- Rate limiting
- Authentication requirement
- OpenAPI docs link
- Auto-start API server toggle

### 7.8 Tab: Security
- API key management (moved from separate page)
- RBAC policy editor
- Audit log viewer
- Token management
- Data encryption settings

### 7.9 Tab: About
- Version info (existing)
- System info (OS, Rust version, Tauri version)
- Changelog viewer
- License info
- Reset to defaults button
- Export/import all settings

---

## Part 8: Complete the Migrations Page

> **Goal**: Full migration history, visual diff, and management.
> **Timeline**: Sprint 4-5 (Weeks 7-10)

### 8.1 Migration History Timeline

```
┌─ Migrations ────────────────────────────────────────────┐
│                                                         │
│ [+ Create Migration] [Run Pending (2)] [Generate Snap]  │
│                                                         │
│ ┌─ Timeline ──────────────────────────────────────────┐ │
│ │                                                     │ │
│ │ ● 003_add_orders_table.sql         Applied 2h ago   │ │
│ │ │  + CREATE TABLE orders (...)                      │ │
│ │ │  Checksum: a3f2b...                               │ │
│ │ │                                                   │ │
│ │ ● 002_add_user_email.sql           Applied 1d ago   │ │
│ │ │  + ALTER TABLE users ADD COLUMN email TEXT         │ │
│ │ │                                                   │ │
│ │ ● 001_create_users.sql             Applied 3d ago   │ │
│ │    + CREATE TABLE users (id, name)                  │ │
│ │                                                     │ │
│ │ ─── Pending ──────────────────────                  │ │
│ │                                                     │ │
│ │ ○ 004_add_products.sql              Pending         │ │
│ │ ○ 005_add_indexes.sql               Pending         │ │
│ │                                                     │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│ ┌─ Migration Details ─────────────────────────────────┐ │
│ │ [UP SQL]  [DOWN SQL]  [DIFF]                        │ │
│ │                                                     │ │
│ │ -- Up                                               │ │
│ │ CREATE TABLE orders (                               │ │
│ │   id INTEGER PRIMARY KEY AUTOINCREMENT,             │ │
│ │   user_id INTEGER NOT NULL REFERENCES users(id),    │ │
│ │   total REAL NOT NULL DEFAULT 0,                    │ │
│ │   status TEXT NOT NULL DEFAULT 'pending'            │ │
│ │ );                                                  │ │
│ │                                                     │ │
│ │ [Rollback this] [View in Git]                       │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 8.2 Migration Features

- **Create from visual** — Edit table in designer → auto-generate migration
- **Create from SQL** — Write raw migration SQL
- **Create from diff** — Compare current schema vs state → auto-generate
- **Squash migrations** — Combine N migrations into one
- **Dry run** — Preview what will happen without applying
- **Selective rollback** — Rollback specific migrations (with dependency check)
- **Migration linting** — Warn about destructive operations (DROP TABLE, DROP COLUMN)
- **Schema snapshots** — Point-in-time schema dumps (existing but unwired)

---

## Part 9: Dashboard — Make It Actually Useful

> **Goal**: Dashboard as a command center with real metrics and quick access.
> **Timeline**: Sprint 5-6 (Weeks 9-12)

### 9.1 Dashboard Layout

```
┌─ Dashboard ─────────────────────────────────────────────┐
│                                                         │
│ my-project • PostgreSQL • Port 54321 • 3.2 GB           │
│                                                         │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│ │ 14      │ │ 3       │ │ 28.5k   │ │ 3.2 GB  │       │
│ │ Tables  │ │ Pending │ │ Total   │ │ Database│       │
│ │         │ │ Migrat. │ │ Rows    │ │ Size    │       │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘       │
│                                                         │
│ ┌─ Quick Actions ────┐ ┌─ Recent Activity ────────────┐ │
│ │ ▶ Open SQL Editor  │ │ 2m ago  — Applied migration  │ │
│ │ ▶ Open NoSQL       │ │ 15m ago — Edited users table │ │
│ │ ▶ Run Migrations   │ │ 1h ago  — Created API key    │ │
│ │ ▶ Start API Server │ │ 2h ago  — Synced to GitHub   │ │
│ │ ▶ View Schema Map  │ │ 1d ago  — Added orders table │ │
│ └────────────────────┘ └──────────────────────────────┘ │
│                                                         │
│ ┌─ Table Overview ──────────────────────────────────┐   │
│ │ Table          │ Rows    │ Size   │ Last Modified │   │
│ │ users          │ 2,847   │ 1.2 MB │ 2h ago       │   │
│ │ orders         │ 14,203  │ 4.8 MB │ 15m ago      │   │
│ │ products       │ 892     │ 0.6 MB │ 3d ago       │   │
│ │ order_items    │ 41,502  │ 12 MB  │ 15m ago      │   │
│ └───────────────────────────────────────────────────┘   │
│                                                         │
│ ┌─ Schema Health ───────────────────────────────────┐   │
│ │ ✅ No missing indexes detected                     │   │
│ │ ⚠️  2 tables have no foreign keys                  │   │
│ │ ⚠️  Column 'data' in events uses generic TEXT type  │   │
│ │ ✅ All migrations applied                          │   │
│ │ ✅ GitHub sync is up to date                       │   │
│ └───────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## Part 10: REST API Server — Actually Wire It Up

> **Goal**: The Axum REST API server is fully built but never started. Wire it to TUI.
> **Timeline**: Sprint 6-7 (Weeks 11-14)

### 10.1 API Server Control

Add a Tauri command to start/stop the API server:

```rust
#[tauri::command]
async fn start_api_server(port: u16, state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap().clone().ok_or("No database")?;
    let api_state = ApiState { db: Arc::new(db) };
    let app = create_router(api_state);
    
    // Spawn on tokio runtime
    let addr = format!("127.0.0.1:{}", port);
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    
    Ok(format!("API server running on http://127.0.0.1:{}", port))
}
```

### 10.2 Auto-Generated API Documentation

When API server starts, generate OpenAPI spec and serve Swagger UI:
```
http://localhost:54321/api/docs  → Swagger UI
http://localhost:54321/api/spec  → OpenAPI JSON
```

### 10.3 API Dashboard in UI

Show API stats, recent requests, curl examples:
```
┌─ API Server ─────────────────────────────────────┐
│ Status: 🟢 Running on :54321                     │
│ Uptime: 2h 34m                                   │
│ Requests: 1,247 (last hour)                      │
│                                                   │
│ Endpoints:                                        │
│ GET  /api/tables          → List tables           │
│ GET  /api/tables/users    → Query users           │
│ POST /api/tables/users    → Insert user           │
│ PUT  /api/tables/users/1  → Update user #1        │
│ DEL  /api/tables/users/1  → Delete user #1        │
│                                                   │
│ Example:                                          │
│ curl -H "X-API-Key: airdb_dev_xxx"               │
│   http://localhost:54321/api/tables/users?limit=10│
│                                                   │
│ [Stop Server] [Open Docs] [Copy Base URL]         │
└───────────────────────────────────────────────────┘
```

### 10.4 GraphQL Generation (Future)

Auto-generate GraphQL schema from SQL tables:
```graphql
type User {
  id: Int!
  name: String!
  email: String
  orders: [Order!]!
}

type Query {
  users(limit: Int, offset: Int, where: UserFilter): [User!]!
  user(id: Int!): User
}
```

---

## Part 11: Wire the Dead Modules

> **Goal**: Activate RBAC, Audit, Observability, Team modules that exist but are unused.
> **Timeline**: Sprint 8-10 (Weeks 15-20)

### 11.1 RBAC Integration

The RBAC module with Policies and Enforcer exists. Wire it:
- Add RBAC settings to API keys (read-only, developer, admin → fine-grained)
- Enforce permissions on API endpoints
- Show permission matrix in Settings → Security

### 11.2 Audit Log

The AuditLog and AuditEntry types exist. Wire them:
- Log every schema change, migration, data modification
- Show audit log in Dashboard → Activity
- Show per-table audit trail
- Export audit log as CSV/JSON

### 11.3 Health Dashboard

The HealthDashboard and MetricsCollector exist. Wire them:
- Show database health metrics
- Query performance stats
- Connection pool utilization
- Disk space monitoring
- Memory usage

### 11.4 Team Workflows

BranchContext and ThreeWayMerge exist. Wire them:
- Schema branching (create branch → edit → merge)
- Conflict resolution UI (already exists in HTML)
- Team member management
- Branch comparison view

---

## Part 12: Production Infrastructure

> **Goal**: Make AirDB shippable and maintainable.
> **Timeline**: Sprint 9-12 (Weeks 17-24)

### 12.1 Testing Strategy

| Level | Tool | Coverage Target |
|-------|------|----------------|
| **Rust Unit** | `cargo test` | 80% of engine modules |
| **Rust Integration** | Custom test harness | All Tauri commands |
| **Frontend Unit** | Vitest + Testing Library | All components |
| **E2E** | Playwright + Tauri driver | Critical user flows |
| **Performance** | Custom benchmarks | Query latency < 50ms |

Priority test scenarios:
1. Create project → Create table → Insert row → Query → Delete
2. PostgreSQL connection → Schema inspection → Migration
3. Auth flow → API key → REST request
4. NoSQL create → Insert → Query → Delete
5. Hybrid relation → Cross-engine query

### 12.2 CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    - cargo test --all
    - npm run test
    
  build:
    matrix: [linux, macos, windows]
    steps:
      - npm run tauri:build
      
  release:
    if: tag
    steps:
      - Build all platforms
      - Generate checksums
      - Create GitHub Release
      - Update manifest
```

### 12.3 Crash Reporting

```rust
// Use sentry-rust or custom crash reporter
fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|info| {
        // Write crash report to ~/.airdb/crashes/
        // Optionally send to telemetry (with user consent)
    }));
}
```

### 12.4 Telemetry (Opt-in)

Anonymous usage metrics:
- Commands used
- Database types
- Error frequency
- Performance timings

Settings toggle: "Help improve AirDB by sending anonymous usage data"

---

## Part 13: UX Polish — The Details That Matter

> **Goal**: Make every interaction feel premium and intentional.
> **Timeline**: Ongoing through all sprints

### 13.1 Keyboard Navigation

| Shortcut | Action |
|----------|--------|
| `Ctrl+1-7` | Navigate sidebar pages |
| `Ctrl+N` | New table / New tab |
| `Ctrl+S` | Save / Execute |
| `Ctrl+Enter` | Execute query (in SQL editor) |
| `Ctrl+Shift+F` | Global search |
| `Ctrl+P` | Quick switch (command palette) |
| `Ctrl+,` | Open Settings |
| `Ctrl+K` | Quick command bar |
| `Escape` | Close modal / Cancel |
| `F5` | Refresh current view |
| `Ctrl+D` | Duplicate row / column |
| `Ctrl+Backspace` | Delete row / column |

### 13.2 Command Palette

Press `Ctrl+K` for a VS Code-style command palette:
```
┌──────────────────────────────────┐
│ 🔍 Type a command...             │
│──────────────────────────────────│
│ > Open SQL Editor                │
│ > Create New Table               │
│ > Run Pending Migrations         │
│ > Start API Server               │
│ > Open Settings                  │
│ > Switch Project...              │
│ > Generate Schema Snapshot       │
│ > Export as SQL Dump              │
└──────────────────────────────────┘
```

### 13.3 Loading States & Skeletons

Every data loading operation should show:
- Skeleton loaders (not spinners) for tables
- Progress bars for migrations
- Optimistic updates for inline edits
- Error states with retry buttons

### 13.4 Responsive Sidebar

- Collapsible sidebar (icon-only mode)
- Project switcher at top
- Connection indicator
- Notification badge for pending migrations

### 13.5 Theme System

Move beyond single dark theme:

| Theme | Description |
|-------|-------------|
| **Void Cyan** (current) | Deep black + cyan accents |
| **Midnight Blue** | Navy background + blue accents |
| **Dracula** | Purple + green + orange |
| **Nord** | Cool blue-grey palette |
| **Light** | Clean white + blue |
| **Solarized Light** | Warm light theme |
| **High Contrast** | Accessibility-focused |

### 13.6 Notifications System

Replace scattered toasts with a unified notification center:
- Toast notifications (current) — stay
- Notification bell in topbar
- Notification drawer (slide-out panel)
- Persistent notifications for important events (migration failures, sync conflicts)
- Desktop notifications (via Tauri notification plugin)

---

## Part 14: CLI Completion

> **Goal**: The CLI structs are fully defined in cli.rs but the actual execution handlers are missing.
> **Timeline**: Sprint 7-9 (Weeks 13-18)

### 14.1 Wire CLI Commands

The `bin/cli.rs` needs to match each `Commands` variant to an action:

```rust
match cli.command {
    Commands::Init { name, visibility, no_github } => {
        init_project(&name, &visibility, no_github, &cli.format)?;
    }
    Commands::Migrate { action } => match action {
        MigrateAction::Create { name } => create_migration(&name, &project_dir)?,
        MigrateAction::Push => push_migrations(&project_dir)?,
        MigrateAction::Check => check_migrations(&project_dir)?,
        MigrateAction::Rollback { count } => rollback_migrations(&project_dir, count)?,
        MigrateAction::List => list_migrations(&project_dir)?,
    },
    Commands::Serve { port, host } => {
        start_api_server(&project_dir, &host, port)?;
    }
    // ... all other commands
}
```

### 14.2 CLI Output Formatting

```rust
// Text mode (human-readable):
$ airdb status
╔══════════════════════════════════════╗
║ AirDB Project: my-project           ║
║ Database: PostgreSQL (localhost)     ║
║ Tables: 14 │ Rows: 28,502          ║
║ Migrations: 12 applied, 2 pending   ║
║ API: Running on :54321              ║
║ Sync: ✅ Up to date with GitHub     ║
╚══════════════════════════════════════╝

// JSON mode (for scripting):
$ airdb status --format json
{"project":"my-project","database":"postgres",...}
```

---

## Part 15: GitHub Sync Completion

> **Goal**: Complete the Git-based schema versioning and team sync.
> **Timeline**: Sprint 8-10 (Weeks 15-20)

### 15.1 Sync Flow

```
Local Schema Edit
   ↓
Auto-generate Migration
   ↓
Commit to local Git
   ↓
Push to GitHub
   ↓
Team members Pull
   ↓
Auto-apply Migrations
   ↓
Conflict? → Resolution UI
```

### 15.2 Conflict Resolution UI

When sync detects conflicts:
```
┌─ Conflict Resolution ──────────────────────────────────┐
│                                                        │
│ ⚠️ 2 conflicts detected during sync                    │
│                                                        │
│ ┌─ sql/migrations/003_add_email.sql ──────────────┐   │
│ │                                                  │   │
│ │  ←── Local ──→        ←── Remote ──→             │   │
│ │  ALTER TABLE users    ALTER TABLE users           │   │
│ │  ADD COLUMN email     ADD COLUMN email_address   │   │
│ │  TEXT NOT NULL;        VARCHAR(255);              │   │
│ │                                                  │   │
│ │  [Use Local] [Use Remote] [Edit Manually]        │   │
│ └──────────────────────────────────────────────────┘   │
│                                                        │
│ [Resolve All → Use Local] [Resolve All → Use Remote]   │
│                                                        │
└────────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### Phase 1: Stabilize (Weeks 1-4)
- [ ] Fix Column type serde mismatch (1.1)
- [ ] Fix generate_table_migration signature (1.2)
- [ ] Fix CSS variable references (1.3)
- [ ] Fix NoSQL collection creation (1.4)
- [ ] Replace confirm()/alert() (1.5)
- [ ] Fix check_lock (1.6)
- [ ] Fix SQL injection in api.rs (1.7)
- [ ] Fix Rust warnings in locks.rs
- [ ] Add CSS variable aliases to theme.css
- [ ] Wire `loadProjectType` (uncomment + fix)
- [ ] Wire `updateStatus` state (uncomment + fix)

### Phase 2: Multi-Database Foundation (Weeks 5-10)
- [ ] Create DatabaseAdapter trait (2.1)
- [ ] Refactor existing SQLite into adapter (2.2)
- [ ] Implement PostgreSQL adapter (2.3)
- [ ] Implement MySQL adapter (2.4)
- [ ] Build SQL dialect generator (2.5)
- [ ] Build unified type system (2.6)
- [ ] Build connection manager backend (2.7)
- [ ] Build connection UI in Settings (2.8)
- [ ] Update TableEditor for dialect awareness
- [ ] Update migration generator for dialects

### Phase 3: Data Browser (Weeks 7-12)
- [ ] Build DataGrid component (3.1)
- [ ] Build inline cell editor (3.2)
- [ ] Build row inspector panel (3.3)
- [ ] Build insert row dialog (3.4)
- [ ] Implement bulk operations (3.5)
- [ ] Add table data Tauri commands (query_table_data, update_row, delete_row, insert_row)
- [ ] Export functionality (CSV, JSON, SQL)

### Phase 4: SQL Editor Upgrade (Weeks 9-14)
- [ ] Integrate CodeMirror 6 (4.1)
- [ ] Build query results grid (4.2)
- [ ] Build multi-tab interface (4.3)
- [ ] Build EXPLAIN visualizer (4.4)
- [ ] Build saved queries system (4.5)
- [ ] Implement autocomplete with schema context

### Phase 5: Visualization (Weeks 11-16)
- [ ] Build ER diagram view (5.1)
- [ ] Build schema diff/compare (5.2)
- [ ] Create schema templates (5.3)
- [ ] Add schema map to sidebar navigation

### Phase 6: AI Assistant (Weeks 13-18)
- [ ] Build NL query bar (6.1)
- [ ] Setup LLM integration (6.2)
- [ ] Implement NL→SQL, explain, fix (6.3)
- [ ] Build AI settings panel (6.4)

### Phase 7: Complete Everything Else (Weeks 15-24)
- [ ] Complete Settings page (7.1-7.9)
- [ ] Complete Migrations page (8.1-8.2)
- [ ] Complete Dashboard (9.1)
- [ ] Wire REST API server (10.1-10.3)
- [ ] Wire RBAC module (11.1)
- [ ] Wire Audit log (11.2)
- [ ] Wire Health dashboard (11.3)
- [ ] Wire Team workflows (11.4)
- [ ] Complete CLI (14.1-14.2)
- [ ] Complete GitHub sync (15.1-15.2)

### Phase 8: Ship It (Weeks 20-24)
- [ ] Testing suite (12.1)
- [ ] CI/CD pipeline (12.2)
- [ ] Crash reporting (12.3)
- [ ] UX polish pass (13.1-13.6)
- [ ] Performance optimization
- [ ] Documentation
- [ ] Public beta release

---

## Technical Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **PostgreSQL crate** | `sqlx` with async | Compile-time query checking, async native |
| **MySQL crate** | `sqlx` (mysql feature) | Same interface as PostgreSQL |
| **SQL Editor** | CodeMirror 6 | Lightweight, extensible, good SQL support |
| **ER Diagram** | React Flow | Already React, good developer experience |
| **AI Integration** | Hybrid (local + API) | Works offline but can use cloud for complex |
| **Testing** | Vitest + Playwright | Modern, fast, good Tauri support |
| **State Management** | Keep React useState | App is small enough; avoid Redux complexity |
| **Connection Encryption** | OS Keyring (existing) | Already in place, secure |

## New Dependencies

### Rust (Cargo.toml additions)
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "mysql", "sqlite"] }
```

### Frontend (package.json additions)
```json
{
  "@codemirror/lang-sql": "^6.8.0",
  "@codemirror/autocomplete": "^6.18.0", 
  "@codemirror/theme-one-dark": "^6.1.0",
  "@codemirror/view": "^6.34.0",
  "@codemirror/state": "^6.4.0",
  "reactflow": "^11.11.0",
  "@tanstack/react-virtual": "^3.10.0"
}
```

---

## Competitive Positioning

| Feature | AirDB | TablePlus | DBeaver | pgAdmin | Supabase |
|---------|-------|-----------|---------|---------|----------|
| Local-first | ✅ | ✅ | ✅ | ❌ | ❌ |
| Multi-DB | ✅ | ✅ | ✅ | PG only | PG only |
| Visual migrations | ✅ | ❌ | ❌ | ❌ | ❌ |
| GitHub sync | ✅ | ❌ | ❌ | ❌ | ❌ |
| Hybrid SQL+NoSQL | ✅ | ❌ | ❌ | ❌ | ❌ |
| Auto REST API | ✅ | ❌ | ❌ | ❌ | ✅ |
| AI assistant | ✅ | ❌ | ❌ | ❌ | ✅ |
| Team collab | ✅ | ❌ | ❌ | ❌ | ✅ |
| Schema templates | ✅ | ❌ | ❌ | ❌ | ❌ |
| ER visualization | ✅ | ❌ | ✅ | ❌ | ❌ |
| Free & open | ✅ | ❌ | ✅ | ✅ | Freemium |
| Cross-platform desktop | ✅ | Mac/Win | ✅ | Web | Web |
| CLI | ✅ | ❌ | ❌ | ✅ | ✅ |
| Schema-as-code | ✅ | ❌ | ❌ | ❌ | ❌ |
| RBAC built-in | ✅ | ❌ | ❌ | ✅ | ✅ |
| Audit trail | ✅ | ❌ | ❌ | ❌ | ❌ |

**AirDB's unique positioning**: The ONLY tool that combines local-first development, visual schema management with auto-migration, GitHub-backed versioning, hybrid SQL+NoSQL, AND auto-generated APIs — all in a beautiful desktop app with a CLI.

---

## Success Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| First project creation | < 30 seconds | Timer from app open to first table |
| Query execution | < 50ms for 10K rows | Performance benchmarks |
| Migration generation | < 200ms | From edit to SQL preview |
| Type-safe API generation | 100% | All tables get endpoints |
| Zero data loss | 100% | Migration rollback tests |
| Cross-DB compatibility | 3 engines | SQLite, PostgreSQL, MySQL |
| Codebase test coverage | > 70% | cargo tarpaulin + vitest |
| Bundle size (frontend) | < 500KB gzip | Vite bundle analysis |
| Startup time | < 2 seconds | Cold start measurement |
| GitHub stars | 1000+ within 6 months | Community interest |

---

## The Vision

AirDB is not just another database GUI. It's a **database development platform** that:

1. **Lets you design databases visually** and automatically generates production-ready SQL
2. **Versions your schema like code** with Git and GitHub
3. **Works with any database** — start with SQLite, move to PostgreSQL without changing a line
4. **Generates APIs automatically** — REST today, GraphQL tomorrow
5. **Bridges SQL and NoSQL** — the only tool that lets you query across both
6. **Helps you write SQL** — AI-powered natural language to SQL
7. **Keeps your team in sync** — schema branches, conflict resolution, audit trails
8. **Runs everywhere** — Desktop app, CLI, all three major OSes

**One tool. Any database. Zero friction.**
