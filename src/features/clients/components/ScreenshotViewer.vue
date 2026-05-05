<script setup lang="ts">
import { ref, shallowRef, onMounted, onBeforeUnmount } from "vue";
import {
    ChevronLeft,
    ChevronRight,
    RefreshCw,
    X,
    ZoomIn,
    ZoomOut,
} from "lucide-vue-next";
import { useI18n } from "vue-i18n";

const props = defineProps<{
    isOpen: boolean;
    screenshotUrls: string[];
    initialIndex?: number;
    clientName: string;
}>();

const emit = defineEmits(["close"]);

const { t } = useI18n();

const currentScreenshotIndex = ref(props.initialIndex || 0);
const isImageLoading = ref(false);
const imageTransitionDirection = ref<"next" | "prev">("next");
const imageRef = shallowRef<HTMLImageElement | null>(null);
const imageContainerRef = shallowRef<HTMLElement | null>(null);

const zoomScale = ref(1);
const zoomPosition = ref({ x: 0, y: 0 });
const isZoomed = ref(false);
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const hasDragged = ref(false);
const maxZoom = 5;
const minZoom = 1;
const zoomStep = 0.5;

const resetZoom = () => {
    zoomScale.value = 1;
    zoomPosition.value = { x: 0, y: 0 };
    isZoomed.value = false;
    isDragging.value = false;
};

const handleImageLoad = () => {
    isImageLoading.value = false;
};

const handleImageError = () => {
    isImageLoading.value = false;
    emit("close");
};

const nextScreenshot = () => {
    if (props.screenshotUrls.length <= 1) return;
    imageTransitionDirection.value = "next";
    isImageLoading.value = true;
    currentScreenshotIndex.value =
        (currentScreenshotIndex.value + 1) % props.screenshotUrls.length;
    resetZoom();
};

const prevScreenshot = () => {
    if (props.screenshotUrls.length <= 1) return;
    imageTransitionDirection.value = "prev";
    isImageLoading.value = true;
    currentScreenshotIndex.value =
        (currentScreenshotIndex.value - 1 + props.screenshotUrls.length) %
        props.screenshotUrls.length;
    resetZoom();
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
            y: (zoomPosition.value.y - clickY) * scaleFactor + clickY,
        };

        zoomScale.value = newScale;
        isZoomed.value = true;

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
        y: event.clientY - zoomPosition.value.y,
    };

    document.addEventListener("mousemove", handleDrag, { passive: false });
    document.addEventListener("mouseup", stopDrag);

    if (imageRef.value) {
        imageRef.value.style.pointerEvents = "none";
    }
};

const handleDrag = (event: MouseEvent) => {
    if (!isDragging.value) return;

    event.preventDefault();
    event.stopPropagation();

    hasDragged.value = true;

    zoomPosition.value = {
        x: event.clientX - dragStart.value.x,
        y: event.clientY - dragStart.value.y,
    };
};

const stopDrag = (event?: MouseEvent) => {
    if (event) {
        event.preventDefault();
        event.stopPropagation();
    }

    isDragging.value = false;
    document.removeEventListener("mousemove", handleDrag);
    document.removeEventListener("mouseup", stopDrag);

    if (imageRef.value) {
        imageRef.value.style.pointerEvents = "";
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
        y: Math.min(Math.max(zoomPosition.value.y, -maxY), maxY),
    };

    if (
        constrainedPosition.x !== zoomPosition.value.x ||
        constrainedPosition.y !== zoomPosition.value.y
    ) {
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
        emit("close");
    }
};

const handleScreenshotKeydown = (event: KeyboardEvent) => {
    switch (event.key) {
        case "Escape":
            if (isZoomed.value) {
                resetZoom();
            } else {
                emit("close");
            }
            break;
        case "ArrowLeft":
            if (!isZoomed.value) {
                prevScreenshot();
            }
            break;
        case "ArrowRight":
            if (!isZoomed.value) {
                nextScreenshot();
            }
            break;
        case "+":
        case "=":
            zoomIn();
            break;
        case "-":
            zoomOut();
            break;
        case "0":
            resetZoom();
            break;
    }
};

onMounted(() => {
    document.addEventListener("keydown", handleScreenshotKeydown);
    document.body.style.overflow = "hidden";
});

onBeforeUnmount(() => {
    document.removeEventListener("keydown", handleScreenshotKeydown);
    document.body.style.overflow = "";
});
</script>

