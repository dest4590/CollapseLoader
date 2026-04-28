<script setup lang="ts">
import {
    Camera,
    ExternalLink,
    Info,
    ScrollText,
    ZoomIn,
} from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import type { Client, ClientDetails } from "../../../types/ui";
import ClientRating from "./ClientRating.vue";

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

const handleClientSourceLink = (url: string) => {
    emit("open-source-link", url);
};

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
                        class="tab"
                        :class="{ 'tab-active': activeTab === 'info' }"
                        :aria-selected="activeTab === 'info'"
                        @click="emit('change-tab', 'info')"
                    >
                        <Info class="w-4 h-4 mr-2" />
                        {{ t("client.details.info_tab") }}
                    </button>
                    <button
                        type="button"
                        role="tab"
                        class="tab"
                        :class="{ 'tab-active': activeTab === 'screenshots' }"
                        :aria-selected="activeTab === 'screenshots'"
                        @click="emit('change-tab', 'screenshots')"
                    >
                        <Camera class="w-4 h-4 mr-2" />
                        {{ t("client.details.screenshots_tab") }}
                    </button>
                </div>

                <div>
                    <transition
                        :name="`tab-slide-${slideDirection}`"
                        mode="out-in"
                    >
                        <div
                            v-if="activeTab === 'info'"
                            key="info"
                            class="tab-pane p-1 space-y-4"
                        >
                            <div
                                class="stats stats-vertical sm:stats-horizontal w-full rounded-xl bg-base-200/40 border border-base-content/5"
                            >
                                <ClientRating
                                    :clientId="client.id"
                                    :ratingAvg="ratingAvg"
                                    :ratingCount="ratingCount"
                                    :myRating="myRating"
                                    @update:my-rating="
                                        emit('update:my-rating', $event)
                                    "
                                    :isAuthenticated="isAuthenticated"
                                    @update:rating-avg="
                                        emit('update:rating-avg', $event)
                                    "
                                    @update:rating-count="
                                        emit('update:rating-count', $event)
                                    "
                                />

                                <div
                                    v-if="clientDetails.created_at"
                                    class="stat"
                                >
                                    <div
                                        class="stat-title text-[10px] font-bold uppercase tracking-widest opacity-60"
                                    >
                                        {{ t("client.details.created") }}
                                    </div>
                                    <div
                                        class="stat-value text-sm font-semibold flex items-center gap-2"
                                    >
                                        <Info
                                            class="w-4 h-4 text-base-content/40"
                                        />
                                        {{
                                            new Date(
                                                clientDetails.created_at
                                            ).toLocaleDateString(undefined, {
                                                dateStyle: "medium",
                                            })
                                        }}
                                    </div>
                                </div>

                                <div
                                    v-if="clientDetails.source_link"
                                    class="stat"
                                >
                                    <div
                                        class="stat-title text-[10px] font-bold uppercase tracking-widest opacity-60"
                                    >
                                        {{ t("client.details.source_link") }}
                                    </div>
                                    <div
                                        class="stat-value text-sm flex items-center gap-2"
                                    >
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-sm justify-start px-2 gap-2 min-h-0 h-8"
                                            @click.stop="
                                                handleClientSourceLink(
                                                    clientDetails.source_link
                                                )
                                            "
                                        >
                                            <ExternalLink class="w-4 h-4" />
                                            <span
                                                class="truncate max-w-[18rem]"
                                                >{{
                                                    clientDetails.source_link
                                                }}</span
                                            >
                                        </button>
                                    </div>
                                </div>
                            </div>

                            <div
                                class="card bg-base-200/40 border border-base-content/5"
                            >
                                <div class="card-body p-4 gap-3">
                                    <div
                                        class="flex items-center justify-between"
                                    >
                                        <h4
                                            class="text-sm font-semibold text-base-content/80 flex items-center gap-2"
                                        >
                                            <ScrollText class="w-4 h-4" />
                                            {{ t("client.details.changelog") }}
                                        </h4>
                                        <span
                                            v-if="
                                                clientDetails.changelog_entries
                                            "
                                            class="badge badge-ghost badge-sm"
                                        >
                                            {{
                                                clientDetails.changelog_entries
                                                    .length
                                            }}
                                        </span>
                                    </div>

                                    <div
                                        v-if="
                                            clientDetails.changelog_entries &&
                                            clientDetails.changelog_entries
                                                .length > 0
                                        "
                                    >
                                        <ul
                                            class="timeline timeline-compact max-h-52 overflow-y-auto ml-2 pr-4 scrollbar-thin scrollbar-thumb-base-content/20 scrollbar-track-transparent"
                                        >
                                            <li
                                                v-for="(
                                                    entry, index
                                                ) in clientDetails.changelog_entries"
                                                :key="entry.version"
                                                class="timeline-item"
                                            >
                                                <div
                                                    class="timeline-start text-xs text-right"
                                                >
                                                    <span
                                                        class="badge badge-ghost text-xs"
                                                    >
                                                        {{
                                                            new Date(
                                                                entry.created_at
                                                            ).toLocaleDateString(
                                                                undefined,
                                                                {
                                                                    month: "short",
                                                                    day: "numeric",
                                                                }
                                                            )
                                                        }}
                                                    </span>
                                                </div>
                                                <div class="timeline-middle">
                                                    <Info
                                                        class="w-4 h-4 text-base-content/50"
                                                    />
                                                </div>
                                                <div
                                                    class="timeline-end timeline-box shadow-sm bg-base-100/40 w-full mb-2 border border-base-content/10"
                                                >
                                                    <div
                                                        class="font-bold text-sm text-base-content"
                                                    >
                                                        v{{ entry.version }}
                                                    </div>
                                                    <div
                                                        class="text-sm whitespace-pre-line text-base-content/80 mt-1 leading-relaxed"
                                                    >
                                                        {{ entry.content }}
                                                    </div>
                                                </div>
                                                <hr
                                                    v-if="
                                                        index <
                                                        clientDetails
                                                            .changelog_entries
                                                            .length -
                                                            1
                                                    "
                                                    class="bg-base-content/10"
                                                />
                                            </li>
                                        </ul>
                                    </div>

                                    <div
                                        v-else
                                        class="flex flex-col items-center justify-center text-center p-8 bg-base-100/30 border border-dashed border-base-content/10 rounded-lg opacity-0 animate-fade-in"
                                    >
                                        <Info
                                            class="w-8 h-8 text-base-content/50 mb-2"
                                        />
                                        <p
                                            class="text-sm font-medium text-base-content/70"
                                        >
                                            {{
                                                t("client.details.no_changelog")
                                            }}
                                        </p>
                                        <p
                                            class="text-xs text-base-content/50 mt-1"
                                        >
                                            {{
                                                t(
                                                    "client.details.no_changelog_desc"
                                                )
                                            }}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div
                            v-else-if="activeTab === 'screenshots'"
                            key="screenshots"
                            class="tab-pane"
                        >
                            <div
                                v-if="
                                    clientDetails.screenshot_urls &&
                                    clientDetails.screenshot_urls.length > 0
                                "
                                class="screenshots-section"
                            >
                                <div
                                    class="grid grid-cols-1 sm:grid-cols-2 gap-2"
                                >
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
                                                handleScreenshotClick(
                                                    $event,
                                                    index
                                                )
                                            "
                                        >
                                            <img
                                                :src="screenshot"
                                                :alt="`${client.name} screenshot ${index + 1}`"
                                                class="w-full h-36 object-cover transition-all duration-200 group-hover:scale-105"
                                                @error="
                                                    ($event.target as HTMLImageElement).closest('.screenshot-container')?.setAttribute('style', 'display:none')
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
                            <div
                                v-else
                                class="text-center py-8 text-base-content/60"
                            >
                                <p>{{ t("client.details.no_screenshots") }}</p>
                            </div>
                        </div>
                    </transition>
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

