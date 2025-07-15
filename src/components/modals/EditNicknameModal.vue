<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    $t('modals.edit_nickname.label')
                    }}</span>
            </label>
            <input v-model="nicknameInput" type="text" :placeholder="$t('modals.edit_nickname.placeholder')"
                class="input input-bordered w-full bg-base-100" maxlength="100" @keyup.enter="updateNickname" />
            <label class="label">
                <span class="label-text-alt text-base-content/70 mt-2">{{
                    $t('modals.edit_nickname.description')
                    }}</span>
            </label>
        </div>

        <div class="flex justify-end space-x-2 mt-3">
            <button @click="updateNickname" class="btn btn-primary">
                <check-icon class="w-4 h-4 mr-2" />
                {{ $t('modals.edit_nickname.update') }}
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
import { useToast } from '../../services/toastService';
import { Check as CheckIcon, X as XIcon } from 'lucide-vue-next';
import { apiPatch } from '../../services/authClient';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
    currentNickname: string;
}>();

const emit = defineEmits(['close', 'nickname-updated']);
const { addToast } = useToast();
const { t } = useI18n();

const nicknameInput = ref(props.currentNickname);

const updateNickname = async () => {
    try {
        await apiPatch(
            '/auth/profile/',
            {
                nickname: nicknameInput.value,
            },
            {
                headers: {
                    Authorization: `Token ${localStorage.getItem('authToken')}`,
                },
            }
        );

        emit('nickname-updated', nicknameInput.value);
        emit('close');
    } catch (error: any) {
        console.error('Failed to update nickname:', error);
        if (error.response && error.response.data) {
            let errorMessage = t('toast.account.nickname_update_failed');
            const errors = error.response.data;
            if (errors.nickname) {
                errorMessage = errors.nickname.join(' ');
            } else if (typeof errors === 'string') {
                errorMessage = errors;
            } else if (errors.detail) {
                errorMessage = errors.detail;
            }
            addToast(errorMessage, 'error');
        } else {
            addToast(t('toast.account.nickname_update_failed'), 'error');
        }
    }
};
</script>
