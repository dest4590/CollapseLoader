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
import { useToast } from '../../services/toastService';
import { userService } from '../../services/userService';
import { useI18n } from 'vue-i18n';
import { apiPost } from '../../services/authClient';
import { getCurrentLanguage } from '../../i18n';

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
            '/auth/users/',
            {
                username: username.value,
                email: email.value,
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
                '/auth/token/login/',
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

            const authToken = loginResponse.data.auth_token;
            if (authToken) {
                localStorage.setItem('authToken', authToken);
                userService.clearCache();
                addToast(t('auth.login.success'), 'success');
                emit('logged-in');
            } else {
                addToast(t('auth.login.no_token'), 'error');
                emit('registered');
            }
        } catch (loginError) {
            console.error('Auto-login failed:', loginError);
            addToast(t('auth.register.auto_login_failed'), 'warning');
            emit('registered');
        }
    } catch (error: any) {
        console.error('Registration failed:', error);
        if (error.response && error.response.data) {
            let errorMessage = t('auth.register.registration_failed');
            const errors = error.response.data;
            if (errors.username) {
                errorMessage = `Username: ${errors.username.join(' ')}`;
            } else if (errors.email) {
                errorMessage = `Email: ${errors.email.join(' ')}`;
            } else if (errors.password) {
                errorMessage = `Password: ${errors.password.join(' ')}`;
            } else if (Array.isArray(errors) && errors.length > 0) {
                errorMessage = errors.join(' ');
            } else if (typeof errors === 'string') {
                errorMessage = errors;
            } else if (errors.detail) {
                errorMessage = errors.detail;
            }
            addToast(errorMessage, 'error');
        } else {
            addToast(t('auth.register.registration_failed'), 'error');
        }
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
