<template>
    <div>
        <p class="text-sm text-base-content/70">
            {{
                $t('modals.delete_account_confirm.message', {
                    username: account?.username,
                })
            }}
        </p>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmDelete" class="btn btn-primary">
                <check-icon class="w-4 h-4 mr-2" />
                {{ $t('modals.delete_account_confirm.yes_delete') }}
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

interface Account {
    id: string;
    username: string;
    tags: string[];
    created_at: string;
    last_used?: string;
    is_active: boolean;
}

const props = defineProps<{
    account: Account;
}>();

const emit = defineEmits(['close', 'account-deleted']);
const { addToast } = useToast();
const { t } = useI18n();

const confirmDelete = async () => {
    try {
        await invoke('remove_account', { id: props.account.id });
        addToast(t('toast.account.account_deleted'), 'success');
        emit('account-deleted');
        emit('close');
    } catch (error) {
        console.error('Failed to delete account:', error);
        addToast(t('toast.account.account_delete_failed', { error }), 'error');
    }
};
</script>
