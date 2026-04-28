<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    t("modals.add_account.username_label")
                }}</span>
                <span
                    class="label-text-alt"
                    :class="username.length > 16 ? 'text-error' : 'text-base-content/50'"
                >
                    {{ username.length }}/16
                </span>
            </label>
            <input
                v-model="username"
                type="text"
                :maxlength="16"
                class="input input-bordered w-full bg-base-100"
                :class="{ 'input-error': usernameError }"
                :placeholder="t('modals.add_account.username_placeholder')"
                @input="validateUsername"
            />
            <div v-if="usernameError" class="label">
                <span class="label-text-alt text-error">{{ usernameError }}</span>
            </div>
            <div v-else class="label">
                <span class="label-text-alt text-base-content/40">{{ t("modals.add_account.username_hint") }}</span>
            </div>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text font-medium mb-1">{{
                    t("modals.add_account.tags_label")
                }}</span>
            </label>
            <input
                v-model="tags"
                type="text"
                class="input input-bordered w-full bg-base-100"
                :placeholder="t('modals.add_account.tags_placeholder')"
            />
        </div>

        <div class="flex justify-end space-x-2 mt-9 w-full">
            <button @click="addAccount" class="btn btn-primary" :disabled="!!usernameError || !username.trim()">
                {{ t("modals.add_account.add_account") }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                {{ t("common.cancel") }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../../../../services/toastService";
import { useI18n } from "vue-i18n";

const emit = defineEmits(["close", "account-added"]);
const { addToast } = useToast();
const { t } = useI18n();

const username = ref("");
const tags = ref("");
const usernameError = ref("");

const MC_REGEX = /^[a-zA-Z0-9_]+$/;

const validateUsername = () => {
    const val = username.value.trim();
    if (!val) {
        usernameError.value = "";
        return;
    }
    if (val.length < 3) {
        usernameError.value = t("modals.add_account.username_too_short");
        return;
    }
    if (val.length > 16) {
        usernameError.value = t("modals.add_account.username_too_long");
        return;
    }
    if (!MC_REGEX.test(val)) {
        usernameError.value = t("modals.add_account.username_invalid_chars");
        return;
    }
    usernameError.value = "";
};

const addAccount = async () => {
    validateUsername();
    if (usernameError.value || !username.value.trim()) {
        addToast(usernameError.value || t("toast.account.username_required"), "error");
        return;
    }

    try {
        const tagList = tags.value
            .split(",")
            .map((tag) => tag.trim())
            .filter((tag) => tag.length > 0);

        await invoke("add_account", {
            username: username.value.trim(),
            tags: tagList,
        });

        username.value = "";
        tags.value = "";
        usernameError.value = "";
        addToast(t("toast.account.account_added"), "success");
        emit("account-added");
        emit("close");
    } catch (error) {
        console.error("Failed to add account:", error);
        addToast(t("toast.account.account_add_failed", { error }), "error");
    }
};
</script>
