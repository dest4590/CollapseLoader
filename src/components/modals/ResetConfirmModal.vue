<template>
    <div>
        <p class="text-sm text-base-content/70">
            {{ $t('modals.reset_confirm.message') }}
        </p>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmReset" class="btn btn-primary">
                <check-icon class="w-4 h-4 mr-2" />
                {{ $t('modals.reset_confirm.yes_reset') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <x-icon class="w-4 h-4 mr-2" />
                {{ $t('common.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { Check as CheckIcon, X as XIcon } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['close', 'settings-reset']);
const { addToast } = useToast();
const { t } = useI18n();

const confirmReset = async () => {
    try {
        await invoke('reset_settings');
        addToast(t('toast.modal.settings_reset_success'), 'success');
        emit('settings-reset');
        emit('close');
    } catch (error) {
        console.error('Failed to reset settings:', error);
        addToast(t('toast.modal.settings_reset_failed', { error }), 'error');
    }
};
</script>
