<script setup lang="ts">
import { computed, ref, onBeforeUnmount, onMounted } from 'vue';
import {
    StopCircle,
    Terminal,
    Download,
    Star,
    AlertTriangle,
    X,
    ChevronLeft,
    ChevronRight,
    ZoomIn,
} from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import gsap from 'gsap';
import type { Client, InstallProgress, ClientDetails } from '../../../types/ui';
import ClientInfo from './ClientInfo.vue';
import { invoke } from '@tauri-apps/api/core';

const { t } = useI18n();

const props = defineProps<{
    client: Client;
    isClientRunning: (id: number) => boolean;
    isClientInstalling: (client: Client) => boolean;
    installationStatus: Map<string, InstallProgress>;
    isRequirementsInProgress: boolean;
    isAnyClientDownloading?: boolean;
    isFavorite?: boolean;
    isSelected?: boolean;
    isMultiSelectMode?: boolean;
    isHashVerifying?: boolean;
    class?: string;
    style?: string | Record<string, any>;
}>();

const emit = defineEmits([
    'launch',
    'download',
    'open-log-viewer',
    'show-context-menu',
    'client-click',
    'expanded-state-changed',
]);

const isExpanded = ref(false);
const isAnimating = ref(false);
const inTransition = ref(false);
const isCollapsing = ref(false);
const cardRef = ref<HTMLElement | null>(null);
const placeholder = ref<HTMLElement | null>(null);
const clientDetails = ref<ClientDetails | null>(null);
const isLoadingDetails = ref(false);
const activeTab = ref<'info' | 'screenshots'>('info');

const previousTab = ref<'info' | 'screenshots'>('info');
const slideDirection = ref<'left' | 'right'>('right');

const isScreenshotViewerOpen = ref(false);
const currentScreenshotIndex = ref(0);
const screenshotViewerRef = ref<HTMLElement | null>(null);
const isImageLoading = ref(false);
const imageTransitionDirection = ref<'next' | 'prev'>('next');
const imageRef = ref<HTMLImageElement | null>(null);
const imageContainerRef = ref<HTMLElement | null>(null);

const zoomScale = ref(1);
const zoomPosition = ref({ x: 0, y: 0 });
const isZoomed = ref(false);
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const hasDragged = ref(false);
const maxZoom = 5;
const minZoom = 1;
const zoomStep = 0.5;

const tabOrder = ['info', 'screenshots'] as const;

const changeTab = (newTab: 'info' | 'screenshots') => {
    const currentIndex = tabOrder.indexOf(activeTab.value);
    const newIndex = tabOrder.indexOf(newTab);

    slideDirection.value = newIndex > currentIndex ? 'right' : 'left';
    previousTab.value = activeTab.value;
    activeTab.value = newTab;
};

const handleLaunchClick = () => {
    emit('launch', props.client);
};

const handleDownloadClick = () => {
    emit('download', props.client.id);
};

const handleOpenLogViewer = () => {
    emit('open-log-viewer', props.client);
};

const handleCardClick = (event: MouseEvent) => {
    if (props.isMultiSelectMode) {
        emit('client-click', props.client, event);
    } else if (!isAnimating.value && !props.isSelected) {
        const target = event.target as HTMLElement;
        if (target.closest('button, a, .progress-bar-container, input, select, textarea')) {
            return;
        }
        expandCard();
    }
};

const handleShowContextMenu = (event: MouseEvent) => {
    emit('show-context-menu', event, props.client);
};

const clientIsRunning = computed(() => props.isClientRunning(props.client.id));
const clientIsInstalling = computed(() => props.isClientInstalling(props.client));
const currentInstallStatus = computed(() => props.installationStatus.get(props.client.filename));
const isHashVerifying = computed(() => props.isHashVerifying || false);

const cardBorderRadius = computed(() => {
    return getComputedStyle(document.documentElement).getPropertyValue('--client-card-radius')?.trim() || '0.5rem';
});
const cardBoxShadow = computed(() => {
    return getComputedStyle(document.documentElement).getPropertyValue('--client-card-shadow')?.trim() || '0 4px 6px -1px rgba(0,0,0,0.1)';
});
const cardPadding = computed(() => {
    return getComputedStyle(document.documentElement).getPropertyValue('--client-card-padding')?.trim() || '1rem';
});

