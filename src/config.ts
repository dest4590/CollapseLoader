import { invoke } from "@tauri-apps/api/core";

let currentApiUrl = "";
let isInitialized = false;
let initPromise: Promise<string> | null = null;
let currentApiVersion = "v1";

export const initializeApiUrl = (): Promise<string> => {
    if (isInitialized) {
        return Promise.resolve(currentApiUrl);
    }

    if (initPromise) {
        return initPromise;
    }

    initPromise = (async () => {
        try {
            const result = await invoke<string>("get_api_url");
            if (result.length > 0) {
                currentApiUrl = result.endsWith("/")
                    ? result.slice(0, -1)
                    : result;
                console.log("API URL from backend:", currentApiUrl);
            } else {
                throw new Error("Invalid API url from backend");
            }
        } catch (error) {
            console.error("Failed to get API URL from backend:", error);
        }

        isInitialized = true;
        return currentApiUrl;
    })();

    return initPromise;
};

export const getApiUrl = (): string => currentApiUrl;
export const getApiVersion = (): string => currentApiVersion;
export const getApiBaseWithVersion = (): string =>
    `${currentApiUrl}/api/${currentApiVersion}`;
export const ensureApiUrl = async (): Promise<string> => {
    if (isInitialized) return currentApiUrl;
    return initializeApiUrl();
};
