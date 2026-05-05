<script setup lang="ts">
import { Camera, ZoomIn } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import type { Client, ClientDetails } from "@shared/types/ui";

const { t } = useI18n();

defineProps<{
    client: Client;
    clientDetails: ClientDetails | null;
    isLoadingDetails: boolean;
    isAuthenticated: boolean;
    ratingAvg: number | null;
    ratingCount: number;
    activeTab: "info" | "screenshots";
    slideDirection: "left" | "right";
    myRating: number | null;
}>();

const emit = defineEmits([
    "change-tab",
    "update:my-rating",
    "update:rating-avg",
    "update:rating-count",
    "update:client-details",
    "screenshot-click",
    "show-user-profile",
    "open-source-link",
]);

const handleScreenshotClick = (event: MouseEvent, index: number) => {
    emit("screenshot-click", event, index);
};
</script>

<template>
    <div class="client-details">
        <div class="space-y-4">
            <div v-if="isLoadingDetails" class="text-center py-4">
                <div class="loading loading-spinner loading-md"></div>
                <p class="text-sm text-base-content/60 mt-2">
                    {{ t("client.details.loading") }}
                </p>
            </div>

            <div v-else-if="clientDetails">
                <div role="tablist" class="tabs tabs-boxed mb-4 w-fit mx-auto">
                    <button
                        type="button"
                        role="tab"
                        class="tab tab-active"
                        aria-selected="true"
                    >
                        <Camera class="w-4 h-4 mr-2" />
                        {{ t("client.details.screenshots_tab") }}
                    </button>
                </div>

                <div
                    v-if="
                        clientDetails.screenshot_urls &&
                        clientDetails.screenshot_urls.length > 0
                    "
                    class="screenshots-section"
                >
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                        <div
                            v-for="(
                                screenshot, index
                            ) in clientDetails.screenshot_urls"
                            :key="index"
                            class="screenshot-container group"
                        >
                            <div
                                class="relative overflow-hidden rounded border border-base-content/10 cursor-pointer"
                                @click.stop="
                                    handleScreenshotClick($event, index)
                                "
                            >
                                <img
                                    :src="screenshot"
                                    :alt="`${client.name} screenshot ${index + 1}`"
                                    class="w-full h-36 object-cover transition-all duration-200 group-hover:scale-105"
                                    @error="
                                        ($event.target as HTMLImageElement)
                                            .closest('.screenshot-container')
                                            ?.setAttribute(
                                                'style',
                                                'display:none'
                                            )
                                    "
                                />
                                <div
                                    class="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors duration-200 flex items-center justify-center pointer-events-none"
                                >
                                    <ZoomIn
                                        class="w-6 h-6 text-white opacity-0 group-hover:opacity-100 transition-opacity duration-200"
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div v-else class="text-center py-8 text-base-content/60">
                    <p>{{ t("client.details.no_screenshots") }}</p>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.client-details {
    opacity: 0;
    max-height: 0;
    overflow: hidden;
    display: none;
    scrollbar-gutter: stable;
}

.screenshot-container {
    position: relative;
    overflow: hidden;
    border-radius: 0.2rem;
    transition: transform 0.2s ease;
}

.screenshot-container:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
</style>
