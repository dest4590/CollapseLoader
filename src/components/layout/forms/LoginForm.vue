<template>
    <form @submit.prevent="handleLogin" class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ t('common.username') }}</span>
            </label>
            <input v-model="username" type="text" :placeholder="t('auth.login.username_placeholder')"
                class="input input-bordered w-full bg-base-100" required :disabled="isLoading" />
        </div>
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ t('common.password') }}</span>
            </label>
            <input v-model="password" type="password" :placeholder="t('auth.login.password_placeholder')"
                class="input input-bordered w-full bg-base-100" required :disabled="isLoading" />
        </div>
        <div class="pt-2">
            <button type="submit" class="btn btn-primary w-full" :disabled="isLoading">
                <span v-if="isLoading" class="loading loading-spinner"></span>
                {{ t('auth.login.login_button') }}
            </button>
        </div>
    </form>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { apiPost } from '../../../services/apiClient';
import { getApiBaseWithVersion } from '../../../config';
import { useToast } from '../../../services/toastService';
import { userService } from '../../../services/userService';
import { invoke } from '@tauri-apps/api/core';

const { t } = useI18n();
const { addToast } = useToast();
const emit = defineEmits(['logged-in', 'unverified']);

const username = ref('');
const password = ref('');
const isLoading = ref(false);

const handleLogin = async () => {
    try {
        isLoading.value = true;
        const response = await apiPost(
            `${getApiBaseWithVersion()}/auth/login`,
            {
                username: username.value,
                password: password.value,
            }
        );

        if (response.token) {
            localStorage.setItem('authToken', response.token);
            userService.clearCache();
            try {
                await invoke('update_presence', {
                    details: 'In Menu',
                    state: 'Browsing clients',
                });
            } catch (e) {
                console.error('Presence update failed', e);
            }
            addToast(t('auth.login.success'), 'success');
            emit('logged-in');
        }
    } catch (error: any) {
        console.error('Login failed:', error);
        const errorMsg = error.response?.data?.error || '';
        console.log('Error message:', errorMsg);
        if (errorMsg.startsWith('Email not verified')) {
            const email = errorMsg.split(': ')[1] || '';
            console.log('Emitting unverified event with email:', email);
            emit('unverified', { username: username.value, email });
        } else {
            addToast(errorMsg || t('auth.login.error'), 'error');
        }
    } finally {
        isLoading.value = false;
    }
};
</script>