const expandCard = async () => {
    const card = cardRef.value;
    if (!card) return;

    if (!clientDetails.value && !isLoadingDetails.value) {
        isLoadingDetails.value = true;
        try {
            const response = await invoke<ClientDetails>('get_client_details', { clientId: props.client.id });
            clientDetails.value = response;
        } catch (error) {
            console.error('Failed to fetch client details:', error);
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

    isAnimating.value = true;
    emit('expanded-state-changed', props.client.id, true);

    placeholder.value = document.createElement('div');
    placeholder.value.style.width = `${card.offsetWidth}px`;
    placeholder.value.style.height = `${card.offsetHeight}px`;
    placeholder.value.style.margin = getComputedStyle(card).margin;
    placeholder.value.style.visibility = 'hidden';
    card.parentNode?.insertBefore(placeholder.value, card);

    const detailsContainer = card.querySelector('.client-details') as HTMLElement | null;
    if (detailsContainer) {
        detailsContainer.style.display = 'block';
        gsap.set(detailsContainer, { opacity: 0, maxHeight: '0px', overflow: 'hidden' });
    }

    card.style.position = 'fixed';
    card.style.top = `${rect.top}px`;
    card.style.left = `${rect.left}px`;
    card.style.width = `${rect.width}px`;
    card.style.height = `${rect.height}px`;
    card.style.zIndex = '1000';
    card.style.overflow = 'hidden';

    const tl = gsap.timeline({
        onComplete: () => {
            if (card) card.style.overflow = 'auto';
            isExpanded.value = true;
        }
    });

    tl.to(card, {
        duration: 0.5,
        top: centerY,
        left: centerX,
        width: modalWidth,
        height: modalHeight,
        boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.8)',
        ease: 'expo.inOut',
    });

    if (detailsContainer) {
        tl.to(detailsContainer, {
            opacity: 1,
            maxHeight: () => detailsContainer.scrollHeight + 'px',
            duration: 0.4,
            ease: 'power2.out',
        }, "-=0.3");


    }

    setTimeout(() => {
        inTransition.value = true;
    }, 300);
};

const collapseCard = () => {
    const card = cardRef.value;
    if (!card || !placeholder.value) return;

    inTransition.value = false;
    isCollapsing.value = true;

    const placeholderRect = placeholder.value.getBoundingClientRect();
    if (card) card.style.overflow = 'hidden';

    const detailsContainer = card.querySelector('.client-details') as HTMLElement | null;

    const tl = gsap.timeline({
        onComplete: () => {
            if (card) {
                card.style.position = '';
                card.style.top = '';
                card.style.left = '';
                card.style.width = '';
                card.style.height = '';
                card.style.zIndex = '';
                card.style.boxShadow = '';
                card.style.overflow = '';
            }

            placeholder.value?.parentNode?.removeChild(placeholder.value);
            placeholder.value = null;

            if (detailsContainer) {
                detailsContainer.style.display = 'none';
                gsap.set(detailsContainer, { opacity: 0, maxHeight: '0px', overflow: 'hidden' });
            }
            isExpanded.value = false;
            isAnimating.value = false;
            isCollapsing.value = false;
            emit('expanded-state-changed', props.client.id, false);
        }
    });

    if (detailsContainer) {
        const extraInfoElements = detailsContainer.querySelectorAll('.extra-info > *');
        if (extraInfoElements.length > 0) {
            tl.to(extraInfoElements, {
                opacity: 0,
                y: 20,
                duration: 0.2,
                stagger: {
                    each: 0.04,
                    from: "start"
                },
                ease: 'power1.in',
            });
        }

        tl.to(detailsContainer, {
            maxHeight: '0px',
            opacity: 0,
            duration: 0.25,
            ease: 'power2.in',
            onStart: () => {
                if (detailsContainer) detailsContainer.style.overflow = 'hidden';
            }
        }, extraInfoElements.length > 0 ? "-=0.1" : ">");
    }

    tl.to(card, {
        duration: 0.5,
        top: placeholderRect.top,
        left: placeholderRect.left,
        width: placeholderRect.width,
        height: placeholderRect.height,
        boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
        ease: 'expo.inOut',
    }, detailsContainer ? "-=0.2" : ">");
};

onBeforeUnmount(() => {
    if (isExpanded.value && cardRef.value) {
        const card = cardRef.value;
        gsap.killTweensOf(card);

        if (placeholder.value) {
            placeholder.value.parentNode?.removeChild(placeholder.value);
            placeholder.value = null;
        }

        card.style.position = '';
        card.style.top = '';
        card.style.left = '';
        card.style.width = '';
        card.style.height = '';
        card.style.zIndex = '';
        card.style.boxShadow = '';
        card.style.overflow = '';

        const detailsContainer = card.querySelector('.client-details') as HTMLElement | null;
        if (detailsContainer) {
            gsap.killTweensOf(detailsContainer);
            gsap.set(detailsContainer, { opacity: 0, maxHeight: '0px', overflow: 'hidden' });
            const extraInfoElements = detailsContainer.querySelectorAll('.extra-info > *');
            gsap.killTweensOf(extraInfoElements);
            gsap.set(extraInfoElements, { opacity: 0, y: 20 });
        }
        isExpanded.value = false;
        emit('expanded-state-changed', props.client.id, false);
    }

    if (isScreenshotViewerOpen.value) {
        document.body.style.overflow = '';
        document.removeEventListener('keydown', handleScreenshotKeydown);
    }
});

const openScreenshotViewer = (index: number) => {
    currentScreenshotIndex.value = index;
    isScreenshotViewerOpen.value = true;
    isImageLoading.value = true;
    imageTransitionDirection.value = 'next';
    resetZoom();
    document.body.style.overflow = 'hidden';
    document.addEventListener('keydown', handleScreenshotKeydown);
};

const handleScreenshotClick = (event: MouseEvent, index: number) => {
    event.preventDefault();
    event.stopPropagation();
    openScreenshotViewer(index);
};

const closeScreenshotViewer = () => {
    isScreenshotViewerOpen.value = false;
    isImageLoading.value = false;
    resetZoom();
    document.body.style.overflow = '';
    document.removeEventListener('keydown', handleScreenshotKeydown);
};

const handleScreenshotViewerClose = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    closeScreenshotViewer();
};

