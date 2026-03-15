<template>
    <div
        class="absolute inset-0 z-10 flex flex-col bg-base-100 font-sans text-sm rounded-xl overflow-hidden border border-base-300 shadow-lg"
    >
        <div
            class="sticky top-0 z-20 flex items-center justify-between px-4 py-2 bg-base-200 border-b border-base-300"
        >
            <div class="flex items-center gap-4">
                <h1 class="font-bold text-base-content flex items-center gap-2">
                    <Globe class="w-4 h-4 text-primary" />
                    Network
                </h1>
                <div class="w-px h-4 bg-base-300"></div>
                <button
                    class="btn btn-sm btn-ghost text-base-content/70 hover:text-error"
                    @click="clearRequests"
                    title="Clear network requests"
                >
                    <Trash2 class="w-4 h-4" />
                </button>
                <button
                    class="btn btn-sm btn-ghost text-base-content/70 hover:text-primary"
                    @click="openInNewWindow"
                    v-if="!isExternalWindow"
                    title="Open in new window"
                >
                    <ExternalLink class="w-4 h-4" />
                </button>
            </div>

            <button
                class="btn btn-sm btn-outline btn-primary"
                @click="doExportReport"
                :disabled="exportingReport"
            >
                <span
                    v-if="exportingReport"
                    class="loading loading-spinner loading-xs"
                ></span>
                <DownloadCloud v-else class="w-3.5 h-3.5 mr-1" />
                Export Report
            </button>
        </div>

        <div class="flex flex-1 overflow-hidden bg-base-100">
            <div
                class="flex flex-col h-full overflow-hidden transition-all duration-300 border-r border-base-300"
                :class="selectedRequest ? 'w-2/5 min-w-75' : 'w-full'"
            >
                <div class="flex-1 overflow-auto custom-scrollbar">
                    <table
                        class="w-full text-left border-collapse whitespace-nowrap"
                    >
                        <thead
                            class="sticky top-0 bg-base-300 z-10 text-xs font-semibold text-base-content/70 border-b border-base-300 shadow-sm"
                        >
                            <tr>
                                <th class="py-1.5 px-3 w-full">Name</th>
                                <th class="py-1.5 px-3">Status</th>
                                <th class="py-1.5 px-3">Method</th>
                                <th class="py-1.5 px-3 text-right">Time</th>
                            </tr>
                        </thead>
                        <tbody class="text-xs font-mono">
                            <tr
                                v-for="req in sortedRequests"
                                :key="req.id"
                                class="border-b border-base-200/50 cursor-pointer transition-colors"
                                :class="{
                                    'bg-primary/10':
                                        selectedRequest?.id === req.id,
                                    'hover:bg-base-200':
                                        selectedRequest?.id !== req.id,
                                    'text-error':
                                        req.status === 0 ||
                                        (req.status && req.status >= 400),
                                }"
                                @click="selectRequest(req)"
                            >
                                <td
                                    class="py-1.5 px-3 max-w-37.5 truncate"
                                    :title="req.url"
                                >
                                    <span class="font-medium">{{
                                        getPathName(req.url)
                                    }}</span>
                                </td>

                                <td class="py-1.5 px-3">
                                    <div class="flex items-center gap-1.5">
                                        <div
                                            class="w-2 h-2 rounded-full"
                                            :class="
                                                getStatusDotColor(req.status)
                                            "
                                        ></div>
                                        <span>{{
                                            formatStatus(req.status)
                                        }}</span>
                                    </div>
                                </td>

                                <td
                                    class="py-1.5 px-3"
                                    :class="getMethodClass(req.method)"
                                >
                                    {{ req.method }}
                                </td>

                                <td class="py-1.5 px-3 text-right opacity-80">
                                    <span v-if="req.duration !== undefined"
                                        >{{ req.duration }} ms</span
                                    >
                                    <span
                                        v-else
                                        class="loading loading-dots loading-xs"
                                    ></span>
                                </td>
                            </tr>
                            <tr v-if="sortedRequests.length === 0">
                                <td
                                    colspan="4"
                                    class="text-center py-8 text-base-content/50 font-sans italic"
                                >
                                    No network requests recorded yet
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div
                v-if="selectedRequest"
                class="flex flex-col flex-1 h-full w-3/5 bg-base-100 overflow-hidden animate-in fade-in slide-in-from-right-2 duration-200"
            >
                <div
                    class="flex items-center bg-base-200 border-b border-base-300 text-xs font-medium"
                >
                    <button
                        class="px-4 py-2 hover:bg-base-300 transition-colors border-b-2"
                        :class="
                            activeTab === 'headers'
                                ? 'border-primary text-primary'
                                : 'border-transparent text-base-content/70'
                        "
                        @click="activeTab = 'headers'"
                    >
                        Headers
                    </button>
                    <button
                        class="px-4 py-2 hover:bg-base-300 transition-colors border-b-2"
                        :class="
                            activeTab === 'payload'
                                ? 'border-primary text-primary'
                                : 'border-transparent text-base-content/70'
                        "
                        @click="activeTab = 'payload'"
                    >
                        Payload
                    </button>
                    <button
                        class="px-4 py-2 hover:bg-base-300 transition-colors border-b-2"
                        :class="
                            activeTab === 'response'
                                ? 'border-primary text-primary'
                                : 'border-transparent text-base-content/70'
                        "
                        @click="activeTab = 'response'"
                    >
                        Response
                    </button>

                    <div class="flex-1"></div>
                    <button
                        class="px-3 py-2 text-base-content/50 hover:text-base-content hover:bg-base-300 transition-colors"
                        @click="selectedRequest = null"
                    >
                        <X class="w-4 h-4" />
                    </button>
                </div>

                <div
                    class="flex-1 overflow-auto p-4 custom-scrollbar text-xs font-mono"
                >
                    <div v-if="activeTab === 'headers'" class="space-y-6">
                        <section>
                            <h3
                                class="font-bold font-sans text-base-content mb-2 border-b border-base-200 pb-1"
                            >
                                General
                            </h3>
                            <div
                                class="grid grid-cols-[120px_1fr] gap-y-1.5 text-base-content/80"
                            >
                                <div>Request URL:</div>
                                <div class="break-all text-base-content">
                                    {{ selectedRequest.url }}
                                </div>

                                <div>Request Method:</div>
                                <div
                                    class="font-bold"
                                    :class="
                                        getMethodClass(selectedRequest.method)
                                    "
                                >
                                    {{ selectedRequest.method }}
                                </div>

                                <div>Status Code:</div>
                                <div class="flex items-center gap-2">
                                    <div
                                        class="w-2 h-2 rounded-full"
                                        :class="
                                            getStatusDotColor(
                                                selectedRequest.status
                                            )
                                        "
                                    ></div>
                                    <span
                                        :class="
                                            getStatusTextColor(
                                                selectedRequest.status
                                            )
                                        "
                                        >{{
                                            formatStatus(selectedRequest.status)
                                        }}</span
                                    >
                                </div>
                            </div>
                        </section>

                        <section v-if="selectedRequest.request_headers">
                            <h3
                                class="font-bold font-sans text-base-content mb-2 border-b border-base-200 pb-1"
                            >
                                Request Headers
                            </h3>
                            <div
                                class="grid grid-cols-[150px_1fr] gap-y-1 gap-x-2 text-base-content/80"
                            >
                                <template
                                    v-for="(
                                        val, key
                                    ) in selectedRequest.request_headers"
                                    :key="key"
                                >
                                    <div
                                        class="capitalize text-base-content/60"
                                    >
                                        {{ key }}:
                                    </div>
                                    <div class="break-all text-base-content">
                                        {{ formatHeaderValue(key, val) }}
                                    </div>
                                </template>
                            </div>
                        </section>
                    </div>

                    <div v-if="activeTab === 'payload'" class="h-full">
                        <div v-if="selectedRequest.request_body">
                            <h3
                                class="font-bold font-sans text-base-content mb-2 border-b border-base-200 pb-1"
                            >
                                Request Payload
                            </h3>
                            <pre
                                class="bg-base-200 p-3 rounded-md overflow-auto text-info border border-base-300"
                                >{{
                                    JSON.stringify(
                                        selectedRequest.request_body,
                                        null,
                                        2
                                    )
                                }}</pre
                            >
                        </div>
                        <div
                            v-else
                            class="flex h-full items-center justify-center text-base-content/40 font-sans italic"
                        >
                            No request payload
                        </div>
                    </div>

                    <div
                        v-if="activeTab === 'response'"
                        class="h-full flex flex-col"
                    >
                        <div
                            v-if="selectedRequest.error_message"
                            class="mb-4 text-error bg-error/10 border border-error/20 p-3 rounded-md break-all"
                        >
                            <span class="font-bold font-sans"
                                >Network Error:
                            </span>
                            {{ selectedRequest.error_message }}
                        </div>

                        <div
                            v-if="selectedRequest.response_body"
                            class="flex-1"
                        >
                            <pre
                                class="bg-base-200 p-3 rounded-md overflow-auto text-success h-full border border-base-300"
                                >{{
                                    JSON.stringify(
                                        selectedRequest.response_body,
                                        null,
                                        2
                                    )
                                }}</pre
                            >
                        </div>
                        <div
                            v-else-if="selectedRequest.response_text"
                            class="flex-1"
                        >
                            <pre
                                class="bg-base-200 p-3 rounded-md overflow-auto whitespace-pre-wrap break-all h-full border border-base-300"
                                >{{ selectedRequest.response_text }}</pre
                            >
                        </div>
                        <div
                            v-else-if="!selectedRequest.status"
                            class="flex h-full items-center justify-center text-base-content/40"
                        >
                            <span
                                class="loading loading-spinner loading-md"
                            ></span>
                        </div>
                        <div
                            v-else
                            class="flex h-full items-center justify-center text-base-content/40 font-sans italic"
                        >
                            This request has no response body
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { Globe, Trash2, DownloadCloud, X, ExternalLink } from "lucide-vue-next";
import {
    getRequestsRef,
    exportReport,
    clearRequestsLocal,
} from "../services/networkDebugService";

