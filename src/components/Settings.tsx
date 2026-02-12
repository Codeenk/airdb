import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Monitor,
    Moon,
    Sun,
    Info,
    Github,
    Database,
    Code,
    GitBranch,
    RefreshCw,
    Globe,
    Shield,
    Plus,
    Trash2,
    Check,
    X,
    TestTube,
    Wifi,
    WifiOff,
    Server,
} from 'lucide-react';
import type { ConnectionConfig } from '../types';
import './Settings.css';

type SettingsTab = 'general' | 'connections' | 'editor' | 'sync' | 'updates' | 'api' | 'security' | 'about';

interface AppSettings {
    theme: string;
    fontSize: number;
    fontFamily: string;
    autoSave: boolean;
    tabSize: number;
    wordWrap: boolean;
    lineNumbers: boolean;
    defaultLimit: number;
    syncEnabled: boolean;
    syncInterval: number;
    updateChannel: string;
    autoCheckUpdates: boolean;
    apiEnabled: boolean;
    apiPort: number;
    apiAutoStart: boolean;
    apiCorsOrigins: string;
    apiRateLimit: number;
}

const defaultSettings: AppSettings = {
    theme: 'void-cyan',
    fontSize: 14,
    fontFamily: 'JetBrains Mono',
    autoSave: true,
    tabSize: 2,
    wordWrap: true,
    lineNumbers: true,
    defaultLimit: 100,
    syncEnabled: false,
    syncInterval: 300,
    updateChannel: 'stable',
    autoCheckUpdates: true,
    apiEnabled: false,
    apiPort: 54321,
    apiAutoStart: false,
    apiCorsOrigins: '*',
    apiRateLimit: 100,
};

