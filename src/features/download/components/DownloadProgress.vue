<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useDownloadSpeedMonitor } from "../../../composables/useDownloadSpeedMonitor";

const { t } = useI18n();
const downloadMonitor = useDownloadSpeedMonitor();

interface ProgressEvent {
    file: string;
    percentage: number;
    downloaded?: number;
    total?: number;
    speed_bps?: number;
    elapsed_ms?: number;
    action?: string;
    files_extracted?: number;
    total_files?: number;
}

interface FileProgressState {
    percentage: number;
    downloadedBytes: number;
    totalBytes: number;
    speedBps: number;
    hasKnownTotal: boolean;
    action: string;
}

const isVisible = ref(false);
const currentFile = ref("");
const fileProgress = ref<Map<string, FileProgressState>>(new Map());
const listeners = ref<any[]>([]);
const activeDownloads = ref<Set<string>>(new Set());

const shouldShowProgress = (filename: string): boolean => {
    return filename.length > 0;
};

const humanSize = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024)
        return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
    if (bytes >= 1024 * 1024)
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
};

const currentState = computed(
    (): FileProgressState | undefined => fileProgress.value.get(currentFile.value)
);

const percentage = computed(() => currentState.value?.percentage ?? 0);
const currentAction = computed(() => currentState.value?.action ?? t("installation.downloading"));

const formattedSpeed = computed((): string => {
    const bps = currentState.value?.speedBps ?? 0;
    if (bps <= 0) return "";
    return `${humanSize(bps)}/s`;
});

const formattedEta = computed((): string => {
    const s = currentState.value;
    if (!s || !s.hasKnownTotal || s.speedBps <= 0 || s.totalBytes <= 0 || s.downloadedBytes >= s.totalBytes)
        return "";
    const remaining = s.totalBytes - s.downloadedBytes;
    const etaSec = Math.ceil(remaining / s.speedBps);
    
    if (etaSec < 60) return `${etaSec}s`;
    if (etaSec < 3600) {
        const min = Math.floor(etaSec / 60);
        const sec = etaSec % 60;
        return `${min}m ${sec}s`;
    }
    const hours = Math.floor(etaSec / 3600);
    const min = Math.floor((etaSec % 3600) / 60);
    return `${hours}h ${min}m`;
});

const formattedProgress = computed((): string => {
    const s = currentState.value;
    if (!s) return "";
    if (s.downloadedBytes > 0 && s.totalBytes > 0 && s.hasKnownTotal) {
        return `${humanSize(s.downloadedBytes)} / ${humanSize(s.totalBytes)}`;
    }
    if (s.downloadedBytes > 0) {
        return humanSize(s.downloadedBytes);
    }
    return "";
});

const isIndeterminate = computed(
    () =>
        isVisible.value &&
        !(currentState.value?.hasKnownTotal ?? false) &&
        currentState.value?.action !== t("installation.extracting")
);

const makeInitialState = (action: string): FileProgressState => ({
    percentage: 0,
    downloadedBytes: 0,
    totalBytes: 0,
    speedBps: 0,
    hasKnownTotal: false,
    action,
});

const updateFileProgress = (file: string, patch: Partial<FileProgressState>) => {
    const existing = fileProgress.value.get(file) ?? makeInitialState(t("installation.downloading"));
    fileProgress.value.set(file, { ...existing, ...patch });
    fileProgress.value = new Map(fileProgress.value);
};

const removeFileProgress = (file: string) => {
    fileProgress.value.delete(file);
    fileProgress.value = new Map(fileProgress.value);
};

const cancelDownload = async (file: string) => {
    if (!file) return;
    try {
        await invoke("cancel_download", { name: file });
    } catch (e) {
        console.error("Failed to cancel download:", e);
    }
};

