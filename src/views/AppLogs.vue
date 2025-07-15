<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../services/toastService';
import { Copy } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const appLogs = ref<string[]>([]);
const logsContainer = ref<HTMLDivElement | null>(null);
const autoScroll = ref(true);
const { addToast } = useToast();
const { t } = useI18n();
const refreshInterval = ref<number | null>(null);

const fetchAllLogs = async () => {
    try {
        appLogs.value = await invoke<string[]>('get_app_logs');
        if (autoScroll.value && logsContainer.value) {
            scrollToBottom();
        }
    } catch (err) {
        console.error('Error fetching app logs:', err);
        addToast(t('toast.logs.fetch_failed', { error: err }), 'error');
    }
};

const scrollToBottom = () => {
    if (logsContainer.value) {
        setTimeout(() => {
            if (logsContainer.value) {
                logsContainer.value.scrollTop =
                    logsContainer.value.scrollHeight;
            }
        }, 50);
    }
};

const copyLogs = () => {
    if (appLogs.value.length > 0) {
        const logsText = appLogs.value.join('\n');
        navigator.clipboard
            .writeText(logsText)
            .then(() => {
                console.log('Logs copied to clipboard');
                addToast(t('toast.logs.copied_success'), 'success');
            })
            .catch((err) => {
                console.error('Failed to copy logs:', err);
                addToast(t('toast.logs.copy_failed', { error: err }), 'error');
            });
    }
};

const handleScroll = () => {
    if (!logsContainer.value) return;

    const { scrollTop, scrollHeight, clientHeight } = logsContainer.value;
    const isScrolledToBottom = scrollTop + clientHeight >= scrollHeight - 10;

    if (!isScrolledToBottom && autoScroll.value) {
        autoScroll.value = false;
    } else if (isScrolledToBottom && !autoScroll.value) {
        autoScroll.value = true;
    }
};

watch(autoScroll, (newValue) => {
    if (newValue) {
        scrollToBottom();
    }
});

onMounted(() => {
    fetchAllLogs();
    if (refreshInterval.value === null) {
        refreshInterval.value = setInterval(
            fetchAllLogs,
            1000
        ) as unknown as number;
    }

    if (autoScroll.value) {
        scrollToBottom();
    }
});

onUnmounted(() => {
    if (refreshInterval.value !== null) {
        clearInterval(refreshInterval.value);
        refreshInterval.value = null;
    }
});
</script>

<template>
    <div class="app-log-view-container slide-up">
        <div class="log-viewer-header">
            <div class="log-viewer-actions">
                <button @click="copyLogs" class="btn btn-ghost">
                    <Copy class="w-4 h-4"></Copy>
                    {{ $t('appLogs.copyLogs') }}
                </button>

                <label class="auto-scroll-toggle">
                    <input class="checkbox" type="checkbox" v-model="autoScroll" />
                    <span>{{ $t('appLogs.autoScroll') }}</span>
                </label>
            </div>
        </div>
        <div class="card bg-base-200 shadow-md border border-base-300">
            <div class="card-body p-2">
                <div ref="logsContainer" class="log-viewer-content" @scroll="handleScroll">
                    <div v-if="appLogs.length === 0" class="no-logs">
                        <p>{{ $t('appLogs.noLogs') }}</p>
                    </div>
                    <pre v-else><code>{{ appLogs.join('\n') }}</code></pre>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.app-log-view-container {
    width: 100%;
    display: flex;
    flex-direction: column;
}

.animate-fadeInUp {
    animation: fadeInUp 0.5s ease-out forwards;
    opacity: 0;
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(10px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.log-viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
}

.log-viewer-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
}

.log-viewer-actions {
    display: flex;
    align-items: center;
    gap: 12px;
}

.auto-scroll-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    cursor: pointer;
}

.auto-scroll-toggle input {
    cursor: pointer;
}

.card {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.card-body {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    padding: 0;
}

.log-viewer-content {
    flex-grow: 1;
    overflow-y: auto;
    padding: 16px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    color: #ddd;
}

.log-viewer-content pre {
    margin: 0;
}

.no-logs {
    text-align: center;
    padding: 20px;
    color: #888;
}
</style>
