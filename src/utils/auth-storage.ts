import { invoke } from '@tauri-apps/api/core';

export interface AuthState {
    authenticated: boolean;
    username?: string;
}

export interface SavedAuthToken {
    token: string;
    userEmail: string;
    timestamp: number;
    expiresAt: number;
}

const AUTH_STORAGE_KEY = 'airdb_github_token';
const TOKEN_EXPIRY_DAYS = 30;

/**
 * Save GitHub authentication token to localStorage
 */
export async function saveAuthToken(token: string, userEmail: string): Promise<void> {
    const timestamp = Date.now();
    const expiresAt = timestamp + (TOKEN_EXPIRY_DAYS * 24 * 60 * 60 * 1000);
    
    const authData: SavedAuthToken = {
        token,
        userEmail,
        timestamp,
        expiresAt,
    };
    
    try {
        localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(authData));
    } catch (e) {
        console.error('Failed to save auth token:', e);
        throw new Error('Failed to save authentication credentials');
    }
}

/**
 * Load saved GitHub authentication token from localStorage
 */
export async function loadAuthToken(): Promise<SavedAuthToken | null> {
    try {
        const stored = localStorage.getItem(AUTH_STORAGE_KEY);
        if (!stored) {
            return null;
        }
        
        const authData: SavedAuthToken = JSON.parse(stored);
        return authData;
    } catch (e) {
        console.error('Failed to load auth token:', e);
        return null;
    }
}

/**
 * Clear saved authentication token
 */
export async function clearAuthToken(): Promise<void> {
    try {
        localStorage.removeItem(AUTH_STORAGE_KEY);
    } catch (e) {
        console.error('Failed to clear auth token:', e);
    }
}

/**
 * Check if saved authentication token is still valid
 */
export async function isAuthValid(savedAuth: SavedAuthToken): Promise<boolean> {
    if (!savedAuth) {
        return false;
    }
    
    const now = Date.now();
    if (now > savedAuth.expiresAt) {
        return false;
    }
    
    return true;
}

/**
 * Check if the user is authenticated.
 */
export async function checkAuthStatus(): Promise<AuthState> {
    try {
        return await invoke<AuthState>('get_auth_status');
    } catch (e) {
        console.error('Failed to get auth status:', e);
        return { authenticated: false };
    }
}

/**
 * Logout from GitHub.
 */
export async function logout(): Promise<void> {
    try {
        await invoke('github_logout');
        await clearAuthToken();
    } catch (e) {
        console.error('Failed to logout:', e);
        throw e;
    }
}
