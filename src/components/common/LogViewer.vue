<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { X } from 'lucide-vue-next';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

interface Props {
    clientId: number;
    clientName: string;
    isOpen: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits(['close']);

const logs = ref<string[]>([]);
const logsContainer = ref<HTMLDivElement | null>(null);
const autoScroll = ref(true);
const refreshInterval = ref<number | null>(null);
const { addToast } = useToast();
const { t } = useI18n();

const fetchLogs = async () => {
    if (props.clientId && props.isOpen) {
        try {
            logs.value = await invoke<string[]>('get_client_logs', {
                id: props.clientId,
            });
            if (autoScroll.value && logsContainer.value) {
                scrollToBottom();
            }
        } catch (err) {
            console.error('Error fetching client logs:', err);
            addToast(t('errors.logs_error', { error: err }), 'error');
        }
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

const handleScroll = () => {
    if (!logsContainer.value) return;

    const { scrollTop, scrollHeight, clientHeight } = logsContainer.value;
    const isScrolledToBottom = scrollTop + clientHeight >= scrollHeight - 10;

    if (isScrolledToBottom !== autoScroll.value) {
        autoScroll.value = isScrolledToBottom;
    }
};

watch(autoScroll, (newValue) => {
    if (newValue) {
        scrollToBottom();
    }
});

watch(
    () => props.isOpen,
    (newVal) => {
        if (newVal) {
            fetchLogs();
            if (refreshInterval.value === null) {
                refreshInterval.value = setInterval(
                    fetchLogs,
                    1000
                ) as unknown as number;
            }
        } else {
            if (refreshInterval.value !== null) {
                clearInterval(refreshInterval.value);
                refreshInterval.value = null;
            }
        }
    }
);

onMounted(() => {
    if (props.isOpen) {
        fetchLogs();
        refreshInterval.value = setInterval(
            fetchLogs,
            1000
        ) as unknown as number;
    }
});

onUnmounted(() => {
    if (refreshInterval.value !== null) {
        clearInterval(refreshInterval.value);
    }
});

const close = () => {
    emit('close');
};
</script>

<template>
    <div v-if="isOpen" class="log-viewer-overlay" @click="close">
        <div class="log-viewer-modal" @click.stop>
            <div class="log-viewer-header">
                <h3>{{ t('logs.title', { client: clientName }) }}</h3>
                <div class="log-viewer-actions">
                    <label class="auto-scroll-toggle">
                        <input type="checkbox" class="checkbox" v-model="autoScroll" />
                        <span>{{ t('appLogs.autoScroll') }}</span>
                    </label>
                    <button class="close-btn" @click="close">
                        <X class="w-4" />
                    </button>
                </div>
            </div>
            <div ref="logsContainer" class="log-viewer-content" @scroll="handleScroll">
                <div v-if="logs.length === 0" class="no-logs">
                    {{ t('appLogs.noLogs') }}
                </div>
                <pre v-else><code>{{ logs.join('\n') }}</code></pre>
            </div>
        </div>
    </div>
</template>

<style scoped>
.log-viewer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
}

.log-viewer-modal {
    width: 80%;
    height: 80%;
    max-height: 90vh;
    background-color: var(--color-base-300, #1d1d1d);
    border-radius: 8px;
    box-shadow: 0 4px 25px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.log-viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
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

.close-btn {
    background: none;
    border: none;
    color: #999;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.close-btn:hover {
    color: white;
    background-color: rgba(255, 255, 255, 0.1);
}

.log-viewer-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 16px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    color: #ddd;
    background-color: var(--color-base-200, #282828);
    min-height: 0;
    word-wrap: break-word;
}

.log-viewer-content::-webkit-scrollbar {
    width: 8px;
}

.log-viewer-content::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
}

.log-viewer-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.3);
    border-radius: 4px;
}

.log-viewer-content::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.4);
}

.no-logs {
    text-align: center;
    padding: 20px;
    color: #888;
}
</style>
