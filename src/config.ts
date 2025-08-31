import { invoke } from '@tauri-apps/api/core';

export const DEV_AUTH_URL = "http://127.0.0.1:8000";

export let USE_DEV_AUTH = false;

await invoke('is_development').then((result) => {
    if (result === true) {
        console.log("Development mode detected. Using development Auth URL.");
        USE_DEV_AUTH = true;
    }
}).catch((error) => {
    console.error("Error checking development mode:", error);
});

let currentAuthUrl = "";
let isInitialized = false;

export const initializeAuthUrl = async (): Promise<string> => {
    if (isInitialized) {
        return currentAuthUrl;
    }

    if (USE_DEV_AUTH) {
        console.log(`Using development Auth URL: ${DEV_AUTH_URL}`);
        currentAuthUrl = DEV_AUTH_URL;
        isInitialized = true;
        return currentAuthUrl;
    }

    try {
        currentAuthUrl = await invoke<string>('get_auth_url');
        console.log('Auth URL from backend:', currentAuthUrl);

        if (currentAuthUrl.endsWith('/')) {
            currentAuthUrl = currentAuthUrl.slice(0, -1);
        }
    } catch (error) {
        console.error('Failed to get Auth URL from backend:', error);
        currentAuthUrl = "https://auth.collapseloader.org";
    }

    isInitialized = true;
    return currentAuthUrl;
};

export const getAuthUrl = (): string => currentAuthUrl;
