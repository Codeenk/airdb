import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { IndexManager } from './IndexManager';
import './TableEditor.css';

interface Column {
    name: string;
    type: string;
    nullable: boolean;
    defaultValue: string | null;
    isPrimaryKey: boolean;
    isUnique: boolean;
    foreignKey: { table: string; column: string } | null;
}

interface Index {
    name: string;
    columns: string[];
    unique: boolean;
}

interface Table {
    name: string;
    columns: Column[];
    indexes: Index[];
}

interface MigrationPreview {
    upSql: string;
    downSql: string;
    version: number;
    name: string;
}

export function TableEditor() {
    const [tables, setTables] = useState<string[]>([]);
    const [selectedTable, setSelectedTable] = useState<string | null>(null);
    const [tableData, setTableData] = useState<Table | null>(null);
    const [editedColumns, setEditedColumns] = useState<Column[]>([]);
    const [migrationPreview, setMigrationPreview] = useState<MigrationPreview | null>(null);
    const [isCreatingTable, setIsCreatingTable] = useState(false);
    const [newTableName, setNewTableName] = useState('');
    const [loading, setLoading] = useState(false);
    const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

    useEffect(() => {
        loadTables();
    }, []);

    useEffect(() => {
        if (selectedTable) {
            loadTableSchema(selectedTable);
        }
    }, [selectedTable]);

    const loadTables = async () => {
        try {
            const result = await invoke<string[]>('get_tables');
            setTables(result);
        } catch (e) {
            showMessage('error', `Failed to load tables: ${e}`);
        }
    };

    const loadTableSchema = async (tableName: string) => {
        setLoading(true);
        try {
            const result = await invoke<Table>('get_table_schema', { tableName });
            setTableData(result);
            setEditedColumns([...result.columns]);
        } catch (e) {
            showMessage('error', `Failed to load schema: ${e}`);
        }
        setLoading(false);
    };

    const showMessage = (type: 'success' | 'error', text: string) => {
        setMessage({ type, text });
        setTimeout(() => setMessage(null), 5000);
    };

    const addColumn = () => {
        setEditedColumns([
            ...editedColumns,
            {
                name: '',
                type: 'TEXT',
                nullable: true,
                defaultValue: null,
                isPrimaryKey: false,
                isUnique: false,
                foreignKey: null,
            },
        ]);
    };

    const removeColumn = (index: number) => {
        setEditedColumns(editedColumns.filter((_, i) => i !== index));
    };

    const updateColumn = (index: number, field: keyof Column, value: any) => {
        const updated = [...editedColumns];
        updated[index] = { ...updated[index], [field]: value };
        setEditedColumns(updated);
    };

    const generateMigration = async () => {
        if (!selectedTable && !isCreatingTable) return;

        try {
            const tableName = isCreatingTable ? newTableName : selectedTable;
            const result = await invoke<MigrationPreview>('generate_table_migration', {
                tableName,
                columns: editedColumns,
                isNew: isCreatingTable,
                originalColumns: tableData?.columns || [],
            });
            setMigrationPreview(result);
        } catch (e) {
            showMessage('error', `Failed to generate migration: ${e}`);
        }
    };

    const applyMigration = async () => {
        if (!migrationPreview) return;

        setLoading(true);
        try {
            await invoke('apply_generated_migration', {
                name: migrationPreview.name,
                upSql: migrationPreview.upSql,
                downSql: migrationPreview.downSql,
            });
            showMessage('success', 'Migration applied successfully!');
            setMigrationPreview(null);
            setIsCreatingTable(false);
            await loadTables();
            if (selectedTable) await loadTableSchema(selectedTable);
        } catch (e) {
            showMessage('error', `Failed to apply migration: ${e}`);
        }
        setLoading(false);
    };

    const startCreateTable = () => {
        setIsCreatingTable(true);
        setSelectedTable(null);
        setTableData(null);
        setNewTableName('');
        setEditedColumns([
            {
                name: 'id',
                type: 'INTEGER',
                nullable: false,
                defaultValue: null,
                isPrimaryKey: true,
                isUnique: false,
                foreignKey: null,
            },
        ]);
    };

    const cancelEdit = () => {
        setIsCreatingTable(false);
        setMigrationPreview(null);
        if (tableData) {
            setEditedColumns([...tableData.columns]);
        }
    };

    return (
        <div className="table-editor">
            <div className="sidebar">
                <div className="sidebar-header">
                    <h3>Tables</h3>
                    <button className="btn-icon" onClick={startCreateTable} title="Create Table">
                        +
                    </button>
                </div>
                <ul className="table-list">
                    {tables.map((table) => (
                        <li
                            key={table}
                            className={selectedTable === table ? 'selected' : ''}
                            onClick={() => {
                                setIsCreatingTable(false);
                                setMigrationPreview(null);
                                setSelectedTable(table);
                            }}
                        >
                            <span className="table-icon">📋</span>
                            {table}
                        </li>
                    ))}
                </ul>
            </div>

            <div className="main-panel">
                {message && (
                    <div className={`message ${message.type}`}>
                        {message.type === 'success' ? '✓' : '✗'} {message.text}
                    </div>
                )}

                {isCreatingTable && (
                    <div className="create-table-header">
                        <h2>Create New Table</h2>
                        <input
                            type="text"
                            placeholder="Table name..."
                            value={newTableName}
                            onChange={(e) => setNewTableName(e.target.value)}
                            className="table-name-input"
                        />
                    </div>
                )}

                {selectedTable && !isCreatingTable && (
                    <div className="table-header">
                        <h2>
                            <span className="table-icon">📋</span>
                            {selectedTable}
                        </h2>
                    </div>
                )}

                {(selectedTable || isCreatingTable) && (
                    <>
                        <div className="columns-section">
                            <div className="section-header">
                                <h3>Columns</h3>
                                <button className="btn-small" onClick={addColumn}>
                                    + Add Column
                                </button>
                            </div>

                            <table className="columns-table">
                                <thead>
                                    <tr>
                                        <th>Name</th>
                                        <th>Type</th>
                                        <th>Nullable</th>
                                        <th>Default</th>
                                        <th>PK</th>
                                        <th>Unique</th>
                                        <th>Actions</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {editedColumns.map((col, idx) => (
                                        <tr key={idx}>
                                            <td>
                                                <input
                                                    type="text"
                                                    value={col.name}
                                                    onChange={(e) => updateColumn(idx, 'name', e.target.value)}
                                                    placeholder="column_name"
                                                />
                                            </td>
                                            <td>
                                                <select
                                                    value={col.type}
                                                    onChange={(e) => updateColumn(idx, 'type', e.target.value)}
                                                >
                                                    <option value="INTEGER">INTEGER</option>
                                                    <option value="TEXT">TEXT</option>
                                                    <option value="REAL">REAL</option>
                                                    <option value="BLOB">BLOB</option>
                                                    <option value="DATETIME">DATETIME</option>
                                                    <option value="BOOLEAN">BOOLEAN</option>
                                                </select>
                                            </td>
                                            <td>
                                                <input
                                                    type="checkbox"
                                                    checked={col.nullable}
                                                    onChange={(e) => updateColumn(idx, 'nullable', e.target.checked)}
                                                />
                                            </td>
                                            <td>
                                                <input
                                                    type="text"
                                                    value={col.defaultValue || ''}
                                                    onChange={(e) => updateColumn(idx, 'defaultValue', e.target.value || null)}
                                                    placeholder="NULL"
                                                />
                                            </td>
                                            <td>
                                                <input
                                                    type="checkbox"
                                                    checked={col.isPrimaryKey}
                                                    onChange={(e) => updateColumn(idx, 'isPrimaryKey', e.target.checked)}
                                                />
                                            </td>
                                            <td>
                                                <input
                                                    type="checkbox"
                                                    checked={col.isUnique}
                                                    onChange={(e) => updateColumn(idx, 'isUnique', e.target.checked)}
                                                />
                                            </td>
                                            <td>
                                                <button
                                                    className="btn-danger-small"
                                                    onClick={() => removeColumn(idx)}
                                                    disabled={col.isPrimaryKey && editedColumns.length === 1}
                                                >
                                                    ✕
                                                </button>
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>

                        {selectedTable && !isCreatingTable && (
                            <IndexManager
                                tableName={selectedTable}
                                availableColumns={tableData?.columns.map(c => c.name) || []}
                                onMigrationGenerated={(upSql, downSql) => {
                                    setMigrationPreview({
                                        upSql,
                                        downSql,
                                        version: Math.floor(Date.now() / 1000) % 1000000,
                                        name: 'index_change'
                                    });
                                }}
                            />
                        )}

                        <div className="actions-bar">
                            <button className="btn-secondary" onClick={cancelEdit}>
                                Cancel
                            </button>
                            <button
                                className="btn-primary"
                                onClick={generateMigration}
                                disabled={loading || (isCreatingTable && !newTableName)}
                            >
                                Preview Migration
                            </button>
                        </div>
                    </>
                )}

                {migrationPreview && (
                    <div className="migration-preview">
                        <h3>Migration Preview</h3>
                        <div className="sql-preview">
                            <div className="sql-section">
                                <h4>Up (Apply)</h4>
                                <pre><code>{migrationPreview.upSql}</code></pre>
                            </div>
                            <div className="sql-section">
                                <h4>Down (Rollback)</h4>
                                <pre><code>{migrationPreview.downSql}</code></pre>
                            </div>
                        </div>
                        <div className="preview-actions">
                            <button className="btn-secondary" onClick={() => setMigrationPreview(null)}>
                                Edit More
                            </button>
                            <button className="btn-success" onClick={applyMigration} disabled={loading}>
                                {loading ? 'Applying...' : 'Apply Migration'}
                            </button>
                        </div>
                    </div>
                )}

                {!selectedTable && !isCreatingTable && (
                    <div className="empty-state">
                        <div className="empty-icon">📋</div>
                        <h3>No table selected</h3>
                        <p>Select a table from the sidebar or create a new one.</p>
                        <button className="btn-primary" onClick={startCreateTable}>
                            Create Table
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}
