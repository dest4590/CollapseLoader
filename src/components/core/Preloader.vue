<template>
    <div
        id="preloader"
        v-if="visible"
        :class="{ 'animate-out': animateOut }"
        class="fixed inset-0 bg-base-300 flex items-center justify-center"
    >
        <div
            class="flex flex-col items-center justify-center h-full w-screen relative z-10"
        >
            <div class="logo-wrapper mb-8 relative" :style="maskStyle">
                <canvas ref="matrixCanvas" class="matrix-canvas"></canvas>
            </div>

            <div class="loading-status mt-2">
                <OdometerText
                    :text="loadingState"
                    :stagger="15"
                    :duration="500"
                    class="text-lg font-bold tracking-wider uppercase font-mono"
                    :class="{
                        'text-gray-800': currentTheme === 'light',
                        'text-white': currentTheme !== 'light',
                    }"
                />
            </div>

            <div
                class="w-96 progress-container mt-6 relative"
                :class="{ invert: currentTheme === 'light' }"
            >
                <div
                    class="bg-white/5 border border-white/10 rounded-full h-1.5 overflow-hidden progress-track relative shadow-inner"
                >
                    <div
                        class="bg-white h-full progress-fill relative"
                        :style="{ width: `${displayProgress}%` }"
                    >
                        <div
                            class="absolute inset-0 bg-linear-to-r from-transparent via-white/30 to-transparent"
                        ></div>
                    </div>
                </div>

                <div
                    class="flex justify-between mt-3 text-[10px] font-mono tracking-[0.2em] text-gray-500"
                >
                    <span class="opacity-70">{{
                        $t("preloader.initializing_progress")
                    }}</span>
                    <span class="percentage-number font-bold text-gray-400">
                        {{ Math.round(displayProgress) }}%
                    </span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import {
    computed,
    ref,
    watch,
    onBeforeUnmount,
    onMounted,
    nextTick,
    toRefs,
} from "vue";
import OdometerText from "./OdometerText.vue";
import { animations, animationKeys } from "../../services/preloaderAnimations";

const props = defineProps({
    show: { type: Boolean, required: true },
    isDev: { type: Boolean, default: false },
    loadingState: { type: String, required: true },
    currentProgress: { type: Number, required: true },
    currentTheme: { type: String, default: "dark" },
    animationType: { type: String, default: "" },
});

const { loadingState, currentTheme } = toRefs(props);

const displayProgress = ref(0);

let _progressRaf: number | null = null;
const LERP_FACTOR = 0.12;
const LERP_EPS = 0.05;

const selectedAnimationKey = ref("");
const _matrixCleanup = ref<(() => void) | null>(null);

const stepProgress = () => {
    const target = props.currentProgress;
    const current = displayProgress.value;
    const next = current + (target - current) * LERP_FACTOR;
    displayProgress.value = Math.abs(target - next) < LERP_EPS ? target : next;

    if (displayProgress.value !== target) {
        _progressRaf = requestAnimationFrame(stepProgress);
    } else {
        _progressRaf = null;
    }
};

watch(
    () => props.currentProgress,
    () => {
        if (_progressRaf) cancelAnimationFrame(_progressRaf);
        _progressRaf = requestAnimationFrame(stepProgress);
    },
    { immediate: true }
);

const pickAnimation = () => {
    if (props.animationType && animations[props.animationType]) {
        selectedAnimationKey.value = props.animationType;
        return;
    }
    const keys = animationKeys;
    selectedAnimationKey.value = keys[Math.floor(Math.random() * keys.length)];
};

const currentSvg = computed(() => {
    const defaultKey = animationKeys[0] || Object.keys(animations)[0];
    return (
        animations[selectedAnimationKey.value]?.svgString ||
        animations[defaultKey]?.svgString ||
        ""
    );
});

const maskStyle = computed(() => {
    const encodedSvg = encodeURIComponent(currentSvg.value);
    const dataUrl = `url("data:image/svg+xml,${encodedSvg}")`;
    return {
        maskImage: dataUrl,
        WebkitMaskImage: dataUrl,
        maskSize: "contain",
        WebkitMaskSize: "contain",
        maskRepeat: "no-repeat",
        WebkitMaskRepeat: "no-repeat",
        maskPosition: "center",
        WebkitMaskPosition: "center",
    };
});

const matrixCanvas = ref<HTMLCanvasElement | null>(null);

const visible = ref(props.show);
const animateOut = ref(false);
const ANIMATE_OUT_MS = 500;

watch(
    () => props.show,
    (val) => {
        if (val) {
            animateOut.value = false;
            visible.value = true;
            nextTick(() => {
                pickAnimation();
                if (_matrixCleanup.value) {
                    _matrixCleanup.value();
                    _matrixCleanup.value = null;
                }
                const anim =
                    animations[selectedAnimationKey.value] || animations.matrix;
                _matrixCleanup.value = anim.initMatrix(
                    matrixCanvas.value,
                    props.currentTheme
                );
            });
        } else {
            animateOut.value = true;
            window.setTimeout(() => {
                animateOut.value = false;
                visible.value = false;
            }, ANIMATE_OUT_MS);
        }
    }
);

onMounted(() => {
    nextTick(() => {
        pickAnimation();
        const anim =
            animations[selectedAnimationKey.value] || animations.matrix;
        _matrixCleanup.value = anim.initMatrix(
            matrixCanvas.value,
            props.currentTheme
        );
    });
});

onBeforeUnmount(() => {
    if (_matrixCleanup.value) {
        _matrixCleanup.value();
        _matrixCleanup.value = null;
    }
    if (_progressRaf) cancelAnimationFrame(_progressRaf);
});
</script>

<style scoped>
#preloader {
    position: fixed;
    width: 100%;
    height: 100%;
    left: 0;
    top: 0;
    background-color: #050505;
    z-index: 1337;
    transition: opacity 0.3s ease;
    color: white;
    -webkit-app-region: drag;
}

#preloader * {
    -webkit-app-region: drag;
}

[data-theme="light"] #preloader {
    background-color: #f0f0f0;
}

#preloader.animate-out {
    opacity: 0;
    pointer-events: none;
}

.logo-wrapper {
    width: 217.5px;
    height: 182.5px;
    position: relative;
    filter: drop-shadow(0 0 15px rgba(255, 255, 255, 0.15));
}

.matrix-canvas {
    width: 100%;
    height: 100%;
    display: block;
}
.invert {
    filter: invert(1);
}

.progress-fill {
    transition: width 1.2s cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow:
        0 0 12px rgba(255, 255, 255, 0.6),
        0 0 4px rgba(255, 255, 255, 0.8);
}
</style>
