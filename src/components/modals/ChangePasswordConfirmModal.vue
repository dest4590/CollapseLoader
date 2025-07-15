<template>
    <div>
        <p class="text-sm text-base-content/70 mb-4">
            {{ $t('modals.change_password_confirm.message') }}
        </p>

        <div class="alert alert-warning mb-4">
            <warning-icon class="w-5 h-5" />
            <span class="text-sm">{{
                $t('modals.change_password_confirm.warning')
                }}</span>
        </div>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmPasswordChange" class="btn btn-primary">
                <check-icon class="w-4 h-4 mr-2" />
                {{ $t('modals.change_password_confirm.yes_change') }}
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
    Check as CheckIcon,
    X as XIcon,
    AlertTriangle as WarningIcon,
} from 'lucide-vue-next';

interface Props {
    currentPassword: string;
    newPassword: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['close', 'confirm']);

const confirmPasswordChange = () => {
    emit('confirm', {
        currentPassword: props.currentPassword,
        newPassword: props.newPassword,
    });
};
</script>
