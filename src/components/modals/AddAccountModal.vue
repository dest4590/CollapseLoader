<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    t('modals.add_account.username_label')
                    }}</span>
            </label>
            <input v-model="username" type="text" class="input input-bordered w-full bg-base-100"
                :placeholder="t('modals.add_account.username_placeholder')" />
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    t('modals.add_account.tags_label')
                    }}</span>
            </label>
            <input v-model="tags" type="text" class="input input-bordered w-full bg-base-100"
                :placeholder="t('modals.add_account.tags_placeholder')" />
        </div>

        <div class="flex justify-end space-x-2 mt-9 w-full">
            <button @click="addAccount" class="btn btn-primary">
                {{ t('modals.add_account.add_account') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                {{ t('common.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['close', 'account-added']);
const { addToast } = useToast();
const { t } = useI18n();

const username = ref('');
const tags = ref('');

const addAccount = async () => {
    if (!username.value.trim()) {
        addToast(t('toast.account.username_required'), 'error');
        return;
    }

    try {
        const tagList = tags.value
            .split(',')
            .map((tag) => tag.trim())
            .filter((tag) => tag.length > 0);

        await invoke('add_account', {
            username: username.value.trim(),
            tags: tagList,
        });

        username.value = '';
        tags.value = '';
        addToast(t('toast.account.account_added'), 'success');
        emit('account-added');
        emit('close');
    } catch (error) {
        console.error('Failed to add account:', error);
        addToast(t('toast.account.account_add_failed', { error }), 'error');
    }
};
</script>
