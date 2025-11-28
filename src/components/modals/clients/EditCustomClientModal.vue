<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../../services/toastService';
import { useModal } from '../../../services/modalService';
import type { CustomClient } from '../../../types/ui';

const { addToast } = useToast();
const { getModals } = useModal();

const emit = defineEmits<{
    'client-edited': [];
    'close': []
}>();

const form = reactive({
    name: '',
    version: '',
    mainClass: '',
});

const loading = ref(false);
const errors = ref<Record<string, string>>({});
const currentClient = ref<CustomClient | null>(null);

const validateForm = () => {
    errors.value = {};

    if (!form.name.trim()) {
        errors.value.name = 'Name is required';
    }

    if (!form.version.trim()) {
        errors.value.version = 'Version is required';
    }

    if (!form.mainClass.trim()) {
        errors.value.mainClass = 'Main class is required';
    }

    return Object.keys(errors.value).length === 0;
};

const handleSubmit = async () => {
    if (!validateForm() || !currentClient.value) {
        return;
    }

    try {
        loading.value = true;

        await invoke('update_custom_client', {
            id: currentClient.value.id,
            name: form.name.trim(),
            version: form.version,
            mainClass: form.mainClass.trim(),
        });

        emit('client-edited');
        emit('close');
    } catch (err) {
        addToast(`Failed to update custom client: ${err}`, 'error');
    } finally {
        loading.value = false;
    }
};

const modals = getModals();

watch(() => modals['edit-custom-client']?.props?.client, (client: CustomClient | undefined) => {
    if (client) {
        currentClient.value = client;
        form.name = client.name;
        form.version = client.version;
        form.mainClass = client.main_class;
    }
}, { immediate: true });
</script>

<template>
    <form @submit.prevent="handleSubmit" class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('modals.edit_custom_client_modal.client_name') }} *</span>
            </label>
            <input v-model="form.name" type="text" :placeholder="$t('modals.edit_custom_client_modal.enter_client_name')" class="input input-bordered"
                :class="{ 'input-error': errors.name }" />
            <label v-if="errors.name" class="label">
                <span class="label-text-alt text-error">{{ errors.name }}</span>
            </label>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('modals.edit_custom_client_modal.minecraft_version') }} *</span>
            </label>
            <input v-model="form.version" type="text" placeholder="e.g. 1.16.5" class="input input-bordered"
                :class="{ 'input-error': errors.version }" />
            <label v-if="errors.version" class="label">
                <span class="label-text-alt text-error">{{ errors.version }}</span>
            </label>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('modals.edit_custom_client_modal.main_class') }} *</span>
            </label>
            <input v-model="form.mainClass" type="text" :placeholder="$t('modals.edit_custom_client_modal.main_class_placeholder')"
                class="input input-bordered" :class="{ 'input-error': errors.mainClass }" />
            <label v-if="errors.mainClass" class="label">
                <span class="label-text-alt text-error">{{ errors.mainClass }}</span>
            </label>
        </div>

        <div class="modal-action">
            <button type="button" class="btn" @click="$emit('close')" :disabled="loading">
                {{ $t('common.cancel') }}
            </button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
                <div v-if="loading" class="loading loading-spinner loading-sm"></div>
                {{ loading ? $t('modals.edit_custom_client_modal.updating') : $t('modals.edit_custom_client_modal.update_client') }}
            </button>
        </div>
    </form>
</template>