<template>
    <div class="flex items-center justify-center min-h-screen p-6">
        <div class="w-full max-w-md space-y-8">
            <div class="text-center space-y-4">
                <div class="flex justify-center">
                    <div class="p-4 bg-primary/10 rounded-full">
                        <Mail class="w-12 h-12 text-primary" />
                    </div>
                </div>
                <h1 class="text-3xl font-bold">
                    {{ t("auth.register.verification_required") }}
                </h1>
                <p class="text-base-content/70">
                    {{ t("auth.register.check_email", { email: email }) }}
                </p>
            </div>

            <div class="space-y-6">
                <div>
                    <label class="block text-sm font-medium mb-2">{{
                        t("auth.verify.code_label")
                    }}</label>
                    <input
                        v-model="verificationCode"
                        type="text"
                        :placeholder="t('auth.verify.code_placeholder')"
                        class="input input-lg input-bordered w-full text-center text-2xl tracking-widest font-mono"
                        maxlength="6"
                        :disabled="isVerifying"
                        @keyup.enter="handleVerify"
                    />
                    <p class="text-sm text-base-content/50 mt-2">
                        Enter the 6-digit code sent to your email
                    </p>
                </div>

                <div class="space-y-3">
                    <button
                        @click="handleVerify"
                        class="btn btn-primary btn-lg w-full"
                        :disabled="isVerifying || verificationCode.length < 6"
                    >
                        <span
                            v-if="isVerifying"
                            class="loading loading-spinner"
                        ></span>
                        {{ t("auth.verify.verify_button") }}
                    </button>

                    <button
                        @click="handleResend"
                        class="btn btn-ghost w-full"
                        :disabled="isResending"
                    >
                        <span
                            v-if="isResending"
                            class="loading loading-spinner"
                        ></span>
                        {{ t("common.send_again") }}
                    </button>

                    <div class="pt-4 border-t border-base-content/10">
                        <button
                            @click="$emit('change-view', 'login')"
                            class="btn btn-ghost btn-sm w-full"
                        >
                            {{ t("common.back_to_login") }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Mail } from "lucide-vue-next";
import { apiPost, apiGet } from "../services/apiClient";
import { getApiBaseWithVersion } from "../config";
import { useToast } from "../services/toastService";

const props = defineProps<{
    email: string;
    code?: string;
}>();

const emit = defineEmits(["verified", "change-view"]);
const { t } = useI18n();
const { addToast } = useToast();

const verificationCode = ref(props.code || "");
const isVerifying = ref(false);
const isResending = ref(false);

watch(
    () => props.code,
    (newCode) => {
        if (newCode) {
            verificationCode.value = newCode;
            if (newCode.length === 6) {
                handleVerify();
            }
        }
    }
);

const handleVerify = async () => {
    if (!verificationCode.value || verificationCode.value.length < 6) return;

    try {
        isVerifying.value = true;
        const response = await apiGet(
            `${getApiBaseWithVersion()}/auth/verify?token=${encodeURIComponent(verificationCode.value)}`
        );
        addToast(t("auth.verify.success"), "success");

        if (response.token) {
            emit("verified", response.token);
        } else {
            emit("verified");
        }
    } catch (e) {
        console.error("Verification failed", e);
        addToast(t("auth.verify.error"), "error");
    } finally {
        isVerifying.value = false;
    }
};

const handleResend = async () => {
    try {
        isResending.value = true;
        await apiPost(
            `${getApiBaseWithVersion()}/auth/resend-verification?email=${encodeURIComponent(props.email)}`
        );
        addToast(t("common.verification_sent"), "success");
    } catch (e) {
        console.error("Resend failed", e);
    } finally {
        isResending.value = false;
    }
};

onMounted(async () => {
    await handleResend();
});
</script>
