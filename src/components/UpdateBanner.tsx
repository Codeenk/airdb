import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './UpdateBanner.css';

interface BannerState {
    visible: boolean;
    version: string | null;
    dismissed: boolean;
}

export function UpdateBanner() {
    const [state, setState] = useState<BannerState>({
        visible: false,
        version: null,
        dismissed: false,
    });

    useEffect(() => {
        checkForUpdates();

        // Check periodically (every 30 minutes)
        const interval = setInterval(checkForUpdates, 30 * 60 * 1000);
        return () => clearInterval(interval);
    }, []);

    const checkForUpdates = async () => {
        if (state.dismissed) return;

        try {
            const result = await invoke<{ update_available: boolean; latest_version: string }>('check_for_updates');
            if (result.update_available) {
                setState({
                    visible: true,
                    version: result.latest_version,
                    dismissed: false,
                });
            }
        } catch (e) {
            // Silent fail for background check
        }
    };

    const dismiss = () => {
        setState(s => ({ ...s, visible: false, dismissed: true }));
    };

    const goToSettings = () => {
        // Emit event to navigate to settings
        window.dispatchEvent(new CustomEvent('navigate', { detail: '/settings/updates' }));
    };

    if (!state.visible || state.dismissed) {
        return null;
    }

    return (
        <div className="update-banner">
            <div className="banner-content">
                <span className="banner-icon">🚀</span>
                <span className="banner-text">
                    Update available: <strong>v{state.version}</strong>
                </span>
                <button className="banner-btn" onClick={goToSettings}>
                    View Details
                </button>
                <button className="banner-dismiss" onClick={dismiss} aria-label="Dismiss">
                    ×
                </button>
            </div>
        </div>
    );
}