const nextScreenshot = () => {
    if (clientDetails.value?.screenshot_urls && currentScreenshotIndex.value < clientDetails.value.screenshot_urls.length - 1) {
        imageTransitionDirection.value = 'next';
        isImageLoading.value = true;
        currentScreenshotIndex.value = currentScreenshotIndex.value + 1;
        resetZoom();
    } else if (clientDetails.value?.screenshot_urls) {
        imageTransitionDirection.value = 'next';
        isImageLoading.value = true;
        currentScreenshotIndex.value = 0;
        resetZoom();
    }
};

const prevScreenshot = () => {
    if (currentScreenshotIndex.value > 0) {
        imageTransitionDirection.value = 'prev';
        isImageLoading.value = true;
        currentScreenshotIndex.value = currentScreenshotIndex.value - 1;
        resetZoom();
    } else if (clientDetails.value?.screenshot_urls) {
        imageTransitionDirection.value = 'prev';
        isImageLoading.value = true;
        currentScreenshotIndex.value = clientDetails.value.screenshot_urls.length - 1;
        resetZoom();
    }
};

const handleImageLoad = () => {
    isImageLoading.value = false;
};

const handleImageError = () => {
    isImageLoading.value = false;
    closeScreenshotViewer();
};

const handleScreenshotKeydown = (event: KeyboardEvent) => {
    switch (event.key) {
        case 'Escape':
            if (isZoomed.value) {
                resetZoom();
            } else {
                closeScreenshotViewer();
            }
            break;
        case 'ArrowLeft':
            if (!isZoomed.value) {
                prevScreenshot();
            }
            break;
        case 'ArrowRight':
            if (!isZoomed.value) {
                nextScreenshot();
            }
            break;
        case '+':
        case '=':
            zoomIn();
            break;
        case '-':
            zoomOut();
            break;
        case '0':
            resetZoom();
            break;
    }
};


const resetZoom = () => {
    zoomScale.value = 1;
    zoomPosition.value = { x: 0, y: 0 };
    isZoomed.value = false;
    isDragging.value = false;
};

const zoomToPoint = (event: MouseEvent) => {
    if (!imageRef.value || !imageContainerRef.value) return;

    const rect = imageContainerRef.value.getBoundingClientRect();

    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const clickX = event.clientX - centerX;
    const clickY = event.clientY - centerY;

    const newScale = Math.min(zoomScale.value + zoomStep, maxZoom);

    if (newScale > 1) {
        const scaleFactor = newScale / zoomScale.value;

        zoomPosition.value = {
            x: (zoomPosition.value.x - clickX) * scaleFactor + clickX,
            y: (zoomPosition.value.y - clickY) * scaleFactor + clickY
        };

        zoomScale.value = newScale;
        isZoomed.value = true;

        setTimeout(() => constrainPosition(), 0);
    }
};

const zoomIn = (event?: MouseEvent) => {
    if (zoomScale.value >= maxZoom) return;

    if (event && imageRef.value) {
        zoomToPoint(event);
    } else {
        const newScale = Math.min(zoomScale.value + zoomStep, maxZoom);
        zoomScale.value = newScale;
        isZoomed.value = newScale > 1;

        if (isZoomed.value) {
            setTimeout(() => constrainPosition(), 0);
        }
    }
};

const zoomOut = () => {
    const newScale = Math.max(zoomScale.value - zoomStep, minZoom);

    if (newScale <= 1) {
        resetZoom();
    } else {
        zoomScale.value = newScale;
        setTimeout(() => constrainPosition(), 0);
    }
};

const handleWheel = (event: WheelEvent) => {
    event.preventDefault();
    event.stopPropagation();

    if (event.deltaY < 0) {
        zoomIn(event);
    } else {
        zoomOut();
    }
};

const startDrag = (event: MouseEvent) => {
    if (!isZoomed.value) return;

    event.preventDefault();
    event.stopPropagation();

    isDragging.value = true;
    hasDragged.value = false;
    dragStart.value = {
        x: event.clientX - zoomPosition.value.x,
        y: event.clientY - zoomPosition.value.y
    };

    document.addEventListener('mousemove', handleDrag, { passive: false });
    document.addEventListener('mouseup', stopDrag);

    if (imageRef.value) {
        imageRef.value.style.pointerEvents = 'none';
    }
};

const handleDrag = (event: MouseEvent) => {
    if (!isDragging.value) return;

    event.preventDefault();
    event.stopPropagation();

    hasDragged.value = true;

    zoomPosition.value = {
        x: event.clientX - dragStart.value.x,
        y: event.clientY - dragStart.value.y
    };
};

