<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { CirclePause, CirclePlay, Pause, Play, X } from "@lucide/vue";
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

interface DownloadSnapshot {
    file: string;
    percentage: number;
    downloaded: number;
    total: number;
    speed_bps: number;
    paused: boolean;
    waiting: boolean;
}

interface FileProgressState {
    percentage: number;
    downloadedBytes: number;
    totalBytes: number;
    speedBps: number;
    hasKnownTotal: boolean;
    action: string;
    paused: boolean;
    waiting: boolean;
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
const currentFile = computed(() => {
    const files = activeFiles.value;
    return (
        files.find((file) => {
            const state = fileProgress.value.get(file);
            return state && !state.paused && !state.waiting;
        }) ??
        files[0] ??
        ""
    );
});
const currentState = computed((): FileProgressState | undefined =>
    fileProgress.value.get(currentFile.value)
);
const isCurrentPaused = computed(() => currentState.value?.paused ?? false);
const isCurrentWaiting = computed(() => currentState.value?.waiting ?? false);

const downloadableFiles = computed(() =>
    activeFiles.value.filter((file) => {
        const state = fileProgress.value.get(file);
        return state && state.action !== t("installation.extracting");
    })
);
const allPaused = computed(
    () =>
        downloadableFiles.value.length > 0 &&
        downloadableFiles.value.every(
            (file) => fileProgress.value.get(file)?.paused
        )
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
    if (bps <= 0 || isCurrentPaused.value || isCurrentWaiting.value) return "";
    return `${humanSize(bps)}/s`;
});

