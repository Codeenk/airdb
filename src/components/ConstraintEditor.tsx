import { useState } from 'react';
import './ConstraintEditor.css';

interface ForeignKey {
    name: string;
    column: string;
    referencedTable: string;
    referencedColumn: string;
    onDelete: 'CASCADE' | 'SET NULL' | 'RESTRICT' | 'NO ACTION';
    onUpdate: 'CASCADE' | 'SET NULL' | 'RESTRICT' | 'NO ACTION';
}

interface CheckConstraint {
    name: string;
    expression: string;
}

interface ConstraintEditorProps {
    tableName: string;
    availableColumns: string[];
    availableTables: string[];
    onMigrationGenerated: (upSql: string, downSql: string) => void;
}

export function ConstraintEditor({
    tableName,
    availableColumns,
    availableTables,
    onMigrationGenerated,
}: ConstraintEditorProps) {
    const [activeTab, setActiveTab] = useState<'fk' | 'check'>('fk');
    const [isCreatingFk, setIsCreatingFk] = useState(false);
    const [isCreatingCheck, setIsCreatingCheck] = useState(false);

    const [newFk, setNewFk] = useState<Partial<ForeignKey>>({
        name: '',
        column: '',
        referencedTable: '',
        referencedColumn: '',
        onDelete: 'NO ACTION',
        onUpdate: 'NO ACTION',
    });

    const [newCheck, setNewCheck] = useState<Partial<CheckConstraint>>({
        name: '',
        expression: '',
    });

    const createForeignKey = () => {
        if (!newFk.name || !newFk.column || !newFk.referencedTable || !newFk.referencedColumn) {
            return;
        }

        // SQLite doesn't support ALTER TABLE ADD CONSTRAINT for FK
        // We need to recreate the table - show warning
        const constraintName = `fk_${tableName}_${newFk.name}`;
        const upSql = `-- Foreign Key: ${constraintName}
-- NOTE: SQLite requires table recreation to add FK constraints
-- This migration generates the constraint definition for reference
-- 
-- To add this in SQLite, you need to:
-- 1. Create new table with FK
-- 2. Copy data
-- 3. Drop old table  
-- 4. Rename new table
--
-- Constraint definition:
-- FOREIGN KEY (${newFk.column}) REFERENCES ${newFk.referencedTable}(${newFk.referencedColumn})
--   ON DELETE ${newFk.onDelete}
--   ON UPDATE ${newFk.onUpdate}

-- For new tables, add this to CREATE TABLE:
-- ${newFk.column} ... REFERENCES ${newFk.referencedTable}(${newFk.referencedColumn}) ON DELETE ${newFk.onDelete} ON UPDATE ${newFk.onUpdate}
`;

        const downSql = `-- Remove foreign key ${constraintName}
-- Requires table recreation in SQLite`;

        onMigrationGenerated(upSql, downSql);
        setIsCreatingFk(false);
        setNewFk({
            name: '',
            column: '',
            referencedTable: '',
            referencedColumn: '',
            onDelete: 'NO ACTION',
            onUpdate: 'NO ACTION',
        });
    };

    const createCheckConstraint = () => {
        if (!newCheck.name || !newCheck.expression) {
            return;
        }

        // SQLite 3.25+ supports ALTER TABLE ADD ... CHECK, but not all versions
        const constraintName = `chk_${tableName}_${newCheck.name}`;
        const upSql = `-- Check Constraint: ${constraintName}
-- Expression: CHECK(${newCheck.expression})
--
-- For SQLite 3.25+:
-- ALTER TABLE ${tableName} ADD CONSTRAINT ${constraintName} CHECK(${newCheck.expression});
--
-- For older SQLite, requires table recreation`;

        const downSql = `-- Remove check constraint ${constraintName}
-- ALTER TABLE ${tableName} DROP CONSTRAINT ${constraintName};`;

        onMigrationGenerated(upSql, downSql);
        setIsCreatingCheck(false);
        setNewCheck({ name: '', expression: '' });
    };

    return (
        <div className="constraint-editor">
            <div className="constraint-tabs">
                <button
                    className={`tab ${activeTab === 'fk' ? 'active' : ''}`}
                    onClick={() => setActiveTab('fk')}
                >
                    🔗 Foreign Keys
                </button>
                <button
                    className={`tab ${activeTab === 'check' ? 'active' : ''}`}
                    onClick={() => setActiveTab('check')}
                >
                    ✓ Check Constraints
                </button>
            </div>

            {activeTab === 'fk' && (
                <div className="constraint-content">
                    {!isCreatingFk ? (
                        <button
                            className="btn-small"
                            onClick={() => setIsCreatingFk(true)}
                        >
                            + Add Foreign Key
                        </button>
                    ) : (
                        <div className="create-constraint-form">
                            <h4>New Foreign Key</h4>

                            <div className="form-row">
                                <label>Name:</label>
                                <input
                                    type="text"
                                    value={newFk.name || ''}
                                    onChange={(e) => setNewFk({ ...newFk, name: e.target.value })}
                                    placeholder="e.g., user_id"
                                />
                            </div>

                            <div className="form-row">
                                <label>Column:</label>
                                <select
                                    value={newFk.column || ''}
                                    onChange={(e) => setNewFk({ ...newFk, column: e.target.value })}
                                >
                                    <option value="">Select column...</option>
                                    {availableColumns.map((col) => (
                                        <option key={col} value={col}>{col}</option>
                                    ))}
                                </select>
                            </div>

                            <div className="form-row">
                                <label>References Table:</label>
                                <select
                                    value={newFk.referencedTable || ''}
                                    onChange={(e) => setNewFk({ ...newFk, referencedTable: e.target.value })}
                                >
                                    <option value="">Select table...</option>
                                    {availableTables.filter(t => t !== tableName).map((t) => (
                                        <option key={t} value={t}>{t}</option>
                                    ))}
                                </select>
                            </div>

                            <div className="form-row">
                                <label>References Column:</label>
                                <input
                                    type="text"
                                    value={newFk.referencedColumn || ''}
                                    onChange={(e) => setNewFk({ ...newFk, referencedColumn: e.target.value })}
                                    placeholder="e.g., id"
                                />
                            </div>

                            <div className="form-row">
                                <label>On Delete:</label>
                                <select
                                    value={newFk.onDelete || 'NO ACTION'}
                                    onChange={(e) => setNewFk({ ...newFk, onDelete: e.target.value as ForeignKey['onDelete'] })}
                                >
                                    <option value="NO ACTION">NO ACTION</option>
                                    <option value="CASCADE">CASCADE</option>
                                    <option value="SET NULL">SET NULL</option>
                                    <option value="RESTRICT">RESTRICT</option>
                                </select>
                            </div>

                            <div className="form-row">
                                <label>On Update:</label>
                                <select
                                    value={newFk.onUpdate || 'NO ACTION'}
                                    onChange={(e) => setNewFk({ ...newFk, onUpdate: e.target.value as ForeignKey['onUpdate'] })}
                                >
                                    <option value="NO ACTION">NO ACTION</option>
                                    <option value="CASCADE">CASCADE</option>
                                    <option value="SET NULL">SET NULL</option>
                                    <option value="RESTRICT">RESTRICT</option>
                                </select>
                            </div>

                            <div className="form-actions">
                                <button className="btn-secondary" onClick={() => setIsCreatingFk(false)}>
                                    Cancel
                                </button>
                                <button
                                    className="btn-primary"
                                    onClick={createForeignKey}
                                    disabled={!newFk.name || !newFk.column || !newFk.referencedTable}
                                >
                                    Generate Migration
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            )}

            {activeTab === 'check' && (
                <div className="constraint-content">
                    {!isCreatingCheck ? (
                        <button
                            className="btn-small"
                            onClick={() => setIsCreatingCheck(true)}
                        >
                            + Add Check Constraint
                        </button>
                    ) : (
                        <div className="create-constraint-form">
                            <h4>New Check Constraint</h4>

                            <div className="form-row">
                                <label>Name:</label>
                                <input
                                    type="text"
                                    value={newCheck.name || ''}
                                    onChange={(e) => setNewCheck({ ...newCheck, name: e.target.value })}
                                    placeholder="e.g., positive_amount"
                                />
                            </div>

                            <div className="form-row">
                                <label>Expression:</label>
                                <input
                                    type="text"
                                    value={newCheck.expression || ''}
                                    onChange={(e) => setNewCheck({ ...newCheck, expression: e.target.value })}
                                    placeholder="e.g., amount > 0"
                                />
                            </div>

                            <div className="form-actions">
                                <button className="btn-secondary" onClick={() => setIsCreatingCheck(false)}>
                                    Cancel
                                </button>
                                <button
                                    className="btn-primary"
                                    onClick={createCheckConstraint}
                                    disabled={!newCheck.name || !newCheck.expression}
                                >
                                    Generate Migration
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
