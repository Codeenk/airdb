import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfirmDialog } from './ConfirmDialog';
import type { DataPage, SortParam, FilterParam } from '../types';
import './DataGrid.css';
import './Skeleton.css';

interface DataGridProps {
  table: string;
  onToast: (type: 'success' | 'error' | 'info', message: string) => void;
}

export function DataGrid({ table, onToast }: DataGridProps) {
  const [data, setData] = useState<DataPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(100);
  const [sort, setSort] = useState<SortParam | null>(null);
  const [filters, setFilters] = useState<FilterParam[]>([]);
  const [selectedRows, setSelectedRows] = useState<Set<number>>(new Set());
  const [editingCell, setEditingCell] = useState<{ row: number; col: string } | null>(null);
  const [editValue, setEditValue] = useState('');
  const [filterColumn, setFilterColumn] = useState('');
  const [filterOp, setFilterOp] = useState('eq');
  const [filterValue, setFilterValue] = useState('');
  const [showFilterBar, setShowFilterBar] = useState(false);
  const [inspectorRow, setInspectorRow] = useState<Record<string, any> | null>(null);
  const [showInsertDialog, setShowInsertDialog] = useState(false);
  const [insertData, setInsertData] = useState<Record<string, string>>({});
  const [confirmDelete, setConfirmDelete] = useState<{ ids: number[] } | null>(null);
  const editInputRef = useRef<HTMLInputElement>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<DataPage>('query_table_data', {
        table,
        limit: pageSize,
        offset: page * pageSize,
        sort: sort || undefined,
        filters: filters.length > 0 ? filters : undefined,
      });
      setData(result);
    } catch (err: any) {
      onToast('error', `Failed to load data: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [table, page, pageSize, sort, filters, onToast]);

  useEffect(() => {
    setPage(0);
    setSort(null);
    setFilters([]);
    setSelectedRows(new Set());
    setInspectorRow(null);
  }, [table]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useEffect(() => {
    if (editingCell && editInputRef.current) {
      editInputRef.current.focus();
      editInputRef.current.select();
    }
  }, [editingCell]);

  const handleSort = (column: string) => {
    setSort((prev) => {
      if (prev?.column === column) {
        return prev.direction === 'asc'
          ? { column, direction: 'desc' }
          : null;
      }
      return { column, direction: 'asc' };
    });
  };

  const handleCellDoubleClick = (rowIndex: number, column: string, value: any) => {
    setEditingCell({ row: rowIndex, col: column });
    setEditValue(value === null ? '' : String(value));
  };

  const handleCellEditSave = async () => {
    if (!editingCell || !data) return;
    const row = data.rows[editingCell.row];
    const pkCol = data.columns.find((c) => c.name === 'id' || c.name === 'ID');
    if (!pkCol) {
      onToast('error', 'Cannot edit: no id column found');
      setEditingCell(null);
      return;
    }

    const id = row[pkCol.name];
    try {
      await invoke('adapter_update_row', {
        table,
        id: Number(id),
        data: { [editingCell.col]: editValue || null },
      });
      onToast('success', 'Cell updated');
      fetchData();
    } catch (err: any) {
      onToast('error', `Update failed: ${err}`);
    }
    setEditingCell(null);
  };

  const handleCellEditCancel = () => {
    setEditingCell(null);
  };

  const handleCellKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleCellEditSave();
    if (e.key === 'Escape') handleCellEditCancel();
  };

  const handleAddFilter = () => {
    if (!filterColumn) return;
    setFilters([...filters, { column: filterColumn, operator: filterOp, value: filterValue }]);
    setFilterColumn('');
    setFilterValue('');
  };

  const handleRemoveFilter = (index: number) => {
    setFilters(filters.filter((_, i) => i !== index));
  };

  const handleRowSelect = (index: number, e: React.MouseEvent) => {
    setSelectedRows((prev) => {
      const next = new Set(prev);
      if (e.shiftKey && prev.size > 0) {
        const last = Math.max(...prev);
        const [start, end] = index > last ? [last, index] : [index, last];
        for (let i = start; i <= end; i++) next.add(i);
      } else if (e.ctrlKey || e.metaKey) {
        if (next.has(index)) next.delete(index);
        else next.add(index);
      } else {
        next.clear();
        next.add(index);
      }
      return next;
    });

    if (data?.rows[index]) {
      setInspectorRow(data.rows[index]);
    }
  };

  const handleDeleteSelected = () => {
    if (!data) return;
    const pkCol = data.columns.find((c) => c.name === 'id' || c.name === 'ID');
    if (!pkCol) {
      onToast('error', 'Cannot delete: no id column found');
      return;
    }
    const ids = Array.from(selectedRows).map((i) => Number(data.rows[i]?.[pkCol.name]));
    setConfirmDelete({ ids });
  };

  const confirmDeleteRows = async () => {
    if (!confirmDelete) return;
    try {
      for (const id of confirmDelete.ids) {
        await invoke('adapter_delete_row', { table, id });
      }
      onToast('success', `Deleted ${confirmDelete.ids.length} row(s)`);
      setSelectedRows(new Set());
      setConfirmDelete(null);
      fetchData();
    } catch (err: any) {
      onToast('error', `Delete failed: ${err}`);
      setConfirmDelete(null);
    }
  };

  const handleInsertRow = async () => {
    try {
      const cleanData: Record<string, any> = {};
      for (const [k, v] of Object.entries(insertData)) {
        if (v !== '') cleanData[k] = v;
      }
      await invoke('adapter_insert_row', { table, data: cleanData });
      onToast('success', 'Row inserted');
      setShowInsertDialog(false);
      setInsertData({});
      fetchData();
    } catch (err: any) {
      onToast('error', `Insert failed: ${err}`);
    }
  };

  const handleExport = (format: 'csv' | 'json') => {
    if (!data) return;
    const rows = selectedRows.size > 0
      ? Array.from(selectedRows).map((i) => data.rows[i])
      : data.rows;

    if (format === 'json') {
      const blob = new Blob([JSON.stringify(rows, null, 2)], { type: 'application/json' });
      downloadBlob(blob, `${table}.json`);
    } else {
      const cols = data.columns.map((c) => c.name);
      const csvRows = [cols.join(',')];
      for (const row of rows) {
        csvRows.push(cols.map((c) => {
          const val = row[c];
          if (val === null || val === undefined) return '';
          const str = String(val);
          return str.includes(',') || str.includes('"') ? `"${str.replace(/"/g, '""')}"` : str;
        }).join(','));
      }
      const blob = new Blob([csvRows.join('\n')], { type: 'text/csv' });
      downloadBlob(blob, `${table}.csv`);
    }
    onToast('success', `Exported ${rows.length} rows as ${format.toUpperCase()}`);
  };

  const downloadBlob = (blob: Blob, filename: string) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  const totalPages = data ? Math.ceil(data.totalCount / pageSize) : 0;

  const sortIcon = (col: string) => {
    if (sort?.column !== col) return '⇅';
    return sort.direction === 'asc' ? '↑' : '↓';
  };

  return (
    <div className="data-grid-container">
      {/* Toolbar */}
      <div className="data-grid-toolbar">
        <span className="data-grid-info">
          {data ? `${data.totalCount.toLocaleString()} rows` : 'Loading...'}
          {data?.executionTimeMs !== undefined && ` · ${data.executionTimeMs}ms`}
        </span>
        <div className="data-grid-actions">
          <button className="btn btn-sm" onClick={() => setShowFilterBar(!showFilterBar)}>
            Filter {filters.length > 0 && `(${filters.length})`}
          </button>
          <button className="btn btn-sm" onClick={() => handleExport('csv')}>CSV</button>
          <button className="btn btn-sm" onClick={() => handleExport('json')}>JSON</button>
          {selectedRows.size > 0 && (
            <button className="btn btn-sm btn-danger" onClick={handleDeleteSelected}>
              Delete ({selectedRows.size})
            </button>
          )}
          <button className="btn btn-sm btn-primary" onClick={() => {
            setInsertData({});
            setShowInsertDialog(true);
          }}>+ Insert</button>
        </div>
      </div>

      {/* Filter Bar */}
      {showFilterBar && (
        <div className="data-grid-filter-bar">
          {filters.map((f, i) => (
            <span key={i} className="filter-tag">
              {f.column} {f.operator} {f.value}
              <button onClick={() => handleRemoveFilter(i)}>×</button>
            </span>
          ))}
          <select
            value={filterColumn}
            onChange={(e) => setFilterColumn(e.target.value)}
          >
            <option value="">Column...</option>
            {data?.columns.map((c) => (
              <option key={c.name} value={c.name}>{c.name}</option>
            ))}
          </select>
          <select value={filterOp} onChange={(e) => setFilterOp(e.target.value)}>
            <option value="eq">=</option>
            <option value="neq">≠</option>
            <option value="gt">&gt;</option>
            <option value="lt">&lt;</option>
            <option value="gte">≥</option>
            <option value="lte">≤</option>
            <option value="like">LIKE</option>
            <option value="is_null">IS NULL</option>
            <option value="is_not_null">IS NOT NULL</option>
          </select>
          <input
            type="text"
            placeholder="Value"
            value={filterValue}
            onChange={(e) => setFilterValue(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAddFilter()}
          />
          <button className="btn btn-sm" onClick={handleAddFilter}>Add</button>
        </div>
      )}

      {/* Table */}
      <div className="data-grid-scroll">
        <table className="data-grid-table">
          <thead>
            <tr>
              <th className="row-number-col">#</th>
              {data?.columns.map((col) => (
                <th
                  key={col.name}
                  onClick={() => handleSort(col.name)}
                  className="sortable-header"
                >
                  {col.name}
                  <span className="sort-icon">{sortIcon(col.name)}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {loading && (
              <>
                {Array.from({ length: 5 }).map((_, i) => (
                  <tr key={`skel-${i}`} className="skeleton-row">
                    <td><div className="skeleton-pulse" style={{ height: 18, borderRadius: 3 }} /></td>
                    {(data?.columns || Array.from({ length: 4 })).map((_, j) => (
                      <td key={j}><div className="skeleton-pulse" style={{ height: 18, borderRadius: 3, width: `${60 + Math.random() * 40}%` }} /></td>
                    ))}
                  </tr>
                ))}
              </>
            )}
            {!loading && data?.rows.map((row, rowIdx) => (
              <tr
                key={rowIdx}
                className={`${selectedRows.has(rowIdx) ? 'selected' : ''}`}
                onClick={(e) => handleRowSelect(rowIdx, e)}
              >
                <td className="row-number-col">{page * pageSize + rowIdx + 1}</td>
                {data.columns.map((col) => (
                  <td
                    key={col.name}
                    onDoubleClick={() => handleCellDoubleClick(rowIdx, col.name, row[col.name])}
                    className={row[col.name] === null ? 'null-cell' : ''}
                  >
                    {editingCell?.row === rowIdx && editingCell.col === col.name ? (
                      <input
                        ref={editInputRef}
                        className="cell-editor"
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        onBlur={handleCellEditSave}
                        onKeyDown={handleCellKeyDown}
                      />
                    ) : (
                      <span className="cell-value">
                        {row[col.name] === null ? (
                          <span className="null-badge">NULL</span>
                        ) : typeof row[col.name] === 'boolean' ? (
                          row[col.name] ? '✓' : '✗'
                        ) : (
                          String(row[col.name])
                        )}
                      </span>
                    )}
                  </td>
                ))}
              </tr>
            ))}
            {!loading && data?.rows.length === 0 && (
              <tr>
                <td colSpan={(data?.columns.length || 1) + 1} className="empty-cell">
                  No rows found
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="data-grid-pagination">
        <div className="pagination-info">
          Showing {data ? page * pageSize + 1 : 0}–{data ? Math.min((page + 1) * pageSize, data.totalCount) : 0} of {data?.totalCount.toLocaleString() || 0}
        </div>
        <div className="pagination-controls">
          <button
            className="btn btn-sm"
            disabled={page === 0}
            onClick={() => setPage(0)}
          >⟨⟨</button>
          <button
            className="btn btn-sm"
            disabled={page === 0}
            onClick={() => setPage(page - 1)}
          >⟨</button>
          <span className="page-indicator">Page {page + 1} of {totalPages || 1}</span>
          <button
            className="btn btn-sm"
            disabled={page >= totalPages - 1}
            onClick={() => setPage(page + 1)}
          >⟩</button>
          <button
            className="btn btn-sm"
            disabled={page >= totalPages - 1}
            onClick={() => setPage(totalPages - 1)}
          >⟩⟩</button>
          <select
            className="page-size-select"
            value={pageSize}
            onChange={(e) => { setPageSize(Number(e.target.value)); setPage(0); }}
          >
            <option value={50}>50/page</option>
            <option value={100}>100/page</option>
            <option value={500}>500/page</option>
            <option value={1000}>1000/page</option>
          </select>
        </div>
      </div>

      {/* Row Inspector Side Panel */}
      {inspectorRow && (
        <div className="row-inspector">
          <div className="inspector-header">
            <span>Row Inspector</span>
            <button onClick={() => setInspectorRow(null)}>×</button>
          </div>
          <div className="inspector-body">
            {Object.entries(inspectorRow).map(([key, value]) => (
              <div key={key} className="inspector-field">
                <span className="inspector-key">{key}</span>
                <span className={`inspector-value ${value === null ? 'null-value' : ''}`}>
                  {value === null ? 'NULL' : typeof value === 'object' ? JSON.stringify(value) : String(value)}
                </span>
              </div>
            ))}
          </div>
          <div className="inspector-actions">
            <button className="btn btn-sm" onClick={() => {
              navigator.clipboard.writeText(JSON.stringify(inspectorRow, null, 2));
              onToast('info', 'Copied as JSON');
            }}>Copy JSON</button>
          </div>
        </div>
      )}

      {/* Insert Dialog */}
      {showInsertDialog && data && (
        <div className="confirm-overlay">
          <div className="insert-dialog">
            <h3>Insert Row into {table}</h3>
            <div className="insert-fields">
              {data.columns.map((col) => (
                <div key={col.name} className="insert-field">
                  <label>{col.name}</label>
                  <input
                    type="text"
                    placeholder={col.type}
                    value={insertData[col.name] || ''}
                    onChange={(e) => setInsertData({ ...insertData, [col.name]: e.target.value })}
                  />
                </div>
              ))}
            </div>
            <div className="insert-actions">
              <button className="btn" onClick={() => setShowInsertDialog(false)}>Cancel</button>
              <button className="btn btn-primary" onClick={handleInsertRow}>Insert</button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation */}
      {confirmDelete && (
        <ConfirmDialog
          isOpen={true}
          title="Delete Rows"
          message={`Are you sure you want to delete ${confirmDelete.ids.length} row(s)? This cannot be undone.`}
          confirmLabel="Delete"
          variant="danger"
          onConfirm={confirmDeleteRows}
          onCancel={() => setConfirmDelete(null)}
        />
      )}
    </div>
  );
}
