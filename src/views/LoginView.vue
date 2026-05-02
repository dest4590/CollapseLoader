<template>
    <div class="max-w-md mx-auto p-6">
        <h1 class="text-2xl font-semibold mb-6 text-primary-focus">
            {{ t("auth.login.title") }}
        </h1>
        <LoginForm
            @logged-in="$emit('logged-in')"
            @unverified="handleUnverified"
        />
        <div v-if="false" class="text-center mt-6">
            <p class="text-sm text-base-content/70">
                {{ t("auth.login.register_link") }}
                <a
                    @click="$emit('change-view', 'register')"
                    class="link link-primary hover:underline cursor-pointer"
                    >{{ t("common.register") }}</a
                >
            </p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import LoginForm from "@layouts/forms/LoginForm.vue";

const { t } = useI18n();
const emit = defineEmits(["logged-in", "change-view", "show-verify"]);

const handleUnverified = (data: { email: string }) => {
    console.log("LoginView: handleUnverified called with:", data);
    emit("show-verify", data.email);
};
</script>