<template>
    <teleport to="body">
        <transition name="screenshot-viewer" appear>
            <div
                v-if="isOpen"
                class="fixed inset-0 z-9999 bg-black/95 backdrop-blur-sm flex items-center justify-center"
                @click="handleViewerBackgroundClick"
            >
                <button
                    @click="emit('close')"
                    class="absolute top-4 right-4 z-10 btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                >
                    <X class="w-6 h-6" />
                </button>

                <button
                    v-if="screenshotUrls.length > 1 && !isZoomed"
                    @click.stop="prevScreenshot"
                    class="absolute left-4 top-1/2 transform -translate-y-1/2 z-10 btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200 hover:scale-110"
                    :disabled="isImageLoading"
                >
                    <ChevronLeft class="w-6 h-6" />
                </button>

                <button
                    v-if="screenshotUrls.length > 1 && !isZoomed"
                    @click.stop="nextScreenshot"
                    class="absolute right-4 top-1/2 transform -translate-y-1/2 z-10 btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200 hover:scale-110"
                    :disabled="isImageLoading"
                >
                    <ChevronRight class="w-6 h-6" />
                </button>

                <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
                    <button
                        @click.stop="zoomIn()"
                        class="btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                        :disabled="zoomScale >= maxZoom"
                    >
                        <ZoomIn class="w-5 h-5" />
                    </button>
                    <button
                        @click.stop="zoomOut"
                        class="btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                        :disabled="zoomScale <= minZoom"
                    >
                        <ZoomOut class="w-5 h-5" />
                    </button>
                    <button
                        @click.stop="resetZoom"
                        class="btn btn-circle btn-ghost text-white hover:bg-white/20 transition-all duration-200"
                        :disabled="!isZoomed"
                        :class="{ 'cursor-not-allowed opacity-50': !isZoomed }"
                    >
                        <RefreshCw class="w-5 h-5" />
                    </button>
                </div>

                <transition name="fade" appear>
                    <div
                        v-if="isZoomed"
                        class="absolute top-4 left-1/2 transform -translate-x-1/2 z-10 bg-black/50 text-white px-3 py-1 rounded-full text-sm backdrop-blur-sm border border-white/10"
                    >
                        {{
                            t("client.details.screenshot_viewer.zoom_percent", {
                                percent: Math.round(zoomScale * 100),
                            })
                        }}
                    </div>
                </transition>

                <div
                    class="relative max-w-[95vw] max-h-[95vh] flex items-center justify-center p-12"
                    @wheel="handleWheel"
                    @click="handleViewerBackgroundClick"
                >
                    <div
                        class="relative w-full h-full flex items-center justify-center"
                    >
                        <transition name="fade">
                            <div
                                v-if="isImageLoading"
                                class="absolute inset-0 flex items-center justify-center z-20"
                            >
                                <div class="skeleton-container">
                                    <div
                                        class="skeleton w-full h-full rounded-lg bg-base-200/20"
                                    ></div>
                                    <div
                                        class="absolute inset-0 flex items-center justify-center"
                                    >
                                        <div
                                            class="loading loading-spinner loading-lg text-white"
                                        ></div>
                                    </div>
                                </div>
                            </div>
                        </transition>

                        <div
                            ref="imageContainerRef"
                            class="relative w-full h-full flex items-center justify-center overflow-hidden"
                            @click="handleViewerBackgroundClick"
                        >
                            <transition
                                :name="`image-slide-${imageTransitionDirection}`"
                                mode="out-in"
                                appear
                            >
                                <img
                                    ref="imageRef"
                                    :key="currentScreenshotIndex"
                                    :src="
                                        screenshotUrls[currentScreenshotIndex]
                                    "
                                    :alt="`${clientName} screenshot ${currentScreenshotIndex + 1}`"
                                    class="max-w-full max-h-full object-contain rounded-lg shadow-2xl select-none image-transition"
                                    :class="{
                                        'opacity-0': isImageLoading,
                                        'opacity-100': !isImageLoading,
                                        'cursor-zoom-in': !isZoomed,
                                        'cursor-grab': isZoomed && !isDragging,
                                        'cursor-grabbing': isDragging,
                                        'smooth-transition': !isDragging,
                                    }"
                                    :style="{
                                        transform: `scale(${zoomScale}) translate(${zoomPosition.x / zoomScale}px, ${zoomPosition.y / zoomScale}px)`,
                                        transformOrigin: 'center center',
                                    }"
                                    @click="handleImageClick"
                                    @mousedown="startDrag"
                                    @dragstart.prevent
                                    @contextmenu.prevent
                                    @load="handleImageLoad"
                                    @error="handleImageError"
                                />
                            </transition>
                        </div>
                    </div>
                </div>

                <transition name="fade-slide-up" appear>
                    <div
                        v-if="screenshotUrls.length > 1"
                        class="absolute bottom-4 left-1/2 transform -translate-x-1/2 bg-black/50 text-white px-3 py-1 rounded-full text-sm border border-white/10 backdrop-blur-md"
                    >
                        {{
                            t(
                                "client.details.screenshot_viewer.image_counter",
                                {
                                    current: currentScreenshotIndex + 1,
                                    total: screenshotUrls.length,
                                }
                            )
                        }}
                    </div>
                </transition>

                <transition name="fade-slide-up" appear>
                    <div
                        class="absolute bottom-4 right-4 text-white/70 text-xs space-y-1 text-right bg-black/40 p-2 rounded border border-white/10 backdrop-blur-md"
                    >
                        <div>
                            {{
                                isZoomed
                                    ? t(
                                          "client.details.screenshot_viewer.controls.esc_reset"
                                      )
                                    : t(
                                          "client.details.screenshot_viewer.controls.esc_close"
                                      )
                            }}
                        </div>
                        <div v-if="screenshotUrls.length > 1 && !isZoomed">
                            {{
                                t(
                                    "client.details.screenshot_viewer.controls.navigate"
                                )
                            }}
                        </div>
                        <div>
                            {{
                                t(
                                    "client.details.screenshot_viewer.controls.click_zoom"
                                )
                            }}
                        </div>
                        <div v-if="isZoomed">
                            {{
                                t(
                                    "client.details.screenshot_viewer.controls.drag_pan"
                                )
                            }}
                        </div>
                    </div>
                </transition>
            </div>
        </transition>
    </teleport>
</template>

<style scoped>
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
</style>
