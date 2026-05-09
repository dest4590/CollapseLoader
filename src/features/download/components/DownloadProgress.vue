<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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

const fileProgress = ref<Map<string, FileProgressState>>(new Map());
const listeners = ref<UnlistenFn[]>([]);
const activeDownloads = ref<Set<string>>(new Set());

const isValidFile = (filename: string): boolean => Boolean(filename?.trim());

const activeCount = computed(() => activeDownloads.value.size);
const isVisible = computed(() => activeCount.value > 0);

const humanSize = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024)
        return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
};

const activeFiles = computed(() => [...activeDownloads.value]);
const currentFile = computed(() => activeFiles.value[0] ?? "");
const currentState = computed((): FileProgressState | undefined =>
    fileProgress.value.get(currentFile.value)
);

const visibleFileEntries = computed(() =>
    activeFiles.value.slice(0, 5).map((file) => ({
        file,
        state: fileProgress.value.get(file),
    }))
);
const hasMoreText = computed(
    () => activeFiles.value.length > visibleFileEntries.value.length
);
const displayText = computed(() => {
    if (visibleFileEntries.value.length === 0) return "";
    const names = visibleFileEntries.value.map((entry) => entry.file);
    return hasMoreText.value ? `${names.join(", ")}, ...` : names.join(", ");
});

const percentage = computed(() =>
    Math.min(100, Math.max(0, currentState.value?.percentage ?? 0))
);
const currentAction = computed(
    () => currentState.value?.action ?? t("installation.downloading")
);

const formattedSpeed = computed((): string => {
    const bps = currentState.value?.speedBps ?? 0;
    if (bps <= 0) return "";
    return `${humanSize(bps)}/s`;
});

const formattedEta = computed((): string => {
    const state = currentState.value;
    if (
        !state ||
        !state.hasKnownTotal ||
        state.speedBps <= 0 ||
        state.totalBytes <= 0 ||
        state.downloadedBytes >= state.totalBytes
    ) {
        return "";
    }
    const remaining = state.totalBytes - state.downloadedBytes;
    const etaSec = Math.ceil(remaining / state.speedBps);

    if (etaSec < 60) return `${etaSec}s`;
    if (etaSec < 3600) {
        const minutes = Math.floor(etaSec / 60);
        const seconds = etaSec % 60;
        return `${minutes}m ${seconds}s`;
    }

    const hours = Math.floor(etaSec / 3600);
    const minutes = Math.floor((etaSec % 3600) / 60);
    return `${hours}h ${minutes}m`;
});

const formattedProgress = computed((): string => {
    const state = currentState.value;
    if (!state) return "";

    if (state.action === t("installation.extracting")) {
        if (state.totalBytes > 0) {
            return `${state.downloadedBytes} / ${state.totalBytes} files`;
        }
        if (state.downloadedBytes > 0) {
            return `${state.downloadedBytes} files`;
        }
        return "";
    }

    if (
        state.downloadedBytes > 0 &&
        state.totalBytes > 0 &&
        state.hasKnownTotal
    ) {
        return `${humanSize(state.downloadedBytes)} / ${humanSize(state.totalBytes)}`;
    }
    if (state.downloadedBytes > 0) {
        return humanSize(state.downloadedBytes);
    }
    return "";
});

