import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Database,
  Table2,
  FileJson,
  GitBranch,
  Share2,
  HardDrive,
  Rows3,
  Play,
  Key,
  Settings,
  RefreshCw,
  AlertTriangle,
  CheckCircle2,
  ArrowRight,
  Clock,
} from 'lucide-react';
import './Dashboard.css';
import { Skeleton, SkeletonStats, SkeletonLines } from './Skeleton';
import './Skeleton.css';

interface TableStat {
  name: string;
  rowCount: number;
}

interface AuditEntry {
  action: string;
  resource_type: string;
  resource_name: string;
  timestamp: string;
  actor?: string;
}

interface DashboardData {
  tables: string[];
  tableStats: TableStat[];
  dbSize: string;
  totalRows: number;
  migrationStatus: { applied_count: number; pending_count: number; pending: string[] };
  recentActivity: AuditEntry[];
}

interface Props {
  projectName: string;
  dbType: string;
  apiPort: number;
  onNavigate: (page: string) => void;
}

export default function Dashboard({ projectName, dbType, apiPort, onNavigate }: Props) {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try {
      setLoading(true);
      const [tablesResult, migStatus, sizeResult, auditResult] = await Promise.allSettled([
        invoke<string[]>('get_tables'),
        invoke<{ applied_count: number; pending_count: number; pending: string[] }>('get_migration_status'),
        invoke<string>('get_database_size'),
        invoke<{ total: number; entries: AuditEntry[] }>('get_audit_log', { limit: 10 }),
      ]);

      const tables = tablesResult.status === 'fulfilled' ? tablesResult.value : [];
      const migration = migStatus.status === 'fulfilled' ? migStatus.value : { applied_count: 0, pending_count: 0, pending: [] };
      const dbSize = sizeResult.status === 'fulfilled' ? sizeResult.value : '—';
      const activity = auditResult.status === 'fulfilled' ? auditResult.value.entries : [];

      // Get row counts for each table
      const stats: TableStat[] = [];
      let total = 0;
      for (const t of tables.slice(0, 20)) {
        try {
          const count = await invoke<number>('get_table_row_count', { tableName: t });
          stats.push({ name: t, rowCount: count });
          total += count;
        } catch {
          stats.push({ name: t, rowCount: 0 });
        }
      }
      stats.sort((a, b) => b.rowCount - a.rowCount);

      setData({ tables, tableStats: stats, dbSize, totalRows: total, migrationStatus: migration, recentActivity: activity });
    } catch {
      // partial data is fine
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  function formatNumber(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
    return String(n);
  }

  const hasPending = (data?.migrationStatus.pending_count ?? 0) > 0;

  function formatTimeAgo(ts: string): string {
    const diff = Date.now() - new Date(ts).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    return `${days}d ago`;
  }

  function formatAction(a: string): string {
    if (typeof a === 'string') return a.replace(/_/g, ' ');
    if (typeof a === 'object' && a !== null) {
      const key = Object.keys(a)[0];
      return key ?? 'custom';
    }
    return String(a);
  }

  if (loading && !data) {
    return (
      <div className="dashboard-page">
        <div className="dashboard-banner">
          <div className="banner-info">
            <Skeleton width="40%" height={28} style={{ marginBottom: 8 }} />
            <Skeleton width="60%" height={14} />
          </div>
        </div>
        <SkeletonStats count={4} />
        <div style={{ marginTop: 'var(--space-4)' }}>
          <SkeletonLines lines={4} />
        </div>
      </div>
    );
  }

  return (
    <div className="dashboard-page">
      {/* Project banner */}
      <div className="dashboard-banner">
        <div className="banner-info">
          <h1 className="banner-title">{projectName}</h1>
          <div className="banner-meta">
            <span className="meta-tag"><Database size={12} /> {dbType || 'SQLite'}</span>
            <span className="meta-tag"><HardDrive size={12} /> {data?.dbSize ?? '—'}</span>
            <span className="meta-tag">Port {apiPort}</span>
          </div>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={load} disabled={loading}>
          <RefreshCw size={14} className={loading ? 'spin' : ''} />
        </button>
      </div>

      {/* Stats row */}
      <div className="dashboard-stats">
        <div className="dash-stat" onClick={() => onNavigate('tables')}>
          <div className="dash-stat-value">{data?.tables.length ?? 0}</div>
          <div className="dash-stat-label">Tables</div>
          <Table2 size={16} className="dash-stat-icon" />
        </div>
        <div className="dash-stat" onClick={() => onNavigate('migrations')}>
          <div className="dash-stat-value">{data?.migrationStatus.pending_count ?? 0}</div>
          <div className="dash-stat-label">Pending Migrations</div>
          <GitBranch size={16} className="dash-stat-icon" />
        </div>
        <div className="dash-stat">
          <div className="dash-stat-value">{formatNumber(data?.totalRows ?? 0)}</div>
          <div className="dash-stat-label">Total Rows</div>
          <Rows3 size={16} className="dash-stat-icon" />
        </div>
        <div className="dash-stat">
          <div className="dash-stat-value">{data?.dbSize ?? '—'}</div>
          <div className="dash-stat-label">Database Size</div>
          <HardDrive size={16} className="dash-stat-icon" />
        </div>
      </div>

      <div className="dashboard-grid">
        {/* Quick Actions */}
        <div className="dash-card">
          <h3 className="dash-card-title">Quick Actions</h3>
          <div className="quick-actions">
            <button className="quick-action" onClick={() => onNavigate('tables')}>
              <Table2 size={16} /> Open SQL Editor <ArrowRight size={12} />
            </button>
            <button className="quick-action" onClick={() => onNavigate('nosql')}>
              <FileJson size={16} /> NoSQL Browser <ArrowRight size={12} />
            </button>
            <button className="quick-action" onClick={() => onNavigate('schema')}>
              <Share2 size={16} /> View Schema Map <ArrowRight size={12} />
            </button>
            <button className="quick-action" onClick={() => onNavigate('migrations')}>
              <Play size={16} /> Run Migrations <ArrowRight size={12} />
            </button>
            <button className="quick-action" onClick={() => onNavigate('keys')}>
              <Key size={16} /> API Keys <ArrowRight size={12} />
            </button>
            <button className="quick-action" onClick={() => onNavigate('settings')}>
              <Settings size={16} /> Settings <ArrowRight size={12} />
            </button>
          </div>
        </div>

        {/* Table Overview */}
        <div className="dash-card">
          <div className="dash-card-header">
            <h3 className="dash-card-title">Table Overview</h3>
            <button className="btn btn-ghost btn-sm" onClick={() => onNavigate('tables')}>View All</button>
          </div>
          {(!data?.tableStats.length) ? (
            <div className="dash-empty">No tables yet</div>
          ) : (
            <div className="table-stats-list">
              <div className="table-stat-row header">
                <span>Table</span>
                <span>Rows</span>
              </div>
              {data.tableStats.map(t => (
                <div key={t.name} className="table-stat-row">
                  <span className="table-name">{t.name}</span>
                  <span className="table-rows">{formatNumber(t.rowCount)}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Recent Activity */}
      <div className="dash-card full-width">
        <h3 className="dash-card-title">Recent Activity</h3>
        {(!data?.recentActivity?.length) ? (
          <div className="dash-empty">No activity recorded yet</div>
        ) : (
          <div className="activity-list">
            {data.recentActivity.slice(0, 8).map((entry, i) => (
              <div key={i} className="activity-item">
                <Clock size={12} className="activity-icon" />
                <span className="activity-time">{formatTimeAgo(entry.timestamp)}</span>
                <span className="activity-action">{formatAction(entry.action)}</span>
                <span className="activity-resource">{entry.resource_type}/{entry.resource_name}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Schema Health */}
      <div className="dash-card full-width">
        <h3 className="dash-card-title">Schema Health</h3>
        <div className="health-checks">
          {hasPending ? (
            <div className="health-item warning">
              <AlertTriangle size={14} />
              <span>{data?.migrationStatus.pending_count} pending migration(s) — <a onClick={() => onNavigate('migrations')}>Review</a></span>
            </div>
          ) : (
            <div className="health-item ok">
              <CheckCircle2 size={14} />
              <span>All migrations applied</span>
            </div>
          )}
          <div className="health-item ok">
            <CheckCircle2 size={14} />
            <span>{data?.migrationStatus.applied_count ?? 0} migration(s) tracked</span>
          </div>
          {(data?.tables.length ?? 0) > 0 ? (
            <div className="health-item ok">
              <CheckCircle2 size={14} />
              <span>{data?.tables.length} table(s) in database</span>
            </div>
          ) : (
            <div className="health-item warning">
              <AlertTriangle size={14} />
              <span>Database has no tables — <a onClick={() => onNavigate('tables')}>Create one</a></span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