const stopDrag = (event?: MouseEvent) => {
    if (event) {
        event.preventDefault();
        event.stopPropagation();
    }

    isDragging.value = false;
    document.removeEventListener('mousemove', handleDrag);
    document.removeEventListener('mouseup', stopDrag);

    if (imageRef.value) {
        imageRef.value.style.pointerEvents = '';
    }

    constrainPosition();

    setTimeout(() => {
        hasDragged.value = false;
    }, 100);
};

const constrainPosition = () => {
    if (!imageRef.value || !imageContainerRef.value || zoomScale.value <= 1) {
        zoomPosition.value = { x: 0, y: 0 };
        return;
    }

    const containerRect = imageContainerRef.value.getBoundingClientRect();
    const imageRect = imageRef.value.getBoundingClientRect();

    const scaledWidth = imageRect.width * zoomScale.value;
    const scaledHeight = imageRect.height * zoomScale.value;

    const maxX = Math.max(0, (scaledWidth - containerRect.width) / 2);
    const maxY = Math.max(0, (scaledHeight - containerRect.height) / 2);

    const constrainedPosition = {
        x: Math.min(Math.max(zoomPosition.value.x, -maxX), maxX),
        y: Math.min(Math.max(zoomPosition.value.y, -maxY), maxY)
    };

    if (constrainedPosition.x !== zoomPosition.value.x || constrainedPosition.y !== zoomPosition.value.y) {
        zoomPosition.value = constrainedPosition;
    }
};

const handleImageClick = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    if (hasDragged.value) return;

    if (isZoomed.value) {
        resetZoom();
    } else {
        zoomToPoint(event);
    }
};

const handleViewerBackgroundClick = (event: MouseEvent) => {
    if (isDragging.value || hasDragged.value) return;

    const target = event.target as HTMLElement;
    if (target === event.currentTarget) {
        closeScreenshotViewer();
    }
};

const handleCardKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && isExpanded.value && !isCollapsing.value) {
        collapseCard();
    }
};

onMounted(() => {
    document.addEventListener('keydown', handleCardKeyDown);
});

onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleCardKeyDown);
});
</script>

