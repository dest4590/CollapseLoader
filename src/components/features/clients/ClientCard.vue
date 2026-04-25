<script setup lang="ts">
import {
    computed,
    nextTick,
    onBeforeUnmount,
    onMounted,
    ref,
    shallowRef,
    watch,
} from "vue";
import { AlertTriangle, Star, X } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import gsap from "gsap";
import type { Client, ClientDetails, InstallProgress } from "../../../types/ui";
import InsecureClientWarningModal from "../../modals/clients/InsecureClientWarningModal.vue";
import ClientInfo from "./ClientInfo.vue";
import ScreenshotViewer from "./ScreenshotViewer.vue";
import ClientCardActions from "./ClientCardActions.vue";
import ClientCardDetails from "./ClientCardDetails.vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { apiGet } from "../../../services/apiClient";
import { useModal } from "../../../services/modalService";
import { useUser } from "../../../composables/useUser";

const { t } = useI18n();
const { showModal, hideModal } = useModal();

const props = defineProps<{
    client: Client;
    isClientRunning?: boolean;
    isClientInstalling?: boolean;
    installationStatus?: InstallProgress | undefined;
    isRequirementsInProgress: boolean;
    isAnyClientDownloading?: boolean;
    isFavorite?: boolean;
    isSelected?: boolean;
    isMultiSelectMode?: boolean;
    isHashVerifying?: boolean;
    isAnyCardExpanded?: boolean;
    class?: string;
    style?: string | Record<string, any>;
}>();

const emit = defineEmits([
    "launch",
    "download",
    "open-log-viewer",
    "show-context-menu",
    "client-click",
    "expanded-state-changed",
    "show-user-profile",
]);

const isExpanded = ref(false);
const isAnimating = ref(false);
const inTransition = ref(false);
const isCollapsing = ref(false);
const cardRef = shallowRef<HTMLElement | null>(null);
const placeholder = shallowRef<HTMLElement | null>(null);
const clientDetails = shallowRef<ClientDetails | null>(null);
const isLoadingDetails = ref(false);
const activeTab = ref<"info" | "screenshots">("info");

const myRating = ref<number | null>(null);
const ratingAvgOverride = ref<number | null>(null);
const ratingCountOverride = ref<number | null>(null);

const { isAuthenticated } = useUser();

let expansionLock = false;
let inTransitionTimeout: number | null = null;

const previousTab = ref<"info" | "screenshots">("info");
const slideDirection = ref<"left" | "right">("right");

const isScreenshotViewerOpen = ref(false);
const currentScreenshotIndex = ref(0);
const backdropRef = shallowRef<HTMLElement | null>(null);
const favoriteRef = shallowRef<HTMLElement | null>(null);

const scrollContainer = shallowRef<HTMLElement | null>(null);
const showScrollbar = ref(false);
const thumbHeight = ref(20);
const thumbTop = ref(0);
const isDraggingScrollbar = ref(false);
const dragStartY = ref(0);
const dragStartTop = ref(0);

let scrollbarUpdateScheduled = false;
let lastScrollUpdate = 0;
const SCROLL_THROTTLE = 16;

const updateScrollbar = () => {
    if (scrollbarUpdateScheduled) return;

    const now = performance.now();
    if (now - lastScrollUpdate < SCROLL_THROTTLE) return;
    lastScrollUpdate = now;

    scrollbarUpdateScheduled = true;
    requestAnimationFrame(() => {
        const el = scrollContainer.value;
        if (!el) {
            scrollbarUpdateScheduled = false;
            return;
        }

        const { clientHeight, scrollHeight, scrollTop } = el;
        const visibleRatio = clientHeight / scrollHeight;

        if (visibleRatio >= 0.99) {
            showScrollbar.value = false;
        } else {
            showScrollbar.value = true;
            const height = Math.max(visibleRatio * clientHeight, 30);
            thumbHeight.value = height;

            const maxThumbTop = clientHeight - height - 20;
            const maxScrollTop = scrollHeight - clientHeight;

            if (maxScrollTop > 0) {
                thumbTop.value = (scrollTop / maxScrollTop) * maxThumbTop;
            } else {
                thumbTop.value = 0;
            }
        }

        scrollbarUpdateScheduled = false;
    });
};

const handleScroll = () => {
    if (!isDraggingScrollbar.value) {
        updateScrollbar();
    }
};

