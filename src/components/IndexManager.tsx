import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './IndexManager.css';

interface Index {
    name: string;
    columns: string[];
    unique: boolean;
}

interface IndexManagerProps {
    tableName: string;
    availableColumns: string[];
    onMigrationGenerated: (upSql: string, downSql: string) => void;
}

export function IndexManager({ tableName, availableColumns, onMigrationGenerated }: IndexManagerProps) {
    const [indexes, setIndexes] = useState<Index[]>([]);
    const [isCreating, setIsCreating] = useState(false);
    const [newIndex, setNewIndex] = useState<Partial<Index>>({
        name: '',
        columns: [],
        unique: false,
    });

    useEffect(() => {
        loadIndexes();
    }, [tableName]);

    const loadIndexes = async () => {
        try {
            const result = await invoke<Index[]>('get_table_indexes', { tableName });
            setIndexes(result);
        } catch (e) {
            console.error('Failed to load indexes:', e);
        }
    };

    const toggleColumn = (column: string) => {
        const cols = newIndex.columns || [];
        if (cols.includes(column)) {
            setNewIndex({ ...newIndex, columns: cols.filter(c => c !== column) });
        } else {
            setNewIndex({ ...newIndex, columns: [...cols, column] });
        }
    };

    const createIndex = () => {
        if (!newIndex.name || !newIndex.columns?.length) return;

        const indexName = `idx_${tableName}_${newIndex.name}`;
        const uniqueStr = newIndex.unique ? 'UNIQUE ' : '';
        const upSql = `CREATE ${uniqueStr}INDEX ${indexName} ON ${tableName}(${newIndex.columns.join(', ')});`;
        const downSql = `DROP INDEX ${indexName};`;

        onMigrationGenerated(upSql, downSql);

        setIsCreating(false);
        setNewIndex({ name: '', columns: [], unique: false });
    };

    const dropIndex = (indexName: string) => {
        const upSql = `DROP INDEX ${indexName};`;
        // For down, we'd need to recreate - simplified here
        const downSql = `-- Recreate ${indexName} if needed`;

        onMigrationGenerated(upSql, downSql);
    };

    return (
        <div className="index-manager">
            <div className="section-header">
                <h3>Indexes</h3>
                {!isCreating && (
                    <button className="btn-small" onClick={() => setIsCreating(true)}>
                        + Add Index
                    </button>
                )}
            </div>

            {indexes.length > 0 && (
                <table className="indexes-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Columns</th>
                            <th>Unique</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {indexes.map((idx) => (
                            <tr key={idx.name}>
                                <td>{idx.name}</td>
                                <td>{idx.columns.join(', ')}</td>
                                <td>{idx.unique ? '✓' : ''}</td>
                                <td>
                                    <button
                                        className="btn-danger-small"
                                        onClick={() => dropIndex(idx.name)}
                                    >
                                        ✕
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}

            {indexes.length === 0 && !isCreating && (
                <div className="no-indexes">No indexes defined</div>
            )}

            {isCreating && (
                <div className="create-index-form">
                    <div className="form-row">
                        <label>Index Name:</label>
                        <input
                            type="text"
                            value={newIndex.name || ''}
                            onChange={(e) => setNewIndex({ ...newIndex, name: e.target.value })}
                            placeholder="e.g., email_created"
                        />
                    </div>

                    <div className="form-row">
                        <label>Columns:</label>
                        <div className="column-selector">
                            {availableColumns.map((col) => (
                                <button
                                    key={col}
                                    className={`column-chip ${newIndex.columns?.includes(col) ? 'selected' : ''}`}
                                    onClick={() => toggleColumn(col)}
                                >
                                    {col}
                                </button>
                            ))}
                        </div>
                    </div>

                    <div className="form-row">
                        <label>
                            <input
                                type="checkbox"
                                checked={newIndex.unique || false}
                                onChange={(e) => setNewIndex({ ...newIndex, unique: e.target.checked })}
                            />
                            Unique Index
                        </label>
                    </div>

                    <div className="form-actions">
                        <button className="btn-secondary" onClick={() => setIsCreating(false)}>
                            Cancel
                        </button>
                        <button
                            className="btn-primary"
                            onClick={createIndex}
                            disabled={!newIndex.name || !newIndex.columns?.length}
                        >
                            Create Index
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
