<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { useModal } from '../../services/modalService';
import { AlertTriangle, Trash2 } from 'lucide-vue-next';
import type { CustomClient } from '../../types/ui';

const { addToast } = useToast();
const { getModals } = useModal();

const emit = defineEmits<{
    'client-deleted': [];
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
    } catch (err) {
        addToast(`Failed to delete custom client: ${err}`, 'error');
    } finally {
        loading.value = false;
    }
};

const modals = getModals();
const isOpen = computed(() => modals['delete-custom-client-confirm']?.open || false);

watch(() => modals['delete-custom-client-confirm']?.props?.client, (client: CustomClient | undefined) => {
    if (client) {
        currentClient.value = client;
    }
}, { immediate: true });
</script>

<template>
    <div v-if="isOpen" class="modal modal-open">
        <div class="modal-box">
            <div class="flex items-center gap-3 mb-4">
                <div class="flex-shrink-0">
                    <div class="w-12 h-12 bg-error/20 rounded-full flex items-center justify-center">
                        <Trash2 class="w-6 h-6 text-error" />
                    </div>
                </div>
                <div>
                    <h3 class="font-bold text-lg">Delete Custom Client</h3>
                    <p class="text-base-content/70 text-sm">
                        This action cannot be undone
                    </p>
                </div>
            </div>

            <div v-if="currentClient" class="mb-6">
                <div class="alert alert-warning">
                    <AlertTriangle class="w-4 h-4" />
                    <span>
                        Are you sure you want to delete <strong>{{ currentClient.name }}</strong>?
                    </span>
                </div>

                <div class="bg-base-200 p-4 rounded-lg mt-4">
                    <h4 class="font-semibold mb-2">Client Details:</h4>
                    <div class="space-y-1 text-sm">
                        <div><strong>Name:</strong> {{ currentClient.name }}</div>
                        <div><strong>Version:</strong> {{ currentClient.version }}</div>
                        <div><strong>Main Class:</strong> {{ currentClient.main_class }}</div>
                        <div><strong>Launches:</strong> {{ currentClient.launches }}</div>
                    </div>
                </div>

                <div class="alert alert-error mt-4">
                    <AlertTriangle class="w-4 h-4" />
                    <span class="text-sm">
                        This will permanently delete the client file and remove it from your custom clients list.
                    </span>
                </div>
            </div>

            <div class="modal-action">
                <button type="button" class="btn" @click="$emit('client-deleted')" :disabled="loading">
                    Cancel
                </button>
                <button type="button" class="btn btn-error" @click="handleDelete" :disabled="loading">
                    <div v-if="loading" class="loading loading-spinner loading-sm"></div>
                    {{ loading ? 'Deleting...' : 'Delete Client' }}
                </button>
            </div>
        </div>
    </div>
</template>