onMounted(async () => {
    listeners.value.push(
        await listen("download-start", (event: any) => {
            const filename = event.payload as string;
            downloadMonitor.startMonitoring();

            if (shouldShowProgress(filename)) {
                activeDownloads.value.add(filename);
                currentFile.value = filename;
                fileProgress.value.set(filename, makeInitialState(t("installation.downloading")));
                fileProgress.value = new Map(fileProgress.value);
                isVisible.value = true;
            }
        })
    );

    listeners.value.push(
        await listen("download-progress", (event: any) => {
            const data = event.payload as ProgressEvent;

            if (data.downloaded !== undefined) {
                downloadMonitor.onProgress(data.downloaded, data.percentage);
            }

            if (
                shouldShowProgress(data.file) &&
                activeDownloads.value.has(data.file)
            ) {
                currentFile.value = data.file;
                isVisible.value = true;

                updateFileProgress(data.file, {
                    action: t("installation.downloading"),
                    percentage: data.percentage,
                    ...(data.downloaded !== undefined ? { downloadedBytes: data.downloaded } : {}),
                    ...(data.total !== undefined && data.total > 0
                        ? { totalBytes: data.total, hasKnownTotal: true }
                        : {}),
                    ...(data.speed_bps !== undefined ? { speedBps: data.speed_bps } : {}),
                });
            }
        })
    );

    listeners.value.push(
        await listen("download-complete", (event: any) => {
            const filename = event.payload as string;
            downloadMonitor.stopMonitoring();

            if (shouldShowProgress(filename)) {
                if (!filename.endsWith(".zip")) {
                    activeDownloads.value.delete(filename);
                    removeFileProgress(filename);
                    if (activeDownloads.value.size === 0) {
                        isVisible.value = false;
                    } else {
                        const remaining = [...activeDownloads.value];
                        currentFile.value = remaining[remaining.length - 1];
                    }
                }
            }
        })
    );

    listeners.value.push(
        await listen("unzip-start", (event: any) => {
            const filename = event.payload as string;
            if (
                shouldShowProgress(filename) &&
                activeDownloads.value.has(filename)
            ) {
                currentFile.value = filename;
                isVisible.value = true;
                updateFileProgress(filename, {
                    action: t("installation.extracting"),
                    percentage: 0,
                    downloadedBytes: 0,
                    totalBytes: 0,
                    speedBps: 0,
                    hasKnownTotal: false,
                });
            }
        })
    );

    listeners.value.push(
        await listen("unzip-progress", (event: any) => {
            const data = event.payload as ProgressEvent;
            if (
                shouldShowProgress(data.file) &&
                activeDownloads.value.has(data.file)
            ) {
                currentFile.value = data.file;
                updateFileProgress(data.file, {
                    action: t("installation.extracting"),
                    percentage: data.percentage,
                    ...(data.files_extracted !== undefined ? { downloadedBytes: data.files_extracted } : {}),
                    ...(data.total_files !== undefined && data.total_files > 0
                        ? { totalBytes: data.total_files, hasKnownTotal: true }
                        : {}),
                });
            }
        })
    );

    listeners.value.push(
        await listen("unzip-complete", (event: any) => {
            const filename = event.payload as string;
            if (shouldShowProgress(filename)) {
                activeDownloads.value.delete(filename);
                removeFileProgress(filename);
                if (activeDownloads.value.size === 0) {
                    isVisible.value = false;
                } else {
                    const remaining = [...activeDownloads.value];
                    currentFile.value = remaining[remaining.length - 1];
                }
            }
        })
    );

    listeners.value.push(
        await listen("requirements-status", (event: any) => {
            const isDownloading = event.payload as boolean;
            if (!isDownloading) {
                activeDownloads.value.clear();
                fileProgress.value = new Map();
                isVisible.value = false;
                downloadMonitor.stopMonitoring();
            }
        })
    );
});

onUnmounted(() => {
    listeners.value.forEach((unlisten) => unlisten());
    downloadMonitor.reset();
});
</script>

<template>
    <Transition name="slide-up">
        <div v-if="isVisible" class="download-progress">
            <div class="progress-info">
                <span class="progress-label">
                    {{ currentAction }}
                    <span class="progress-file">{{ currentFile }}</span>
                </span>
                <span class="progress-meta">
                    <span v-if="formattedProgress" class="progress-size">{{ formattedProgress }}</span>
                    <span v-if="formattedSpeed" class="progress-speed">{{ formattedSpeed }}</span>
                    <span v-if="formattedEta" class="progress-eta">{{ formattedEta }}</span>
                    <span class="progress-pct">{{ isIndeterminate ? "..." : `${percentage}%` }}</span>
                    <button 
                        v-if="currentAction === t('installation.downloading')"
                        @click="cancelDownload(currentFile)" 
                        class="cancel-btn"
                        :title="t('common.cancel')"
                    >
                        ✕
                    </button>
                </span>
            </div>
            <div class="progress-bar-container">
                <div
                    v-if="!isIndeterminate"
                    class="progress-bar"
                    :style="{ width: `${percentage}%` }"
                />
                <div v-else class="progress-bar progress-bar--indeterminate" />
            </div>
        </div>
    </Transition>
</template>

<style scoped>
.download-progress {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: rgba(18, 18, 18, 0.95);
    padding: 10px 16px;
    z-index: 1000;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.progress-info {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 6px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.9);
    gap: 8px;
    min-width: 0;
}

.progress-label {
    display: flex;
    gap: 6px;
    align-items: baseline;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
}

.progress-file {
    color: rgba(255, 255, 255, 0.6);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
}

.progress-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
    white-space: nowrap;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.65);
}

.progress-size {
    color: rgba(255, 255, 255, 0.5);
}

.progress-speed {
    color: rgba(255, 255, 255, 0.75);
    font-weight: 500;
}

.progress-eta {
    color: rgba(255, 255, 255, 0.5);
}

.progress-pct {
    color: rgba(255, 255, 255, 0.9);
    font-weight: 500;
    min-width: 3ch;
    text-align: right;
}

.cancel-btn {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.4);
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
    transition: all 0.2s;
    line-height: 1;
}

.cancel-btn:hover {
    color: #ff4d4d;
    background-color: rgba(255, 77, 77, 0.1);
}

.progress-bar-container {
    height: 4px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
}

.progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #3b3b3b, #777777);
    border-radius: 2px;
    transition: width 0.2s ease-out;
}

.progress-bar--indeterminate {
    width: 40%;
    animation: indeterminate 1.4s ease-in-out infinite;
}

@keyframes indeterminate {
    0% {
        transform: translateX(-100%);
    }
    100% {
        transform: translateX(350%);
    }
}

.slide-up-enter-active,
.slide-up-leave-active {
    transition:
        transform 0.3s ease,
        opacity 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
    transform: translateY(100%);
    opacity: 0;
}
</style>
