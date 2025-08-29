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
    <div class="w-full flex flex-col">
        <div class="flex justify-between items-center p-3">
            <div class="flex items-center gap-3">
                <button @click="copyLogs" class="btn btn-ghost">
                    <Copy class="w-4 h-4" />
                    {{ $t('appLogs.copyLogs') }}
                </button>

                <label class="flex items-center gap-2 text-sm cursor-pointer">
                    <input class="toggle toggle-sm" type="checkbox" v-model="autoScroll" />
                    <span>{{ $t('appLogs.autoScroll') }}</span>
                </label>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 flex-1">
            <div class="card-body p-2 flex flex-col">
                <div ref="logsContainer"
                    class="flex-1 h-64 overflow-y-auto p-4 font-mono text-sm leading-6 whitespace-pre-wrap bg-base-100 rounded"
                    @scroll="handleScroll">
                    <div v-if="appLogs.length === 0" class="text-center p-5 text-neutral">
                        <p>{{ $t('appLogs.noLogs') }}</p>
                    </div>
                    <pre v-else class="whitespace-pre-wrap"><code>{{ appLogs.join('\n') }}</code></pre>
                </div>
            </div>
        </div>
    </div>
</template>

<!-- Styling replaced by Tailwind + daisyUI utility classes -->
