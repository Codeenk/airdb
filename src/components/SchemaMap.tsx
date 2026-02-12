import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  MarkerType,
  Panel,
  type Node,
  type Edge,
  type NodeTypes,
  Handle,
  Position,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import type { SchemaGraph, SchemaTable, SchemaEdge } from '../types';
import './SchemaMap.css';

/* ─── Table Node Component ─── */
function TableNode({ data }: { data: { table: SchemaTable; onNavigate?: (table: string) => void } }) {
  const { table, onNavigate } = data;
  return (
    <div className="er-table-node">
      <div className="er-table-header" onDoubleClick={() => onNavigate?.(table.name)}>
        <span className="er-table-name">{table.name}</span>
        <span className="er-table-count">{table.rowCount.toLocaleString()} rows</span>
      </div>
      <div className="er-table-columns">
        {table.columns.map((col, idx) => (
          <div key={col.name} className={`er-column ${col.isPk ? 'pk' : ''} ${col.isFk ? 'fk' : ''}`}>
            <Handle
              type="target"
              position={Position.Left}
              id={`${table.name}.${col.name}-target`}
              style={{ top: 40 + idx * 26 + 13, background: 'var(--accent)', width: 8, height: 8 }}
            />
            <span className="er-col-icon">
              {col.isPk ? '🔑' : col.isFk ? '🔗' : col.type.includes('INT') ? '#' : col.type.includes('TEXT') || col.type.includes('VARCHAR') ? 'Aa' : '•'}
            </span>
            <span className="er-col-name">{col.name}</span>
            <span className="er-col-type">{col.type}</span>
            {!col.isNullable && <span className="er-col-not-null" title="NOT NULL">*</span>}
            <Handle
              type="source"
              position={Position.Right}
              id={`${table.name}.${col.name}-source`}
              style={{ top: 40 + idx * 26 + 13, background: 'var(--accent)', width: 8, height: 8 }}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

const nodeTypes: NodeTypes = { tableNode: TableNode };

/* ─── Auto Layout (simple grid for now) ─── */
function autoLayout(tables: SchemaTable[]): Node[] {
  const cols = Math.ceil(Math.sqrt(tables.length));
  const xGap = 320;
  const yGap = 50;

  return tables.map((table, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const height = 40 + table.columns.length * 26 + 12;
    return {
      id: table.name,
      type: 'tableNode',
      position: { x: col * xGap, y: row * (yGap + height) },
      data: { table },
    };
  });
}

function buildEdges(schemaEdges: SchemaEdge[]): Edge[] {
  return schemaEdges.map((e, i) => ({
    id: `edge-${i}`,
    source: e.from,
    target: e.to,
    sourceHandle: `${e.from}.${e.fromColumn}-source`,
    targetHandle: `${e.to}.${e.toColumn}-target`,
    type: 'smoothstep',
    animated: true,
    markerEnd: { type: MarkerType.ArrowClosed, color: 'var(--accent)' },
    style: { stroke: 'var(--accent)', strokeWidth: 2, opacity: 0.7 },
    label: `${e.fromColumn} → ${e.toColumn}`,
    labelStyle: { fill: 'var(--text-secondary)', fontSize: 10 },
    labelBgStyle: { fill: 'var(--surface-1)', fillOpacity: 0.9 },
    labelBgPadding: [4, 2] as [number, number],
    labelBgBorderRadius: 3,
  }));
}

/* ─── Main Component ─── */
interface SchemaMapProps {
  onNavigateToTable?: (tableName: string) => void;
}

export function SchemaMap({ onNavigateToTable }: SchemaMapProps) {
  const [schema, setSchema] = useState<SchemaGraph | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [filter, setFilter] = useState('');

  const loadSchema = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<SchemaGraph>('get_schema_graph');
      setSchema(result);
    } catch (err: any) {
      setError(err?.toString() || 'Failed to load schema');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSchema();
  }, [loadSchema]);

  // Rebuild nodes/edges when schema or filter changes
  useEffect(() => {
    if (!schema) return;

    const filteredTables = filter
      ? schema.tables.filter(t => t.name.toLowerCase().includes(filter.toLowerCase()))
      : schema.tables;

    const newNodes = autoLayout(filteredTables).map(n => ({
      ...n,
      data: { ...n.data, onNavigate: onNavigateToTable },
    }));
    const newEdges = buildEdges(
      schema.edges.filter(e =>
        filteredTables.some(t => t.name === e.from) && filteredTables.some(t => t.name === e.to)
      )
    );
    setNodes(newNodes);
    setEdges(newEdges);
  }, [schema, filter, onNavigateToTable, setNodes, setEdges]);

  const stats = useMemo(() => {
    if (!schema) return null;
    const totalRows = schema.tables.reduce((sum, t) => sum + t.rowCount, 0);
    const totalCols = schema.tables.reduce((sum, t) => sum + t.columns.length, 0);
    return { tables: schema.tables.length, columns: totalCols, rows: totalRows, edges: schema.edges.length };
  }, [schema]);

  if (loading) {
    return (
      <div className="schema-map-loading">
        <div className="schema-map-spinner" />
        <span>Loading schema...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="schema-map-error">
        <p>{error}</p>
        <button className="btn btn-primary" onClick={loadSchema}>Retry</button>
      </div>
    );
  }

  if (!schema || schema.tables.length === 0) {
    return (
      <div className="schema-map-empty">
        <h3>No tables found</h3>
        <p>Create tables in the Table Editor to see the schema diagram</p>
      </div>
    );
  }

  return (
    <div className="schema-map-container">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.1}
        maxZoom={2}
        defaultEdgeOptions={{ animated: true }}
        proOptions={{ hideAttribution: true }}
      >
        <Background color="var(--surface-2)" gap={24} size={1} />
        <Controls
          style={{ background: 'var(--surface-1)', borderColor: 'var(--surface-3)', borderRadius: '8px' }}
        />
        <MiniMap
          nodeColor={() => 'var(--accent)'}
          maskColor="rgba(8, 9, 13, 0.7)"
          style={{ background: 'var(--surface-0)', borderRadius: '8px', border: '1px solid var(--surface-3)' }}
        />
        <Panel position="top-left">
          <div className="schema-map-panel">
            <input
              type="text"
              className="schema-map-filter"
              placeholder="Filter tables..."
              value={filter}
              onChange={e => setFilter(e.target.value)}
            />
            {stats && (
              <div className="schema-map-stats">
                <span>{stats.tables} tables</span>
                <span>{stats.columns} columns</span>
                <span>{stats.rows.toLocaleString()} rows</span>
                <span>{stats.edges} relations</span>
              </div>
            )}
            <button className="btn btn-sm" onClick={loadSchema} title="Refresh">↻ Refresh</button>
          </div>
        </Panel>
      </ReactFlow>
    </div>
  );
}