interface NetworkRequest {
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

const requests = getRequestsRef();
const selectedRequest = ref<NetworkRequest | null>(null);
const exportingReport = ref(false);
const activeTab = ref<"headers" | "payload" | "response">("headers");
const isExternalWindow = window.location.search.includes("window=network");

const sortedRequests = computed(() => {
    return Object.values(requests.value).sort(
        (a, b) => b.timestamp - a.timestamp
    );
});

const selectRequest = (req: NetworkRequest) => {
    selectedRequest.value = req;
    if (req.status === 0 || (req.status && req.status >= 400)) {
        activeTab.value = "response";
    } else {
        activeTab.value = "headers";
    }
};

const clearRequests = async () => {
    await clearRequestsLocal();
    selectedRequest.value = null;
};

const doExportReport = async () => {
    exportingReport.value = true;
    try {
        const path = await exportReport();
        console.log(`Report exported to ${path}`);
    } catch (err: any) {
        console.error(err);
    } finally {
        exportingReport.value = false;
    }
};

const openInNewWindow = async () => {
    try {
        const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
        const webview = new WebviewWindow("network-debug", {
            url: "index.html?window=network",
            title: "Network Debugger",
            width: 1000,
            height: 800,
            resizable: true,
            decorations: true,
        });

        webview.once("tauri://created", function () {
            console.log("Network window created");
        });

        webview.once("tauri://error", function (e) {
            console.error("Error creating window:", e);
        });
    } catch (e) {
        console.error("Failed to open new window:", e);
    }
};

const getPathName = (urlStr: string) => {
    try {
        const url = new URL(urlStr);
        const path = url.pathname === "/" ? url.hostname : url.pathname;
        return path + url.search;
    } catch {
        return urlStr;
    }
};

const getMethodClass = (method: string) => {
    method = method.toUpperCase();
    if (method === "GET") return "text-success font-semibold";
    if (method === "POST") return "text-info font-semibold";
    if (method === "PUT" || method === "PATCH")
        return "text-warning font-semibold";
    if (method === "DELETE") return "text-error font-semibold";
    return "font-semibold";
};

const getStatusDotColor = (status?: number) => {
    if (!status && status !== 0) return "bg-base-300 animate-pulse";
    if (status === 0) return "bg-error";
    if (status >= 200 && status < 300) return "bg-success";
    if (status >= 400) return "bg-error";
    return "bg-warning";
};

const getStatusTextColor = (status?: number) => {
    if (status === 0 || (status && status >= 400))
        return "text-error font-bold";
    if (status && status >= 200 && status < 300)
        return "text-success font-bold";
    return "text-base-content/70";
};

const formatHeaderValue = (key: string, value: string) => {
    if (key.toLowerCase() === "authorization") {
        const parts = value.split(" ");

        if (parts.length === 2) {
            return `${parts[0]} ****`;
        }

        return "****";
    }
    return value;
};

const formatStatus = (status?: number) => {
    if (status === 0) return "(failed)";
    if (!status) return "Pending";
    return status.toString();
};
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: oklch(var(--bc) / 0.2);
    border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background-color: oklch(var(--bc) / 0.4);
}
</style>
