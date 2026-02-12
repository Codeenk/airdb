import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Plus, Play, Camera, RefreshCw, ChevronDown, ChevronRight, Check, Clock, FileText } from 'lucide-react';
import './Migrations.css';

interface MigrationItem {
  name: string;
  sql: string;
  status: 'applied' | 'pending';
  checksum?: string;
}

interface MigrationsData {
  migrations: MigrationItem[];
  applied_count: number;
  pending_count: number;
}

export default function Migrations() {
  const [data, setData] = useState<MigrationsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState('');
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<{ type: 'success' | 'error' | 'info'; msg: string } | null>(null);

  const loadMigrations = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<MigrationsData>('list_all_migrations');
      setData(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadMigrations(); }, [loadMigrations]);

  function showToast(type: 'success' | 'error' | 'info', msg: string) {
    setToast({ type, msg });
    setTimeout(() => setToast(null), 4000);
  }

  function toggleExpand(name: string) {
    setExpanded(prev => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  }

  async function handleRunPending() {
    try {
      setRunning(true);
      const applied = await invoke<string[]>('run_migrations');
      if (applied.length > 0) {
        showToast('success', `Applied ${applied.length} migration(s)`);
      } else {
        showToast('info', 'No pending migrations');
      }
      await loadMigrations();
    } catch (e) {
      showToast('error', String(e));
    } finally {
      setRunning(false);
    }
  }

  async function handleCreate() {
    const name = newName.trim();
    if (!name) return;
    try {
      const path = await invoke<string>('create_migration', { name });
      showToast('success', `Created: ${path.split('/').pop()}`);
      setNewName('');
      setCreating(false);
      await loadMigrations();
    } catch (e) {
      showToast('error', String(e));
    }
  }

  async function handleSnapshot() {
    try {
      const path = await invoke<string>('generate_snapshot');
      showToast('success', `Snapshot saved: ${path.split('/').pop()}`);
    } catch (e) {
      showToast('error', String(e));
    }
  }

  function parseMigrationName(filename: string): { timestamp: string; label: string } {
    // Format: 20260212_153000_add_users.sql
    const match = filename.match(/^(\d{8}_\d{6})_(.+)\.sql$/);
    if (match) {
      const ts = match[1];
      const year = ts.slice(0, 4);
      const month = ts.slice(4, 6);
      const day = ts.slice(6, 8);
      const hour = ts.slice(9, 11);
      const min = ts.slice(11, 13);
      return {
        timestamp: `${year}-${month}-${day} ${hour}:${min}`,
        label: match[2].replace(/_/g, ' '),
      };
    }
    return { timestamp: '', label: filename.replace('.sql', '') };
  }

  if (loading && !data) {
    return (
      <div className="migrations-page">
        <div className="migrations-loading">
          <RefreshCw size={20} className="spin" />
          <span>Loading migrations...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="migrations-page">
        <div className="migrations-error">
          <p>{error}</p>
          <button className="btn btn-primary btn-sm" onClick={loadMigrations}>Retry</button>
        </div>
      </div>
    );
  }

  const applied = data?.migrations.filter(m => m.status === 'applied') ?? [];
  const pending = data?.migrations.filter(m => m.status === 'pending') ?? [];

  return (
    <div className="migrations-page">
      {/* Header */}
      <div className="migrations-header">
        <div className="migrations-title-row">
          <h1 className="page-title">Migrations</h1>
          <div className="migrations-actions">
            <button className="btn btn-ghost btn-sm" onClick={loadMigrations} disabled={loading}>
              <RefreshCw size={14} className={loading ? 'spin' : ''} /> Refresh
            </button>
            <button className="btn btn-ghost btn-sm" onClick={handleSnapshot}>
              <Camera size={14} /> Snapshot
            </button>
            <button className="btn btn-ghost btn-sm" onClick={() => setCreating(true)}>
              <Plus size={14} /> New Migration
            </button>
            <button
              className="btn btn-primary btn-sm"
              onClick={handleRunPending}
              disabled={running || pending.length === 0}
            >
              {running ? <><RefreshCw size={14} className="spin" /> Running...</> : <><Play size={14} /> Run Pending ({pending.length})</>}
            </button>
          </div>
        </div>

        {/* Stats bar */}
        <div className="migrations-stats">
          <div className="stat-pill applied">
            <Check size={12} /> {data?.applied_count ?? 0} Applied
          </div>
          <div className="stat-pill pending">
            <Clock size={12} /> {data?.pending_count ?? 0} Pending
          </div>
          <div className="stat-pill total">
            <FileText size={12} /> {(data?.applied_count ?? 0) + (data?.pending_count ?? 0)} Total
          </div>
        </div>
      </div>

      {/* Create migration inline form */}
      {creating && (
        <div className="migration-create-form">
          <input
            type="text"
            placeholder="Migration name (e.g. add_users_table)"
            value={newName}
            onChange={e => setNewName(e.target.value)}
            onKeyDown={e => { if (e.key === 'Enter') handleCreate(); if (e.key === 'Escape') setCreating(false); }}
            autoFocus
          />
          <button className="btn btn-primary btn-sm" onClick={handleCreate} disabled={!newName.trim()}>
            Create
          </button>
          <button className="btn btn-ghost btn-sm" onClick={() => { setCreating(false); setNewName(''); }}>
            Cancel
          </button>
        </div>
      )}

      {/* Timeline */}
      <div className="migrations-timeline">
        {data?.migrations.length === 0 && (
          <div className="migrations-empty">
            <FileText size={32} />
            <p>No migrations yet</p>
            <p className="text-secondary">Create your first migration to start tracking schema changes</p>
          </div>
        )}

        {/* Pending section */}
        {pending.length > 0 && (
          <div className="migration-section">
            <div className="section-label pending-label">
              <Clock size={14} /> Pending
            </div>
            {pending.map(m => {
              const { timestamp, label } = parseMigrationName(m.name);
              const isOpen = expanded.has(m.name);
              return (
                <div key={m.name} className="migration-item pending">
                  <div className="migration-dot pending" />
                  <div className="migration-content">
                    <div className="migration-row" onClick={() => toggleExpand(m.name)}>
                      <span className="migration-expand">
                        {isOpen ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                      </span>
                      <span className="migration-label">{label}</span>
                      <span className="migration-badge pending">pending</span>
                      <span className="migration-timestamp">{timestamp}</span>
                    </div>
                    {isOpen && (
                      <div className="migration-sql">
                        <pre><code>{m.sql || '-- Empty migration'}</code></pre>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Applied section */}
        {applied.length > 0 && (
          <div className="migration-section">
            <div className="section-label applied-label">
              <Check size={14} /> Applied
            </div>
            {[...applied].reverse().map(m => {
              const { timestamp, label } = parseMigrationName(m.name);
              const isOpen = expanded.has(m.name);
              return (
                <div key={m.name} className="migration-item applied">
                  <div className="migration-dot applied" />
                  <div className="migration-content">
                    <div className="migration-row" onClick={() => toggleExpand(m.name)}>
                      <span className="migration-expand">
                        {isOpen ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                      </span>
                      <span className="migration-label">{label}</span>
                      <span className="migration-badge applied">applied</span>
                      <span className="migration-timestamp">{timestamp}</span>
                    </div>
                    {isOpen && (
                      <div className="migration-sql">
                        <pre><code>{m.sql || '-- Migration file not found on disk'}</code></pre>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Toast */}
      {toast && (
        <div className={`migration-toast ${toast.type}`}>
          {toast.msg}
        </div>
      )}
    </div>
  );
}