.timeline {
    scrollbar-gutter: stable;
    scrollbar-width: thin;
}

.timeline::-webkit-scrollbar {
    width: 10px;
    height: 10px;
}

.timeline::-webkit-scrollbar-track {
    background: transparent;
}

.timeline::-webkit-scrollbar-thumb {
    background-color: rgba(100, 100, 100, 0.35);
    border-radius: 999px;
    border: 3px solid transparent;
    background-clip: content-box;
}

.tab-slide-enter-active,
.tab-slide-leave-active {
    transition: all 0.3s ease;
}

.tab-slide-enter-from {
    opacity: 0;
    transform: translateX(20px);
}

.tab-slide-leave-to {
    opacity: 0;
    transform: translateX(-20px);
}

.tab-slide-right-enter-active,
.tab-slide-right-leave-active,
.tab-slide-left-enter-active,
.tab-slide-left-leave-active {
    transition: all 0.3s ease;
}

.tab-slide-right-enter-from {
    opacity: 0;
    transform: translateX(20px);
}

.tab-slide-right-leave-to {
    opacity: 0;
    transform: translateX(-20px);
}

.tab-slide-left-enter-from {
    opacity: 0;
    transform: translateX(-20px);
}

.tab-slide-left-leave-to {
    opacity: 0;
    transform: translateX(20px);
}

.tab-pane {
    min-height: 200px;
}

.tabs {
    justify-content: center;
}

.tab {
    transition: all 0.2s ease;
    border-radius: 4rem;
}

.tab:hover {
    background-color: rgba(255, 255, 255, 0.1);
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

@keyframes fade-in {
    from {
        opacity: 0;
        transform: translateY(10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.animate-fade-in {
    animation: fade-in 0.4s ease-out forwards;
}

.scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: var(--thumb-color) var(--track-color);
}
</style>
