import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Home,
  LayoutDashboard,
  GitBranch,
  Key,
  Settings,
  FolderOpen,
  Database,
  Github,
  Plus,
  X,
  AlertCircle,
  CheckCircle2,
  ChevronRight,
  Pin,
  LogOut,
  RefreshCw,
  Table2,
  FileJson,
  Share2,
  Bell,
} from 'lucide-react';
import { Logo } from './components/Logo';
import { TableEditor } from './components/TableEditor';
import { NoSqlBrowser } from './components/NoSqlBrowser';
import { Settings as SettingsPage } from './components/Settings';
import { SchemaMap } from './components/SchemaMap';
import Migrations from './components/Migrations';
import Dashboard from './components/Dashboard';
import CommandPalette from './components/CommandPalette';
import './components/CommandPalette.css';
import { saveAuthToken, loadAuthToken, clearAuthToken, isAuthValid } from './utils/auth-storage';
import './App.css';
import { ApiKey, AuthStatus, DeviceCode, MigrationStatus, Project, ProjectStatus, Toast, UpdateStatus } from './types';

/* ─── Types (Local or Imported) ─── */
// Most types moved to ./types/index.ts. Importing specific ones.

type Page = 'home' | 'dashboard' | 'tables' | 'nosql' | 'schema' | 'migrations' | 'keys' | 'settings' | 'login';

interface ModalState {
  open: boolean;
  type: 'api-key-name' | 'api-key-role' | 'api-key-result' | 'confirm-revoke' | null;
  data?: Record<string, string>;
}

let toastId = 0;