const isIndeterminate = computed(
    () =>
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

const updateFileProgress = (
    file: string,
    patch: Partial<FileProgressState>
) => {
    const existing =
        fileProgress.value.get(file) ??
        makeInitialState(t("installation.downloading"));
    fileProgress.value.set(file, { ...existing, ...patch });
    fileProgress.value = new Map(fileProgress.value);
};

const removeFileProgress = (file: string) => {
    fileProgress.value.delete(file);
    fileProgress.value = new Map(fileProgress.value);
};

const updateMonitoringState = () => {
    if (activeDownloads.value.size > 0) {
        downloadMonitor.startMonitoring();
    } else {
        downloadMonitor.stopMonitoring();
    }
};

const deactivateFile = (file: string) => {
    activeDownloads.value.delete(file);
    removeFileProgress(file);
    updateMonitoringState();
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
            if (!isValidFile(filename)) return;

            activeDownloads.value.add(filename);
            updateFileProgress(
                filename,
                makeInitialState(t("installation.downloading"))
            );
            updateMonitoringState();
        })
    );

    listeners.value.push(
        await listen("download-progress", (event: any) => {
            const data = event.payload as ProgressEvent;
            if (data.downloaded !== undefined) {
                downloadMonitor.onProgress(data.downloaded, data.percentage);
            }

            if (
                !isValidFile(data.file) ||
                !activeDownloads.value.has(data.file)
            )
                return;

            updateFileProgress(data.file, {
                action: t("installation.downloading"),
                percentage: data.percentage,
                ...(data.downloaded !== undefined
                    ? { downloadedBytes: data.downloaded }
                    : {}),
                ...(data.total !== undefined && data.total > 0
                    ? { totalBytes: data.total, hasKnownTotal: true }
                    : {}),
                ...(data.speed_bps !== undefined
                    ? { speedBps: data.speed_bps }
                    : {}),
            });
        })
    );

    listeners.value.push(
        await listen("download-complete", (event: any) => {
            const filename = event.payload as string;
            if (!isValidFile(filename)) return;
            if (filename.endsWith(".zip")) return;

            deactivateFile(filename);
        })
    );

    listeners.value.push(
        await listen("unzip-start", (event: any) => {
            const filename = event.payload as string;
            if (!isValidFile(filename) || !activeDownloads.value.has(filename))
                return;

            updateFileProgress(filename, {
                action: t("installation.extracting"),
                percentage: 0,
                downloadedBytes: 0,
                totalBytes: 0,
                speedBps: 0,
                hasKnownTotal: false,
            });
        })
    );

    listeners.value.push(
        await listen("unzip-progress", (event: any) => {
            const data = event.payload as ProgressEvent;
            if (
                !isValidFile(data.file) ||
                !activeDownloads.value.has(data.file)
            )
                return;

            updateFileProgress(data.file, {
                action: t("installation.extracting"),
                percentage: data.percentage,
                ...(data.files_extracted !== undefined
                    ? { downloadedBytes: data.files_extracted }
                    : {}),
                ...(data.total_files !== undefined && data.total_files > 0
                    ? { totalBytes: data.total_files, hasKnownTotal: true }
                    : {}),
            });
        })
    );

    listeners.value.push(
        await listen("unzip-complete", (event: any) => {
            const filename = event.payload as string;
            if (!isValidFile(filename)) return;

            deactivateFile(filename);
        })
    );

    listeners.value.push(
        await listen("requirements-status", (event: any) => {
            const isDownloading = event.payload as boolean;
            if (!isDownloading) {
                activeDownloads.value.clear();
                fileProgress.value = new Map();
                updateMonitoringState();
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
        <div
            v-if="isVisible"
            class="download-progress"
            role="status"
            aria-live="polite"
        >
            <div class="progress-info">
                <span class="progress-label">
                    {{ currentAction }}
                </span>
                <span class="progress-meta">
                    <span v-if="formattedProgress" class="progress-size">{{
                        formattedProgress
                    }}</span>
                    <span v-if="formattedSpeed" class="progress-speed">{{
                        formattedSpeed
                    }}</span>
                    <span v-if="formattedEta" class="progress-eta">{{
                        formattedEta
                    }}</span>
                    <span class="progress-pct">{{
                        isIndeterminate ? "..." : `${percentage}%`
                    }}</span>
                    <button
                        v-if="currentAction === t('installation.downloading')"
                        @click="cancelDownload(currentFile)"
                        class="cancel-btn"
                        :aria-label="t('common.cancel')"
                        :title="t('common.cancel')"
                    >
                        ✕
                    </button>
                </span>
            </div>
            <TransitionGroup
                name="queue-slide"
                tag="div"
                class="progress-file-queue"
                :title="displayText"
            >
                <span
                    v-for="entry in visibleFileEntries"
                    :key="entry.file"
                    class="progress-file-item"
                    :style="{ '--fill': `${entry.state?.percentage ?? 0}%` }"
                >
                    {{ entry.file }}
                </span>
                <span v-if="hasMoreText" key="more-count" class="progress-more">
                    +{{ activeFiles.length - visibleFileEntries.length }} more
                </span>
            </TransitionGroup>
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

.queue-animate {
    color: rgba(255, 255, 255, 0.7);
}

.queue-static {
    color: rgba(255, 255, 255, 0.7);
}

.progress-file-queue {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
}

.progress-file-item {
    position: relative;
    display: inline-block;
    align-items: center;
    padding: 0 4px;
    font-size: 12px;
    line-height: 1.2;
    color: transparent;
    white-space: nowrap;
    min-width: max-content;
    will-change: transform, opacity;
    backface-visibility: hidden;
    transform-origin: left center;
    background: linear-gradient(
        90deg,
        rgba(255, 255, 255, 1) 0%,
        rgba(255, 255, 255, 1) var(--fill),
        rgba(255, 255, 255, 0.35) var(--fill),
        rgba(255, 255, 255, 0.35) 100%
    );
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
}

.progress-more {
    color: rgba(255, 255, 255, 0.65);
    font-size: 12px;
    white-space: nowrap;
}

.queue-slide-enter-from {
    opacity: 0;
    transform: translateX(12px);
}

.queue-slide-enter-to {
    opacity: 1;
    transform: translateX(0);
}

.queue-slide-leave-from {
    opacity: 1;
    transform: translateX(0);
}

.queue-slide-leave-to {
    opacity: 0;
    transform: translateX(-12px);
}

.queue-slide-enter-active {
    transition:
        opacity 220ms ease,
        transform 220ms ease;
}

.queue-slide-leave-active {
    transition:
        opacity 220ms ease,
        transform 220ms ease;
    position: absolute;
}

.queue-slide-move {
    transition:
        transform 300ms cubic-bezier(0.2, 0.8, 0.2, 1),
        opacity 180ms ease;
}

.progress-file-item {
    flex: 0 0 auto;
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