<template>
    <div ref="cardRef" class="card bg-base-300 shadow-lg client-card" :class="{
        'border-primary/50 ring-2 ring-primary/30 bg-primary/5': isSelected,
        'border-neutral/10': !isSelected,
        'cursor-pointer': (isMultiSelectMode || !isAnimating),
        'hover:border-primary/30': isMultiSelectMode && !isSelected,
        'hover:bg-primary/10': isMultiSelectMode && !isSelected,
        'transition-all duration-200 ease-out': !isExpanded
    }" :data-client-id="client.id" @contextmenu="handleShowContextMenu" @click="handleCardClick" :style="{
        borderRadius: cardBorderRadius,
        boxShadow: cardBoxShadow,
        padding: cardPadding
    }">
        <div v-if="isSelected && !isExpanded" class="absolute -z-0" style="right: 1.1rem; top: 1.1rem;">
            <div class="w-5 h-5 bg-primary rounded-full flex items-center justify-center">
                <svg class="w-3 h-3 text-primary-content" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd" />
                </svg>
            </div>
        </div>
        <transition name="fade">
            <div v-if="isMultiSelectMode && !isSelected && !isExpanded" class="absolute z-0"
                style="right: 1.1rem; top: 1.1rem;">
                <div class="w-5 h-5 border-2 border-base-content/30 rounded-full bg-base-100"></div>
            </div>
        </transition>

        <transition name="fade">
            <button v-if="isAnimating" @click="!isCollapsing && collapseCard()"
                class="btn btn-sm btn-circle btn-ghost absolute top-3 right-3 z-[1001] text-base-content hover:bg-base-content/10"
                :disabled="isCollapsing">
                <X class="w-5 h-5" />
            </button>
        </transition>

        <div class="card-body p-4 flex flex-col">
            <div class="flex justify-between items-start">
                <h2 class="card-title text-base">
                    {{ client.name }}
                    <AlertTriangle v-if="client.insecure" class="text-warning w-4 h-4" />
                </h2>
                <transition name="fade" appear>
                    <div v-if="isFavorite && !isAnimating" class="favorite-indicator">
                        <Star class="w-4 h-4 fill-yellow-400 text-yellow-400" />
                    </div>
                </transition>
            </div>

            <ClientInfo :client="client" :expanded="inTransition" />

            <div class="card-actions justify-end mt-2">
                <button v-if="clientIsRunning && !clientIsInstalling && !isAnimating" @click.stop="handleOpenLogViewer"
                    class="btn btn-sm btn-ghost btn-circle text-info hover:bg-info/20 focus:ring-info">
                    <Terminal class="w-4 h-4" />
                </button>
                <transition name="fade-transform" mode="out-in">
                    <div v-if="clientIsInstalling && !isAnimating" class="w-full">
                        <div class="flex justify-between mb-1 text-xs text-base-content">
                            <span class="truncate max-w-[90%]">
                                {{ currentInstallStatus?.action }}
                                {{ client.name }}
                            </span>
                            <span>
                                {{ currentInstallStatus?.percentage }}%
                            </span>
                        </div>
                        <div class="progress-bar-container">
                            <div class="progress-bar" :style="{
                                width: `${currentInstallStatus?.percentage}%`,
                            }"></div>
                        </div>
                    </div>
                    <div v-else-if="isHashVerifying && !isAnimating" class="w-full">
                        <div class="flex justify-between mb-1 text-xs text-base-content">
                            <span class="truncate max-w-[90%]">
                                {{ t('home.verifying_hash') }}
                                {{ client.name }}
                            </span>
                            <span class="loading loading-spinner loading-xs"></span>
                        </div>
                        <div class="progress-bar-container">
                            <div class="progress-bar animate-pulse" style="width: 100%;"></div>
                        </div>
                    </div>
                    <div v-else-if="!isAnimating" class="flex items-center space-x-2">
                        <button v-if="!client.meta.installed" @click="handleDownloadClick"
                            class="btn btn-sm btn-primary min-w-[7rem] download-btn relative overflow-hidden" :disabled="isRequirementsInProgress || !client.working
                                ">
                            <span class="flex items-center download-text">
                                <Download v-if="client.working" class="w-4 h-4 mr-1" />
                                <span v-if="client.working">{{
                                    t('home.download')
                                }}</span>
                                <span v-else-if="!client.working">{{
                                    t('home.unavailable')
                                }}</span>
                            </span>
                            <span class="flex items-center get-text absolute inset-0 opacity-0">
                                {{ client.meta.size || '0' }} MB
                            </span>
                        </button>
                        <button v-else @click="handleLaunchClick" class="btn btn-sm min-w-[5rem] launch-btn"
                            :disabled="isRequirementsInProgress || isHashVerifying" :class="clientIsRunning
                                ? 'btn-error focus:ring-error'
                                : 'btn-primary focus:ring-primary'
                                ">
                            <StopCircle class="w-4 h-4 mr-1" v-if="clientIsRunning" />
                            <span v-if="isHashVerifying" class="loading loading-spinner loading-sm mr-1"></span>
                            {{
                                isHashVerifying
                                    ? t('home.verifying')
                                    : clientIsRunning
                                        ? t('home.stop')
                                        : t('home.launch')
                            }}
                        </button>
                    </div>
                </transition>
            </div>

            <div class="client-details">
                <div class="space-y-4">
                    <div v-if="isLoadingDetails" class="text-center py-4">
                        <div class="loading loading-spinner loading-md"></div>
                        <p class="text-sm text-base-content/60 mt-2">{{ t('client.details.loading') }}</p>
                    </div>

                    <div v-else-if="clientDetails">
                        <div class="tabs tabs-boxed mb-4">
                            <a class="tab" :class="{ 'tab-active': activeTab === 'info' }" @click="changeTab('info')">
                                {{ t('client.details.info_tab') }}
                            </a>
                            <a class="tab" :class="{ 'tab-active': activeTab === 'screenshots' }"
                                @click="changeTab('screenshots')">
                                {{ t('client.details.screenshots_tab') }}
                            </a>
                        </div>

                        <div>
                            <transition :name="`tab-slide-${slideDirection}`" mode="out-in">
                                <div v-if="activeTab === 'info'" key="info" class="tab-pane p-1">
                                    <div class="mb-6 space-y-2">
                                        <dl class="divide-y divide-base-content/10">
                                            <div v-if="clientDetails.source_link"
                                                class="py-3 grid grid-cols-1 md:grid-cols-3 gap-2 md:gap-4 items-center transition-all duration-300 hover:bg-base-content/5 -mx-4 px-4 rounded-lg">
                                                <dt class="text-sm font-medium text-base-content/70">{{
                                                    t('client.details.source_link') }}</dt>
                                                <dd class="text-sm text-base-content col-span-1 md:col-span-2">
                                                    <a :href="clientDetails.source_link" target="_blank"
                                                        rel="noopener noreferrer"
                                                        class="link link-primary link-hover inline-flex items-center gap-1.5 text-xs truncate">
                                                        <span>{{ clientDetails.source_link }}</span>
                                                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12"
                                                            viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                                            stroke-width="2.5" stroke-linecap="round"
                                                            stroke-linejoin="round" class="opacity-70 flex-shrink-0">
                                                            <path
                                                                d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6">
                                                            </path>
                                                            <polyline points="15 3 21 3 21 9"></polyline>
                                                            <line x1="10" y1="14" x2="21" y2="3"></line>
                                                        </svg>
                                                    </a>
                                                </dd>
                                            </div>
                                            <div v-if="clientDetails.created_at"
                                                class="py-3 transition-all duration-300 hover:bg-base-content/5 -mx-4 px-4 rounded-lg">
                                                <dt class="text-sm font-medium text-base-content/70">{{
                                                    t('client.details.created') }}</dt>
                                                <dd class="text-base-content col-span-1 md:col-span-2 text-xs">
                                                    {{ new Date(clientDetails.created_at).toLocaleDateString('en-GB') }}
                                                </dd>
                                            </div>
                                        </dl>
                                    </div>

                                    <div class="divider before:bg-base-content/10 after:bg-base-content/10 my-0">
                                        <h4 class="text-sm font-semibold text-base-content/80 flex items-center gap-2">
                                            <svg class="w-4 h-4 text-primary" fill="none" stroke="currentColor"
                                                viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                    d="M19 7v4a2 2 0 01-2 2H7a2 2 0 01-2-2V7" />
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                    d="M5 7V5a2 2 0 012-2h10a2 2 0 012 2v2" />
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                    d="M5 17h14" />
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                    d="M9 21h6" />
                                            </svg>
                                            {{ t('client.details.changelog') }}
                                        </h4>
                                    </div>

                                    <div v-if="clientDetails.changelog_entries && clientDetails.changelog_entries.length > 0"
                                        class="mt-4">
                                        <ul
                                            class="timeline timeline-compact max-h-52 overflow-y-auto ml-2 pr-4 scrollbar-thin scrollbar-thumb-base-content/20 scrollbar-track-transparent">
                                            <li v-for="(entry, index) in clientDetails.changelog_entries"
                                                :key="entry.version" class="timeline-item">

                                                <div class="timeline-start text-xs text-right">
                                                    <span class="badge badge-ghost text-xs">
                                                        {{ new Date(entry.created_at).toLocaleDateString(undefined, {
                                                            month: 'short', day: 'numeric'
                                                        }) }}
                                                    </span>
                                                </div>
                                                <div class="timeline-middle">
                                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"
                                                        fill="currentColor" class="w-5 h-5 text-primary/70">
                                                        <path fill-rule="evenodd"
                                                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.06 0l4-5.5z"
                                                            clip-rule="evenodd" />
                                                    </svg>
                                                </div>
                                                <div
                                                    class="timeline-end timeline-box shadow-sm bg-base-200/40 w-full mb-2 transition-all duration-300 border-primary/10">
                                                    <div class="font-bold text-sm text-base-content">
                                                        v{{ entry.version }}
                                                    </div>
                                                    <div
                                                        class="text-sm whitespace-pre-line text-base-content/80 mt-1 leading-relaxed">
                                                        {{ entry.content }}
                                                    </div>
                                                </div>
                                                <hr v-if="index < clientDetails.changelog_entries.length - 1"
                                                    class="bg-primary/20" style="width: 119%;" />
                                            </li>
                                        </ul>
                                    </div>
                                    <div v-else
                                        class="mt-6 flex flex-col items-center justify-center text-center p-8 bg-base-200/50 border-2 border-dashed border-base-content/10 rounded-lg opacity-0 animate-fade-in">
                                        <svg xmlns="http://www.w3.org/2000/svg"
                                            class="w-10 h-10 text-base-content/30 mb-3" fill="none" viewBox="0 0 24 24"
                                            stroke="currentColor" stroke-width="1.5">
                                            <path stroke-linecap="round" stroke-linejoin="round"
                                                d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
                                        </svg>
                                        <p class="text-sm font-medium text-base-content/70">{{
                                            t('client.details.no_changelog') }}</p>
                                        <p class="text-xs text-base-content/50 mt-1">{{
                                            t('client.details.no_changelog_desc') }}</p>
                                    </div>
                                </div>

                                <div v-else-if="activeTab === 'screenshots'" key="screenshots" class="tab-pane">
                                    <div v-if="clientDetails.screenshot_urls && clientDetails.screenshot_urls.length > 0"
                                        class="screenshots-section">
                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                                            <div v-for="(screenshot, index) in clientDetails.screenshot_urls"
                                                :key="index" class="screenshot-container group">
                                                <div class="relative overflow-hidden rounded border border-base-content/10 cursor-pointer"
                                                    @click.stop="handleScreenshotClick($event, index)">
                                                    <img :src="screenshot"
                                                        :alt="`${client.name} screenshot ${index + 1}`"
                                                        class="w-full h-36 object-cover transition-all duration-200 group-hover:scale-105"
                                                        @error="($event.target as HTMLImageElement).style.display = 'none'" />
                                                    <div
                                                        class="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors duration-200 flex items-center justify-center pointer-events-none">
                                                        <ZoomIn
                                                            class="w-6 h-6 text-white opacity-0 group-hover:opacity-100 transition-opacity duration-200" />
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    <div v-else class="text-center py-8 text-base-content/60">
                                        <p>{{ t('client.details.no_screenshots') }}</p>
                                    </div>
                                </div>
                            </transition>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <teleport to="body">
        <transition name="screenshot-viewer" appear>
            <div v-if="isScreenshotViewerOpen && clientDetails?.screenshot_urls" ref="screenshotViewerRef"
                class="fixed inset-0 z-[9999] bg-black/95 backdrop-blur-sm flex items-center justify-center"
                @click="handleViewerBackgroundClick">

                <button @click="handleScreenshotViewerClose"
                    class="absolute top-4 right-4 z-10 btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200">
                    <X class="w-6 h-6" />
                </button>

                <button v-if="clientDetails.screenshot_urls.length > 1 && !isZoomed" @click.stop="prevScreenshot"
                    class="absolute left-4 top-1/2 transform -translate-y-1/2 z-10 btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200 hover:scale-110"
                    :disabled="isImageLoading">
                    <ChevronLeft class="w-6 h-6" />
                </button>

                <button v-if="clientDetails.screenshot_urls.length > 1 && !isZoomed" @click.stop="nextScreenshot"
                    class="absolute right-4 top-1/2 transform -translate-y-1/2 z-10 btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200 hover:scale-110"
                    :disabled="isImageLoading">
                    <ChevronRight class="w-6 h-6" />
                </button>

                <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
                    <button @click.stop="zoomIn()"
                        class="btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                        :disabled="zoomScale >= maxZoom" :class="{
                            'cursor-not-allowed opacity-50': zoomScale >= maxZoom
                        }">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                        </svg>
                    </button>
                    <button @click.stop="zoomOut"
                        class="btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                        :disabled="zoomScale <= minZoom" :class="{
                            'cursor-not-allowed opacity-50': zoomScale <= minZoom
                        }">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
                        </svg>
                    </button>
                    <button @click.stop="resetZoom"
                        class="btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                        :disabled="!isZoomed" :class="{
                            'cursor-not-allowed opacity-50': !isZoomed
                        }">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                    </button>
                </div>

                <transition name="fade" appear>
                    <div v-if="isZoomed"
                        class="absolute top-4 left-1/2 transform -translate-x-1/2 z-10 bg-black/50 text-white px-3 py-1 rounded-full text-sm backdrop-blur-sm border border-white/10">
                        {{ t('client.details.screenshot_viewer.zoom_percent', { percent: Math.round(zoomScale * 100) })
                        }}
                    </div>
                </transition>


                <div class="relative max-w-[95vw] max-h-[95vh] flex items-center justify-center p-12"
                    @wheel="handleWheel" @click="handleViewerBackgroundClick">
                    <div class="relative w-full h-full flex items-center justify-center">

                        <transition name="fade">
                            <div v-if="isImageLoading" class="absolute inset-0 flex items-center justify-center z-20">
                                <div class="skeleton-container">
                                    <div
                                        class="skeleton-image animate-pulse bg-gradient-to-r from-gray-300 via-gray-200 to-gray-300 bg-[length:200%_100%] rounded-lg">
                                    </div>
                                    <div class="absolute inset-0 flex items-center justify-center">
                                        <div class="loading loading-spinner loading-lg text-white"></div>
                                    </div>
                                </div>
                            </div>
                        </transition>

                        <div ref="imageContainerRef"
                            class="relative w-full h-full flex items-center justify-center overflow-hidden"
                            @click="handleViewerBackgroundClick">
                            <transition :name="`image-slide-${imageTransitionDirection}`" mode="out-in" appear>
                                <img ref="imageRef" :key="currentScreenshotIndex"
                                    :src="clientDetails.screenshot_urls[currentScreenshotIndex]"
                                    :alt="`${client.name} screenshot ${currentScreenshotIndex + 1}`"
                                    class="max-w-full max-h-full object-contain rounded-lg shadow-2xl select-none image-transition"
                                    :class="{
                                        'opacity-0': isImageLoading,
                                        'opacity-100': !isImageLoading,
                                        'cursor-zoom-in': !isZoomed,
                                        'cursor-grab': isZoomed && !isDragging,
                                        'cursor-grabbing': isDragging,
                                        'smooth-transition': !isDragging
                                    }" :style="{
                                        transform: `scale(${zoomScale}) translate(${zoomPosition.x / zoomScale}px, ${zoomPosition.y / zoomScale}px)`,
                                        transformOrigin: 'center center'
                                    }" @click="handleImageClick" @mousedown="startDrag" @dragstart.prevent
                                    @contextmenu.prevent @load="handleImageLoad" @error="handleImageError" />
                            </transition>
                        </div>
                    </div>
                </div>

                <transition name="fade-slide-up" appear>
                    <div v-if="clientDetails.screenshot_urls.length > 1"
                        class="absolute bottom-4 left-1/2 transform -translate-x-1/2 bg-black/50 text-white px-3 py-1 rounded-full text-sm border border-white/10 backdrop-blur-md">
                        {{ t('client.details.screenshot_viewer.image_counter', {
                            current: currentScreenshotIndex + 1,
                            total: clientDetails.screenshot_urls.length
                        }) }}
                    </div>
                </transition>

                <transition name="fade-slide-up" appear>
                    <div
                        class="absolute bottom-4 right-4 text-white/70 text-xs space-y-1 text-right bg-black/40 p-2 rounded border border-white/10 backdrop-blur-md">
                        <div>{{ isZoomed ? t('client.details.screenshot_viewer.controls.esc_reset') :
                            t('client.details.screenshot_viewer.controls.esc_close') }}</div>
                        <div v-if="clientDetails.screenshot_urls.length > 1 && !isZoomed">{{
                            t('client.details.screenshot_viewer.controls.navigate') }}</div>
                        <div>{{ t('client.details.screenshot_viewer.controls.click_zoom') }}</div>
                        <div v-if="isZoomed">{{ t('client.details.screenshot_viewer.controls.drag_pan') }}</div>
                    </div>
                </transition>
            </div>
        </transition>
    </teleport>
