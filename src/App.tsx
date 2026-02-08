import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TableEditor } from './components/TableEditor';
import { UpdateSettings } from './components/UpdateSettings';
import { UpdateBanner } from './components/UpdateBanner';
import './App.css';

interface ProjectStatus {
  initialized: boolean;
  project_name?: string;
  db_type?: string;
  api_port?: number;
}

interface MigrationStatus {
  applied_count: number;
  pending_count: number;
  pending: string[];
}

interface ApiKey {
  id: string;
  name: string;
  role: string;
  created_at: string;
}

interface Project {
  name: string;
  path: string;
  configured: boolean;
}

interface AuthStatus {
  authenticated: boolean;
  username?: string;
}

interface DeviceCode {
  user_code: string;
  verification_uri: string;
  device_code: string;
  expires_in: number;
  interval: number;
}

interface UpdateStatus {
  current_version: string;
  update_available: boolean;
  latest_version: string;
  channel: string;
  pending_version?: string;
}

type Page = 'home' | 'dashboard' | 'tables' | 'migrations' | 'keys' | 'settings' | 'updates' | 'login';

function App() {
  const [page, setPage] = useState<Page>('home');
  const [status, setStatus] = useState<ProjectStatus | null>(null);
  const [migrationStatus, setMigrationStatus] = useState<MigrationStatus | null>(null);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectName, setProjectName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [deviceCode, setDeviceCode] = useState<DeviceCode | null>(null);
  const [loginPolling, setLoginPolling] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);

  useEffect(() => {
    checkAuthStatus();
    loadProjects();
    checkForUpdates();
  }, []);

  async function checkAuthStatus() {
    try {
      const result = await invoke<AuthStatus>('get_auth_status');
      setAuthStatus(result);
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
      setUpdateStatus(result);
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
    } catch (e) {
      console.error('Failed to get migration status:', e);
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

  async function handleStartLogin() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DeviceCode>('start_github_login');
      setDeviceCode(result);
      setPage('login');

      // Open browser (use window.open as fallback)
      window.open(result.verification_uri, '_blank');

      // Start polling
      setLoginPolling(true);
      pollForLogin(result.device_code, result.interval);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function pollForLogin(code: string, interval: number) {
    try {
      const result = await invoke<{ success: boolean; username: string }>('complete_github_login', {
        deviceCode: code,
        interval
      });
      if (result.success) {
        setAuthStatus({ authenticated: true, username: result.username });
        setDeviceCode(null);
        setLoginPolling(false);
        setPage('home');
      }
    } catch (e) {
      // Still polling or error
      console.log('Login polling:', e);
      setLoginPolling(false);
      setError(String(e));
    }
  }

  async function handleLogout() {
    try {
      await invoke('github_logout');
      setAuthStatus({ authenticated: false });
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleCreateProject() {
    if (!projectName.trim()) {
      setError('Please enter a project name');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      await invoke('init_project', { name: projectName });
      await loadProjects();
      setProjectName('');
    } catch (e) {
      setError(String(e));
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
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleRunMigrations() {
    setLoading(true);
    setError(null);
    try {
      const applied = await invoke<string[]>('run_migrations');
      if (applied.length > 0) {
        alert(`Applied ${applied.length} migration(s)`);
      } else {
        alert('No pending migrations');
      }
      await loadMigrationStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleCreateApiKey() {
    const name = prompt('API Key name:');
    if (!name) return;

    const role = prompt('Role (admin/developer/readonly):', 'readonly');
    if (!role) return;

    try {
      const result = await invoke<{ key: string }>('create_api_key', { name, role });
      alert(`API Key created!\n\nKey (save this):\n${result.key}`);
      await loadApiKeys();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRevokeKey(keyId: string) {
    if (!confirm('Revoke this API key?')) return;
    try {
      await invoke('revoke_api_key', { keyId });
      await loadApiKeys();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="app">
      {/* Update Banner */}
      <UpdateBanner />

      {/* Version indicator (always visible) */}
      <div className="version-badge">
        v{updateStatus?.current_version || '0.1.0'}
      </div>

      <nav className="sidebar">
        <div className="logo">
          <h2>✨ AirDB</h2>
        </div>
        <ul className="nav-links">
          <li className={page === 'home' ? 'active' : ''}>
            <button onClick={() => setPage('home')}>
              🏠 Home
            </button>
          </li>
          <li className={page === 'dashboard' ? 'active' : ''}>
            <button onClick={() => setPage('dashboard')} disabled={!status?.initialized}>
              📊 Dashboard
            </button>
          </li>
          <li className={page === 'tables' ? 'active' : ''}>
            <button onClick={() => setPage('tables')} disabled={!status?.initialized}>
              📋 Tables
            </button>
          </li>
          <li className={page === 'migrations' ? 'active' : ''}>
            <button onClick={() => setPage('migrations')} disabled={!status?.initialized}>
              🔄 Migrations
            </button>
          </li>
          <li className={page === 'keys' ? 'active' : ''}>
            <button onClick={() => setPage('keys')} disabled={!status?.initialized}>
              🔑 API Keys
            </button>
          </li>
          <li className={page === 'settings' ? 'active' : ''}>
            <button onClick={() => setPage('settings')}>
              ⚙️ Settings
            </button>
          </li>
        </ul>
        <div className="nav-footer">
          {authStatus?.authenticated ? (
            <div className="user-info">
              <span className="user-badge">🟢 {authStatus.username || 'Connected'}</span>
            </div>
          ) : (
            <button className="btn-link" onClick={handleStartLogin} disabled={loading}>
              🔐 Login with GitHub
            </button>
          )}
          <span className="version">v0.1.0</span>
        </div>
      </nav>

      <main className="content">
        {error && (
          <div className="error-banner">
            {error}
            <button onClick={() => setError(null)}>×</button>
          </div>
        )}

        {page === 'login' && deviceCode && (
          <div className="page login">
            <div className="card login-card">
              <h2>🔐 GitHub Login</h2>
              <p>Complete authorization in your browser</p>
              <div className="code-display">
                <span className="label">Enter this code:</span>
                <span className="code">{deviceCode.user_code}</span>
              </div>
              <p className="url">
                <a href={deviceCode.verification_uri} target="_blank" rel="noopener">
                  {deviceCode.verification_uri}
                </a>
              </p>
              {loginPolling && (
                <div className="polling">
                  <span className="spinner"></span>
                  Waiting for authorization...
                </div>
              )}
              <button className="btn" onClick={() => { setPage('home'); setDeviceCode(null); }}>
                Cancel
              </button>
            </div>
          </div>
        )}

        {page === 'home' && (
          <div className="page home">
            <h1>Welcome to AirDB</h1>
            <p className="subtitle">Local-first, GitHub-backed database platform</p>

            <div className="home-grid">
              <div className="card">
                <h3>➕ Create New Project</h3>
                <div className="form-group">
                  <input
                    type="text"
                    value={projectName}
                    onChange={(e) => setProjectName(e.target.value)}
                    placeholder="my-project"
                  />
                </div>
                <button
                  className="btn primary"
                  onClick={handleCreateProject}
                  disabled={loading}
                >
                  {loading ? 'Creating...' : 'Create Project'}
                </button>
              </div>

              <div className="card">
                <h3>📁 Your Projects</h3>
                {projects.length === 0 ? (
                  <p className="empty">No projects yet</p>
                ) : (
                  <ul className="project-list">
                    {projects.map((p) => (
                      <li key={p.path}>
                        <span className="project-name">{p.name}</span>
                        <button
                          className="btn small"
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

            {!authStatus?.authenticated && (
              <div className="card github-cta">
                <h3>🔗 Connect GitHub</h3>
                <p>Enable cloud sync, collaboration, and version control</p>
                <button className="btn primary" onClick={handleStartLogin} disabled={loading}>
                  {loading ? 'Connecting...' : 'Login with GitHub'}
                </button>
              </div>
            )}
          </div>
        )}

        {page === 'dashboard' && status?.initialized && (
          <div className="page dashboard">
            <h1>Dashboard</h1>
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-icon">📁</div>
                <div className="stat-info">
                  <h3>{status.project_name}</h3>
                  <p>Project Name</p>
                </div>
              </div>
              <div className="stat-card">
                <div className="stat-icon">🗄️</div>
                <div className="stat-info">
                  <h3>{status.db_type}</h3>
                  <p>Database Type</p>
                </div>
              </div>
              <div className="stat-card">
                <div className="stat-icon">🌐</div>
                <div className="stat-info">
                  <h3>{status.api_port}</h3>
                  <p>API Port</p>
                </div>
              </div>
              <div className="stat-card">
                <div className="stat-icon">🔄</div>
                <div className="stat-info">
                  <h3>{migrationStatus?.applied_count ?? 0}</h3>
                  <p>Applied Migrations</p>
                </div>
              </div>
            </div>

            <div className="card">
              <h3>Quick Actions</h3>
              <div className="actions">
                <button className="btn" onClick={() => setPage('migrations')}>
                  View Migrations
                </button>
                <button className="btn" onClick={() => setPage('keys')}>
                  Manage API Keys
                </button>
                <button className="btn" onClick={() => setPage('home')}>
                  ← Back to Home
                </button>
              </div>
            </div>
          </div>
        )}

        {page === 'migrations' && (
          <div className="page migrations">
            <h1>Migrations</h1>
            <div className="card">
              <div className="card-header">
                <h3>Migration Status</h3>
                <button
                  className="btn primary"
                  onClick={handleRunMigrations}
                  disabled={loading || (migrationStatus?.pending_count ?? 0) === 0}
                >
                  {loading ? 'Running...' : 'Run Migrations'}
                </button>
              </div>
              <div className="migration-info">
                <p><strong>Applied:</strong> {migrationStatus?.applied_count ?? 0}</p>
                <p><strong>Pending:</strong> {migrationStatus?.pending_count ?? 0}</p>
              </div>
              {(migrationStatus?.pending?.length ?? 0) > 0 && (
                <div className="pending-list">
                  <h4>Pending Migrations:</h4>
                  <ul>
                    {migrationStatus?.pending.map((m) => (
                      <li key={m}>{m}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
            <div className="card">
              <h3>Create Migration</h3>
              <p>Use the CLI to create new migrations:</p>
              <pre><code>airdb migrate create my_migration_name</code></pre>
            </div>
          </div>
        )}

        {page === 'keys' && (
          <div className="page keys">
            <h1>API Keys</h1>
            <div className="card">
              <div className="card-header">
                <h3>Your API Keys</h3>
                <button className="btn primary" onClick={handleCreateApiKey}>
                  + Create Key
                </button>
              </div>
              {apiKeys.length === 0 ? (
                <p className="empty">No API keys yet. Create one to get started.</p>
              ) : (
                <table className="keys-table">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Role</th>
                      <th>Created</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {apiKeys.map((key) => (
                      <tr key={key.id}>
                        <td>{key.name}</td>
                        <td><span className={`role-badge ${key.role}`}>{key.role}</span></td>
                        <td>{new Date(key.created_at).toLocaleDateString()}</td>
                        <td>
                          <button
                            className="btn small danger"
                            onClick={() => handleRevokeKey(key.id)}
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

        {page === 'tables' && (
          <div className="page tables-page">
            <h1>Schema Editor</h1>
            <TableEditor />
          </div>
        )}

        {page === 'settings' && (
          <div className="page settings">
            <h1>Settings</h1>
            <div className="card">
              <h3>GitHub Connection</h3>
              {authStatus?.authenticated ? (
                <div className="settings-row">
                  <div>
                    <p><strong>Status:</strong> Connected</p>
                    <p><strong>User:</strong> {authStatus.username || 'Unknown'}</p>
                  </div>
                  <button className="btn danger" onClick={handleLogout}>
                    Disconnect
                  </button>
                </div>
              ) : (
                <div className="settings-row">
                  <p>Not connected to GitHub</p>
                  <button className="btn primary" onClick={handleStartLogin}>
                    Connect GitHub
                  </button>
                </div>
              )}
            </div>

            <UpdateSettings />

            <div className="card">
              <h3>System</h3>
              <div className="settings-row">
                <div>
                  <p><strong>CLI PATH Status</strong></p>
                  <p className="text-muted">Check if airdb is in your system PATH</p>
                </div>
                <button
                  className="btn"
                  onClick={async () => {
                    try {
                      await invoke('add_to_path');
                      alert('Added to PATH!');
                    } catch (e) {
                      alert('Error: ' + e);
                    }
                  }}
                >
                  Add to PATH
                </button>
              </div>
            </div>

            <div className="card">
              <h3>About</h3>
              <p>AirDB v0.1.0</p>
              <p>Local-first, GitHub-backed database platform</p>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
