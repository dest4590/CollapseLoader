<template>
    <div class="space-y-5">
        <div class="alert alert-info text-sm">
            <BadgeCheck class="w-5 h-5 shrink-0" />
            <span>{{ t("modals.microsoft_account.description") }}</span>
        </div>

        <div
            v-if="!selectedMethod"
            class="grid grid-cols-1 sm:grid-cols-2 gap-3"
        >
            <button
                type="button"
                class="btn btn-outline h-auto min-h-20 flex-col gap-2 py-3"
                :disabled="signingIn"
                @click="signInWithOAuth"
            >
                <ExternalLink class="w-5 h-5" />
                {{ t("modals.microsoft_account.oauth") }}
            </button>
            <button
                type="button"
                class="btn btn-outline h-auto min-h-20 flex-col gap-2 py-3"
                :disabled="signingIn"
                @click="selectMethod('access')"
            >
                <KeyRound class="w-5 h-5" />
                {{ t("modals.microsoft_account.access_token") }}
            </button>
            <button
                type="button"
                class="btn btn-outline h-auto min-h-20 flex-col gap-2 py-3"
                :disabled="signingIn"
                @click="selectMethod('refresh')"
            >
                <RefreshCw class="w-5 h-5" />
                {{ t("modals.microsoft_account.refresh_token") }}
            </button>
            <button
                type="button"
                class="btn btn-outline h-auto min-h-20 flex-col gap-2 py-3"
                :disabled="signingIn"
                @click="selectMethod('cookies')"
            >
                <Cookie class="w-5 h-5" />
                {{ t("modals.microsoft_account.cookies") }}
            </button>
        </div>

        <form v-else class="space-y-4" @submit.prevent="submitCredential">
            <div class="flex items-center gap-2">
                <button
                    type="button"
                    class="btn btn-ghost btn-sm btn-square"
                    :disabled="signingIn"
                    @click="resetMethod"
                >
                    <ArrowLeft class="w-4 h-4" />
                </button>
                <h3 class="font-semibold">{{ methodTitle }}</h3>
            </div>

            <p class="text-sm text-base-content/70">
                {{ methodHint }}
            </p>

            <textarea
                v-if="selectedMethod === 'cookies'"
                v-model="credential"
                class="textarea textarea-bordered w-full min-h-36 font-mono text-xs"
                :placeholder="t('modals.microsoft_account.cookies_placeholder')"
                :disabled="signingIn"
                autocomplete="off"
                spellcheck="false"
                required
            ></textarea>
            <input
                v-else
                v-model="credential"
                type="password"
                class="input input-bordered w-full font-mono"
                :placeholder="methodTitle"
                :disabled="signingIn"
                autocomplete="off"
                required
            />

            <button
                type="submit"
                class="btn btn-primary w-full"
                :disabled="signingIn || !credential.trim()"
            >
                <span
                    v-if="signingIn"
                    class="loading loading-spinner loading-sm"
                ></span>
                {{
                    signingIn
                        ? t("modals.microsoft_account.checking")
                        : t("modals.microsoft_account.add_with_method")
                }}
            </button>
        </form>

        <p v-if="!selectedMethod" class="text-xs text-base-content/60">
            {{ t("modals.microsoft_account.security_hint") }}
        </p>

        <div class="flex justify-end">
            <button
                type="button"
                class="btn btn-ghost btn-sm"
                :disabled="signingIn"
                @click="emit('close')"
            >
                {{ t("common.cancel") }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
    ArrowLeft,
    BadgeCheck,
    Cookie,
    ExternalLink,
    KeyRound,
    RefreshCw,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@shared/composables/useToast";

type CredentialMethod = "access" | "refresh" | "cookies";

const emit = defineEmits(["close", "account-added"]);
const { t } = useI18n();
const { addToast } = useToast();
const signingIn = ref(false);
const selectedMethod = ref<CredentialMethod | null>(null);
const credential = ref("");

const methodTitle = computed(() => {
    if (selectedMethod.value === "access") {
        return t("modals.microsoft_account.access_token");
    }
    if (selectedMethod.value === "refresh") {
        return t("modals.microsoft_account.refresh_token");
    }
    return t("modals.microsoft_account.cookies");
});

const methodHint = computed(() => {
    if (selectedMethod.value === "access") {
        return t("modals.microsoft_account.access_hint");
    }
    if (selectedMethod.value === "refresh") {
        return t("modals.microsoft_account.refresh_hint");
    }
    return t("modals.microsoft_account.cookies_hint");
});

const selectMethod = (method: CredentialMethod) => {
    selectedMethod.value = method;
    credential.value = "";
};

const resetMethod = () => {
    selectedMethod.value = null;
    credential.value = "";
};

const finishLogin = () => {
    credential.value = "";
    addToast(t("modals.microsoft_account.success"), "success");
    emit("account-added");
    emit("close");
};

const handleError = (error: unknown) => {
    console.error("Microsoft login failed");
    const message = String(error);
    if (!message.toLowerCase().includes("cancel")) {
        addToast(message, "error");
    }
};

const signInWithOAuth = async () => {
    signingIn.value = true;
    try {
        await invoke("login_microsoft_account");
        finishLogin();
    } catch (error) {
        handleError(error);
    } finally {
        signingIn.value = false;
    }
};

const submitCredential = async () => {
    const value = credential.value.trim();
    if (!selectedMethod.value || !value) return;

    signingIn.value = true;
    try {
        if (selectedMethod.value === "access") {
            await invoke("login_microsoft_account_with_access_token", {
                accessToken: value,
            });
        } else if (selectedMethod.value === "refresh") {
            await invoke("login_microsoft_account_with_refresh_token", {
                refreshToken: value,
            });
        } else {
            await invoke("login_microsoft_account_with_cookies", {
                cookies: value,
            });
        }
        finishLogin();
    } catch (error) {
        handleError(error);
    } finally {
        signingIn.value = false;
    }
};
</script>