</template>

<style scoped>
.client-card {
    position: relative;
    border-radius: var(--client-card-radius, 0.5rem);
    box-shadow: var(--client-card-shadow, 0 4px 6px -1px rgba(0, 0, 0, 0.1));
    padding: var(--client-card-padding, 1rem);
}

.client-details {
    opacity: 0;
    max-height: 0px;
    overflow: hidden;
    display: none;
    padding-left: 1rem;
    padding-right: 1rem;
}

.status-section {
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    padding-top: 1rem;
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid;
    transition: all 0.2s ease;
}

.status-success {
    background: rgba(34, 197, 94, 0.1);
    border-color: rgba(34, 197, 94, 0.3);
    color: rgb(34, 197, 94);
}

.status-warning {
    background: rgba(245, 158, 11, 0.1);
    border-color: rgba(245, 158, 11, 0.3);
    color: rgb(245, 158, 11);
}

.status-error {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
    color: rgb(239, 68, 68);
}

.status-info {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
    color: rgb(59, 130, 246);
}

.status-dot {
    width: 0.375rem;
    height: 0.375rem;
    border-radius: 50%;
    background: currentColor;
}

.progress-bar-container {
    height: 6px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
}

.progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #3b3b3b, #777777);
    border-radius: 2px;
    transition: width 0.2s ease-out;
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

