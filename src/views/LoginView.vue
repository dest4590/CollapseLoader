<template>
    <div class="max-w-md mx-auto p-6">
        <h1 class="text-2xl font-semibold mb-6 text-primary-focus">
            {{ t('auth.login.title') }}
        </h1>
        <form @submit.prevent="handleLogin">
            <div class="form-control mb-4">
                <label class="label">
                    <span class="label-text mb-1">{{ t('common.username') }}</span>
                </label>
                <input v-model="username" type="text" :placeholder="t('auth.login.username_placeholder')"
                    class="input input-bordered w-full bg-base-100" required />
            </div>
            <div class="form-control mb-6">
                <label class="label">
                    <span class="label-text mb-1">{{ t('common.password') }}</span>
                </label>
                <input v-model="password" type="password" :placeholder="t('auth.login.password_placeholder')"
                    class="input input-bordered w-full bg-base-100" required />
            </div>
            <button type="submit" class="btn btn-primary w-full mb-4">
                {{ t('auth.login.login_button') }}
            </button>
            <div class="text-center">
                <p class="text-sm text-base-content/70">
                    {{ t('auth.login.register_link') }}
                    <a @click="$emit('change-view', 'register')"
                        class="link link-primary hover:underline cursor-pointer">{{ t('common.register') }}</a>
                </p>
            </div>
        </form>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useToast } from '../services/toastService';
import { userService } from '../services/userService';
import { useI18n } from 'vue-i18n';
import { apiPost } from '../services/apiClient';
import { getApiBaseWithVersion } from '../config';
import { getCurrentLanguage } from '../i18n';
import { invoke } from '@tauri-apps/api/core';

const { t } = useI18n();
const { addToast } = useToast();
const emit = defineEmits(['logged-in', 'change-view']);

const username = ref('');
const password = ref('');

const handleLogin = async () => {
    try {
        const response = await apiPost(
            `${getApiBaseWithVersion()}/auth/login`,
            {
                username: username.value,
                password: password.value,
            },
            {
                headers: {
                    'Accept-Language': getCurrentLanguage() || 'en',
                    'Content-Type': 'application/json',
                },
            }
        );

        console.log('Login response:', response);

        const authToken = response.token;

        if (authToken) {
            localStorage.setItem('authToken', authToken);

            userService.clearCache();

            try {
                await invoke('update_presence', {
                    details: 'In Menu',
                    state: 'Browsing clients',
                });
            } catch (error) {
                console.error('Failed to initialize Discord presence:', error);
            }

            addToast(t('auth.login.success'), 'success');
            emit('logged-in');
        } else {
            console.error('No auth token found in response:', response);
            addToast(t('auth.login.no_token'), 'error');
        }
    } catch (error: any) {
        console.error('Login failed:', error);
        console.error('Error response:', error.response);
        addToast(t('auth.login.error'), 'error');
    }
};
</script>
