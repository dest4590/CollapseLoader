<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { apiDelete, apiGet, apiPost } from "../../../services/apiClient";
import { ensureApiUrl } from "../../../config";
import { invoke } from "@tauri-apps/api/core";
import type { ClientDetails } from "../../../types/ui";

const props = defineProps<{
    clientId: number;
    ratingAvg: number | null;
    ratingCount: number;
    myRating: number | null;
    isAuthenticated: boolean;
}>();

const emit = defineEmits([
    "update:ratingAvg",
    "update:ratingCount",
    "update:myRating",
]);

const { t } = useI18n();

const isSubmittingRating = ref(false);
const isLoadingMyRating = ref(false);

const ratingRounded = computed(() => {
    if (props.ratingAvg === null) return null;
    return Math.round(props.ratingAvg * 2) / 2;
});

const getRatingEndpoint = () => {
    return `/clients/${props.clientId}/rating`;
};

const fetchMyRating = async () => {
    if (!props.isAuthenticated) return;
    if (isLoadingMyRating.value) return;
    if (props.myRating !== null) return;

    isLoadingMyRating.value = true;
    try {
        await ensureApiUrl();
        const data = await apiGet<{ my_rating?: number | null }>(
            getRatingEndpoint()
        );
        if (typeof data?.my_rating === "number") {
            emit("update:myRating", data.my_rating);
        }
    } catch (error) {
        console.warn("Failed to fetch my rating:", error);
    } finally {
        isLoadingMyRating.value = false;
    }
};

const handleRatingClick = async (value: number) => {
    if (isSubmittingRating.value) return;
    if (!props.isAuthenticated) return;

    if (props.myRating === value) {
        await removeRating();
        return;
    }

    await submitRating(value);
};

const submitRating = async (value: number) => {
    if (isSubmittingRating.value) return;
    if (!props.isAuthenticated) return;

    const previousRating = props.myRating;
    emit("update:myRating", value);

    isSubmittingRating.value = true;
    try {
        await ensureApiUrl();
        const data = await apiPost<{
            rating_avg: number | null;
            rating_count: number;
            my_rating?: number | null;
        }>(getRatingEndpoint(), { rating: value });

        if (typeof data?.my_rating === "number") {
            emit("update:myRating", data.my_rating);
        } else {
            emit("update:myRating", value);
        }

        if (typeof data?.rating_avg === "number" || data?.rating_avg === null) {
            emit("update:ratingAvg", data.rating_avg);
        }
        if (typeof data?.rating_count === "number") {
            emit("update:ratingCount", data.rating_count);
        }
    } catch (error) {
        console.error("Failed to submit rating:", error);
        emit("update:myRating", previousRating);
    } finally {
        isSubmittingRating.value = false;
    }
};

const removeRating = async () => {
    if (isSubmittingRating.value) return;
    if (!props.isAuthenticated) return;
    if (props.myRating === null) return;

    const previousRating = props.myRating;
    emit("update:myRating", null);

    isSubmittingRating.value = true;
    try {
        await ensureApiUrl();
        await apiDelete(getRatingEndpoint());

        try {
            const updated = await invoke<ClientDetails>("get_client_details", {
                clientId: props.clientId,
            });
            emit("update:ratingAvg", updated.rating_avg);
            emit("update:ratingCount", updated.rating_count);
        } catch (e) {
            console.warn(
                "Failed to refetch client details after rating delete:",
                e
            );
        }
    } catch (error) {
        console.error("Failed to remove rating:", error);
        emit("update:myRating", previousRating);
    } finally {
        isSubmittingRating.value = false;
    }
};

watch(
    () => props.isAuthenticated,
    (newVal) => {
        if (newVal) fetchMyRating();
    }
);

onMounted(() => {
    if (props.isAuthenticated) fetchMyRating();
});
</script>

<template>
    <div class="stat">
        <div
            class="stat-title text-[10px] font-bold uppercase tracking-widest opacity-60"
        >
            {{ t("client.details.rating") }}
        </div>
        <div class="stat-value text-base flex flex-col gap-2">
            <div class="flex items-center gap-3">
                <div
                    class="rating rating-half rating-sm pointer-events-none opacity-80"
                >
                    <input
                        type="radio"
                        :name="`rating-avg-display-${clientId}`"
                        class="rating-hidden"
                        :checked="ratingRounded === null"
                        disabled
                    />
                    <template v-for="i in 5" :key="i">
                        <input
                            type="radio"
                            :name="`rating-avg-display-${clientId}`"
                            class="mask mask-star-2 mask-half-1 bg-warning"
                            :checked="ratingRounded === i - 0.5"
                            disabled
                        />
                        <input
                            type="radio"
                            :name="`rating-avg-display-${clientId}`"
                            class="mask mask-star-2 mask-half-2 bg-warning"
                            :checked="ratingRounded === i"
                            disabled
                        />
                    </template>
                </div>
                <div
                    class="text-xs font-semibold text-base-content/70 whitespace-nowrap"
                >
                    <span v-if="ratingAvg !== null"
                        >{{ ratingAvg.toFixed(1) }}/5</span
                    >
                    <span v-else>—</span>
                    <span class="text-base-content/50">
                        ({{ ratingCount }})</span
                    >
                </div>
            </div>

            <div v-if="isAuthenticated" class="flex items-center gap-3">
                <span
                    class="text-[10px] font-bold uppercase tracking-widest opacity-40"
                >
                    {{ t("client.details.your_rating") }}:
                </span>
                <div
                    :key="`my-rating-${myRating}`"
                    class="rating rating-half rating-sm"
                >
                    <input
                        type="radio"
                        :name="`my-rating-input-${clientId}`"
                        class="rating-hidden"
                        :checked="myRating === null"
                        @click="removeRating"
                    />
                    <template v-for="i in 5" :key="i">
                        <input
                            type="radio"
                            :name="`my-rating-input-${clientId}`"
                            class="mask mask-star-2 mask-half-1 bg-warning"
                            :checked="myRating === i - 0.5"
                            :disabled="isSubmittingRating"
                            @click="handleRatingClick(i - 0.5)"
                        />
                        <input
                            type="radio"
                            :name="`my-rating-input-${clientId}`"
                            class="mask mask-star-2 mask-half-2 bg-warning"
                            :checked="myRating === i"
                            :disabled="isSubmittingRating"
                            @click="handleRatingClick(i)"
                        />
                    </template>
                </div>
            </div>
            <div v-else class="text-xs font-medium text-base-content/60 italic">
                {{ t("client.details.login_to_rate") }}
            </div>
        </div>
    </div>
</template>
