<template>
    <div>
        <p class="text-sm text-base-content/70 mb-4">
            {{ $t('modals.logout_confirm.message') }}
        </p>

        <div class="alert alert-info mb-4">
            <info-icon class="w-5 h-5" />
            <span class="text-sm">{{ $t('modals.logout_confirm.info') }}</span>
        </div>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmLogout" class="btn btn-error">
                <log-out-icon class="w-4 h-4 mr-2" />
                {{ $t('modals.logout_confirm.yes_logout') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <x-icon class="w-4 h-4 mr-2" />
                {{ $t('common.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import {
    LogOut as LogOutIcon,
    X as XIcon,
    Info as InfoIcon,
} from 'lucide-vue-next';

const emit = defineEmits(['close', 'confirm']);

const confirmLogout = () => {
    emit('confirm');
};
</script>