const formattedEta = computed((): string => {
    const state = currentState.value;
    if (
        !state ||
        state.paused ||
        state.waiting ||
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
        !isCurrentPaused.value &&
        !isCurrentWaiting.value &&
        currentState.value?.action !== t("installation.extracting")
);

const makeInitialState = (action: string): FileProgressState => ({
    percentage: 0,
    downloadedBytes: 0,
    totalBytes: 0,
    speedBps: 0,
    hasKnownTotal: false,
    action,
    paused: false,
    waiting: false,
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
    const hasRunning = activeFiles.value.some((file) => {
        const state = fileProgress.value.get(file);
        return state && !state.paused && !state.waiting;
    });
    if (hasRunning) {
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

const pauseDownload = async (file: string) => {
    if (!file) return;
    try {
        const ok = await invoke<boolean>("pause_download", { name: file });
        if (ok) {
            updateFileProgress(file, {
                paused: true,
                waiting: false,
                action: t("installation.paused"),
                speedBps: 0,
            });
            updateMonitoringState();
        }
    } catch (e) {
        console.error("Failed to pause download:", e);
    }
};

const resumeDownload = async (file: string) => {
    if (!file) return;
    try {
        const ok = await invoke<boolean>("resume_download", { name: file });
        if (ok) {
            updateFileProgress(file, {
                paused: false,
                waiting: false,
                action: t("installation.downloading"),
            });
            updateMonitoringState();
        }
    } catch (e) {
        console.error("Failed to resume download:", e);
    }
};

const toggleAll = async () => {
    const shouldResume = allPaused.value;
    for (const file of downloadableFiles.value) {
        const state = fileProgress.value.get(file);
        if (!state) continue;
        if (shouldResume && state.paused) {
            await resumeDownload(file);
        } else if (!shouldResume && !state.paused) {
            await pauseDownload(file);
        }
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
            if (
                !isValidFile(data.file) ||
                !activeDownloads.value.has(data.file)
            )
                return;

            const isPaused = data.action === "paused";
            const isWaiting = data.action === "waiting";
            const previous = fileProgress.value.get(data.file);
            const wasIdle =
                (previous?.paused ?? false) || (previous?.waiting ?? false);

            if (!isPaused && !isWaiting && data.downloaded !== undefined) {
                downloadMonitor.onProgress(data.downloaded, data.percentage);
            }

            updateFileProgress(data.file, {
                action: isPaused
                    ? t("installation.paused")
                    : isWaiting
                      ? t("installation.waiting_network")
                      : t("installation.downloading"),
                paused: isPaused,
                waiting: isWaiting,
                percentage: data.percentage,
                ...(data.downloaded !== undefined
                    ? { downloadedBytes: data.downloaded }
                    : {}),
                ...(data.total !== undefined && data.total > 0
                    ? { totalBytes: data.total, hasKnownTotal: true }
                    : {}),
                speedBps: isPaused || isWaiting ? 0 : (data.speed_bps ?? 0),
            });

            if ((isPaused || isWaiting) !== wasIdle) {
                updateMonitoringState();
            }
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
                paused: false,
                waiting: false,
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
                paused: false,
                waiting: false,
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

    try {
        const snapshots = await invoke<DownloadSnapshot[]>(
            "get_download_states"
        );
        for (const snapshot of snapshots || []) {
            if (!isValidFile(snapshot.file)) continue;
            activeDownloads.value.add(snapshot.file);
            updateFileProgress(snapshot.file, {
                percentage: snapshot.percentage,
                downloadedBytes: snapshot.downloaded,
                totalBytes: snapshot.total,
                hasKnownTotal: snapshot.total > 0,
                speedBps: snapshot.speed_bps,
                paused: snapshot.paused,
                waiting: snapshot.waiting,
                action: snapshot.paused
                    ? t("installation.paused")
                    : snapshot.waiting
                      ? t("installation.waiting_network")
                      : t("installation.downloading"),
            });
        }
        updateMonitoringState();
    } catch (e) {
        console.error("Failed to restore download states:", e);
    }
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
            :class="{
                'download-progress--paused':
                    isCurrentPaused || isCurrentWaiting,
            }"
            role="status"
            aria-live="polite"
        >
            <div class="progress-info">
                <span
                    class="progress-label"
                    :class="{
                        'progress-label--paused':
                            isCurrentPaused || isCurrentWaiting,
                    }"
                >
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

                    <span class="ctrl-cluster">
                        <template
                            v-if="
                                currentAction !== t('installation.extracting')
                            "
                        >
                            <button
                                v-if="downloadableFiles.length > 1"
                                type="button"
                                class="icon-btn tooltip tooltip-top"
                                :class="{ 'icon-btn--resume': allPaused }"
                                :data-tip="
                                    allPaused
                                        ? t('installation.resume_all')
                                        : t('installation.pause_all')
                                "
                                :aria-label="
                                    allPaused
                                        ? t('installation.resume_all')
                                        : t('installation.pause_all')
                                "
                                @click="toggleAll"
                            >
                                <CirclePlay
                                    v-if="allPaused"
                                    :size="14"
                                    :stroke-width="2"
                                />
                                <CirclePause
                                    v-else
                                    :size="14"
                                    :stroke-width="2"
                                />
                            </button>
                            <button
                                v-if="!isCurrentPaused"
                                type="button"
                                class="icon-btn tooltip tooltip-top"
                                :data-tip="t('installation.pause')"
                                :aria-label="t('installation.pause')"
                                @click="pauseDownload(currentFile)"
                            >
                                <Pause :size="14" :stroke-width="2" />
                            </button>
                            <button
                                v-else
                                type="button"
                                class="icon-btn icon-btn--resume tooltip tooltip-top"
                                :data-tip="t('installation.resume')"
                                :aria-label="t('installation.resume')"
                                @click="resumeDownload(currentFile)"
                            >
                                <Play :size="14" :stroke-width="2" />
                            </button>
                            <button
                                type="button"
                                class="icon-btn icon-btn--danger tooltip tooltip-top"
                                :data-tip="t('common.cancel')"
                                :aria-label="t('common.cancel')"
                                @click="cancelDownload(currentFile)"
                            >
                                <X :size="14" :stroke-width="2" />
                            </button>
                        </template>
                    </span>
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
                    class="progress-file-entry"
                    :class="{
                        'progress-file-entry--paused':
                            entry.state?.paused || entry.state?.waiting,
                    }"
                >
                    <span
                        class="progress-file-item"
                        :style="{
                            '--fill': `${entry.state?.percentage ?? 0}%`,
                        }"
                    >
                        {{ entry.file }}
                    </span>
                    <span
                        v-if="
                            entry.state &&
                            entry.state.action !== t('installation.extracting')
                        "
                        class="queue-controls"
                    >
                        <button
                            v-if="!entry.state.paused"
                            type="button"
                            class="queue-btn tooltip tooltip-top"
                            :data-tip="t('installation.pause')"
                            :aria-label="`${t('installation.pause')}: ${entry.file}`"
                            @click="pauseDownload(entry.file)"
                        >
                            <Pause :size="10" :stroke-width="2.5" />
                        </button>
                        <button
                            v-else
                            type="button"
                            class="queue-btn queue-btn--resume tooltip tooltip-top"
                            :data-tip="t('installation.resume')"
                            :aria-label="`${t('installation.resume')}: ${entry.file}`"
                            @click="resumeDownload(entry.file)"
                        >
                            <Play :size="10" :stroke-width="2.5" />
                        </button>
                        <button
                            type="button"
                            class="queue-btn queue-btn--cancel tooltip tooltip-top"
                            :data-tip="t('common.cancel')"
                            :aria-label="`${t('common.cancel')}: ${entry.file}`"
                            @click="cancelDownload(entry.file)"
                        >
                            <X :size="10" :stroke-width="2.5" />
                        </button>
                    </span>
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

.download-progress--paused {
    border-top-color: rgba(251, 191, 36, 0.35);
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

.progress-label--paused {
    color: #fbbf24;
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

.progress-file-entry {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex: 0 0 auto;
    min-width: 0;
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

.progress-file-entry--paused .progress-file-item {
    background: linear-gradient(
        90deg,
        rgba(251, 191, 36, 1) 0%,
        rgba(251, 191, 36, 1) var(--fill),
        rgba(251, 191, 36, 0.35) var(--fill),
        rgba(251, 191, 36, 0.35) 100%
    );
    -webkit-background-clip: text;
    background-clip: text;
}

.queue-controls {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    opacity: 0.55;
    transition: opacity 0.15s ease;
}

.progress-file-entry:hover .queue-controls {
    opacity: 1;
}

.queue-btn {
    width: 18px;
    height: 18px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 5px;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    padding: 0;
    transition:
        transform 0.15s ease,
        background-color 0.12s ease,
        color 0.12s ease;
}

.queue-btn:hover {
    color: #ffffff;
    background-color: rgba(255, 255, 255, 0.12);
    transform: scale(1.08);
}

.queue-btn:active {
    transform: scale(0.9);
}

.queue-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.25);
}

.queue-btn--resume {
    color: rgba(251, 191, 36, 0.85);
}

.queue-btn--resume:hover {
    color: #fcd34d;
    background-color: rgba(251, 191, 36, 0.15);
}

.queue-btn--cancel:hover {
    color: #ff6b6b;
    background-color: rgba(255, 77, 77, 0.12);
}

.progress-file-entry:focus-within .queue-controls {
    opacity: 1;
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

.ctrl-cluster {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
}

.icon-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.65);
    cursor: pointer;
    padding: 0;
    transition:
        transform 0.15s ease,
        background-color 0.12s ease,
        color 0.12s ease;
}

.icon-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: #ffffff;
    transform: scale(1.06);
}

.icon-btn:active {
    transform: scale(0.92);
}

.icon-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.25);
}

.icon-btn--resume {
    color: #fbbf24;
}

.icon-btn--resume:hover {
    background-color: rgba(251, 191, 36, 0.15);
    color: #fcd34d;
}

.icon-btn--danger:hover {
    background-color: rgba(255, 77, 77, 0.12);
    color: #ff6b6b;
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

.download-progress--paused .progress-bar {
    background: linear-gradient(90deg, #b45309, #fbbf24);
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

@media (max-width: 640px) {
    .progress-info {
        flex-wrap: wrap;
        row-gap: 4px;
    }

    .progress-meta {
        flex-shrink: 1;
        min-width: 0;
        flex-wrap: wrap;
        row-gap: 4px;
        gap: 8px;
    }
}

@media (prefers-reduced-motion: reduce) {
    .icon-btn,
    .queue-btn {
        transition: none !important;
        transform: none !important;
    }
}
</style>
