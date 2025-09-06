import { invoke } from '@tauri-apps/api/core';

let currentAuthUrl = "";
let isInitialized = false;

export const initializeAuthUrl = async (): Promise<string> => {
    if (isInitialized) {
        return currentAuthUrl;
    }

    try {
        const result = await invoke<string>('get_auth_url');
        if (typeof result === 'string' && result.length > 0) {
            currentAuthUrl = result.endsWith('/') ? result.slice(0, -1) : result;
            console.log('Auth URL from backend:', currentAuthUrl);
        } else {
            throw new Error('Invalid auth url from backend');
        }
    } catch (error) {
        console.error('Failed to get Auth URL from backend:', error);
        currentAuthUrl = "https://auth.collapseloader.org";
    }

    isInitialized = true;
    return currentAuthUrl;
};

export const getAuthUrl = (): string => currentAuthUrl;