.download-btn {
    transition: all 0.3s ease;
}

.download-btn:hover .download-text {
    opacity: 0;
    transform: translateY(-20px);
}

.download-btn:hover .get-text {
    opacity: 1;
    transform: translateY(0);
}

.download-text,
.get-text {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    transition: all 0.3s ease;
}

.get-text {
    transform: translateY(20px);
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

.version-badge-container {
    height: 1.5rem;
    margin-bottom: 0.5rem;
}

.version-badge {
    height: 1.25rem;
    min-height: 1.25rem;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    white-space: nowrap;
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

.tab-content {
    position: relative;
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

.screenshot-viewer-enter-active,
.screenshot-viewer-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.screenshot-viewer-enter-from,
.screenshot-viewer-leave-to {
    opacity: 0;
    backdrop-filter: blur(0px);
    transform: scale(0.95);
}

.screenshot-viewer-enter-to,
.screenshot-viewer-leave-from {
    opacity: 1;
    backdrop-filter: blur(4px);
    transform: scale(1);
}

.image-slide-next-enter-active {
    transition: all 0.5s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.image-slide-next-leave-active {
    transition: all 0.3s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.image-slide-prev-enter-active {
    transition: all 0.5s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.image-slide-prev-leave-active {
    transition: all 0.3s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.image-slide-next-enter-from {
    opacity: 0;
    transform: translateX(150px) scale(0.9) rotateY(15deg);
    filter: blur(2px);
}

.image-slide-next-leave-to {
    opacity: 0;
    transform: translateX(-150px) scale(0.9) rotateY(-15deg);
    filter: blur(2px);
}

.image-slide-prev-enter-from {
    opacity: 0;
    transform: translateX(-150px) scale(0.9) rotateY(-15deg);
    filter: blur(2px);
}

.image-slide-prev-leave-to {
    opacity: 0;
    transform: translateX(150px) scale(0.9) rotateY(15deg);
    filter: blur(2px);
}

.image-slide-next-enter-to,
.image-slide-next-leave-from,
.image-slide-prev-enter-to,
.image-slide-prev-leave-from {
    opacity: 1;
    transform: translateX(0) scale(1) rotateY(0deg);
    filter: blur(0px);
}

.skeleton-container {
    position: relative;
    width: 100%;
    height: 100%;
    max-width: 800px;
    max-height: 600px;
    min-width: 400px;
    min-height: 300px;
}

.skeleton-image {
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg,
            rgba(255, 255, 255, 0.1) 25%,
            rgba(255, 255, 255, 0.2) 50%,
            rgba(255, 255, 255, 0.1) 75%);
    background-size: 200% 100%;
    animation: skeleton-wave 2s infinite;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

@keyframes skeleton-wave {
    0% {
        background-position: 200% 0;
    }

    100% {
        background-position: -200% 0;
    }
}

.image-transition {
    transition: opacity 0.3s ease;
}

.fade-slide-up-enter-active {
    transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    transition-delay: 0.2s;
}

.fade-slide-up-leave-active {
    transition: all 0.3s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.fade-slide-up-enter-from {
    opacity: 0;
    transform: translateY(20px);
}

.fade-slide-up-leave-to {
    opacity: 0;
    transform: translateY(10px);
}

.fade-slide-up-enter-to,
.fade-slide-up-leave-from {
    opacity: 1;
    transform: translateY(0);
}

.smooth-transition {
    transition: transform 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

img.cursor-grabbing {
    transition: none;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.screenshot-viewer-nav,
.btn {
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    transform: translateZ(0);
    will-change: transform, opacity;
}

.btn {
    transition: all 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.btn:not(:disabled):hover {
    transform: scale(1.05) translateZ(0);
}

.btn:not(:disabled):active {
    transform: scale(0.98) translateZ(0);
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