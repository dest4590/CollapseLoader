<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { useModal } from '../../services/modalService';
import { Trash2 } from 'lucide-vue-next';
import type { CustomClient } from '../../types/ui';

const { addToast } = useToast();
const { getModals } = useModal();

const emit = defineEmits<{
    'client-deleted': [];
    'close': []
}>();

const loading = ref(false);
const currentClient = ref<CustomClient | null>(null);

const handleDelete = async () => {
    if (!currentClient.value) {
        return;
    }

    try {
        loading.value = true;

        await invoke('remove_custom_client', {
            id: currentClient.value.id,
        });

        emit('client-deleted');
        emit('close');
    } catch (err) {
        addToast(`Failed to delete custom client: ${err}`, 'error');
    } finally {
        loading.value = false;
    }
};

const modals = getModals();

watch(() => modals['delete-custom-client-confirm']?.props?.client, (client: CustomClient | undefined) => {
    if (client) {
        currentClient.value = client;
    }
}, { immediate: true });
</script>

<template>
    <div class="flex items-center gap-3 mb-4">
        <div class="flex-shrink-0">
            <div class="w-12 h-12 bg-error/20 rounded-full flex items-center justify-center">
                <Trash2 class="w-6 h-6 text-error" />
            </div>
        </div>
        <div>
            <h3 class="font-bold text-lg">{{ $t('modals.delete_custom_client_confirm_modal.title') }}</h3>
            <p class="text-base-content/70 text-sm">
                {{ $t('modals.delete_custom_client_confirm_modal.description') }}
            </p>
        </div>
    </div>

    <div class="modal-action">
        <button type="button" class="btn" @click="$emit('close')" :disabled="loading">
            {{ $t('common.cancel') }}
        </button>
        <button type="button" class="btn btn-error" @click="handleDelete" :disabled="loading">
            <div v-if="loading" class="loading loading-spinner loading-sm"></div>
            {{ loading ? $t('modals.delete_custom_client_confirm_modal.deleting') : $t('modals.delete_custom_client_confirm_modal.delete_client') }}
        </button>
    </div>
</template>