const startScrollbarDrag = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    isDraggingScrollbar.value = true;
    dragStartY.value = event.clientY;
    dragStartTop.value = thumbTop.value;

    document.addEventListener("mousemove", onScrollbarDrag);
    document.addEventListener("mouseup", stopScrollbarDrag);
    document.body.style.userSelect = "none";
};

const onScrollbarDrag = (event: MouseEvent) => {
    if (!isDraggingScrollbar.value || !scrollContainer.value) return;

    const deltaY = event.clientY - dragStartY.value;
    const el = scrollContainer.value;
    const { clientHeight, scrollHeight } = el;
    const maxThumbTop = clientHeight - thumbHeight.value;
    const maxScrollTop = scrollHeight - clientHeight;

    let newThumbTop = dragStartTop.value + deltaY;
    newThumbTop = Math.max(0, Math.min(newThumbTop, maxThumbTop - 20));

    thumbTop.value = newThumbTop;

    const scrollRatio = newThumbTop / maxThumbTop;
    el.scrollTop = scrollRatio * maxScrollTop;
};

const stopScrollbarDrag = () => {
    isDraggingScrollbar.value = false;
    document.removeEventListener("mousemove", onScrollbarDrag);
    document.removeEventListener("mouseup", stopScrollbarDrag);
    document.body.style.userSelect = "";
};

watch(
    isExpanded,
    (newVal) => {
        if (newVal) {
            document.body.style.overflow = "hidden";
            nextTick(() => {
                updateScrollbar();
            });
        } else {
            document.body.style.overflow = "";
        }
    },
    { flush: "post" }
);

watch(
    clientDetails,
    () => {
        if (isExpanded.value) {
            nextTick(() => {
                updateScrollbar();
            });
        }
    },
    { flush: "post" }
);

const tabOrder = ["info", "screenshots"] as const;

const changeTab = (newTab: "info" | "screenshots") => {
    const currentIndex = tabOrder.indexOf(activeTab.value);
    const newIndex = tabOrder.indexOf(newTab);

    slideDirection.value = newIndex > currentIndex ? "right" : "left";
    previousTab.value = activeTab.value;
    activeTab.value = newTab;
};

const handleLaunchClick = () => {
    emit("launch", props.client);
};

const handleDownloadClick = () => {
    emit("download", props.client.id);
};

const handleOpenLogViewer = () => {
    emit("open-log-viewer", props.client);
};

const handleCardClick = (event: MouseEvent) => {
    if (isAnimating.value || isCollapsing.value || inTransition.value) {
        event.preventDefault();
        event.stopPropagation();
        return;
    }

    if (props.isMultiSelectMode) {
        emit("client-click", props.client, event);
        return;
    }

    if (!props.isSelected && !props.client.meta.is_custom) {
        const target = event.target as HTMLElement;
        if (
            target.closest(
                'button, a, .progress-bar-container, input, select, textarea, div[class*="tooltip"]'
            )
        ) {
            return;
        }
        expandCard();
    }
};

const handleShowContextMenu = (event: MouseEvent) => {
    emit("show-context-menu", event, props.client);
};

const showInsecureWarning = () => {
    showModal(
        `insecure-warning-${props.client.id}`,
        InsecureClientWarningModal,
        {
            title: t("modals.insecure_client_warning.modal_title"),
        },
        { client: props.client, infoVariant: true },
        {
            close: () => hideModal(`insecure-warning-${props.client.id}`),
        }
    );
};

const clientIsRunning = computed(() => !!props.isClientRunning);
const clientIsInstalling = computed(() => !!props.isClientInstalling);
const currentInstallStatus = computed(() => props.installationStatus);

const ratingAvg = computed(() => {
    if (ratingAvgOverride.value !== null) {
        return ratingAvgOverride.value;
    }
    if (!isExpanded.value) {
        const avg = props.client.rating_avg;
        return typeof avg === "number" ? avg : null;
    }
    const avg = clientDetails.value?.rating_avg ?? props.client.rating_avg;
    return typeof avg === "number" ? avg : null;
});

const ratingCount = computed(() => {
    if (ratingCountOverride.value !== null) {
        return ratingCountOverride.value;
    }
    if (!isExpanded.value) {
        const count = props.client.rating_count;
        return typeof count === "number" ? count : 0;
    }
    const count =
        clientDetails.value?.rating_count ?? props.client.rating_count;
    return typeof count === "number" ? count : 0;
});

