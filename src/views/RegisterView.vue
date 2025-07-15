<template>
    <div class="max-w-md mx-auto p-6">
        <h1 class="text-2xl font-semibold mb-6 text-primary-focus">
            {{ t('auth.register.title') }}
        </h1>
        <RegistrationForm @registered="handleRegistered" @logged-in="handleLoggedIn" />
        <div class="text-center mt-4">
            <p class="text-sm text-base-content/70">
                {{ t('auth.register.login_link') }}
                <a @click="$emit('change-view', 'login')" class="link link-primary hover:underline cursor-pointer">{{
                    t('common.login') }}</a>
            </p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import RegistrationForm from '../components/forms/RegistrationForm.vue';

const { t } = useI18n();
const emit = defineEmits(['registered', 'logged-in', 'change-view']);

const handleRegistered = () => {
    emit('registered');
};

const handleLoggedIn = () => {
    emit('logged-in');
};
</script>