export function Settings() {
    const [tab, setTab] = useState<SettingsTab>('general');
    const [settings, setSettings] = useState<AppSettings>(defaultSettings);
    const [connections, setConnections] = useState<ConnectionConfig[]>([]);
    const [showNewConn, setShowNewConn] = useState(false);
    const [testResult, setTestResult] = useState<{ ok: boolean; msg: string } | null>(null);
    const [apiRunning, setApiRunning] = useState(false);
    const [apiMsg, setApiMsg] = useState<string | null>(null);
    const [newConn, setNewConn] = useState({
        name: '',
        dialect: 'sqlite' as 'sqlite' | 'postgres' | 'mysql',
        path: '',
        host: 'localhost',
        port: 5432,
        database: '',
        username: '',
        password: '',
        ssl: false,
    });

    const loadConnections = useCallback(async () => {
        try {
            const conns = await invoke<ConnectionConfig[]>('list_connections');
            setConnections(conns);
        } catch { /* ignore if no project open */ }
    }, []);

    useEffect(() => { loadConnections(); }, [loadConnections]);

    // Check API server status on mount
    useEffect(() => {
        invoke<{ running: boolean; port: number | null }>('get_api_server_status')
            .then(s => setApiRunning(s.running))
            .catch(() => {});
    }, []);

    const handleToggleApiServer = async () => {
        try {
            if (apiRunning) {
                await invoke<string>('stop_api_server');
                setApiRunning(false);
                setApiMsg('Server stopped');
            } else {
                const msg = await invoke<string>('start_api_server', { port: settings.apiPort });
                setApiRunning(true);
                setApiMsg(msg);
            }
            setTimeout(() => setApiMsg(null), 4000);
        } catch (e) {
            setApiMsg(String(e));
            setTimeout(() => setApiMsg(null), 4000);
        }
    };

    const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
        setSettings(prev => ({ ...prev, [key]: value }));
    };

    const handleAddConnection = async () => {
        const config = newConn.dialect === 'sqlite'
            ? { type: 'sqlite' as const, path: newConn.path }
            : newConn.dialect === 'postgres'
                ? { type: 'postgres' as const, host: newConn.host, port: newConn.port, database: newConn.database, username: newConn.username, password: newConn.password, sslMode: newConn.ssl ? 'require' : 'disable' }
                : { type: 'mysql' as const, host: newConn.host, port: newConn.port, database: newConn.database, username: newConn.username, password: newConn.password, ssl: newConn.ssl };
        try {
            await invoke('add_connection', { name: newConn.name, dialect: newConn.dialect, config });
            setShowNewConn(false);
            setNewConn({ name: '', dialect: 'sqlite', path: '', host: 'localhost', port: 5432, database: '', username: '', password: '', ssl: false });
            await loadConnections();
        } catch (err: any) { console.error(err); }
    };

    const handleRemoveConnection = async (id: string) => {
        try { await invoke('remove_connection', { id }); await loadConnections(); } catch (err: any) { console.error(err); }
    };

    const handleTestConnection = async () => {
        try {
            const config = newConn.dialect === 'sqlite'
                ? { type: 'sqlite' as const, path: newConn.path }
                : newConn.dialect === 'postgres'
                    ? { type: 'postgres' as const, host: newConn.host, port: newConn.port, database: newConn.database, username: newConn.username, password: newConn.password, sslMode: newConn.ssl ? 'require' : 'disable' }
                    : { type: 'mysql' as const, host: newConn.host, port: newConn.port, database: newConn.database, username: newConn.username, password: newConn.password, ssl: newConn.ssl };
            await invoke('test_connection', { dialect: newConn.dialect, config });
            setTestResult({ ok: true, msg: 'Connection successful!' });
        } catch (err: any) { setTestResult({ ok: false, msg: err?.toString() || 'Connection failed' }); }
        setTimeout(() => setTestResult(null), 4000);
    };

    const tabs: { key: SettingsTab; label: string; icon: React.ReactNode }[] = [
        { key: 'general', label: 'General', icon: <Monitor size={16} /> },
        { key: 'connections', label: 'Connections', icon: <Database size={16} /> },
        { key: 'editor', label: 'Editor', icon: <Code size={16} /> },
        { key: 'sync', label: 'Sync', icon: <GitBranch size={16} /> },
        { key: 'updates', label: 'Updates', icon: <RefreshCw size={16} /> },
        { key: 'api', label: 'API', icon: <Globe size={16} /> },
        { key: 'security', label: 'Security', icon: <Shield size={16} /> },
        { key: 'about', label: 'About', icon: <Info size={16} /> },
    ];

    return (
        <div className="settings-page">
            <div className="page-header">
                <h1 className="page-title">Settings</h1>
                <p className="page-subtitle">Customize your AirDB experience</p>
            </div>
            <div className="settings-layout">
                <div className="settings-tabs">
                    {tabs.map(t => (
                        <button key={t.key} className={`settings-tab ${tab === t.key ? 'active' : ''}`} onClick={() => setTab(t.key)}>
                            {t.icon}<span>{t.label}</span>
                        </button>
                    ))}
                </div>
                <div className="settings-content">
                    {tab === 'general' && (
                        <div className="settings-section">
                            <h2 className="section-title">Appearance</h2>
                            <div className="setting-card">
                                <SettingRow label="Theme" desc="Select your color theme">
                                    <div className="theme-toggle">
                                        <button className={`theme-btn ${settings.theme === 'void-cyan' ? 'active' : ''}`} onClick={() => updateSetting('theme', 'void-cyan')}><Moon size={14} /> Void Cyan</button>
                                        <button className={`theme-btn ${settings.theme === 'midnight' ? 'active' : ''}`} onClick={() => updateSetting('theme', 'midnight')}><Moon size={14} /> Midnight</button>
                                        <button className="theme-btn" disabled><Sun size={14} /> Light</button>
                                    </div>
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Font Size" desc="Code editor font size">
                                    <select className="setting-select" value={settings.fontSize} onChange={e => updateSetting('fontSize', Number(e.target.value))}>
                                        {[12, 13, 14, 15, 16, 18, 20].map(s => <option key={s} value={s}>{s}px</option>)}
                                    </select>
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Font Family" desc="Code editor font">
                                    <select className="setting-select" value={settings.fontFamily} onChange={e => updateSetting('fontFamily', e.target.value)}>
                                        <option value="JetBrains Mono">JetBrains Mono</option>
                                        <option value="Fira Code">Fira Code</option>
                                        <option value="Source Code Pro">Source Code Pro</option>
                                        <option value="monospace">System Monospace</option>
                                    </select>
                                </SettingRow>
                            </div>
                        </div>
                    )}

                    {tab === 'connections' && (
                        <div className="settings-section">
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                                <h2 className="section-title" style={{ margin: 0 }}>Database Connections</h2>
                                <button className="btn btn-primary btn-sm" onClick={() => setShowNewConn(true)}><Plus size={14} /> Add</button>
                            </div>
                            {connections.length === 0 && !showNewConn && (
                                <div className="setting-card" style={{ textAlign: 'center', padding: '32px' }}>
                                    <Database size={32} style={{ color: 'var(--text-tertiary)', marginBottom: '12px' }} />
                                    <p style={{ color: 'var(--text-secondary)' }}>No connections configured</p>
                                </div>
                            )}
                            {connections.map(conn => (
                                <div key={conn.id} className="setting-card conn-card">
                                    <SettingRow label={conn.name} desc={`${conn.dialect.toUpperCase()} ${conn.isDefault ? '(Default)' : ''}`}>
                                        <button className="btn btn-ghost btn-sm" onClick={() => handleRemoveConnection(conn.id)}><Trash2 size={14} /></button>
                                    </SettingRow>
                                </div>
                            ))}
                            {showNewConn && (
                                <div className="setting-card" style={{ marginTop: '12px' }}>
                                    <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '16px' }}>New Connection</h3>
                                    <div className="conn-form">
                                        <div className="form-row"><label>Name</label><input type="text" value={newConn.name} onChange={e => setNewConn(p => ({ ...p, name: e.target.value }))} placeholder="My Database" /></div>
                                        <div className="form-row"><label>Type</label>
                                            <select value={newConn.dialect} onChange={e => { const d = e.target.value as any; setNewConn(p => ({ ...p, dialect: d, port: d === 'postgres' ? 5432 : d === 'mysql' ? 3306 : 0 })); }}>
                                                <option value="sqlite">SQLite</option><option value="postgres">PostgreSQL</option><option value="mysql">MySQL</option>
                                            </select>
                                        </div>
                                        {newConn.dialect === 'sqlite' ? (
                                            <div className="form-row"><label>Database Path</label><input type="text" value={newConn.path} onChange={e => setNewConn(p => ({ ...p, path: e.target.value }))} placeholder="/path/to/db.sqlite" /></div>
                                        ) : (
                                            <>
                                                <div className="form-row-inline">
                                                    <div className="form-row" style={{ flex: 2 }}><label>Host</label><input type="text" value={newConn.host} onChange={e => setNewConn(p => ({ ...p, host: e.target.value }))} /></div>
                                                    <div className="form-row" style={{ flex: 1 }}><label>Port</label><input type="number" value={newConn.port} onChange={e => setNewConn(p => ({ ...p, port: Number(e.target.value) }))} /></div>
                                                </div>
                                                <div className="form-row"><label>Database</label><input type="text" value={newConn.database} onChange={e => setNewConn(p => ({ ...p, database: e.target.value }))} /></div>
                                                <div className="form-row-inline">
                                                    <div className="form-row" style={{ flex: 1 }}><label>Username</label><input type="text" value={newConn.username} onChange={e => setNewConn(p => ({ ...p, username: e.target.value }))} /></div>
                                                    <div className="form-row" style={{ flex: 1 }}><label>Password</label><input type="password" value={newConn.password} onChange={e => setNewConn(p => ({ ...p, password: e.target.value }))} /></div>
                                                </div>
                                                <label className="toggle-label"><input type="checkbox" checked={newConn.ssl} onChange={e => setNewConn(p => ({ ...p, ssl: e.target.checked }))} /><span>Use SSL</span></label>
                                            </>
                                        )}
                                        <div style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
                                            <button className="btn btn-primary" onClick={handleAddConnection} disabled={!newConn.name}><Check size={14} /> Save</button>
                                            <button className="btn btn-ghost" onClick={handleTestConnection}><TestTube size={14} /> Test</button>
                                            <button className="btn btn-ghost" onClick={() => setShowNewConn(false)}><X size={14} /> Cancel</button>
                                        </div>
                                        {testResult && (
                                            <div style={{ marginTop: '8px', padding: '8px 12px', borderRadius: '6px', fontSize: '12px', background: testResult.ok ? 'rgba(0,212,170,0.1)' : 'rgba(255,85,85,0.1)', color: testResult.ok ? 'var(--success)' : 'var(--danger)', display: 'flex', alignItems: 'center', gap: '6px' }}>
                                                {testResult.ok ? <Wifi size={12} /> : <WifiOff size={12} />} {testResult.msg}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            )}
                        </div>
                    )}

                    {tab === 'editor' && (
                        <div className="settings-section">
                            <h2 className="section-title">SQL Editor</h2>
                            <div className="setting-card">
                                <SettingRow label="Auto-save" desc="Auto-save queries periodically">
                                    <ToggleSwitch checked={settings.autoSave} onChange={v => updateSetting('autoSave', v)} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Tab Size" desc="Spaces per tab">
                                    <select className="setting-select" value={settings.tabSize} onChange={e => updateSetting('tabSize', Number(e.target.value))}>
                                        <option value={2}>2 spaces</option><option value={4}>4 spaces</option>
                                    </select>
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Word Wrap" desc="Wrap long lines in the editor">
                                    <ToggleSwitch checked={settings.wordWrap} onChange={v => updateSetting('wordWrap', v)} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Line Numbers" desc="Show line numbers in gutter">
                                    <ToggleSwitch checked={settings.lineNumbers} onChange={v => updateSetting('lineNumbers', v)} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Default Row Limit" desc="Default LIMIT for data browsing">
                                    <select className="setting-select" value={settings.defaultLimit} onChange={e => updateSetting('defaultLimit', Number(e.target.value))}>
                                        {[50, 100, 500, 1000, 5000].map(l => <option key={l} value={l}>{l}</option>)}
                                    </select>
                                </SettingRow>
                            </div>
                        </div>
                    )}

                    {tab === 'sync' && (
                        <div className="settings-section">
                            <h2 className="section-title">GitHub Sync</h2>
                            <div className="setting-card">
                                <SettingRow label="Enable Sync" desc="Sync schema and migrations with GitHub">
                                    <ToggleSwitch checked={settings.syncEnabled} onChange={v => updateSetting('syncEnabled', v)} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Sync Interval" desc="Check for remote changes">
                                    <select className="setting-select" value={settings.syncInterval} onChange={e => updateSetting('syncInterval', Number(e.target.value))} disabled={!settings.syncEnabled}>
                                        <option value={60}>Every minute</option><option value={300}>Every 5 min</option><option value={600}>Every 10 min</option><option value={1800}>Every 30 min</option>
                                    </select>
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Conflict Strategy" desc="Default resolution">
                                    <select className="setting-select" disabled={!settings.syncEnabled}>
                                        <option value="manual">Manual Review</option><option value="local">Keep Local</option><option value="remote">Keep Remote</option>
                                    </select>
                                </SettingRow>
                            </div>
                        </div>
                    )}

                    {tab === 'updates' && (
                        <div className="settings-section">
                            <h2 className="section-title">Updates</h2>
                            <div className="setting-card">
                                <SettingRow label="Current Version" desc="v0.8.0">
                                    <span className="badge badge-success">Up to date</span>
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Update Channel" desc="Release channel">
                                    <select className="setting-select" value={settings.updateChannel} onChange={e => updateSetting('updateChannel', e.target.value)}>
                                        <option value="stable">Stable</option><option value="beta">Beta</option><option value="nightly">Nightly</option>
                                    </select>
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Auto-check" desc="Check on startup">
                                    <ToggleSwitch checked={settings.autoCheckUpdates} onChange={v => updateSetting('autoCheckUpdates', v)} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Check Now" desc="Manual update check">
                                    <button className="btn btn-primary btn-sm"><RefreshCw size={14} /> Check</button>
                                </SettingRow>
                            </div>
                        </div>
                    )}

                    {tab === 'api' && (
                        <div className="settings-section">
                            <h2 className="section-title">REST API Server</h2>
                            <div className="setting-card">
                                <SettingRow label="Enable API Server" desc="Expose tables as REST endpoints">
                                    <ToggleSwitch checked={settings.apiEnabled} onChange={v => updateSetting('apiEnabled', v)} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Port" desc="API listen port">
                                    <input type="number" className="setting-input-sm" value={settings.apiPort} onChange={e => updateSetting('apiPort', Number(e.target.value))} disabled={!settings.apiEnabled} min={1024} max={65535} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Auto-start" desc="Start API when project opens">
                                    <ToggleSwitch checked={settings.apiAutoStart} onChange={v => updateSetting('apiAutoStart', v)} disabled={!settings.apiEnabled} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="CORS Origins" desc="Allowed origins">
                                    <input type="text" className="setting-input" value={settings.apiCorsOrigins} onChange={e => updateSetting('apiCorsOrigins', e.target.value)} disabled={!settings.apiEnabled} />
                                </SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Rate Limit" desc="Max requests/min">
                                    <input type="number" className="setting-input-sm" value={settings.apiRateLimit} onChange={e => updateSetting('apiRateLimit', Number(e.target.value))} disabled={!settings.apiEnabled} min={1} />
                                </SettingRow>
                                {settings.apiEnabled && (
                                    <>
                                        <div className="setting-divider" />
                                        <div style={{ padding: '12px 0', display: 'flex', gap: '8px', alignItems: 'center' }}>
                                            <button className={`btn btn-sm ${apiRunning ? 'btn-danger' : 'btn-primary'}`} onClick={handleToggleApiServer}>
                                                <Server size={14} /> {apiRunning ? 'Stop Server' : 'Start Server'}
                                            </button>
                                            {apiRunning && (
                                                <a href={`http://localhost:${settings.apiPort}/api/health`} target="_blank" rel="noopener" className="btn btn-ghost btn-sm">Health Check</a>
                                            )}
                                            {apiRunning && <span className="badge badge-success" style={{ marginLeft: 8 }}>● Running on :{settings.apiPort}</span>}
                                        </div>
                                        {apiMsg && <div style={{ padding: '4px 0', fontSize: 12, color: 'var(--text-secondary)' }}>{apiMsg}</div>}
                                    </>
                                )}
                            </div>
                        </div>
                    )}

                    {tab === 'security' && (
                        <div className="settings-section">
                            <h2 className="section-title">Security</h2>
                            <div className="setting-card">
                                <SettingRow label="API Key Management" desc="Manage authentication keys"><span className="badge">API Keys page</span></SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="RBAC Policies" desc="Role-based access control"><span className="badge">Coming soon</span></SettingRow>
                                <div className="setting-divider" />
                                <SettingRow label="Audit Log" desc="View all database operations"><span className="badge">Coming soon</span></SettingRow>
                            </div>
                        </div>
                    )}

                    {tab === 'about' && (
                        <div className="settings-section">
                            <h2 className="section-title">About AirDB</h2>
                            <div className="setting-card">
                                <div className="about-content">
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                                        <div style={{ width: '48px', height: '48px', background: 'linear-gradient(135deg, var(--accent), rgba(0,212,170,0.6))', borderRadius: '12px', display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 'bold', color: 'black', fontSize: '18px' }}>AD</div>
                                        <div>
                                            <div style={{ fontWeight: '700', fontSize: '18px' }}>AirDB</div>
                                            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Version 0.2.6</div>
                                        </div>
                                    </div>
                                    <p style={{ lineHeight: '1.7', color: 'var(--text-secondary)', marginBottom: '20px', fontSize: '13px' }}>
                                        AirDB is a local-first database development platform supporting SQLite, PostgreSQL, and MySQL.
                                        Visual schema design, auto-generated migrations, GitHub sync, hybrid SQL/NoSQL, and auto-generated REST APIs.
                                    </p>
                                    <div className="about-grid">
                                        <div className="about-item"><span className="about-label">Platform</span><span className="about-value">Tauri 2.10.2</span></div>
                                        <div className="about-item"><span className="about-label">Frontend</span><span className="about-value">React 19.1.0</span></div>
                                        <div className="about-item"><span className="about-label">Backend</span><span className="about-value">Rust 2021</span></div>
                                        <div className="about-item"><span className="about-label">License</span><span className="about-value">MIT</span></div>
                                    </div>
                                    <div style={{ marginTop: '20px', display: 'flex', gap: '8px' }}>
                                        <a href="https://github.com/airdb/airdb" target="_blank" rel="noopener" className="btn btn-ghost btn-sm"><Github size={14} /> Source</a>
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

/* ─── Reusable sub-components ─── */
function SettingRow({ label, desc, children }: { label: string; desc: string; children: React.ReactNode }) {
    return (
        <div className="setting-row">
            <div className="setting-info">
                <span className="setting-label">{label}</span>
                <span className="setting-desc">{desc}</span>
            </div>
            {children}
        </div>
    );
}

function ToggleSwitch({ checked, onChange, disabled }: { checked: boolean; onChange: (v: boolean) => void; disabled?: boolean }) {
    return (
        <label className={`toggle-switch ${disabled ? 'disabled' : ''}`}>
            <input type="checkbox" checked={checked} onChange={e => onChange(e.target.checked)} disabled={disabled} />
            <span className="toggle-slider" />
        </label>
    );
}
