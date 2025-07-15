<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

interface ProgressEvent {
    file: string;
    percentage: number;
    downloaded?: number;
    total?: number;
    action?: string;
    files_extracted?: number;
    total_files?: number;
}

const isVisible = ref(false);
const percentage = ref(0);
const currentFile = ref('');
const currentAction = ref('Downloading');
const listeners = ref<any[]>([]);
const activeDownloads = ref<Set<string>>(new Set());

const isEssentialFile = (filename: string): boolean => {
    const essentialFiles = [
        'jdk-21.0.2.zip',
        'assets.zip',
        'natives.zip',
        'libraries.zip',
        'natives-1.12.zip',
        'libraries-1.12.zip',
    ];

    return essentialFiles.includes(filename);
};

const isClientFile = (filename: string): boolean => {
    return filename.endsWith('.jar');
};

const shouldShowProgress = (filename: string): boolean => {
    return isEssentialFile(filename) && !isClientFile(filename);
};

onMounted(async () => {
    listeners.value.push(
        await listen('download-start', (event: any) => {
            const filename = event.payload as string;
            if (shouldShowProgress(filename)) {
                activeDownloads.value.add(filename);
                currentFile.value = filename;
                currentAction.value = t('installation.downloading');
                percentage.value = 0;
                isVisible.value = true;
            }
        })
    );

    listeners.value.push(
        await listen('download-progress', (event: any) => {
            const data = event.payload as ProgressEvent;
            if (
                shouldShowProgress(data.file) &&
                activeDownloads.value.has(data.file)
            ) {
                currentFile.value = data.file;
                currentAction.value = t('installation.downloading');
                percentage.value = data.percentage;
                isVisible.value = true;
            }
        })
    );

    listeners.value.push(
        await listen('download-complete', (event: any) => {
            const filename = event.payload as string;
            if (shouldShowProgress(filename)) {
                if (!filename.endsWith('.zip')) {
                    activeDownloads.value.delete(filename);
                    if (activeDownloads.value.size === 0) {
                        isVisible.value = false;
                    }
                }
            }
        })
    );

    listeners.value.push(
        await listen('unzip-start', (event: any) => {
            const filename = event.payload as string;
            if (
                shouldShowProgress(filename) &&
                activeDownloads.value.has(filename)
            ) {
                currentFile.value = filename;
                currentAction.value = t('installation.extracting');
                percentage.value = 0;
                isVisible.value = true;
            }
        })
    );

    listeners.value.push(
        await listen('unzip-progress', (event: any) => {
            const data = event.payload as ProgressEvent;
            if (
                shouldShowProgress(data.file) &&
                activeDownloads.value.has(data.file)
            ) {
                currentFile.value = data.file;
                currentAction.value = t('installation.extracting');
                percentage.value = data.percentage;
            }
        })
    );

    listeners.value.push(
        await listen('unzip-complete', (event: any) => {
            const filename = event.payload as string;
            if (shouldShowProgress(filename)) {
                activeDownloads.value.delete(filename);
                if (activeDownloads.value.size === 0) {
                    isVisible.value = false;
                }
            }
        })
    );

    listeners.value.push(
        await listen('requirements-status', (event: any) => {
            const isDownloading = event.payload as boolean;
            if (!isDownloading) {
                activeDownloads.value.clear();
                isVisible.value = false;
            }
        })
    );
});

onUnmounted(() => {
    listeners.value.forEach((unlisten) => unlisten());
});
</script>

<template>
    <Transition name="slide-up">
        <div v-if="isVisible" class="download-progress">
            <div class="progress-info">
                <span>{{ currentAction }} {{ currentFile }}</span>
                <span>{{ percentage }}%</span>
            </div>
            <div class="progress-bar-container">
                <div class="progress-bar" :style="{ width: `${percentage}%` }"></div>
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
    padding: 12px 16px;
    z-index: 1000;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.progress-info {
    display: flex;
    justify-content: space-between;
    margin-bottom: 6px;
    font-size: 14px;
    color: rgba(255, 255, 255, 0.9);
}

.progress-bar-container {
    height: 6px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    overflow: hidden;
}

.progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #3b3b3b, #777777);
    border-radius: 3px;
    transition: width 0.2s ease-out;
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
