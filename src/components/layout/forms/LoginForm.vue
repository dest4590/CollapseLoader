<template>
    <form @submit.prevent="handleLocalLogin" class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ t("common.username") }}</span>
            </label>
            <input
                v-model="username"
                type="text"
                :placeholder="t('auth.login.username_placeholder')"
                class="input input-bordered w-full bg-base-100"
                required
                :disabled="isLoading"
            />
        </div>

        <div class="pt-2">
            <button
                type="submit"
                class="btn btn-primary w-full"
                :disabled="isLoading"
            >
                <span v-if="isLoading" class="loading loading-spinner"></span>
                {{ t("auth.login.local_login") || "Войти локально" }}
            </button>
        </div>
    </form>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "../../../services/toastService";
import { localUserService } from "../../../services/localUserService";

const { t } = useI18n();
const { addToast } = useToast();
const emit = defineEmits(["logged-in", "unverified"]);

const username = ref("");
const isLoading = ref(false);

const handleLocalLogin = async () => {
    if (!username.value.trim()) {
        addToast(
            t("auth.login.enter_username") || "Введите имя пользователя",
            "warning"
        );
        return;
    }

    try {
        isLoading.value = true;
        localUserService.createProfile(username.value.trim());
        addToast(
            t("auth.login.local_success") || "Локальный профиль создан!",
            "success"
        );
        emit("logged-in");
    } catch (e) {
        console.error("Local login failed", e);
        addToast("Failed to create local profile", "error");
    } finally {
        isLoading.value = false;
    }
};
</script>
