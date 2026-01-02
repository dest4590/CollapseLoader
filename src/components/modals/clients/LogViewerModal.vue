<template>
    <div class="flex flex-col h-full max-h-[80vh]">
        <div class="flex flex-col gap-4 pb-4 border-b border-base-300/30 shrink-0">
            <div class="flex flex-wrap items-center justify-between gap-4">
                <div class="flex items-center gap-4">
                    <label class="flex items-center gap-2 text-sm cursor-pointer">
                        <input type="checkbox" class="checkbox checkbox-sm checkbox-primary" v-model="autoScroll" />
                        <span>{{ t('appLogs.autoScroll') }}</span>
                    </label>

                    <div class="join">
                        <div class="join-item flex items-center px-3 bg-base-200 border border-base-300">
                            <Search class="w-4 h-4 opacity-50" />
                        </div>
                        <input v-model="searchQuery" type="text" :placeholder="t('appLogs.searchPlaceholder')"
                            class="input input-sm join-item bg-base-200 border-base-300 focus:outline-none" />
                    </div>
                </div>

                <div class="flex items-center gap-2">
                    <button @click="downloadLogs" class="btn btn-sm btn-ghost gap-2" :title="t('appLogs.download')">
                        <Download class="w-4 h-4" />
                        <span class="hidden sm:inline">{{ t('appLogs.download') }}</span>
                    </button>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
                <span class="text-xs font-semibold opacity-50 uppercase mr-2">{{ t('appLogs.filterByLevel') }}:</span>
                <button v-for="level in levels" :key="level.id" @click="toggleLevel(level.id)"
                    class="badge cursor-pointer transition-all duration-200" :class="[
                        selectedLevels.includes(level.id)
                            ? `${level.color} border-transparent`
                            : 'badge-ghost opacity-40 grayscale'
                    ]">
                    {{ level.id }}
                </button>
            </div>
        </div>

        <div ref="logsContainer"
            class="flex-1 p-4 font-mono text-sm leading-6 whitespace-pre-wrap wrap-break-word text-base-content/90 bg-base-300/30 rounded mt-4 overflow-y-auto min-h-0"
            @scroll="handleScroll" style="max-height: calc(80vh - 160px);">
            <div v-if="filteredLogs.length === 0" class="text-center py-8 text-base-content/60">
                {{ searchQuery || selectedLevels.length < levels.length ? t('appLogs.noMatchingLogs') :
                    t('appLogs.noLogs') }} </div>
                    <div v-else class="space-y-1">
                        <div v-for="(log, idx) in filteredLogs" :key="idx"
                            class="w-full rounded px-2 py-0.5 border-l-2 border-transparent hover:bg-base-200/30 transition-colors"
                            :class="getLogLineClass(log)">
                            {{ log }}
                        </div>
                    </div>
            </div>
        </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../../services/toastService';
import { useI18n } from 'vue-i18n';
import { Search, Download } from 'lucide-vue-next';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';

interface Props {
    clientId: number;
    clientName: string;
}

const props = defineProps<Props>();

defineEmits(['close']);

const logs = ref<string[]>([]);
const logsContainer = ref<HTMLDivElement | null>(null);
const autoScroll = ref(true);
const refreshInterval = ref<number | null>(null);
const { addToast } = useToast();
const { t } = useI18n();

const searchQuery = ref('');
const selectedLevels = ref<string[]>(['INFO', 'WARN', 'ERROR', 'DEBUG', 'FATAL', 'COLLAPSE']);
const levels = [
    { id: 'INFO', color: 'badge-info' },
    { id: 'WARN', color: 'badge-warning' },
    { id: 'ERROR', color: 'badge-error' },
    { id: 'DEBUG', color: 'badge-neutral' },
    { id: 'FATAL', color: 'badge-error' },
    { id: 'COLLAPSE', color: 'badge-primary' }
];

const toggleLevel = (level: string) => {
    if (selectedLevels.value.includes(level)) {
        selectedLevels.value = selectedLevels.value.filter(l => l !== level);
    } else {
        selectedLevels.value.push(level);
    }
};

const filteredLogs = computed(() => {
    if (!searchQuery.value && selectedLevels.value.length === levels.length) {
        return logs.value;
    }

    return logs.value.filter(log => {
        const logUpper = log.toUpperCase();
        const matchesSearch = !searchQuery.value || log.toLowerCase().includes(searchQuery.value.toLowerCase());

        const logHasLevel = levels.find(l => {
            if (l.id === 'COLLAPSE') {
                const low = log.toLowerCase();
                return low.includes('[collapsewarden]') || low.includes('[collapseagent]') || low.includes('[collapsenative]');
            }

            return logUpper.includes(l.id);
        });

        if (logHasLevel) {
            return matchesSearch && selectedLevels.value.includes(logHasLevel.id);
        }

        return matchesSearch;
    });
});

const getLogLineClass = (log: string) => {
    const lower = log.toLowerCase();

    if (lower.includes('[collapsewarden]')) {
        return 'bg-info/10 border-info';
    }

    if (lower.includes('[collapseagent]')) {
        return 'bg-success/10 border-success';
    }

    if (lower.includes('[collapsenative]')) {
        return 'bg-warning/10 border-warning';
    }

    return '';
};

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

const downloadLogs = async () => {
    try {
        const filePath = await save({
            filters: [{
                name: 'Log Files',
                extensions: ['log', 'txt']
            }],
            defaultPath: `${props.clientName}_logs.log`
        });

        if (filePath) {
            await writeTextFile(filePath, logs.value.join('\n'));
            addToast(t('toast.logs.download_success'), 'success');
        }
    } catch (err) {
        console.error('Error downloading logs:', err);
        addToast(t('toast.logs.download_failed', { error: err }), 'error');
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
</style>
