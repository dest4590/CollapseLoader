import { invoke } from "@tauri-apps/api/core";

async function tryInvokeIsDevelopment(): Promise<boolean> {
    try {
        const result = await invoke('is_development');
        return Boolean(result);
    } catch (e) {
        console.warn('isDevelopment check failed:', e);
        return false;
    }
}

const isDevelopment: Promise<boolean> = tryInvokeIsDevelopment();

export default isDevelopment;

export async function getIsDevelopment(): Promise<boolean> {
    return await isDevelopment;
}