const expandCard = async () => {
    const card = cardRef.value;
    if (!card || isAnimating.value || isCollapsing.value || inTransition.value)
        return;
    if (props.isAnyCardExpanded) return;
    if (props.client.meta.is_custom) return;

    if (expansionLock) return;
    expansionLock = true;

    isAnimating.value = true;

    if (!clientDetails.value && !isLoadingDetails.value) {
        isLoadingDetails.value = true;
        try {
            clientDetails.value = await apiGet<ClientDetails>(
                `/clients/${props.client.id}/detailed`
            );
        } catch (error) {
            console.error("Failed to fetch client details:", error);
            if (inTransitionTimeout !== null) {
                clearTimeout(inTransitionTimeout);
                inTransitionTimeout = null;
            }
            inTransition.value = false;
            isAnimating.value = false;
            expansionLock = false;
            return;
        } finally {
            isLoadingDetails.value = false;
        }
    }

    const rect = card.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    const modalWidth = Math.min(600, viewportWidth * 0.9);
    const modalHeight = Math.min(600, viewportHeight * 0.8);

    const centerX = (viewportWidth - modalWidth) / 2;
    const centerY = (viewportHeight - modalHeight) / 2;

    emit("expanded-state-changed", props.client.id, true);

    placeholder.value = document.createElement("div");
    placeholder.value.style.width = `${card.offsetWidth}px`;
    placeholder.value.style.height = `${card.offsetHeight}px`;
    placeholder.value.style.margin = getComputedStyle(card).margin;
    placeholder.value.style.visibility = "hidden";
    card.parentNode?.insertBefore(placeholder.value, card);

    const detailsContainer = card.querySelector(
        ".client-details"
    ) as HTMLElement | null;
    if (detailsContainer) {
        detailsContainer.style.display = "block";
        gsap.set(detailsContainer, {
            opacity: 0,
            maxHeight: "0px",
            overflow: "hidden",
        });
    }

    card.style.position = "fixed";
    card.style.top = `${rect.top}px`;
    card.style.left = `${rect.left}px`;
    card.style.width = `${rect.width}px`;
    card.style.height = `${rect.height}px`;
    card.style.zIndex = "1000";
    card.style.overflow = "hidden";

    document.body.appendChild(card);

    const fixedRect = card.getBoundingClientRect();
    const offsetX = fixedRect.left - rect.left;
    const offsetY = fixedRect.top - rect.top;

    if (offsetX !== 0 || offsetY !== 0) {
        card.style.left = `${rect.left - offsetX}px`;
        card.style.top = `${rect.top - offsetY}px`;
    }

    const backdrop = document.createElement("div");
    Object.assign(backdrop.style, {
        position: "fixed",
        top: "0",
        left: "0",
        width: "100vw",
        height: "100vh",
        backgroundColor: "rgba(0, 0, 0, 0)",
        backdropFilter: "blur(0px)",
        zIndex: "999",
        pointerEvents: "auto",
        transition: "background-color 0.5s ease, backdrop-filter 0.5s ease",
    });
    document.body.appendChild(backdrop);
    backdropRef.value = backdrop;

    backdrop.offsetHeight;

    Object.assign(backdrop.style, {
        backgroundColor: "rgba(0, 0, 0, 0.6)",
        backdropFilter: "blur(4px)",
    });

    backdrop.addEventListener("click", collapseCard);

    const tl = gsap.timeline({
        onComplete: () => {
            isExpanded.value = true;
            isAnimating.value = false;
        },
    });

    if (favoriteRef.value) {
        tl.to(
            favoriteRef.value,
            {
                autoAlpha: 0,
                duration: 0.3,
                ease: "power2.out",
            },
            0
        );
    }

    tl.to(
        card,
        {
            duration: 0.5,
            top: centerY - offsetY,
            left: centerX - offsetX,
            width: modalWidth,
            height: modalHeight,
            borderRadius: "1rem",
            boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.5)",
            ease: "back.out(0.6)",
        },
        0
    );

    if (detailsContainer) {
        tl.to(
            detailsContainer,
            {
                opacity: 1,
                maxHeight: () => detailsContainer.scrollHeight + "px",
                duration: 0.4,
                ease: "power2.out",
            },
            "-=0.3"
        );
    }

    if (inTransitionTimeout !== null) {
        clearTimeout(inTransitionTimeout);
    }
    inTransitionTimeout = window.setTimeout(() => {
        inTransition.value = true;
        inTransitionTimeout = null;
    }, 300);
};

