<template>
    <div class="space-y-4">
        <div class="flex flex-col gap-1">
            <h2 class="text-xl font-bold">
                {{ t("reports.title", { name: username }) }}
            </h2>
            <p class="text-sm text-base-content/70">
                {{ t("reports.description") }}
            </p>
        </div>

        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{{ t("reports.reason_label") }}</span>
            </label>
            <select v-model="reason" class="select select-bordered w-full">
                <option disabled value="">
                    {{ t("reports.select_reason") }}
                </option>
                <option v-for="r in reasons" :key="r" :value="r">
                    {{ t(`reports.reasons.${r}`) }}
                </option>
            </select>
        </div>

        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{{ t("reports.details_label") }}</span>
            </label>
            <textarea
                v-model="details"
                class="textarea textarea-bordered h-24"
                :placeholder="t('reports.details_placeholder')"
            ></textarea>
        </div>

        <div class="flex justify-end gap-2 mt-6">
            <button class="btn btn-ghost" @click="$emit('close')">
                {{ t("common.cancel") }}
            </button>
            <button
                class="btn btn-primary"
                :disabled="!reason || loading"
                @click="submitReport"
            >
                <span
                    v-if="loading"
                    class="loading loading-spinner loading-xs"
                ></span>
                {{ t("reports.submit") }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { reportService } from "../../../services/reportService";
import { useToast } from "@shared/composables/useToast";

const props = defineProps<{
    userId: number;
    username: string;
}>();

const emit = defineEmits(["close", "success"]);

const { t } = useI18n();

const { addToast } = useToast();

const reason = ref("");
const details = ref("");
const loading = ref(false);

const reasons = ["HARASSMENT", "SPAM", "INAPPROPRIATE_CONTENT", "OTHER"];

const submitReport = async () => {
    if (!reason.value) return;

    loading.value = true;
    try {
        await reportService.createReport({
            reportedUserId: props.userId,
            reason: reason.value,
            description: details.value,
        });
        addToast(t("reports.success_message"), "success");
        emit("success");
        emit("close");
    } catch (e) {
        console.error("Failed to submit report", e);
        addToast(t("reports.error_message"), "error");
    } finally {
        loading.value = false;
    }
};
</script>
