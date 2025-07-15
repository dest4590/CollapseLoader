<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    $t('modals.edit_account.username_label')
                    }}</span>
            </label>
            <input v-model="username" type="text" class="input input-bordered w-full bg-base-100" />
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    $t('modals.edit_account.tags_label')
                    }}</span>
            </label>
            <input v-model="tags" type="text" class="input input-bordered w-full bg-base-100" />
        </div>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="updateAccount" class="btn btn-primary">
                <check-icon class="w-4 h-4 mr-2" />
                {{ $t('modals.edit_account.update') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <x-icon class="w-4 h-4 mr-2" />
                {{ $t('common.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
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

const emit = defineEmits(['close', 'account-updated']);
const { addToast } = useToast();
const { t } = useI18n();

const username = ref(props.account.username);
const tags = ref(props.account.tags.join(', '));

const updateAccount = async () => {
    if (!username.value.trim()) {
        addToast(t('toast.account.username_required'), 'error');
        return;
    }

    try {
        const tagList = tags.value
            .split(',')
            .map((tag) => tag.trim())
            .filter((tag) => tag.length > 0);

        await invoke('update_account', {
            id: props.account.id,
            username: username.value.trim(),
            tags: tagList,
        });

        addToast(t('toast.account.account_updated'), 'success');
        emit('account-updated');
        emit('close');
    } catch (error) {
        console.error('Failed to update account:', error);
        addToast(t('toast.account.account_update_failed', { error }), 'error');
    }
};
</script>
