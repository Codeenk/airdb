import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
    import {
    Table2,
    Plus,
    Trash2,
    Play,
    FileCode,
    LayoutTemplate,
    RefreshCw,
    Code,
    GripVertical,
    ChevronDown,
    Grid3X3,
} from 'lucide-react';
import { Column, TableSchema, MigrationPreview } from '../types';
import { DataGrid } from './DataGrid';
import { SqlEditor } from './SqlEditor';
import './TableEditor.css';

interface TableEditorProps { }

export function TableEditor({ }: TableEditorProps) {
    /* ─── State ─── */
    const [tables, setTables] = useState<string[]>([]);
    const [selectedTable, setSelectedTable] = useState<string | null>(null);
    const [columns, setColumns] = useState<Column[]>([]);
    const [tableName, setTableName] = useState('');
    const [isNewTable, setIsNewTable] = useState(false);
    const [mode, setMode] = useState<'designer' | 'raw' | 'data'>('designer');

    // Drag and Drop State
    const [draggedIdx, setDraggedIdx] = useState<number | null>(null);

    // Migration / Preview
    const [preview, setPreview] = useState<MigrationPreview | null>(null);
    const [rawSql, setRawSql] = useState('');
    const [rawOutput, setRawOutput] = useState<string>('');
    const [rawResults, setRawResults] = useState<{ columns: string[]; rows: Record<string, any>[] } | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [toastMessages, setToastMessages] = useState<{ id: number; type: 'success' | 'error' | 'info'; message: string }[]>([]);

    const showToast = (type: 'success' | 'error' | 'info', message: string) => {
        const id = Date.now();
        setToastMessages(prev => [...prev, { id, type, message }]);
        setTimeout(() => setToastMessages(prev => prev.filter(t => t.id !== id)), 3000);
    };

    // Build autocomplete schema from known tables + current columns
    const sqlSchema = useMemo(() => {
        const schema: Record<string, string[]> = {};
        for (const t of tables) {
            schema[t] = [];
        }
        if (selectedTable && columns.length > 0) {
            schema[selectedTable] = columns.map(c => c.name);
        }
        return schema;
    }, [tables, selectedTable, columns]);

    /* ─── Effects ─── */
    useEffect(() => {
        loadTables();
    }, []);

    useEffect(() => {
        if (selectedTable && !isNewTable) {
            loadTableSchema(selectedTable);
        }
    }, [selectedTable]);

    useEffect(() => {
        // Debounce preview generation
        if (mode === 'designer' && tableName) {
            const timer = setTimeout(() => {
                generatePreview();
            }, 500);
            return () => clearTimeout(timer);
        }
    }, [columns, tableName, isNewTable, mode]);

    // Keyboard Shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                applyChanges();
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'n') { // New Column shortcut? Or New Table?
                // Let's make it New Column if in designer mode
                if (mode === 'designer' && (selectedTable || isNewTable)) {
                    e.preventDefault();
                    addColumn();
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [preview, mode, selectedTable, isNewTable]); // Dependencies for shortcuts

    /* ─── Actions ─── */
    async function loadTables() {
        try {
            const result = await invoke<string[]>('get_tables');
            setTables(result);
        } catch (e) {
            console.error('Failed to load tables', e);
        }
    }

    async function loadTableSchema(name: string) {
        setLoading(true);
        try {
            const schema = await invoke<TableSchema>('get_table_schema', { tableName: name });
            setColumns(schema.columns.map(c => ({
                name: c.name,
                type: c.type,
                is_pk: c.is_pk,
                is_nullable: c.is_nullable,
                is_unique: c.is_unique,
                default_value: c.default_value,
                foreign_key: c.foreign_key,
            })));
            setTableName(schema.name);
            setIsNewTable(false);
            setPreview(null);
        } catch (e) {
            console.error('Failed to load schema', e);
        } finally {
            setLoading(false);
        }
    }

    function handleNewTable() {
        setSelectedTable(null);
        setIsNewTable(true);
        setTableName('new_table');
        setColumns([
            { name: 'id', type: 'INTEGER', is_pk: true, is_nullable: false, is_unique: true }
        ]);
        setPreview(null);
        setMode('designer');
    }

    /* ─── Column Management ─── */
    function addColumn() {
        setColumns([
            ...columns,
            { name: 'new_column', type: 'TEXT', is_pk: false, is_nullable: true, is_unique: false }
        ]);
    }

    function updateColumn(index: number, field: keyof Column, value: any) {
        const newCols = [...columns];
        newCols[index] = { ...newCols[index], [field]: value };
        setColumns(newCols);
    }

    function removeColumn(index: number) {
        const newCols = columns.filter((_, i) => i !== index);
        setColumns(newCols);
    }

    /* ─── Drag and Drop ─── */
    function handleDragStart(e: React.DragEvent, index: number) {
        setDraggedIdx(index);
        e.dataTransfer.effectAllowed = 'move';
        // e.dataTransfer.setDragImage(e.currentTarget, 20, 20); // Optional: Custom drag image
        // Add a class for styling being dragged? handled by CSS :active usually or manually
    }

    function handleDragOver(e: React.DragEvent, index: number) {
        e.preventDefault();
        if (draggedIdx === null || draggedIdx === index) return;

        // Reorder locally
        const newCols = [...columns];
        const draggedItem = newCols[draggedIdx];
        newCols.splice(draggedIdx, 1);
        newCols.splice(index, 0, draggedItem);

        setColumns(newCols);
        setDraggedIdx(index);
    }

    function handleDragEnd() {
        setDraggedIdx(null);
    }

    /* ─── Migration / SQL ─── */
    async function generatePreview() {
        if (!tableName) return;
        try {
            const result = await invoke<MigrationPreview>('generate_table_migration', {
                tableName,
                columns,
                isNew: isNewTable
            });
            setPreview(result);
            setError(null);
        } catch (e) {
            // console.error(e);
            // Quiet fail for preview
        }
    }

    async function applyChanges() {
        if (!preview) return;
        setLoading(true);
        try {
            await invoke('apply_generated_migration', {
                name: preview.name,
                upSql: preview.upSql,
                downSql: preview.downSql,
            });
            await loadTables();
            if (isNewTable) {
                setSelectedTable(tableName);
                setIsNewTable(false);
            }
            setPreview(null);
            setError(null);
        } catch (e) {
            setError(String(e));
        } finally {
            setLoading(false);
        }
    }

    async function executeRawSql() {
        if (!rawSql.trim()) return;
        setLoading(true);
        setRawResults(null);
        try {
            const result = await invoke<any>('execute_raw_sql', { sql: rawSql });
            if (result.rows) {
                const rows = result.rows as Record<string, any>[];
                const columns = rows.length > 0 ? Object.keys(rows[0]) : [];
                setRawResults({ columns, rows });
                setRawOutput(`${result.rowCount} row(s) returned`);
            } else {
                setRawResults(null);
                setRawOutput(result.message || `${result.affectedRows} row(s) affected`);
            }
            await loadTables();
        } catch (e) {
            setRawResults(null);
            setRawOutput(`Error: ${e}`);
        } finally {
            setLoading(false);
        }
    }

    /* ─── Types List ─── */
    const DATA_TYPES = ['INTEGER', 'TEXT', 'REAL', 'BLOB', 'BOOLEAN', 'DATETIME'];

    return (
        <div className="table-editor-container">
            {/* ─── SIDEBAR: Table List ─── */}
            <div className="te-sidebar">
                <div className="te-sidebar-header">
                    <span className="te-sidebar-title">Tables</span>
                    <button className="btn btn-sm btn-ghost" onClick={handleNewTable} title="New Table">
                        <Plus size={16} />
                    </button>
                </div>

                <div className="te-table-list">
                    {tables.map(table => (
                        <div
                            key={table}
                            className={`te-table-item ${selectedTable === table ? 'active' : ''}`}
                            onClick={() => setSelectedTable(table)}
                        >
                            <Table2 size={16} />
                            <span>{table}</span>
                        </div>
                    ))}
                    {isNewTable && (
                        <div className="te-table-item active">
                            <Plus size={16} />
                            <span>{tableName || 'New Table'}</span>
                            <span className="badge badge-accent" style={{ marginLeft: 'auto', fontSize: '10px' }}>NEW</span>
                        </div>
                    )}
                </div>

                <div className="te-sidebar-footer" style={{ padding: '12px', borderTop: '1px solid var(--border)' }}>
                    <div
                        className={`te-table-item ${mode === 'data' ? 'active' : ''}`}
                        onClick={() => { if (selectedTable) setMode('data'); }}
                        style={{ opacity: selectedTable ? 1 : 0.4 }}
                    >
                        <Grid3X3 size={16} />
                        <span>Browse Data</span>
                    </div>
                    <div
                        className={`te-table-item ${mode === 'raw' ? 'active' : ''}`}
                        onClick={() => setMode('raw')}
                    >
                        <Code size={16} />
                        <span>Raw SQL</span>
                    </div>
                </div>
            </div>

            {mode === 'data' ? (
                /* ─── DATA BROWSER MODE ─── */
                <div className="te-main" style={{ flexDirection: 'column', overflow: 'hidden' }}>
                    {selectedTable ? (
                        <DataGrid table={selectedTable} onToast={showToast} />
                    ) : (
                        <div className="te-empty-state">
                            <Grid3X3 size={48} className="te-empty-icon" />
                            <h3>Select a table to browse</h3>
                            <p>Choose a table from the sidebar to view its data</p>
                        </div>
                    )}
                </div>
            ) : mode === 'raw' ? (
                /* ─── RAW SQL MODE ─── */
                <div className="te-main" style={{ flexDirection: 'column' }}>
                    <div className="te-toolbar">
                        <h2 style={{ fontSize: '16px', fontWeight: 600 }}>Raw SQL Execution</h2>
                        <button
                            className="btn btn-primary"
                            onClick={executeRawSql}
                            disabled={loading}
                        >
                            <Play size={14} /> Run Query
                        </button>
                    </div>
                    <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                        <SqlEditor
                            value={rawSql}
                            onChange={setRawSql}
                            onExecute={executeRawSql}
                            tables={sqlSchema}
                            placeholder="SELECT * FROM users;  (Ctrl+Enter to run)"
                            minHeight="200px"
                        />
                        <div style={{ flex: 1, background: 'var(--surface-0)', overflow: 'auto', display: 'flex', flexDirection: 'column', minHeight: '200px' }}>
                            <div style={{ padding: '8px 16px', color: 'var(--text-secondary)', fontSize: '12px', borderBottom: '1px solid var(--surface-3)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <span>OUTPUT</span>
                                <span style={{ color: rawOutput.startsWith('Error') ? 'var(--danger)' : 'var(--success)', fontFamily: 'var(--font-mono)' }}>{rawOutput}</span>
                            </div>
                            {rawResults && rawResults.rows.length > 0 ? (
                                <div style={{ overflow: 'auto', flex: 1 }}>
                                    <table className="data-grid-table">
                                        <thead>
                                            <tr>
                                                {rawResults.columns.map(col => (
                                                    <th key={col}>{col}</th>
                                                ))}
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rawResults.rows.map((row, idx) => (
                                                <tr key={idx}>
                                                    {rawResults.columns.map(col => (
                                                        <td key={col}>
                                                            {row[col] === null ? <span style={{ color: 'var(--text-tertiary)', fontStyle: 'italic' }}>NULL</span> : String(row[col])}
                                                        </td>
                                                    ))}
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            ) : (
                                !rawOutput && (
                                    <div style={{ padding: '16px', color: 'var(--text-tertiary)', fontFamily: 'var(--font-mono)', fontSize: '13px' }}>
                                        Press Ctrl+Enter to execute
                                    </div>
                                )
                            )}
                        </div>
                    </div>
                </div>
            ) : (
                /* ─── DESIGNER MODE ─── */
                <>
                    {/* ─── MAIN: Schema Editor ─── */}
                    <div className="te-main">
                        {(!selectedTable && !isNewTable) ? (
                            <div className="te-empty-state">
                                <LayoutTemplate size={48} className="te-empty-icon" />
                                <h3>Select a table to edit</h3>
                                <p>Or create a new one to get started</p>
                                <button className="btn btn-primary" onClick={handleNewTable} style={{ marginTop: '16px' }}>
                                    Create Table
                                </button>
                            </div>
                        ) : (
                            <>
                                <div className="te-toolbar">
                                    <input
                                        type="text"
                                        className="te-table-name-input"
                                        value={tableName}
                                        onChange={(e) => setTableName(e.target.value)}
                                        placeholder="Table Name"
                                    />
                                    <div style={{ display: 'flex', gap: '8px' }}>
                                        <div className="te-shortcuts" style={{ fontSize: '10px', color: 'var(--text-tertiary)', marginRight: '12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                                            <span>⌘S Save</span>
                                            <span>⌘N Add Col</span>
                                        </div>
                                        <button className="btn btn-ghost" onClick={() => loadTableSchema(selectedTable || tableName)} title="Reset">
                                            <RefreshCw size={14} />
                                        </button>
                                    </div>
                                </div>

                                <div className="te-columns-area">
                                    {columns.map((col, idx) => (
                                        <div
                                            key={idx}
                                            className={`te-column-row ${draggedIdx === idx ? 'dragging' : ''}`}
                                            draggable
                                            onDragStart={(e) => handleDragStart(e, idx)}
                                            onDragOver={(e) => handleDragOver(e, idx)}
                                            onDragEnd={handleDragEnd}
                                        >
                                            <span className="te-drag-handle" style={{ cursor: 'grab' }}>
                                                <GripVertical size={14} />
                                            </span>

                                            <input
                                                className="te-input"
                                                placeholder="Column Name"
                                                value={col.name}
                                                onChange={(e) => updateColumn(idx, 'name', e.target.value)}
                                            />

                                            <div className="select-wrapper" style={{ position: 'relative', width: '100%' }}>
                                                <select
                                                    className="te-input"
                                                    value={col.type}
                                                    onChange={(e) => updateColumn(idx, 'type', e.target.value)}
                                                    style={{ appearance: 'none', paddingRight: '24px' }}
                                                >
                                                    {DATA_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
                                                </select>
                                                <ChevronDown size={14} style={{ position: 'absolute', right: 8, top: '50%', transform: 'translateY(-50%)', pointerEvents: 'none', color: 'var(--text-tertiary)' }} />
                                            </div>

                                            <div className="te-constraints">
                                                <label className={`te-pill-check ${col.is_pk ? 'checked' : ''}`} title="Primary Key">
                                                    <input
                                                        type="checkbox"
                                                        checked={col.is_pk}
                                                        onChange={e => updateColumn(idx, 'is_pk', e.target.checked)}
                                                    />
                                                    PK
                                                </label>
                                                <label className={`te-pill-check ${col.is_nullable ? 'checked' : ''}`} title="Allow Null">
                                                    <input
                                                        type="checkbox"
                                                        checked={col.is_nullable}
                                                        onChange={e => updateColumn(idx, 'is_nullable', e.target.checked)}
                                                    />
                                                    Null
                                                </label>
                                                <label className={`te-pill-check ${col.is_unique ? 'checked' : ''}`} title="Unique Constraint">
                                                    <input
                                                        type="checkbox"
                                                        checked={col.is_unique}
                                                        onChange={e => updateColumn(idx, 'is_unique', e.target.checked)}
                                                    />
                                                    Unq
                                                </label>
                                            </div>

                                            <button className="btn btn-icon btn-ghost btn-danger" onClick={() => removeColumn(idx)}>
                                                <Trash2 size={14} />
                                            </button>
                                        </div>
                                    ))}

                                    <button className="te-add-column" onClick={addColumn} title="Add Column (Cmd+N)">
                                        <Plus size={14} /> Add Column
                                    </button>
                                </div>
                            </>
                        )}
                    </div>

                    {/* ─── RIGHT: Preview ─── */}
                    {(selectedTable || isNewTable) && (
                        <div className="te-preview">
                            <div className="te-preview-header">
                                <span className="te-preview-title">
                                    <FileCode size={14} /> SQL Preview
                                </span>
                            </div>

                            <div style={{ flex: 1, position: 'relative' }}>
                                <textarea
                                    className="te-sql-editor"
                                    value={preview ? preview.upSql : '-- Make changes to generate SQL'}
                                    readOnly
                                    style={{ height: '100%', position: 'absolute', top: 0, left: 0 }}
                                />
                            </div>

                            {error && (
                                <div style={{ padding: '12px', background: 'rgba(239, 68, 68, 0.1)', color: 'var(--error)', fontSize: '12px', borderTop: '1px solid var(--error)' }}>
                                    {error}
                                </div>
                            )}

                            <div className="te-preview-actions">
                                <button
                                    className="btn btn-primary"
                                    onClick={applyChanges}
                                    disabled={!preview || loading}
                                    title="Apply Changes (Cmd+S)"
                                >
                                    {loading ? 'Applying...' : 'Apply Changes'}
                                </button>
                            </div>
                        </div>
                    )}
                </>
            )}

            {/* Toast Messages */}
            {toastMessages.length > 0 && (
                <div style={{ position: 'fixed', bottom: '24px', right: '24px', zIndex: 9999, display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {toastMessages.map(t => (
                        <div key={t.id} style={{
                            padding: '12px 20px',
                            borderRadius: '8px',
                            background: t.type === 'error' ? 'var(--danger)' : t.type === 'success' ? 'var(--success)' : 'var(--accent)',
                            color: '#fff',
                            fontSize: '13px',
                            fontWeight: 500,
                            boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
                            animation: 'fadeIn 0.2s ease'
                        }}>
                            {t.message}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
