<template>
    <div class="client-crash-modal">
        <p v-if="error" class="error-message">
            {{ $t('modals.client_crash.error_occurred', { error }) }}
        </p>
        <p v-else>
            {{ $t('modals.client_crash.crashed_message', { clientName }) }}
        </p>

        <div class="logs-container">
            <pre><code>{{ formattedLogs }}</code></pre>
        </div>

        <div class="modal-actions">
            <button @click="copyLogs" class="btn btn-sm btn-outline">
                {{ $t('modals.client_crash.copy_logs') }}
            </button>
            <button @click="closeModal" class="btn btn-sm btn-primary">
                {{ $t('modals.client_crash.close') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

interface Props {
    clientId: number;
    clientName: string;
    logs: string[];
    error?: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['close']);

const { addToast } = useToast();
const { t } = useI18n();

const formattedLogs = computed(() => {
    if (!props.logs || props.logs.length === 0) {
        return t('modals.client_crash.no_logs');
    }
    return props.logs.join('\n');
});

const closeModal = () => {
    emit('close');
};

const copyLogs = async () => {
    try {
        await navigator.clipboard.writeText(formattedLogs.value);
        addToast(t('toast.logs.client_logs_copied'), 'success');
    } catch (err) {
        console.error('Failed to copy logs: ', err);
        addToast(t('toast.logs.client_logs_copy_failed'), 'error');
    }
};
</script>

<style scoped>
.client-crash-modal {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: calc(100vh - 9rem);
}

.error-message {
    color: var(--color-error, #f44336);
    font-weight: bold;
}

.logs-container {
    background-color: var(--color-base-300, #1d1d1d);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.375rem;
    padding: 0.75rem;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    max-height: calc(60vh - 6rem);
}

.logs-container h3 {
    margin-top: 0;
    margin-bottom: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
}

.logs-container pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
    font-family: monospace;
    font-size: 0.875rem;
    color: var(--color-base-content, white);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: auto;
}
</style>
