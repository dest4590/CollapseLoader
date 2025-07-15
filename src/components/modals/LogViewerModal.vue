<template>
    <div class="flex flex-col h-full max-h-[80vh]">
        <div class="flex items-center justify-start gap-3 pb-4 border-b border-base-300/30 flex-shrink-0">
            <label class="flex items-center gap-2 text-sm cursor-pointer">
                <input type="checkbox" class="checkbox checkbox-sm" v-model="autoScroll" />
                <span>{{ t('appLogs.autoScroll') }}</span>
            </label>
        </div>

        <div ref="logsContainer"
            class="flex-1 p-4 font-mono text-sm leading-6 whitespace-pre-wrap break-words text-base-content/90 bg-base-300/30 rounded mt-4 overflow-y-auto min-h-0"
            @scroll="handleScroll" style="max-height: calc(80vh - 120px);">
            <div v-if="logs.length === 0" class="text-center py-8 text-base-content/60">
                {{ t('appLogs.noLogs') }}
            </div>
            <pre v-else
                class="m-0 whitespace-pre-wrap break-words overflow-wrap-anywhere"><code>{{ logs.join('\n') }}</code></pre>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

interface Props {
    clientId: number;
    clientName: string;
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
    if (props.clientId) {
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

onMounted(() => {
    fetchLogs();
    refreshInterval.value = setInterval(fetchLogs, 1000) as unknown as number;
});

onUnmounted(() => {
    if (refreshInterval.value !== null) {
        clearInterval(refreshInterval.value);
        refreshInterval.value = null;
    }
});
</script>

<style scoped>
.font-mono {
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

/* Custom scrollbar for logs container */
.overflow-y-auto::-webkit-scrollbar {
    width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.3);
    border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.4);
}

@media (max-width: 768px) {
    .text-sm {
        font-size: 0.75rem;
    }

    .p-4 {
        padding: 0.75rem;
    }

    .gap-3 {
        gap: 0.5rem;
    }
}

@media (max-width: 480px) {
    .text-sm {
        font-size: 0.6875rem;
    }

    .p-4 {
        padding: 0.5rem;
    }
}
</style>
