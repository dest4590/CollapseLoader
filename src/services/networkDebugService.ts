import { ref } from "vue";
import { listen, emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export interface NetworkRequest {
    id: string;
    method: string;
    url: string;
    status?: number;
    duration?: number;
    timestamp: number;
    request_headers?: Record<string, string> | null;
    request_body?: any | null;
    response_body?: any | null;
    response_text?: string | null;
    error_message?: string | null;
}

const requests = ref<Record<string, NetworkRequest>>({});
let inited = false;
let unlistenReq: (() => void) | null = null;
let unlistenRes: (() => void) | null = null;

export function getRequestsRef() {
    return requests;
}

export async function initNetworkDebug() {
    if (inited) return;
    inited = true;

    try {
        const hist = await invoke<NetworkRequest[]>("get_network_history");
        for (const r of hist) {
            requests.value[r.id] = r;
        }
    } catch (e) {
        console.warn("Failed to load network history:", e);
    }

    unlistenReq = await listen<NetworkRequest>("network-request", (e) => {
        requests.value[e.payload.id] = e.payload;
    });

    unlistenRes = await listen<NetworkRequest>("network-response", (e) => {
        requests.value[e.payload.id] = e.payload;
    });

    await listen("network-clear-local", () => {
        requests.value = {};
    });
}

export function destroyNetworkDebug() {
    if (unlistenReq) {
        unlistenReq();
        unlistenReq = null;
    }
    if (unlistenRes) {
        unlistenRes();
        unlistenRes = null;
    }
    inited = false;
}

export async function clearRequestsLocal() {
    requests.value = {};
    await invoke("clear_network_history");
    emit("network-clear-local");
}

export async function exportReport(): Promise<string> {
    return await invoke<string>("export_network_report");
}

export default {
    initNetworkDebug,
    destroyNetworkDebug,
    getRequestsRef,
    clearRequestsLocal,
    exportReport,
};
