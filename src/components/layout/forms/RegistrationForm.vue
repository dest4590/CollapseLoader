<template>
    <div class="space-y-4">
        <div class="form-control">
            <input v-model="username" type="text" :placeholder="t('auth.register.username_placeholder')"
                class="input input-bordered w-full bg-base-100" required :disabled="isRegistering" />
        </div>
        <div class="form-control">
            <input v-model="email" type="email" :placeholder="t('auth.register.email_placeholder')"
                class="input input-bordered w-full bg-base-100" required :disabled="isRegistering" />
        </div>
        <div class="form-control">
            <input v-model="password" type="password" :placeholder="t('auth.register.password_placeholder')"
                class="input input-bordered w-full bg-base-100" required :disabled="isRegistering" />
        </div>
        <div class="form-control">
            <input v-model="confirmPassword" type="password"
                :placeholder="t('auth.register.confirm_password_placeholder')"
                class="input input-bordered w-full bg-base-100" required :disabled="isRegistering" />
        </div>
        <div class="flex gap-2 pt-3">
            <button @click="handleRegister" class="btn btn-primary flex-1" :disabled="isRegistering">
                {{
                    isRegistering
                        ? t('modals.initial_setup.registration.registering')
                        : t('auth.register.register_button')
                }}
            </button>
            <button v-if="showCancelButton" class="btn btn-outline" :disabled="isRegistering" @click="emit('cancel')">
                {{ t('common.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useToast } from '../../../services/toastService';
import { userService } from '../../../services/userService';
import { useI18n } from 'vue-i18n';
import { apiPost } from '../../../services/apiClient';
import { getCurrentLanguage } from '../../../i18n';
import { getApiBaseWithVersion } from '../../../config';

interface Props {
    showCancelButton?: boolean;
    compact?: boolean;
}

withDefaults(defineProps<Props>(), {
    showCancelButton: false,
    compact: false,
});

const emit = defineEmits(['registered', 'logged-in', 'cancel']);
const { t } = useI18n();
const { addToast } = useToast();

const username = ref('');
const email = ref('');
const password = ref('');
const confirmPassword = ref('');
const isRegistering = ref(false);

const extractAuthToken = (payload: any): string | null => {
    if (!payload) return null;
    const candidate =
        payload.token ||
        payload.auth_token ||
        payload.access_token ||
        payload.accessToken ||
        payload?.tokens?.token ||
        payload?.tokens?.access_token ||
        payload?.tokens?.accessToken ||
        payload?.data?.token ||
        payload?.data?.auth_token ||
        payload?.data?.access_token ||
        payload?.data?.accessToken;
    return typeof candidate === 'string' && candidate.length > 0 ? candidate : null;
};

const handleRegister = async () => {
    if (password.value !== confirmPassword.value) {
        addToast(t('validation.passwords_no_match'), 'error');
        return;
    }
    if (password.value.length < 8) {
        addToast(
            t('validation.min_length', {
                field: t('common.password'),
                length: 8,
            }),
            'error'
        );
        return;
    }
    try {
        isRegistering.value = true;

        await apiPost(
            `${getApiBaseWithVersion()}/auth/register`,
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

        try {
            const loginResponse = await apiPost(
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

            const authToken = extractAuthToken(loginResponse);

            if (authToken) {
                localStorage.setItem('authToken', authToken);
                userService.clearCache();
                addToast(t('auth.login.success'), 'success');
                emit('logged-in');
            } else {
                console.error('No auth token found in auto-login response:', loginResponse);
                addToast(t('auth.login.no_token'), 'error');
                emit('registered');
            }
        } catch (loginError) {
            console.error('Auto-login failed:', loginError);
            emit('registered');
        }
    } catch (error: any) {
        console.error('Registration failed:', error);
        console.error('Registration error response:', error.response);
    } finally {
        isRegistering.value = false;
    }
};

const clearForm = () => {
    username.value = '';
    email.value = '';
    password.value = '';
    confirmPassword.value = '';
};

defineExpose({
    clearForm,
});
</script>