function App() {
  const [page, setPage] = useState<Page>('home');
  const [status, setStatus] = useState<ProjectStatus | null>(null);
  const [_migrationStatus, setMigrationStatus] = useState<MigrationStatus | null>(null);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectName, setProjectName] = useState('');
  const [loading, setLoading] = useState(false);
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [deviceCode, setDeviceCode] = useState<DeviceCode | null>(null);
  const [loginPolling, setLoginPolling] = useState(false);
  // const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  // const [showUpdateToast, setShowUpdateToast] = useState(false);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [modal, setModal] = useState<ModalState>({ open: false, type: null });
  const [modalInput, setModalInput] = useState('');
  const [pendingKeyName, setPendingKeyName] = useState('');
  const [revokeTarget, setRevokeTarget] = useState<string | null>(null);
  const [_projectType, setProjectType] = useState<string>('sql');
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false);
  const [sidebarPinned, setSidebarPinned] = useState(false);
  const [notifOpen, setNotifOpen] = useState(false);
  const [notifications, setNotifications] = useState<Array<{ id: number; type: 'info' | 'warn' | 'error'; text: string; time: string }>>([]);

  /* ─── Keyboard Shortcuts ─── */
  useEffect(() => {
    const pages: Page[] = ['home', 'dashboard', 'tables', 'nosql', 'schema', 'migrations', 'keys'];
    function handleKeyDown(e: KeyboardEvent) {
      // Ctrl+K or Ctrl+P → command palette
      if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'p')) {
        e.preventDefault();
        setCmdPaletteOpen(prev => !prev);
        return;
      }
      // Ctrl+, → settings
      if ((e.ctrlKey || e.metaKey) && e.key === ',') {
        e.preventDefault();
        setPage('settings');
        return;
      }
      // Ctrl+1-7 → navigate pages
      if ((e.ctrlKey || e.metaKey) && e.key >= '1' && e.key <= '7') {
        const idx = parseInt(e.key) - 1;
        if (pages[idx]) {
          e.preventDefault();
          setPage(pages[idx]);
        }
        return;
      }
      // Ctrl+B → toggle sidebar pin
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        setSidebarPinned(prev => !prev);
        return;
      }
      // Escape → close palette
      if (e.key === 'Escape') {
        setCmdPaletteOpen(false);
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  /* ─── Toast System ─── */
  const showToast = useCallback((type: Toast['type'], message: string) => {
    const id = ++toastId;
    setToasts(prev => [...prev, { id, type, message }]);
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, 4000);
  }, []);

  const dismissToast = useCallback((id: number) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  /* ─── Data Loading ─── */
  useEffect(() => {
    checkAuthStatus();
    loadProjects();
    checkForUpdates();
    loadProjectType();
  }, []);

  async function checkAuthStatus() {
    try {
      const result = await invoke<AuthStatus>('get_auth_status');
      if (!result.authenticated) {
        const savedAuth = await loadAuthToken();
        if (savedAuth && savedAuth.userEmail && await isAuthValid(savedAuth)) {
          setAuthStatus({ authenticated: true, username: savedAuth.userEmail });
          showToast('success', `Welcome back, ${savedAuth.userEmail}`);
        } else if (savedAuth) {
          await clearAuthToken();
        }
      } else {
        setAuthStatus(result);
      }
    } catch (e) {
      console.error('Failed to get auth status:', e);
    }
  }

  async function loadProjects() {
    try {
      const result = await invoke<Project[]>('list_projects');
      setProjects(result);
    } catch (e) {
      console.error('Failed to load projects:', e);
    }
  }

  async function checkForUpdates() {
    try {
      const result = await invoke<UpdateStatus>('check_for_updates');
      // setUpdateStatus(result);
      if (result.update_available) {
        // setShowUpdateToast(true);
        showToast('info', `Update available: ${result.latest_version}`);
      }
    } catch (e) {
      console.error('Failed to check for updates:', e);
    }
  }

  async function checkStatus() {
    try {
      const result = await invoke<ProjectStatus>('get_status');
      setStatus(result);
      if (result.initialized) {
        setPage('dashboard');
        await loadMigrationStatus();
        await loadApiKeys();
      }
    } catch (e) {
      console.error('Failed to get status:', e);
    }
  }

  async function loadMigrationStatus() {
    try {
      const result = await invoke<MigrationStatus>('get_migration_status');
      setMigrationStatus(result);
      // Auto-generate notifications for pending migrations
      if (result.pending_count > 0) {
        setNotifications(prev => {
          const exists = prev.some(n => n.text.includes('pending migration'));
          if (exists) return prev;
          return [...prev, {
            id: Date.now(),
            type: 'warn' as const,
            text: `${result.pending_count} pending migration${result.pending_count > 1 ? 's' : ''} need to be applied`,
            time: new Date().toISOString(),
          }];
        });
      }
    } catch (e) {
      console.error('Failed to get migration status:', e);
    }
  }

  async function loadProjectType() {
    try {
      const type = await invoke<string>('get_project_type');
      setProjectType(type);
    } catch (e) {
      // Project not open yet, will load when project is opened
    }
  }

  async function loadApiKeys() {
    try {
      const result = await invoke<ApiKey[]>('list_api_keys');
      setApiKeys(result);
    } catch (e) {
      console.error('Failed to load API keys:', e);
    }
  }

  /* ─── Auth ─── */
  async function handleStartLogin() {
    setLoading(true);
    try {
      const result = await invoke<DeviceCode>('start_github_login');
      setDeviceCode(result);
      setPage('login');
      window.open(result.verification_uri, '_blank');
      setLoginPolling(true);
      pollForLogin(result.device_code, result.interval);
    } catch (e) {
      showToast('error', String(e));
    } finally {
      setLoading(false);
    }
  }

  async function pollForLogin(code: string, interval: number) {
    try {
      const result = await invoke<{ success: boolean; username: string; token?: string }>('complete_github_login', {
        deviceCode: code,
        interval
      });
      if (result.success) {
        setAuthStatus({ authenticated: true, username: result.username });
        if (result.token || result.username) {
          await saveAuthToken(result.token || 'legacy_token', result.username);
        }
        setDeviceCode(null);
        setLoginPolling(false);
        setPage('home');
        showToast('success', `Signed in as ${result.username}`);
      }
    } catch (e) {
      setLoginPolling(false);
      showToast('error', String(e));
    }
  }

  async function handleLogout() {
    try {
      await invoke('github_logout');
      await clearAuthToken();
      setAuthStatus({ authenticated: false });
      showToast('info', 'Signed out of GitHub');
    } catch (e) {
      showToast('error', String(e));
    }
  }

  /* ─── Projects ─── */
  async function handleCreateProject() {
    if (!projectName.trim()) {
      showToast('error', 'Please enter a project name');
      return;
    }
    setLoading(true);
    try {
      await invoke('init_project', { name: projectName });
      await loadProjects();
      setProjectName('');
      showToast('success', `Project "${projectName}" created`);
    } catch (e) {
      showToast('error', String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleOpenProject(path: string) {
    setLoading(true);
    try {
      await invoke('open_project', { path });
      await checkStatus();
      setPage('dashboard');
    } catch (e) {
      showToast('error', String(e));
    } finally {
      setLoading(false);
    }
  }

  /* ─── API Keys (Modal Flow) ─── */
  function startCreateApiKey() {
    setModalInput('');
    setPendingKeyName('');
    setModal({ open: true, type: 'api-key-name' });
  }

  function handleModalSubmitKeyName() {
    if (!modalInput.trim()) return;
    setPendingKeyName(modalInput);
    setModalInput('readonly');
    setModal({ open: true, type: 'api-key-role' });
  }

  async function handleModalSubmitKeyRole() {
    if (!modalInput.trim()) return;
    setModal({ open: false, type: null });
    try {
      const result = await invoke<{ key: string }>('create_api_key', {
        name: pendingKeyName,
        role: modalInput
      });
      setModal({
        open: true,
        type: 'api-key-result',
        data: { key: result.key }
      });
      await loadApiKeys();
    } catch (e) {
      showToast('error', String(e));
    }
  }

  function startRevokeKey(keyId: string) {
    setRevokeTarget(keyId);
    setModal({ open: true, type: 'confirm-revoke' });
  }

  async function confirmRevokeKey() {
    if (!revokeTarget) return;
    setModal({ open: false, type: null });
    try {
      await invoke('revoke_api_key', { keyId: revokeTarget });
      await loadApiKeys();
      showToast('success', 'API key revoked');
    } catch (e) {
      showToast('error', String(e));
    }
    setRevokeTarget(null);
  }

  /* ─── Page Labels ─── */
  const pageLabels: Record<Page, string> = {
    home: 'Home',
    dashboard: 'Dashboard',
    tables: 'SQL Tables',
    nosql: 'NoSQL Collections',
    schema: 'Schema Map',
    migrations: 'Migrations',
    keys: 'API Keys',
    settings: 'Settings',
    login: 'Login',
  };

  return (
    <div className="app">
      {/* ── Command Palette ── */}
      <CommandPalette
        open={cmdPaletteOpen}
        onClose={() => setCmdPaletteOpen(false)}
        onNavigate={(p) => { setPage(p as Page); setCmdPaletteOpen(false); }}
        projectOpen={!!status?.initialized}
      />

      {/* ── Sidebar ── */}
      <nav className={`sidebar ${sidebarPinned ? 'pinned' : ''}`}>
        <button
          className={`sidebar-pin ${sidebarPinned ? 'active' : ''}`}
          onClick={() => setSidebarPinned(p => !p)}
          title={sidebarPinned ? 'Unpin sidebar (Ctrl+B)' : 'Pin sidebar open (Ctrl+B)'}
        >
          <Pin size={13} />
        </button>
        <div className="sidebar-logo">
          <Logo size={22} showText={false} />
          <span className="sidebar-logo-text">AirDB</span>
        </div>

        <ul className="nav-links">
          <li>
            <button
              className={`nav-item ${page === 'home' ? 'active' : ''}`}
              onClick={() => setPage('home')}
            >
              <Home size={20} />
              <span className="nav-label">Home</span>
            </button>
          </li>
          <li>
            <button
              className={`nav-item ${page === 'dashboard' ? 'active' : ''}`}
              onClick={() => setPage('dashboard')}
              disabled={!status?.initialized}
            >
              <LayoutDashboard size={20} />
              <span className="nav-label">Dashboard</span>
            </button>
          </li>
          <li>
            <button
              className={`nav-item ${page === 'tables' ? 'active' : ''}`}
              onClick={() => setPage('tables')}
              disabled={!status?.initialized}
            >
              <Table2 size={20} />
              <span className="nav-label">SQL Tables</span>
            </button>
          </li>
          <li>
            <button
              className={`nav-item ${page === 'nosql' ? 'active' : ''}`}
              onClick={() => setPage('nosql')}
              disabled={!status?.initialized}
            >
              <FileJson size={20} />
              <span className="nav-label">NoSQL</span>
            </button>
          </li>
          <li>
            <button
              className={`nav-item ${page === 'schema' ? 'active' : ''}`}
              onClick={() => setPage('schema')}
              disabled={!status?.initialized}
            >
              <Share2 size={20} />
              <span className="nav-label">Schema Map</span>
            </button>
          </li>
          <li>
            <button
              className={`nav-item ${page === 'migrations' ? 'active' : ''}`}
              onClick={() => setPage('migrations')}
              disabled={!status?.initialized}
            >
              <GitBranch size={20} />
              <span className="nav-label">Migrations</span>
            </button>
          </li>
          <li>
            <button
              className={`nav-item ${page === 'keys' ? 'active' : ''}`}
              onClick={() => setPage('keys')}
              disabled={!status?.initialized}
            >
              <Key size={20} />
              <span className="nav-label">API Keys</span>
            </button>
          </li>

          <hr className="nav-separator" />

          <li>
            <button
              className={`nav-item ${page === 'settings' ? 'active' : ''}`}
              onClick={() => setPage('settings')}
            >
              <Settings size={20} />
              <span className="nav-label">Settings</span>
            </button>
          </li>
        </ul>

        <div className="nav-footer">
          {authStatus?.authenticated ? (
            <button className="nav-item" onClick={handleLogout}>
              <LogOut size={20} />
              <span className="nav-label">Sign Out</span>
            </button>
          ) : (
            <button className="nav-item" onClick={handleStartLogin} disabled={loading}>
              <Github size={20} />
              <span className="nav-label">Sign In</span>
            </button>
          )}
        </div>
      </nav>

      {/* Spacer reserves the collapsed sidebar width in document flow */}
      <div className="sidebar-spacer" />

      {/* ── Main ── */}
      <div className="main-wrapper">
        {/* Top Bar */}
        <header className="topbar">
          <div className="topbar-left">
            <div className="breadcrumb">
              <span>AirDB</span>
              <ChevronRight size={14} className="breadcrumb-sep" />
              <span className="breadcrumb-current">{pageLabels[page]}</span>
            </div>
          </div>
          <div className="topbar-right">
            {/* Notification bell */}
            <button className="topbar-icon-btn" onClick={() => setNotifOpen(o => !o)} title="Notifications">
              <Bell size={16} />
              {notifications.length > 0 && <span className="notif-badge">{notifications.length}</span>}
            </button>
            {notifOpen && (
              <div className="notif-drawer">
                <div className="notif-drawer-header">
                  <span>Notifications</span>
                  {notifications.length > 0 && (
                    <button className="btn btn-ghost btn-xs" onClick={() => setNotifications([])}>Clear all</button>
                  )}
                </div>
                {notifications.length === 0 ? (
                  <div className="notif-empty">No notifications</div>
                ) : (
                  <ul className="notif-list">
                    {notifications.map(n => (
                      <li key={n.id} className={`notif-item notif-${n.type}`}>
                        <span className="notif-text">{n.text}</span>
                        <button className="notif-dismiss" onClick={() => setNotifications(prev => prev.filter(x => x.id !== n.id))}>×</button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
            {authStatus?.authenticated && (
              <div className="user-indicator" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <div style={{
                  width: '8px',
                  height: '8px',
                  borderRadius: '50%',
                  backgroundColor: 'var(--success)'
                }}></div>
                <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                  {authStatus.username}
                </span>
              </div>
            )}
          </div>
        </header>

        {/* Content */}
        <main className="content">
          {/* ── LOGIN ── */}
          {page === 'login' && deviceCode && (
            <div className="page">
              <div className="card-glass" style={{ maxWidth: '400px', margin: '40px auto', textAlign: 'center' }}>
                <div className="page-header">
                  <Github size={32} style={{ color: 'var(--text-primary)', marginBottom: 12 }} />
                  <h2 className="page-title">GitHub Authorization</h2>
                  <p className="page-subtitle">Complete sign-in in your browser</p>
                </div>

                <div style={{
                  background: 'var(--surface-2)',
                  padding: '16px',
                  borderRadius: 'var(--radius-md)',
                  margin: '20px 0',
                  fontFamily: 'var(--font-mono)',
                  fontSize: '24px',
                  letterSpacing: '2px'
                }}>
                  {deviceCode.user_code}
                </div>

                <p style={{ marginBottom: '20px' }}>
                  <a
                    href={deviceCode.verification_uri}
                    target="_blank"
                    rel="noopener"
                    style={{ color: 'var(--accent)', textDecoration: 'none' }}
                  >
                    Click here to open GitHub
                  </a>
                </p>

                {loginPolling && (
                  <div style={{ color: 'var(--text-tertiary)', fontSize: '12px' }}>
                    Waiting for authorization...
                  </div>
                )}

                <button
                  className="btn btn-ghost"
                  onClick={() => { setPage('home'); setDeviceCode(null); }}
                  style={{ marginTop: '20px' }}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {/* ── HOME ── */}
          {page === 'home' && (
            <div className="page">
              <div className="welcome-hero">
                <Logo size="lg" showText={true} />
                <p className="welcome-subtitle" style={{ marginTop: 12 }}>
                  Local-first, GitHub-backed database platform
                </p>
              </div>

              <div className="home-grid">
                <div className="card-glass">
                  <div className="card-header">
                    <h3 className="card-title">New Project</h3>
                    <Plus size={16} style={{ color: 'var(--text-tertiary)' }} />
                  </div>
                  <div style={{ marginBottom: '16px' }}>
                    <input
                      className="input"
                      type="text"
                      value={projectName}
                      onChange={(e) => setProjectName(e.target.value)}
                      placeholder="my-project"
                      onKeyDown={(e) => e.key === 'Enter' && handleCreateProject()}
                    />
                  </div>
                  <button
                    className="btn btn-primary"
                    onClick={handleCreateProject}
                    disabled={loading}
                    style={{ width: '100%' }}
                  >
                    {loading ? <><RefreshCw size={14} className="spin" /> Creating...</> : 'Create Project'}
                  </button>
                </div>

                <div className="card-glass">
                  <div className="card-header">
                    <h3 className="card-title">Your Projects</h3>
                    <FolderOpen size={16} style={{ color: 'var(--text-tertiary)' }} />
                  </div>
                  {projects.length === 0 ? (
                    <div className="empty-state">
                      <Database size={32} className="empty-icon" />
                      <p className="empty-text">No projects yet</p>
                    </div>
                  ) : (
                    <ul style={{ listStyle: 'none' }}>
                      {projects.map((p) => (
                        <li key={p.path} style={{
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'center',
                          padding: '8px 0',
                          borderBottom: '1px solid var(--surface-3)'
                        }}>
                          <span style={{ fontWeight: 500 }}>{p.name}</span>
                          <button
                            className="btn btn-sm btn-ghost"
                            onClick={() => handleOpenProject(p.path)}
                          >
                            Open
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* ── DASHBOARD ── */}
          {page === 'dashboard' && status?.initialized && (
            <div className="page page-fullheight">
              <Dashboard
                projectName={status.project_name ?? 'Project'}
                dbType={status.db_type || 'SQLite'}
                apiPort={status.api_port ?? 54321}
                onNavigate={(p) => setPage(p as Page)}
              />
            </div>
          )}

          {/* ── SQL TABLES ── */}
          {page === 'tables' && status?.initialized && (
            <div className="page page-fullheight">
              <TableEditor />
            </div>
          )}

          {/* ── NOSQL COLLECTIONS ── */}
          {page === 'nosql' && status?.initialized && (
            <div className="page page-fullheight">
              <NoSqlBrowser />
            </div>
          )}

          {/* ── SCHEMA MAP (ER Diagram) ── */}
          {page === 'schema' && status?.initialized && (
            <div className="page page-fullheight">
              <SchemaMap onNavigateToTable={() => {
                setPage('tables');
                // The table selection is internal to TableEditor
              }} />
            </div>
          )}

          {/* ── SETTINGS ── */}
          {page === 'settings' && (
            <SettingsPage />
          )}

          {/* ── MIGRATIONS ── */}
          {page === 'migrations' && status?.initialized && (
            <div className="page page-fullheight">
              <Migrations />
            </div>
          )}

          {/* ── API KEYS ── */}
          {page === 'keys' && (
            <div className="page">
              <div className="page-header">
                <h1 className="page-title">API Keys</h1>
              </div>

              <div className="card">
                <div className="card-header">
                  <h3 className="card-title">Your Keys</h3>
                  <button className="btn btn-primary btn-sm" onClick={startCreateApiKey}>
                    <Plus size={12} /> Create Key
                  </button>
                </div>

                {apiKeys.length === 0 ? (
                  <div className="empty-state">
                    <Key size={32} className="empty-icon" />
                    <p className="empty-text">No API keys yet. Create one to get started.</p>
                  </div>
                ) : (
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Name</th>
                        <th>Role</th>
                        <th>Created</th>
                        <th style={{ textAlign: 'right' }}>Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {apiKeys.map((key) => (
                        <tr key={key.id}>
                          <td>{key.name}</td>
                          <td>
                            <span className={`badge ${key.role === 'admin' ? 'badge-danger' :
                              key.role === 'developer' ? 'badge-accent' :
                                'badge-muted'
                              }`}>
                              {key.role}
                            </span>
                          </td>
                          <td style={{ color: 'var(--text-secondary)' }}>
                            {new Date(key.created_at).toLocaleDateString()}
                          </td>
                          <td style={{ textAlign: 'right' }}>
                            <button
                              className="btn btn-danger btn-sm"
                              onClick={() => startRevokeKey(key.id)}
                            >
                              Revoke
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </div>
          )}

        </main>

        {/* Toast Messages */}
        <div className="toast-container">
          {toasts.map(toast => (
            <div key={toast.id} className={`toast toast-${toast.type}`}>
              {toast.type === 'success' ? <CheckCircle2 size={16} /> :
                toast.type === 'error' ? <AlertCircle size={16} /> :
                  <AlertCircle size={16} />}
              <span>{toast.message}</span>
              <button
                onClick={() => dismissToast(toast.id)}
                style={{ background: 'none', border: 'none', color: 'inherit', cursor: 'pointer', marginLeft: 'auto' }}
              >
                <X size={14} />
              </button>
            </div>
          ))}
        </div>

        {/* ── MODALS ── */}
        {modal.open && (
          <div className="modal-overlay">
            <div className="modal">
              {modal.type === 'api-key-name' && (
                <>
                  <div className="modal-header">
                    <h3>Create API Key</h3>
                    <button onClick={() => setModal({ open: false, type: null })}><X size={18} /></button>
                  </div>
                  <div className="modal-body">
                    <p style={{ marginBottom: 12 }}>Enter a names for this key:</p>
                    <input
                      className="input"
                      value={modalInput}
                      onChange={e => setModalInput(e.target.value)}
                      placeholder="e.g. mobile-app-prod"
                      autoFocus
                    />
                  </div>
                  <div className="modal-footer">
                    <button className="btn btn-ghost" onClick={() => setModal({ open: false, type: null })}>Cancel</button>
                    <button className="btn btn-primary" onClick={handleModalSubmitKeyName}>Next</button>
                  </div>
                </>
              )}

              {modal.type === 'api-key-role' && (
                <>
                  <div className="modal-header">
                    <h3>Select Role</h3>
                    <button onClick={() => setModal({ open: false, type: null })}><X size={18} /></button>
                  </div>
                  <div className="modal-body">
                    <p style={{ marginBottom: 12 }}>Select permissions for <strong>{pendingKeyName}</strong>:</p>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                      {['read-only', 'developer', 'admin'].map(role => (
                        <label key={role} style={{
                          display: 'flex',
                          alignItems: 'center',
                          padding: '12px',
                          border: `1px solid ${modalInput === role ? 'var(--accent)' : 'var(--surface-3)'}`,
                          borderRadius: 'var(--radius-md)',
                          cursor: 'pointer',
                          background: modalInput === role ? 'var(--surface-2)' : 'transparent'
                        }}>
                          <input
                            type="radio"
                            name="role"
                            value={role}
                            checked={modalInput === role}
                            onChange={e => setModalInput(e.target.value)}
                            style={{ marginRight: 12 }}
                          />
                          <span style={{ textTransform: 'capitalize' }}>{role}</span>
                        </label>
                      ))}
                    </div>
                  </div>
                  <div className="modal-footer">
                    <button className="btn btn-ghost" onClick={() => setModal({ open: false, type: null })}>Cancel</button>
                    <button className="btn btn-primary" onClick={handleModalSubmitKeyRole}>Create Key</button>
                  </div>
                </>
              )}

              {modal.type === 'api-key-result' && (
                <>
                  <div className="modal-header">
                    <h3>API Key Created</h3>
                    <button onClick={() => setModal({ open: false, type: null })}><X size={18} /></button>
                  </div>
                  <div className="modal-body">
                    <div className="alert alert-success">
                      <CheckCircle2 size={16} /> Key created successfully
                    </div>
                    <p style={{ margin: '16px 0 8px' }}>Copy this key now. You won't see it again.</p>
                    <div style={{
                      background: 'var(--surface-3)',
                      padding: '12px',
                      borderRadius: 'var(--radius-md)',
                      fontFamily: 'var(--font-mono)',
                      wordBreak: 'break-all',
                      border: '1px solid var(--border)'
                    }}>
                      {modal.data?.key}
                    </div>
                  </div>
                  <div className="modal-footer">
                    <button className="btn btn-primary" onClick={() => setModal({ open: false, type: null })}>Done</button>
                  </div>
                </>
              )}

              {modal.type === 'confirm-revoke' && (
                <>
                  <div className="modal-header">
                    <h3>Revoke API Key</h3>
                    <button onClick={() => setModal({ open: false, type: null })}><X size={18} /></button>
                  </div>
                  <div className="modal-body">
                    <p>Are you sure you want to revoke this API key? This action cannot be undone and any applications using this key will stop working.</p>
                  </div>
                  <div className="modal-footer">
                    <button className="btn btn-ghost" onClick={() => setModal({ open: false, type: null })}>Cancel</button>
                    <button className="btn btn-danger" onClick={confirmRevokeKey}>Revoke Key</button>
                  </div>
                </>
              )}
            </div>
          </div>
        )}

      </div>
    </div>
  );
}

export default App;
