<template>
    <div v-if="!showVerificationMessage" class="space-y-4">
        <div class="form-control">
            <input
                v-model="username"
                type="text"
                :placeholder="t('auth.register.username_placeholder')"
                class="input input-bordered w-full bg-base-100"
                required
                :disabled="isRegistering"
            />
        </div>
        <div class="form-control">
            <input
                v-model="email"
                type="email"
                :placeholder="t('auth.register.email_placeholder')"
                class="input input-bordered w-full bg-base-100"
                required
                :disabled="isRegistering"
            />
        </div>
        <div class="form-control">
            <input
                v-model="password"
                type="password"
                :placeholder="t('auth.register.password_placeholder')"
                class="input input-bordered w-full bg-base-100"
                required
                :disabled="isRegistering"
            />
        </div>
        <div class="form-control">
            <input
                v-model="confirmPassword"
                type="password"
                :placeholder="t('auth.register.confirm_password_placeholder')"
                class="input input-bordered w-full bg-base-100"
                required
                :disabled="isRegistering"
            />
        </div>
        <div class="flex gap-2 pt-3">
            <button
                @click="handleRegister"
                class="btn btn-primary flex-1"
                :disabled="isRegistering"
            >
                {{
                    isRegistering
                        ? t("modals.initial_setup.registration.registering")
                        : t("auth.register.register_button")
                }}
            </button>
            <button
                v-if="showCancelButton"
                class="btn btn-outline"
                :disabled="isRegistering"
                @click="emit('cancel')"
            >
                {{ t("common.cancel") }}
            </button>
        </div>
    </div>
    <div v-else class="text-center space-y-4 py-4">
        <div class="text-4xl">📧</div>
        <h3 class="text-xl font-bold">
            {{ t("auth.register.verification_required") }}
        </h3>
        <p class="text-sm opacity-70">
            {{ t("auth.register.check_email", { email: registrationEmail }) }}
        </p>
        <button
            v-if="showCancelButton"
            @click="emit('cancel')"
            class="btn btn-ghost"
        >
            {{ t("common.close") }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useToast } from "@shared/composables/useToast";
import { useI18n } from "vue-i18n";
import { apiPost } from "@api/clients/internal";
import { getCurrentLanguage } from "@core/i18n";
import { getApiBaseWithVersion } from "@/config";

interface Props {
    showCancelButton?: boolean;
    compact?: boolean;
}

withDefaults(defineProps<Props>(), {
    showCancelButton: false,
    compact: false,
});

const emit = defineEmits(["registered", "logged-in", "cancel"]);
const { t } = useI18n();
const { addToast } = useToast();

const username = ref("");
const email = ref("");
const password = ref("");
const confirmPassword = ref("");
const isRegistering = ref(false);
const registrationEmail = ref("");
const showVerificationMessage = ref(false);

const handleRegister = async () => {
    if (password.value !== confirmPassword.value) {
        addToast(t("validation.passwords_no_match"), "error");
        return;
    }
    if (password.value.length < 8) {
        addToast(
            t("validation.min_length", {
                field: t("common.password"),
                length: 8,
            }),
            "error"
        );
        return;
    }
    try {
        isRegistering.value = true;
        registrationEmail.value = email.value;

        await apiPost(
            `${getApiBaseWithVersion()}/auth/register`,
            {
                username: username.value,
                password: password.value,
                email: email.value,
            },
            {
                headers: {
                    "Accept-Language": getCurrentLanguage() || "en",
                    "Content-Type": "application/json",
                },
            }
        );

        showVerificationMessage.value = true;
        addToast(t("auth.register.verification_required"), "info");
        emit("registered", { username: username.value, email: email.value });
    } catch (error: any) {
        console.error("Registration failed:", error);
        const errorMsg =
            error.response?.data?.error ||
            t("auth.register.registration_failed");
        addToast(errorMsg, "error");
    } finally {
        isRegistering.value = false;
    }
};

const clearForm = () => {
    username.value = "";
    email.value = "";
    password.value = "";
    confirmPassword.value = "";
};

defineExpose({
    clearForm,
});
</script>