const collapseCard = () => {
    const card = cardRef.value;
    if (!card || !placeholder.value || isCollapsing.value) return;

    ratingAvgOverride.value = null;
    ratingCountOverride.value = null;

    if (inTransitionTimeout !== null) {
        clearTimeout(inTransitionTimeout);
        inTransitionTimeout = null;
    }
    inTransition.value = false;
    isCollapsing.value = true;
    isAnimating.value = true;

    const placeholderRect = placeholder.value.getBoundingClientRect();
    if (card) card.style.overflow = "hidden";

    const detailsContainer = card.querySelector(
        ".client-details"
    ) as HTMLElement | null;

    if (backdropRef.value) {
        const backdrop = backdropRef.value;
        backdrop.removeEventListener("click", collapseCard);
        Object.assign(backdrop.style, {
            backgroundColor: "rgba(0, 0, 0, 0)",
            backdropFilter: "blur(0px)",
        });

        setTimeout(() => {
            if (backdrop.parentNode) {
                backdrop.parentNode.removeChild(backdrop);
            }
        }, 500);
        backdropRef.value = null;
    }

    const currentRect = card.getBoundingClientRect();
    const styleTop = parseFloat(card.style.top);
    const styleLeft = parseFloat(card.style.left);
    const offsetX = currentRect.left - styleLeft;
    const offsetY = currentRect.top - styleTop;

    const tl = gsap.timeline({
        onComplete: () => {
            if (card) {
                card.style.position = "";
                card.style.top = "";
                card.style.left = "";
                card.style.width = "";
                card.style.height = "";
                card.style.zIndex = "";
                card.style.boxShadow = "";
                card.style.overflow = "";
                card.style.borderRadius = "";

                if (placeholder.value && placeholder.value.parentNode) {
                    placeholder.value.parentNode.insertBefore(
                        card,
                        placeholder.value
                    );
                }
            }

            placeholder.value?.parentNode?.removeChild(placeholder.value);
            placeholder.value = null;

            if (detailsContainer) {
                detailsContainer.style.display = "none";
                gsap.set(detailsContainer, {
                    opacity: 0,
                    maxHeight: "0px",
                    overflow: "hidden",
                });
            }
            isExpanded.value = false;
            isAnimating.value = false;
            isCollapsing.value = false;
            expansionLock = false;
            emit("expanded-state-changed", props.client.id, false);
        },
    });

    if (detailsContainer) {
        const extraInfoElements =
            detailsContainer.querySelectorAll(".extra-info > *");
        if (extraInfoElements.length > 0) {
            tl.to(extraInfoElements, {
                opacity: 0,
                y: 20,
                duration: 0.2,
                stagger: {
                    each: 0.04,
                    from: "start",
                },
                ease: "power1.in",
            });
        }

        tl.to(
            detailsContainer,
            {
                maxHeight: "0px",
                opacity: 0,
                duration: 0.25,
                ease: "power2.in",
                onStart: () => {
                    if (detailsContainer)
                        detailsContainer.style.overflow = "hidden";
                },
            },
            extraInfoElements.length > 0 ? "-=0.1" : ">"
        );
    }

    tl.to(
        card,
        {
            duration: 0.5,
            top: placeholderRect.top - offsetY,
            left: placeholderRect.left - offsetX,
            width: placeholderRect.width,
            height: placeholderRect.height,
            borderRadius: "0.5rem",
            boxShadow: "0 4px 6px -1px rgba(0, 0, 0, 0.1)",
            ease: "power3.inOut",
        },
        detailsContainer ? "-=0.2" : ">"
    );

    if (favoriteRef.value) {
        tl.to(
            favoriteRef.value,
            {
                autoAlpha: 1,
                duration: 0.3,
                ease: "power2.out",
                clearProps: "all",
            },
            "-=0.3"
        );
    }
};

const openScreenshotViewer = (index: number) => {
    currentScreenshotIndex.value = index;
    isScreenshotViewerOpen.value = true;
};

const handleScreenshotClick = (event: MouseEvent, index: number) => {
    event.preventDefault();
    event.stopPropagation();
    openScreenshotViewer(index);
};

const handleCardKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && isExpanded.value && !isCollapsing.value) {
        collapseCard();
    }
};

const handleClientSourceLink = async (url: string) => {
    try {
        await openUrl(url);
    } catch (error) {
        console.error("Failed to open URL:", error);
    }
};

