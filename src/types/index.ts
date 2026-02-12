
export interface ProjectStatus {
    initialized: boolean;
    project_name?: string;
    db_type?: string;
    api_port?: number;
}

export interface MigrationStatus {
    applied_count: number;
    pending_count: number;
    pending: string[];
    applied?: string[];
}

export interface ApiKey {
    id: string;
    name: string;
    role: string;
    created_at: string;
}

export interface Project {
    name: string;
    path: string;
    configured: boolean;
}

export interface AuthStatus {
    authenticated: boolean;
    username?: string;
}

export interface DeviceCode {
    user_code: string;
    verification_uri: string;
    device_code: string;
    expires_in: number;
    interval: number;
}

export interface UpdateStatus {
    current_version: string;
    update_available: boolean;
    latest_version: string;
    channel: string;
    pending_version?: string;
}

export interface Toast {
    id: number;
    type: 'error' | 'success' | 'info';
    message: string;
}

/* ─── Table Editor Types ─── */

export interface Column {
    name: string;
    type: string;
    is_pk: boolean;
    is_nullable: boolean;
    is_unique: boolean;
    default_value?: string;
    foreign_key?: { table: string; column: string } | null;
}

export interface Index {
    name: string;
    columns: string[];
    unique: boolean;
}

export interface TableSchema {
    name: string;
    columns: Column[];
    indexes: Index[];
}

export interface Table {
    name: string;
    columns: Column[];
}

export interface MigrationPreview {
    upSql: string;
    downSql: string;
    version: number;
    name: string;
}

/* ─── NoSQL Types ─── */

export interface Collection {
    name: string;
    count: number;
    size_bytes: number;
}

export interface Document {
    id: string;
    data: any;
    created_at?: string;
    updated_at?: string;
}

/* ─── Data Browser Types ─── */

export interface ColumnMeta {
    name: string;
    type: string;
}

export interface DataPage {
    rows: Record<string, any>[];
    totalCount: number;
    columns: ColumnMeta[];
    executionTimeMs: number;
}

export interface SortParam {
    column: string;
    direction: 'asc' | 'desc';
}

export interface FilterParam {
    column: string;
    operator: string;
    value: string;
}

/* ─── Connection Types ─── */

export interface ConnectionConfig {
    id: string;
    name: string;
    dialect: 'sqlite' | 'postgres' | 'mysql';
    config: SqliteConfig | PostgresConfig | MysqlConfig;
    color?: string;
    isDefault: boolean;
    createdAt?: string;
}

export interface SqliteConfig {
    type: 'sqlite';
    path: string;
}

export interface PostgresConfig {
    type: 'postgres';
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
    sslMode: string;
}

export interface MysqlConfig {
    type: 'mysql';
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
    ssl: boolean;
}

/* ─── ER Diagram Types ─── */

export interface SchemaGraph {
    tables: SchemaTable[];
    edges: SchemaEdge[];
}

export interface SchemaTable {
    name: string;
    columns: SchemaColumn[];
    rowCount: number;
}

export interface SchemaColumn {
    name: string;
    type: string;
    isPk: boolean;
    isFk: boolean;
    isNullable: boolean;
    isUnique: boolean;
    defaultValue?: string;
    fkTable?: string;
    fkColumn?: string;
}

export interface SchemaEdge {
    from: string;
    fromColumn: string;
    to: string;
    toColumn: string;
}
