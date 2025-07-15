import { invoke } from '@tauri-apps/api/core';

export const DEV_AUTH_URL = "http://127.0.0.1:8000";
export const DEV_API_URL = "http://127.0.0.1:8500";
export const USE_DEV_API = false;
export const USE_DEV_AUTH = false;

let currentApiUrl = "";
let currentAuthUrl = "";
let isInitialized = false;

export const initializeApiUrl = async (): Promise<string> => {
    if (isInitialized) {
        return currentApiUrl;
    }

    if (USE_DEV_API || USE_DEV_AUTH) {
        if (USE_DEV_API) {
            console.log(`Using development API URL: ${DEV_API_URL}`);
            currentApiUrl = DEV_API_URL;
        }

        if (USE_DEV_AUTH) {
            console.log(`Using development Auth URL: ${DEV_AUTH_URL}`);
            currentAuthUrl = DEV_AUTH_URL;
        }

        isInitialized = true;
        return currentApiUrl;
    }

    try {
        currentAuthUrl = await invoke<string>('get_auth_url');
        console.log('Auth URL from backend:', currentAuthUrl);

        if (currentAuthUrl.endsWith('/')) {
            currentAuthUrl = currentAuthUrl.slice(0, -1);
        }

        currentApiUrl = await invoke<string>('get_api_url');
        console.log('API URL from backend:', currentApiUrl);

        if (currentApiUrl.endsWith('/')) {
            currentApiUrl = currentApiUrl.slice(0, -1);
        }
    } catch (error) {
        console.error('Failed to get URLs from backend:', error);
        currentApiUrl = "https://api.collapseloader.org";
        currentAuthUrl = "https://auth.collapseloader.org";
    }

    isInitialized = true;
    return currentApiUrl;
};

export const getApiUrl = (): string => currentApiUrl;

export const getAuthUrl = (): string => currentAuthUrl;