onMounted(() => {
    document.addEventListener("keydown", handleCardKeyDown);
});

const cleanupExpandedCard = () => {
    if (inTransitionTimeout !== null) {
        clearTimeout(inTransitionTimeout);
        inTransitionTimeout = null;
    }
    inTransition.value = false;

    if (!isExpanded.value || !cardRef.value) return;

    const card = cardRef.value;
    gsap.killTweensOf(card);

    if (placeholder.value) {
        placeholder.value.parentNode?.removeChild(placeholder.value);
        placeholder.value = null;
    }

    if (backdropRef.value) {
        backdropRef.value.parentNode?.removeChild(backdropRef.value);
        backdropRef.value = null;
    }

    card.style.position = "";
    card.style.top = "";
    card.style.left = "";
    card.style.width = "";
    card.style.height = "";
    card.style.zIndex = "";
    card.style.boxShadow = "";
    card.style.overflow = "";
    card.style.borderRadius = "";

    const detailsContainer = card.querySelector(
        ".client-details"
    ) as HTMLElement | null;
    if (detailsContainer) {
        gsap.killTweensOf(detailsContainer);
        gsap.set(detailsContainer, {
            opacity: 0,
            maxHeight: "0px",
            overflow: "hidden",
        });
        const extraInfoElements =
            detailsContainer.querySelectorAll(".extra-info > *");
        gsap.killTweensOf(extraInfoElements);
        gsap.set(extraInfoElements, { opacity: 0, y: 20 });
    }

    isExpanded.value = false;
    expansionLock = false;
    emit("expanded-state-changed", props.client.id, false);
};

onBeforeUnmount(() => {
    if (inTransitionTimeout !== null) {
        clearTimeout(inTransitionTimeout);
        inTransitionTimeout = null;
    }
    cleanupExpandedCard();
    document.removeEventListener("keydown", handleCardKeyDown);
});
</script>

