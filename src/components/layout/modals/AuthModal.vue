<template>
    <div v-if="modelValue" class="fixed inset-0 flex items-center justify-center z-2000 slide-up backdrop-blur-sm">
        <div class="custom-modal-overlay" @click.self="closeModal">
            <div class="custom-modal-content max-w-md w-full">
                <div class="custom-modal-header">
                    <h2 class="text-xl font-bold">{{ modalTitle }}</h2>
                    <button @click="closeModal" class="custom-modal-close-button">×</button>
                </div>

                <div class="custom-modal-body p-4">
                    <LoginForm v-if="view === 'LOGIN'" @logged-in="handleLoggedIn" @unverified="handleUnverified" />

                    <RegistrationForm v-else-if="view === 'REGISTER'" @registered="handleRegistered"
                        @cancel="view = 'LOGIN'" :show-cancel-button="false" />

                    <div v-else-if="view === 'VERIFY'" class="text-center space-y-6 py-4">
                        <div class="flex justify-center">
                            <div class="p-4 bg-primary/10 rounded-full">
                                <Mail class="w-12 h-12 text-primary" />
                            </div>
                        </div>
                        <div class="space-y-2">
                            <h3 class="text-lg font-bold">{{ t('auth.register.verification_required') }}</h3>
                            <p class="text-sm opacity-70">
                                {{ t('auth.register.check_email', { email: pendingEmail }) }}
                            </p>
                        </div>

                        <div class="pt-4 space-y-3">
                            <button @click="handleResend" class="btn btn-primary w-full" :disabled="isResending">
                                <span v-if="isResending" class="loading loading-spinner"></span>
                                {{ t('common.send_again') || 'Resend Email' }}
                            </button>
                            <button @click="view = 'LOGIN'" class="btn btn-ghost w-full">
                                {{ t('common.back_to_login') || 'Back to Login' }}
                            </button>
                        </div>
                    </div>

                    <div v-if="view !== 'VERIFY' && view !== 'SUCCESS'" class="mt-6 text-center text-sm">
                        <template v-if="view === 'LOGIN'">
                            <span class="opacity-70">{{ t('auth.login.register_link') }}</span>
                            <button @click="view = 'REGISTER'" class="link link-primary ml-1">
                                {{ t('common.register') }}
                            </button>
                        </template>
                        <template v-else>
                            <span class="opacity-70">{{ t('auth.register.login_link') }}</span>
                            <button @click="view = 'LOGIN'" class="link link-primary ml-1">
                                {{ t('common.login') }}
                            </button>
                        </template>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Mail } from 'lucide-vue-next';
import LoginForm from '../forms/LoginForm.vue';
import RegistrationForm from '../forms/RegistrationForm.vue';
import { apiPost } from '../../../services/apiClient';
import { getApiBaseWithVersion } from '../../../config';
import { useToast } from '../../../services/toastService';

const props = defineProps<{
    modelValue: boolean;
    initialView?: 'LOGIN' | 'REGISTER' | 'VERIFY';
    email?: string;
}>();

const emit = defineEmits(['update:modelValue', 'logged-in']);
const { t } = useI18n();
const { addToast } = useToast();

const view = ref<'LOGIN' | 'REGISTER' | 'VERIFY' | 'SUCCESS'>(props.initialView || 'LOGIN');
const pendingEmail = ref(props.email || '');
const isResending = ref(false);

const modalTitle = computed(() => {
    switch (view.value) {
        case 'LOGIN': return t('auth.login.title');
        case 'REGISTER': return t('auth.register.title');
        case 'VERIFY': return t('auth.register.verification_required');
        default: return '';
    }
});

const closeModal = () => {
    emit('update:modelValue', false);
};

const handleLoggedIn = () => {
    emit('logged-in');
    closeModal();
};

const handleRegistered = (data: { email: string }) => {
    pendingEmail.value = data.email;
    view.value = 'VERIFY';
};

const handleUnverified = (data: { email: string }) => {
    pendingEmail.value = data.email;
    view.value = 'VERIFY';
};

const handleResend = async () => {
    try {
        isResending.value = true;
        await apiPost(`${getApiBaseWithVersion()}/auth/resend-verification?email=${encodeURIComponent(pendingEmail.value)}`);
        addToast(t('common.verification_sent') || 'Verification email resent!', 'success');
    } catch (e) {
        console.error('Resend failed', e);
    } finally {
        isResending.value = false;
    }
};
</script>

<style scoped>
.custom-modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.75);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    animation: blur 0.3s ease-out forwards;
}

@keyframes blur {
    from {
        backdrop-filter: blur(0px);
    }

    to {
        backdrop-filter: blur(5px);
    }
}

.custom-modal-content {
    background-color: var(--color-base-200, oklch(20% 0 0));
    padding: 1.5rem;
    border-radius: 0.5rem;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.custom-modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.custom-modal-close-button {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: currentColor;
    opacity: 0.7;
}

.custom-modal-close-button:hover {
    opacity: 1;
}
</style>
