import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './UpdateSettings.css';

interface UpdateInfo {
    currentVersion: string;
    latestVersion: string | null;
    updateAvailable: boolean;
    channel: string;
    lastCheck: string | null;
    changelog: string | null;
}

interface UpdateState {
    status: 'idle' | 'checking' | 'downloading' | 'ready' | 'applying' | 'failed' | 'rolledBack';
    progress: number;
    error: string | null;
}

export function UpdateSettings() {
    const [info, setInfo] = useState<UpdateInfo>({
        currentVersion: '0.1.0',
        latestVersion: null,
        updateAvailable: false,
        channel: 'stable',
        lastCheck: null,
        changelog: null,
    });

    const [state, setState] = useState<UpdateState>({
        status: 'idle',
        progress: 0,
        error: null,
    });

    const [isLocked, setIsLocked] = useState(false);

    useEffect(() => {
        loadUpdateInfo();
        checkLockStatus();
    }, []);

    const loadUpdateInfo = async () => {
        try {
            const data = await invoke<UpdateInfo>('get_update_info');
            setInfo(data);
        } catch (e) {
            console.error('Failed to load update info:', e);
        }
    };

    const checkLockStatus = async () => {
        try {
            const blocked = await invoke<boolean>('is_update_blocked');
            setIsLocked(blocked);
        } catch (e) {
            console.error('Failed to check lock status:', e);
        }
    };

    const checkForUpdates = async () => {
        setState({ status: 'checking', progress: 0, error: null });
        try {
            const result = await invoke<UpdateInfo>('check_for_updates');
            setInfo(result);
            setState({ status: 'idle', progress: 0, error: null });
        } catch (e) {
            setState({ status: 'failed', progress: 0, error: String(e) });
        }
    };

    const applyUpdate = async () => {
        if (isLocked) {
            setState({
                status: 'failed',
                progress: 0,
                error: 'Cannot update: operation in progress (migration/backup/serve)'
            });
            return;
        }

        setState({ status: 'downloading', progress: 0, error: null });
        try {
            await invoke('apply_update', {
                onProgress: (p: number) => setState(s => ({ ...s, progress: p }))
            });
            setState({ status: 'ready', progress: 100, error: null });
        } catch (e) {
            setState({ status: 'failed', progress: 0, error: String(e) });
        }
    };

    const getStatusIcon = () => {
        switch (state.status) {
            case 'checking': return '🔍';
            case 'downloading': return '⬇️';
            case 'ready': return '✅';
            case 'applying': return '⚙️';
            case 'failed': return '❌';
            case 'rolledBack': return '⚠️';
            default: return '📦';
        }
    };

    const getStatusText = () => {
        switch (state.status) {
            case 'checking': return 'Checking for updates...';
            case 'downloading': return `Downloading... ${state.progress}%`;
            case 'ready': return 'Update ready. Restart to apply.';
            case 'applying': return 'Applying update...';
            case 'failed': return state.error || 'Update failed';
            case 'rolledBack': return 'Rolled back to previous version';
            default: return info.updateAvailable ? 'Update available' : 'Up to date';
        }
    };

    return (
        <div className="update-settings">
            <div className="update-header">
                <h2>🔄 Updates</h2>
                {isLocked && (
                    <div className="lock-warning">
                        ⚠️ Updates locked: operation in progress
                    </div>
                )}
            </div>

            <div className="update-info">
                <div className="version-row">
                    <span className="label">Current Version:</span>
                    <span className="value">{info.currentVersion}</span>
                </div>

                <div className="version-row">
                    <span className="label">Channel:</span>
                    <select
                        value={info.channel}
                        onChange={(e) => setInfo({ ...info, channel: e.target.value })}
                        disabled={isLocked}
                    >
                        <option value="stable">Stable</option>
                        <option value="beta">Beta</option>
                        <option value="nightly">Nightly</option>
                    </select>
                </div>

                {info.lastCheck && (
                    <div className="version-row">
                        <span className="label">Last Check:</span>
                        <span className="value muted">{info.lastCheck}</span>
                    </div>
                )}
            </div>

            <div className="update-status">
                <span className="status-icon">{getStatusIcon()}</span>
                <span className="status-text">{getStatusText()}</span>
            </div>

            {state.status === 'downloading' && (
                <div className="progress-bar">
                    <div
                        className="progress-fill"
                        style={{ width: `${state.progress}%` }}
                    />
                </div>
            )}

            <div className="update-actions">
                <button
                    className="btn-primary"
                    onClick={checkForUpdates}
                    disabled={state.status === 'checking' || state.status === 'downloading' || isLocked}
                >
                    Check for Updates
                </button>

                {info.updateAvailable && (
                    <button
                        className="btn-success"
                        onClick={applyUpdate}
                        disabled={state.status !== 'idle' || isLocked}
                    >
                        Apply Update
                    </button>
                )}

                {info.changelog && (
                    <button
                        className="btn-secondary"
                        onClick={() => window.open(info.changelog!, '_blank')}
                    >
                        View Changelog
                    </button>
                )}
            </div>

            {state.status === 'rolledBack' && (
                <div className="rollback-notice">
                    <h4>⚠️ Automatic Rollback</h4>
                    <p>The update failed to start correctly and was automatically rolled back.</p>
                    <p>{state.error}</p>
                </div>
            )}
        </div>
    );
}