<template>
    <div
        ref="cardRef"
        class="card card-border bg-base-300 shadow-lg border-base-content/10 client-card"
        :class="{
            'border-primary/50 ring-2 ring-primary/30 bg-primary/5': isSelected,
            'border-neutral/10': !isSelected,
            'cursor-pointer': isMultiSelectMode || !isAnimating,
            'hover:border-primary/30': isMultiSelectMode && !isSelected,
            'hover:bg-primary/10': isMultiSelectMode && !isSelected,
            'transition-all duration-200 ease-out hover:shadow-xl': !isExpanded,
        }"
        :data-client-id="client.id"
        @contextmenu="handleShowContextMenu"
        @click="handleCardClick"
    >
        <div
            v-if="isSelected && !isExpanded"
            class="absolute z-0"
            style="right: 1.1rem; top: 1.1rem"
        >
            <div
                class="w-5 h-5 bg-primary rounded-full flex items-center justify-center"
            >
                <svg
                    class="w-3 h-3 text-primary-content"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                >
                    <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                    />
                </svg>
            </div>
        </div>
        <div
            class="absolute z-0 transition-opacity duration-200"
            :class="[
                isMultiSelectMode && !isSelected && !isExpanded
                    ? 'opacity-100'
                    : 'opacity-0 pointer-events-none',
            ]"
            style="right: 1.1rem; top: 1.1rem"
        >
            <div
                class="w-5 h-5 border-2 border-base-content/30 rounded-full bg-base-100"
            ></div>
        </div>

        <button
            @click="!isCollapsing && collapseCard()"
            :disabled="isCollapsing"
            class="close-btn btn btn-sm btn-circle btn-ghost absolute top-3 right-3 z-50 text-base-content transition-opacity duration-200"
            :class="{
                'opacity-0 pointer-events-none':
                    (!isExpanded && !isAnimating) || isCollapsing,
                'opacity-100 pointer-events-auto mr-2':
                    (isExpanded || isAnimating) && !isCollapsing,
            }"
        >
            <X class="w-5 h-5" />
        </button>

        <div
            class="scroll-container h-full w-full overflow-y-auto custom-scrollbar-hide relative"
            ref="scrollContainer"
            @scroll="handleScroll"
        >
            <div class="p-4 min-h-full">
                <div class="card-body flex flex-col p-0">
                    <div class="flex justify-between items-start">
                        <h2 class="card-title text-base">
                            {{ client.name }}
                            <div v-if="client.insecure">
                                <div
                                    class="tooltip tooltip-bottom"
                                    :data-tip="t('common.click')"
                                    @click="showInsecureWarning"
                                >
                                    <AlertTriangle
                                        class="text-warning w-4 h-4"
                                    />
                                </div>
                            </div>
                        </h2>
                        <transition name="fade" appear>
                            <div
                                ref="favoriteRef"
                                v-if="isFavorite"
                                class="favorite-indicator"
                            >
                                <Star
                                    class="w-4 h-4 fill-yellow-400 text-yellow-400"
                                />
                            </div>
                        </transition>
                    </div>

                    <ClientInfo :client="client" :expanded="inTransition" />

                    <ClientCardActions
                        :client="client"
                        :clientIsInstalling="clientIsInstalling"
                        :currentInstallStatus="currentInstallStatus"
                        :clientIsRunning="clientIsRunning"
                        :isRequirementsInProgress="isRequirementsInProgress"
                        :isAnimatingOrExpanded="isAnimating || isExpanded"
                        @launch="handleLaunchClick"
                        @download="handleDownloadClick"
                        @open-log-viewer="handleOpenLogViewer"
                    />

                    <ClientCardDetails
                        :client="client"
                        :clientDetails="clientDetails"
                        :isLoadingDetails="isLoadingDetails"
                        :isAuthenticated="isAuthenticated"
                        :ratingAvg="ratingAvg"
                        :ratingCount="ratingCount"
                        :activeTab="activeTab"
                        :slideDirection="slideDirection"
                        :myRating="myRating"
                        @update:my-rating="myRating = $event"
                        @update:rating-avg="ratingAvgOverride = $event"
                        @update:rating-count="ratingCountOverride = $event"
                        @update:client-details="clientDetails = $event"
                        @change-tab="changeTab"
                        @screenshot-click="handleScreenshotClick"
                        @show-user-profile="emit('show-user-profile', $event)"
                        @open-source-link="handleClientSourceLink"
                    />
                </div>
            </div>
        </div>

        <transition name="fade">
            <div
                v-if="showScrollbar"
                class="custom-scrollbar-track absolute right-1 top-1 bottom-1 w-1.5 m-1 h-[95%] bg-base-content/5 rounded-full z-50 transition-opacity duration-200"
                :class="{
                    'opacity-0': !isExpanded || isCollapsing,
                    'opacity-100': isExpanded && !isCollapsing,
                }"
            >
                <div
                    class="custom-scrollbar-thumb absolute w-full bg-base-content/20 hover:bg-base-content/40 rounded-full transition-colors duration-200 cursor-pointer"
                    :style="{
                        height: thumbHeight + 'px',
                        top: thumbTop + 'px',
                    }"
                    @mousedown="startScrollbarDrag"
                ></div>
            </div>
        </transition>
    </div>

    <ScreenshotViewer
        v-if="isScreenshotViewerOpen && clientDetails?.screenshot_urls"
        :isOpen="isScreenshotViewerOpen"
        :screenshotUrls="clientDetails.screenshot_urls"
        :initialIndex="currentScreenshotIndex"
        :clientName="client.name"
        @close="isScreenshotViewerOpen = false"
    />
</template>

<style scoped>
.client-card {
    position: relative;
    border-radius: 0.5rem;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    -webkit-overflow-scrolling: touch;
    will-change: transform, width, height, top, left;
}

.custom-scrollbar-hide {
    scrollbar-width: none;
    -ms-overflow-style: none;
}

.custom-scrollbar-hide::-webkit-scrollbar {
    display: none;
}

.client-details {
    opacity: 0;
    max-height: 0;
    overflow: hidden;
    display: none;
    scrollbar-gutter: stable;
}

.fade-transform-enter-active,
.fade-transform-leave-active {
    transition:
        opacity 0.2s ease,
        transform 0.2s ease;
}

.fade-transform-enter-from,
.fade-transform-leave-to {
    opacity: 0;
}

.fade-transform-enter-to,
.fade-transform-leave-from {
    opacity: 1;
}

.favorite-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    margin-left: 8px;
    color: #fbbf24;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.fade-transform-enter-active,
.fade-transform-leave-active {
    transition: all 0.3s ease;
}

.fade-transform-enter-from {
    opacity: 0;
    transform: translateX(10px);
}

.fade-transform-leave-to {
    opacity: 0;
    transform: translateX(-10px);
}
</style>
